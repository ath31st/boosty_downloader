use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use crate::{ConsoleLogger, ProgressMessage, get_logger};

pub enum ProgressReporter {
    Bar(ProgressBar),
    Logger { total: u64, downloaded: AtomicU64 },
}

impl ProgressReporter {
    pub fn new(total: u64) -> Result<Self> {
        if get_logger().as_any().is::<ConsoleLogger>() {
            let pb = create_progress_bar(total)?;
            Ok(Self::Bar(pb))
        } else {
            Ok(Self::Logger {
                total,
                downloaded: AtomicU64::new(0),
            })
        }
    }

    pub fn inc(&self, bytes: u64) {
        match self {
            Self::Bar(pb) => pb.inc(bytes),
            Self::Logger { total, downloaded } => {
                let current = downloaded.fetch_add(bytes, Ordering::Relaxed) + bytes;
                get_logger().progress(ProgressMessage {
                    current,
                    total: *total,
                });
            }
        }
    }

    pub fn finish(&self) {
        if let Self::Bar(pb) = self {
            pb.finish_and_clear();
        }
    }
}

fn create_progress_bar(total_size: u64) -> Result<ProgressBar> {
    let pb = if total_size > 0 {
        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
            )?
            .progress_chars("=> "),
        );
        pb
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::with_template(
            "{spinner:.green} Downloading file... {bytes}",
        )?);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    };

    Ok(pb)
}
