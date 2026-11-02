import * as Tooltip from '@radix-ui/react-tooltip';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { ArrowUpCircle, LoaderCircle } from 'lucide-react';
import { useState } from 'react';
import { toast } from 'sonner';

interface UpdateButtonProps {
  latestVersion: string;
}

export function UpdateButton({ latestVersion }: UpdateButtonProps) {
  const [isUpdating, setIsUpdating] = useState(false);
  const [progress, setProgress] = useState<number | null>(null);

  const handleUpdate = async () => {
    const confirmed = window.confirm(
      `Доступна новая версия ${latestVersion}.\nОбновить приложение сейчас?`,
    );
    if (!confirmed) return;

    setIsUpdating(true);
    setProgress(null);

    try {
      const update = await check();
      if (!update) {
        toast.error('Не удалось найти обновление');
        return;
      }

      let downloaded = 0;
      let total = 0;

      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          total = event.data.contentLength ?? 0;
        } else if (event.event === 'Progress') {
          downloaded += event.data.chunkLength;
          if (total > 0) {
            setProgress(Math.round((downloaded / total) * 100));
          }
        } else if (event.event === 'Finished') {
          setProgress(100);
        }
      });

      await relaunch();
    } catch (e) {
      console.error(e);
      toast.error(`Ошибка обновления: ${e}`);
      setIsUpdating(false);
      setProgress(null);
    }
  };

  const tooltipText = isUpdating
    ? progress !== null
      ? `Скачивание... ${progress}%`
      : 'Подготовка к обновлению...'
    : `Доступна новая версия: ${latestVersion}`;

  return (
    <Tooltip.Provider delayDuration={100}>
      <Tooltip.Root>
        <Tooltip.Trigger asChild>
          <button
            type="button"
            onClick={handleUpdate}
            disabled={isUpdating}
            className="cursor-pointer text-(--meta-text) transition-colors hover:text-(--accent) disabled:cursor-not-allowed disabled:opacity-60"
            aria-label={`Обновить до версии ${latestVersion}`}
          >
            {isUpdating ? (
              <LoaderCircle
                size={16}
                className="animate-spin text-(--accent)"
              />
            ) : (
              <ArrowUpCircle size={16} />
            )}
          </button>
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content
            side="right"
            className="z-50 max-w-[350px] rounded-lg border border-(--border) bg-(--tooltip-bg) px-2 py-1 text-(--text) text-sm shadow-md"
          >
            {tooltipText}
            <Tooltip.Arrow className="fill-(--tooltip-bg)" />
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
}
