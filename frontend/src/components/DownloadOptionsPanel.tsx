import {
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

interface DownloadOptionsPanelProps {
  value: DownloadOptions;
  onChange: (value: DownloadOptions) => void;
  disabled?: boolean;
}

const OPTIONS: {
  key: DownloadOption;
  label: string;
  icon: LucideIcon;
}[] = [
  { key: 'Video', label: 'Видео контент', icon: Video },
  { key: 'Audio', label: 'Аудио контент', icon: Music },
  { key: 'Images', label: 'Изображения', icon: Image },
  { key: 'Texts', label: 'Текстовый контент', icon: FileText },
  { key: 'Files', label: 'Файлы', icon: File },
];

export function DownloadOptionsPanel({
  value,
  onChange,
  disabled,
}: DownloadOptionsPanelProps) {
  const toggle = (opt: DownloadOption) => {
    onChange(
      value.includes(opt) ? value.filter((x) => x !== opt) : [...value, opt],
    );
  };

  return (
    <div className="flex items-center justify-around gap-5 rounded-lg border border-(--border) bg-(--secondary-bg) px-3 py-2">
      {OPTIONS.map(({ key, label, icon: Icon }) => {
        const checked = value.includes(key);

        return (
          <label htmlFor={key} key={key} className="flex items-center gap-2">
            <Checkbox
              checked={checked}
              disabled={disabled}
              onCheckedChange={() => toggle(key)}
            />

            <Icon size={24} />
            <HintIcon text={label} />
          </label>
        );
      })}
    </div>
  );
}
