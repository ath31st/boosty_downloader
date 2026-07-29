use anyhow::Result;
use boosty_api::media_content::ContentItem;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use once_cell::sync::Lazy;
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::{ConsoleLogger, ProgressMessage, get_logger};

const EMIT_INTERVAL: Duration = Duration::from_millis(120);

static SESSION: Lazy<Mutex<Option<Session>>> = Lazy::new(|| Mutex::new(None));

struct Session {
    files_done: u64,
    files_total: u64,
    multi: Option<MultiProgress>,
    overall: Option<ProgressBar>,
    current_bar: Option<ProgressBar>,
    file_name: Option<String>,
    file_current: u64,
    file_total: u64,
    last_emit: Instant,
    use_cli: bool,
}

pub struct SessionGuard;

impl SessionGuard {
    pub fn new(files_total: u64) -> Self {
        begin(files_total);
        Self
    }
}

impl Drop for SessionGuard {
    fn drop(&mut self) {
        end();
    }
}

pub fn count_downloadable_files(items: &[ContentItem]) -> u64 {
    items.iter().map(count_item).sum()
}

fn count_item(item: &ContentItem) -> u64 {
    match item {
        ContentItem::Image { .. }
        | ContentItem::OkVideo { .. }
        | ContentItem::Audio { .. }
        | ContentItem::File { .. }
        | ContentItem::Smile { .. } => 1,
        ContentItem::List { items, .. } => items.iter().flatten().map(count_item).sum(),
        _ => 0,
    }
}

fn begin(files_total: u64) {
    let use_cli = get_logger().as_any().is::<ConsoleLogger>();
    let (multi, overall) = if use_cli {
        let multi = MultiProgress::new();
        let overall = multi.add(ProgressBar::new(files_total));
        overall.set_style(
            ProgressStyle::with_template("{bar:40.cyan/blue} {pos}/{len} files")
                .expect("overall progress style")
                .progress_chars("=> "),
        );
        if files_total == 0 {
            overall.set_position(0);
        }
        (Some(multi), Some(overall))
    } else {
        (None, None)
    };

    let mut slot = SESSION.lock().expect("progress session lock");
    *slot = Some(Session {
        files_done: 0,
        files_total,
        multi,
        overall,
        current_bar: None,
        file_name: None,
        file_current: 0,
        file_total: 0,
        last_emit: Instant::now()
            .checked_sub(EMIT_INTERVAL)
            .unwrap_or_else(Instant::now),
        use_cli,
    });
    drop(slot);
    emit(true);
}

pub fn add_files_total(extra: u64) {
    if extra == 0 {
        return;
    }
    let mut slot = SESSION.lock().expect("progress session lock");
    let Some(session) = slot.as_mut() else {
        return;
    };
    session.files_total = session.files_total.saturating_add(extra);
    if let Some(overall) = &session.overall {
        overall.set_length(session.files_total);
    }
    drop(slot);
    emit(true);
}

pub fn start_file(name: &str, total: u64) -> Result<()> {
    let mut slot = SESSION.lock().expect("progress session lock");
    let Some(session) = slot.as_mut() else {
        return Ok(());
    };

    clear_current_bar(session);

    session.file_name = Some(name.to_string());
    session.file_current = 0;
    session.file_total = total;

    if session.use_cli {
        let bar = create_file_bar(total)?;
        if let Some(multi) = &session.multi {
            let bar = multi.add(bar);
            bar.set_message(name.to_string());
            session.current_bar = Some(bar);
        } else {
            session.current_bar = Some(bar);
        }
    }

    drop(slot);
    emit(true);
    Ok(())
}

pub fn inc(bytes: u64) {
    let mut slot = SESSION.lock().expect("progress session lock");
    let Some(session) = slot.as_mut() else {
        return;
    };

    session.file_current = session.file_current.saturating_add(bytes);
    if let Some(bar) = &session.current_bar {
        bar.inc(bytes);
    }

    let should_emit = !session.use_cli && session.last_emit.elapsed() >= EMIT_INTERVAL;
    if should_emit {
        session.last_emit = Instant::now();
    }
    drop(slot);

    if should_emit {
        emit(false);
    }
}

pub fn finish_file() {
    let mut slot = SESSION.lock().expect("progress session lock");
    let Some(session) = slot.as_mut() else {
        return;
    };

    clear_current_bar(session);
    session.file_name = None;
    session.file_current = 0;
    session.file_total = 0;
    session.files_done = session.files_done.saturating_add(1);
    if let Some(overall) = &session.overall {
        overall.inc(1);
    }

    drop(slot);
    emit(true);
}

pub fn abandon_file() {
    let mut slot = SESSION.lock().expect("progress session lock");
    let Some(session) = slot.as_mut() else {
        return;
    };

    clear_current_bar(session);
    session.file_name = None;
    session.file_current = 0;
    session.file_total = 0;

    drop(slot);
    emit(true);
}

pub fn suspend_for<R>(f: impl FnOnce() -> R) -> R {
    let slot = SESSION.lock().expect("progress session lock");
    if let Some(session) = slot.as_ref()
        && let Some(multi) = &session.multi
    {
        return multi.suspend(f);
    }
    drop(slot);
    f()
}

fn end() {
    let mut slot = SESSION.lock().expect("progress session lock");
    if let Some(mut session) = slot.take() {
        clear_current_bar(&mut session);
        if let Some(overall) = session.overall.take() {
            overall.finish_and_clear();
        }
    }
}

fn clear_current_bar(session: &mut Session) {
    if let Some(bar) = session.current_bar.take() {
        bar.finish_and_clear();
    }
}

fn emit(force_update_clock: bool) {
    let mut slot = SESSION.lock().expect("progress session lock");
    let Some(session) = slot.as_mut() else {
        return;
    };
    if session.use_cli {
        return;
    }
    if force_update_clock {
        session.last_emit = Instant::now();
    }

    let msg = ProgressMessage {
        files_done: session.files_done,
        files_total: session.files_total,
        file_name: session.file_name.clone(),
        current: session.file_current,
        total: session.file_total,
    };
    drop(slot);
    get_logger().progress(msg);
}

fn create_file_bar(total_size: u64) -> Result<ProgressBar> {
    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}",
            )?
            .progress_chars("=> "),
        );
        pb
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::with_template(
            "{spinner:.green} {msg} {bytes}",
        )?);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    };

    Ok(pb)
}
