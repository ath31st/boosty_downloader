import { formatBytes } from '@/utils/formatBytes';
import { formatEta } from '@/utils/formatEta';
import { useEffect, useRef, useState } from 'react';

interface DownloadProgressProps {
  filesDone: number;
  filesTotal: number;
  fileName: string | null;
  current: number;
  total: number;
  isDownloading: boolean;
}

export function DownloadProgress({
  filesDone,
  filesTotal,
  fileName,
  current,
  total,
  isDownloading,
}: DownloadProgressProps) {
  const [fileStartedAt, setFileStartedAt] = useState<number | null>(null);
  const activeFileKey = useRef<string | null>(null);

  useEffect(() => {
    if (!isDownloading || !fileName) {
      activeFileKey.current = null;
      setFileStartedAt(null);
      return;
    }

    const key = `${filesDone}:${fileName}:${total}`;
    if (activeFileKey.current !== key) {
      activeFileKey.current = key;
      setFileStartedAt(Date.now());
    }
  }, [isDownloading, fileName, filesDone, total]);

  const idle = !isDownloading || filesTotal === 0;
  const fileIndex =
    fileName != null
      ? Math.min(filesDone + 1, Math.max(filesTotal, 1))
      : filesDone;
  const percent = total > 0 ? Math.min(100, (current / total) * 100) : 0;
  const indeterminate = isDownloading && fileName != null && total === 0;

  let eta = 0;
  if (fileStartedAt && current > 0 && total > current) {
    const elapsed = (Date.now() - fileStartedAt) / 1000;
    const speed = elapsed > 0 ? current / elapsed : 0;
    eta = speed > 0 ? (total - current) / speed : 0;
  }

  return (
    <div className={`flex w-full flex-col gap-1.5 ${idle ? 'opacity-50' : ''}`}>
      <div className="flex items-baseline justify-between gap-3 text-sm">
        <span className="truncate text-(--text)">
          {fileName
            ? `Файл ${fileIndex}/${filesTotal}: ${fileName}`
            : filesTotal > 0
              ? `Файлы ${filesDone}/${filesTotal}`
              : ''}
        </span>
        <span className="shrink-0 text-(--meta-text)">
          {indeterminate
            ? formatBytes(current)
            : total > 0
              ? `${formatBytes(current)} / ${formatBytes(total)}`
              : filesTotal > 0
                ? `${filesDone}/${filesTotal}`
                : '0 B / 0 B'}
          {eta > 0 && <span className="ml-2">(~{formatEta(eta)} ост.)</span>}
        </span>
      </div>

      <div className="relative h-2.5 w-full overflow-hidden rounded-md bg-(--border)">
        {indeterminate ? (
          <div className="progress-indeterminate absolute inset-y-0 rounded-md bg-(--button-bg)" />
        ) : (
          <div
            className="h-full rounded-md bg-(--button-bg) transition-[width] duration-150"
            style={{
              width: `${total > 0 ? percent : filesTotal > 0 ? (filesDone / filesTotal) * 100 : 0}%`,
            }}
          />
        )}
      </div>
    </div>
  );
}
