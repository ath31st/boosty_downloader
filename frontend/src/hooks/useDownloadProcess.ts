import { useState, useRef, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'sonner';
import type { LogMessage } from '@/types/logMessage';
import type { ProgressMessage } from '@/types/progressMessage';
import { useUrlValidation } from '@/hooks/useUrlValidation';

export function useDownloadProcess(setDownloading: (v: boolean) => void) {
  const [url, setUrl] = useState('');
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

  return {
    url,
    setUrl,
    urlError,
    logs,
    progress,
    startTime,
    startDownload,
    logsEndRef,
  };
}
