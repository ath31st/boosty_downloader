import { useState, useRef, useEffect } from 'react';
import { Button } from '../components/Button';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { DownloadProgress } from '@/components/DownloadProgress';
import type { LogMessage } from '@/types/logMessage';
import { FormatLog } from '@/components/FormatLog';
import type { ProgressMessage } from '@/types/progressMessage';

interface MainPageProps {
  isDownloading: boolean;
  setDownloading: (value: boolean) => void;
}

export default function MainPage({
  isDownloading,
  setDownloading,
}: MainPageProps) {
  const [url, setUrl] = useState('');
  const [logs, setLogs] = useState<LogMessage[]>([]);
  const [progress, setProgress] = useState({ current: 0, total: 0 });
  const [startTime, setStartTime] = useState<number | null>(null);

  const logsEndRef = useRef<HTMLDivElement>(null);

  // biome-ignore lint/correctness/useExhaustiveDependencies: crying linter with red text
  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  useEffect(() => {
    const unlistenLog = listen('log', (event) => {
      const msg = event.payload as LogMessage;
      setLogs((prev) => [...prev, msg]);
    });

    const unlistenProgress = listen('progress', (event) => {
      const msg = event.payload as ProgressMessage;
      setProgress(msg);
    });

    return () => {
      unlistenLog.then((f) => f());
      unlistenProgress.then((f) => f());
    };
  }, []);

  const startDownload = async () => {
    if (!url) return;

    setLogs([]);
    setDownloading(true);
    setProgress({ current: 0, total: 0 });
    setStartTime(Date.now());

    try {
      await invoke('process_boosty_url_gui', { input: url });
    } catch (e) {
      console.error(e);
    } finally {
      setDownloading(false);
    }
  };

  return (
    <div className="flex flex-col gap-4 rounded-lg border border-(--border) bg-(--background) p-4 text-(--text)">
      <div className="flex gap-4">
        <input
          type="text"
          placeholder="Введите URL адрес блога или конкретного поста"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          disabled={isDownloading}
          className="flex-1 rounded-lg border border-(--border) bg-(--secondary-bg) p-2 text-(--text) focus:outline-none focus:ring-(--button-bg) focus:ring-2"
        />
        <Button onClick={startDownload} disabled={isDownloading || !url}>
          Скачать
        </Button>
      </div>

      <div className="h-70 overflow-y-auto rounded-lg border border-(--border) bg-(--secondary-bg) p-2">
        {logs.map((msg) => (
          <p
            key={msg.message}
            ref={logsEndRef}
            className="text-(--meta-text) text-sm"
          >
            {FormatLog(msg)}
          </p>
        ))}
        <div ref={logsEndRef} />
      </div>

      <DownloadProgress
        current={progress.current}
        total={progress.total}
        startTime={startTime ?? Date.now()}
      />
    </div>
  );
}
