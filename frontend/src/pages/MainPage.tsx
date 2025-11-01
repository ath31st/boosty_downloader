import { Button } from '../components/Button';
import { DownloadProgress } from '@/components/DownloadProgress';
import { FormatLog } from '@/components/FormatLog';
import { OpenFolderButton } from '@/components/OpenFolderButton';
import { DownloadIcon } from 'lucide-react';
import { Input } from '@/components/Input';
import { useDownloadProcess } from '@/hooks/useDownloadProcess';

interface MainPageProps {
  isDownloading: boolean;
  setDownloading: (value: boolean) => void;
}

export default function MainPage({
  isDownloading,
  setDownloading,
}: MainPageProps) {
  const {
    url,
    setUrl,
    urlError,
    logs,
    progress,
    startTime,
    startDownload,
    logsEndRef,
  } = useDownloadProcess(setDownloading);

  return (
    <div className="flex flex-col gap-4 rounded-lg border border-(--border) bg-(--background) p-4 text-(--text)">
      <div className="flex gap-4">
        <Input
          placeholder="Введите URL адрес блога или конкретного поста"
          value={url}
          onChange={(value) => setUrl(String(value))}
          disabled={isDownloading}
          className="flex-1"
        />
        <Button onClick={startDownload} disabled={isDownloading || !url}>
          <DownloadIcon />
        </Button>
        <OpenFolderButton />
      </div>

      {urlError && <p className="text-(--error) text-sm">{urlError}</p>}

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
