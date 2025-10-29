import { useMemo } from 'react';

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${(bytes / k ** i).toFixed(2)} ${sizes[i]}`;
}

function formatEta(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = Math.floor(seconds % 60);

  if (h > 0)
    return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
  return `${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}

interface DownloadProgressProps {
  current: number;
  total: number;
  startTime: number;
}

export function DownloadProgress({
  current,
  total,
  startTime,
}: DownloadProgressProps) {
  const { percent, eta, formatted } = useMemo(() => {
    const percent = total > 0 ? (current / total) * 100 : 0;
    const elapsed = (Date.now() - startTime) / 1000;
    const speed = current > 0 && elapsed > 0 ? current / elapsed : 0;
    const remaining = total > 0 ? total - current : 0;
    const eta = speed > 0 ? remaining / speed : 0;

    return {
      percent,
      eta,
      formatted: {
        current: formatBytes(current),
        total: formatBytes(total),
      },
    };
  }, [current, total, startTime]);

  return (
    <div
      className={`relative flex w-full flex-col gap-2 ${total === 0 ? 'opacity-50' : ''}`}
    >
      <div className="relative h-6 w-full overflow-hidden rounded-md bg-(--border)">
        <div
          className="absolute top-0 left-0 h-full bg-(--button-bg) transition-all duration-150"
          style={{ width: `${percent}%` }}
        />
        <div className="absolute inset-0 flex items-center justify-center font-medium text-(--text) text-sm">
          {formatted.current} / {formatted.total}
          {eta > 0 && (
            <span className="ml-2 text-(--meta-text)">
              ({formatEta(eta)} left)
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
