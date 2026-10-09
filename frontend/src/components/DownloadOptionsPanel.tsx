import {
  ChevronDown,
  File,
  FileText,
  Image,
  Music,
  Video,
  type LucideIcon,
} from 'lucide-react';

import { Checkbox } from '@/components/Checkbox';
import { HintIcon } from '@/components/HintIcon';
import type { DownloadOption, DownloadOptions } from '@/types/downloadOptions';
import { VIDEO_QUALITY_OPTIONS, type VideoQuality } from '@/types/videoQuality';

interface DownloadOptionsPanelProps {
  value: DownloadOptions;
  onChange: (value: DownloadOptions) => void;
  videoQuality: VideoQuality;
  onVideoQualityChange: (value: VideoQuality) => void;
  disabled?: boolean;
}

const OPTIONS: {
  key: DownloadOption;
  label: string;
  icon: LucideIcon;
}[] = [
  {
    key: 'Video',
    label:
      'Видео контент (включая ссылки на видео с других площадок, например, YouTube)',
    icon: Video,
  },
  { key: 'Audio', label: 'Аудио контент', icon: Music },
  { key: 'Images', label: 'Изображения', icon: Image },
  {
    key: 'Texts',
    label: 'Текст поста (включая ссылки и эмодзи)',
    icon: FileText,
  },
  { key: 'Files', label: 'Файлы', icon: File },
];

export function DownloadOptionsPanel({
  value,
  onChange,
  videoQuality,
  onVideoQualityChange,
  disabled,
}: DownloadOptionsPanelProps) {
  const toggle = (opt: DownloadOption) => {
    onChange(
      value.includes(opt) ? value.filter((x) => x !== opt) : [...value, opt],
    );
  };

  return (
    <div className="flex items-center gap-4 rounded-lg py-1">
      <div className="flex flex-1 items-center justify-around">
        {OPTIONS.map(({ key, label, icon: Icon }) => {
          const checked = value.includes(key);

          return (
            <label htmlFor={key} key={key} className="flex items-center gap-2">
              <Checkbox
                checked={checked}
                disabled={disabled}
                onCheckedChange={() => toggle(key)}
              />

              <Icon size={28} />
              <HintIcon size={20} text={label} />
            </label>
          );
        })}
      </div>
      <label className="flex shrink-0 items-center gap-2">
        <span className="relative">
          <select
            value={videoQuality}
            disabled={disabled}
            onChange={(event) =>
              onVideoQualityChange(event.target.value as VideoQuality)
            }
            className="cursor-pointer appearance-none rounded-lg border border-(--border) bg-(--secondary-bg) py-1 pr-8 pl-2 text-(--text) focus:outline-none focus:ring-(--button-bg) focus:ring-2 disabled:cursor-not-allowed disabled:opacity-50"
          >
            {VIDEO_QUALITY_OPTIONS.map((option) => (
              <option
                key={option.value}
                value={option.value}
                className="bg-white text-neutral-900"
              >
                {option.label}
              </option>
            ))}
          </select>
          <ChevronDown
            size={16}
            aria-hidden
            className="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2 text-(--meta-text)"
          />
        </span>

        <HintIcon
          size={20}
          text="Потолок качества только для видео размещенных на Boosty. Ролики с других площадок (например, YouTube) не ограничиваются. Уже скачанный файл заменяется через «Перекачать»."
        />
      </label>
    </div>
  );
}
