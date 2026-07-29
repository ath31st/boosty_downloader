import { useState, useRef, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'sonner';
import type { LogMessage } from '@/types/logMessage';
import type { ProgressMessage } from '@/types/progressMessage';
import { useUrlValidation } from '@/hooks/useUrlValidation';
import { isBlogUrl } from '@/utils/isBlogUrl';
import { isSameBlogUrl } from '@/utils/isSameBlogUrl';
import type { DownloadOptions } from '@/types/downloadOptions';

export function useDownloadProcess(setDownloading: (v: boolean) => void) {
  const [url, setUrl] = useState(() => {
    if (typeof window === 'undefined') return '';
    return sessionStorage.getItem('url') ?? '';
  });

  const [offsetUrl, setOffsetUrl] = useState(() => {
    if (typeof window === 'undefined') return '';
    return sessionStorage.getItem('offsetUrl') ?? '';
  });

  const [logs, setLogs] = useState<LogMessage[]>([]);
  const [progress, setProgress] = useState({ current: 0, total: 0 });
  const [startTime, setStartTime] = useState<number | null>(null);
  const { urlError, validateUrl } = useUrlValidation();
  const logsEndRef = useRef<HTMLDivElement>(null);
  const [downloadOptions, setDownloadOptions] = useState<DownloadOptions>([
    'Video',
    'Audio',
    'Images',
    'Texts',
    'Files',
  ]);

  useEffect(() => {
    if (url) sessionStorage.setItem('url', url);
    else sessionStorage.removeItem('url');
  }, [url]);

  useEffect(() => {
    if (offsetUrl) sessionStorage.setItem('offsetUrl', offsetUrl);
    else sessionStorage.removeItem('offsetUrl');
  }, [offsetUrl]);

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
      await invoke('download_content', {
        url: url.trim(),
        offsetUrl: offsetUrl.trim() !== '' ? offsetUrl : undefined,
        downloadOptions,
      });
      toast.success('Загрузка завершена');
    } catch (e) {
      console.error(e);
      if (String(e) === 'Download cancelled by user') {
        toast.info('Загрузка отменена');
      } else {
        toast.error('Не удалось произвести загрузку');
      }
    } finally {
      setDownloading(false);
    }
  };

  const cancelDownload = async () => {
    try {
      await invoke('cancel_download');
    } catch (e) {
      console.error(e);
      toast.error('Не удалось отменить загрузку');
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
    cancelDownload,
    logsEndRef,
    isOffsetUrlDisabled,
    isDifferentBlogs,
    downloadOptions,
    setDownloadOptions,
  };
}
