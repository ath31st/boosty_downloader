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
  disabled,
}: DownloadOptionsPanelProps) {
  const toggle = (opt: DownloadOption) => {
    onChange(
      value.includes(opt) ? value.filter((x) => x !== opt) : [...value, opt],
    );
  };

  return (
    <div className="flex items-center justify-around rounded-lg py-2">
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
