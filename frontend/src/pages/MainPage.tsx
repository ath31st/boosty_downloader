import { useState, useRef, useEffect } from 'react';
import { Button } from '../components/Button';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export default function MainPage() {
  const [url, setUrl] = useState('');
  const [logs, setLogs] = useState<string[]>([]);
  const [progress, setProgress] = useState(0);
  const [downloading, setDownloading] = useState(false);

  const logsEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    logsEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  useEffect(() => {
    const unlisten = listen('log', (event) => {
      const msg = event.payload as {
        level: 'Info' | 'Warn' | 'Error';
        message: string;
      };
      setLogs((prev) => [...prev, `[${msg.level}] ${msg.message}`]);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const startDownload = async () => {
    if (!url) return;

    setLogs([]);
    setProgress(0);
    setDownloading(true);

    try {
      await invoke('process_boosty_url_gui', { input: url });
      setLogs((prev) => [...prev, 'Скачивание завершено!']);
    } catch (e) {
      setLogs((prev) => [...prev, `Ошибка: ${e}`]);
    } finally {
      setDownloading(false);
    }
  };

  return (
    <div className="flex flex-col gap-4 rounded-lg border border-(--border) bg-(--background) p-4 text-(--text)">
      <div className="flex gap-4">
        <input
          type="text"
          placeholder="Введите URL"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          disabled={downloading}
          className="flex-1 rounded-lg border border-(--border) bg-(--background) p-2 text-(--text) focus:outline-none focus:ring-(--button-bg) focus:ring-2"
        />
        <Button onClick={startDownload} disabled={downloading || !url}>
          Скачать
        </Button>
      </div>

      <div className="h-60 overflow-y-auto rounded-lg border border-(--border) bg-(--background) p-2">
        {logs.map((line) => (
          <p key={line} ref={logsEndRef} className="text-(--meta-text) text-sm">
            {line}
          </p>
        ))}
        <div ref={logsEndRef} />
      </div>

      <div className="h-4 w-full rounded-lg bg-(--border)">
        <div
          className="h-4 rounded-lg bg-(--button-bg)"
          style={{ width: `${progress}%` }}
        ></div>
      </div>
    </div>
  );
}
