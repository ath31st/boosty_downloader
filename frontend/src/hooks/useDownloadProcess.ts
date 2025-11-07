import { useState, useRef, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'sonner';
import type { LogMessage } from '@/types/logMessage';
import type { ProgressMessage } from '@/types/progressMessage';
import { useUrlValidation } from '@/hooks/useUrlValidation';
import { isBlogUrl } from '@/utils/isBlogUrl';
import { isSameBlogUrl } from '@/utils/isSameBlogUrl';

export function useDownloadProcess(setDownloading: (v: boolean) => void) {
  const [url, setUrl] = useState('');
  const [offsetUrl, setOffsetUrl] = useState('');
  const [logs, setLogs] = useState<LogMessage[]>([]);
  const [progress, setProgress] = useState({ current: 0, total: 0 });
  const [startTime, setStartTime] = useState<number | null>(null);
  const { urlError, validateUrl } = useUrlValidation();
  const logsEndRef = useRef<HTMLDivElement>(null);

  // biome-ignore lint/correctness/useExhaustiveDependencies: crying linter with red text
  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  useEffect(() => {
    const unlistenLog = listen('log', (event) => {
      setLogs((prev) => [...prev, event.payload as LogMessage]);
    });
    const unlistenProgress = listen('progress', (event) => {
      setProgress(event.payload as ProgressMessage);
    });

    return () => {
      unlistenLog.then((f) => f());
      unlistenProgress.then((f) => f());
    };
  }, []);

  const startDownload = async () => {
    if (!url) return;
    if (!validateUrl(url)) {
      toast.error('Введите корректный URL');
      return;
    }

    setLogs([]);
    setProgress({ current: 0, total: 0 });
    setStartTime(Date.now());
    setDownloading(true);

    try {
      await invoke('process_boosty_url_gui', { input: url });
      toast.success('Загрузка завершена');
    } catch (e) {
      console.error(e);
      toast.error('Не удалось произвести загрузку');
    } finally {
      setDownloading(false);
    }
  };

  const isOffsetUrlDisabled = useMemo(() => {
    if (!url) return true;
    if (!isBlogUrl(url)) return true;
    return false;
  }, [url]);

  const isDifferentBlogs = useMemo(() => {
    if (!url || !offsetUrl) return false;
    return !isSameBlogUrl(url, offsetUrl);
  }, [url, offsetUrl]);

  useEffect(() => {
    if (isDifferentBlogs) toast.error('Введены разные блоги');
  }, [isDifferentBlogs]);

  return {
    url,
    offsetUrl,
    setUrl,
    setOffsetUrl,
    urlError,
    logs,
    progress,
    startTime,
    startDownload,
    logsEndRef,
    isOffsetUrlDisabled,
    isDifferentBlogs,
  };
}
