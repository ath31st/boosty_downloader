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
import type { DownloadOptions } from '@/types/downloadOptions';

interface DownloadOptionsPanelProps {
  value: DownloadOptions;
  onChange: (value: DownloadOptions) => void;
  disabled?: boolean;
}

const OPTIONS: {
  key: keyof DownloadOptions;
  label: string;
  icon: LucideIcon;
}[] = [
  {
    key: 'video',
    label: 'Видео контент',
    icon: Video,
  },
  {
    key: 'audio',
    label: 'Аудио контент',
    icon: Music,
  },
  {
    key: 'images',
    label: 'Изображения',
    icon: Image,
  },
  {
    key: 'texts',
    label: 'Текстовый контент',
    icon: FileText,
  },
  {
    key: 'files',
    label: 'Файлы',
    icon: File,
  },
];

export function DownloadOptionsPanel({
  value,
  onChange,
  disabled,
}: DownloadOptionsPanelProps) {
  return (
    <div className="flex items-center justify-around gap-5 rounded-lg border border-(--border) bg-(--secondary-bg) px-3 py-2">
      {OPTIONS.map(({ key, label, icon: Icon }) => (
        <label htmlFor={key} key={key} className="flex items-center gap-2">
          <Checkbox
            checked={value[key]}
            disabled={disabled}
            onCheckedChange={(checked) =>
              onChange({
                ...value,
                [key]: Boolean(checked),
              })
            }
          />

          <Icon size={24} />

          <HintIcon text={label} />
        </label>
      ))}
    </div>
  );
}
