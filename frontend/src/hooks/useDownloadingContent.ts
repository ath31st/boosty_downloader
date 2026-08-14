import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { toast } from 'sonner';
import type { DownloadOptions } from '@/types/downloadOptions';
import type { LogMessage } from '@/types/logMessage';
import type { ProgressMessage } from '@/types/progressMessage';

const EMPTY_PROGRESS: ProgressMessage = {
  files_done: 0,
  files_total: 0,
  file_name: null,
  current: 0,
  total: 0,
};

const DEFAULT_OPTIONS: DownloadOptions = [
  'Video',
  'Audio',
  'Images',
  'Texts',
  'Files',
];

export function useDownloadingContent() {
  const [isDownloading, setDownloading] = useState(false);
  const [downloadOptions, setDownloadOptions] =
    useState<DownloadOptions>(DEFAULT_OPTIONS);
  const [logs, setLogs] = useState<LogMessage[]>([]);
  const [progress, setProgress] = useState(EMPTY_PROGRESS);
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

  const resetDownloadUi = () => {
    setLogs([]);
    setProgress(EMPTY_PROGRESS);
  };

  const resetProgress = () => {
    setProgress(EMPTY_PROGRESS);
  };

  const cancelDownload = async () => {
    try {
      await invoke('cancel_download');
    } catch (e) {
      console.error(e);
      toast.error('Не удалось отменить загрузку');
    }
  };

  return {
    isDownloading,
    setDownloading,
    downloadOptions,
    setDownloadOptions,
    logs,
    progress,
    logsEndRef,
    resetDownloadUi,
    resetProgress,
    cancelDownload,
  };
}

export type DownloadSession = ReturnType<typeof useDownloadingContent>;
