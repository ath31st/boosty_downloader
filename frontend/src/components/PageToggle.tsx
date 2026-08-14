import type { Page } from '@/constants/pages';
import { Button } from './Button';
import { CogIcon, List, House } from 'lucide-react';

interface PageToggleProps {
  currentPage: Page;
  setCurrentPage: (page: Page) => void;
  isDownloading: boolean;
}

const ITEMS: { page: Page; icon: typeof House; label: string }[] = [
  { page: 'main', icon: House, label: 'Загрузка' },
  { page: 'downloaded', icon: List, label: 'Скачанное' },
  { page: 'config', icon: CogIcon, label: 'Настройки' },
];

export function PageToggle({
  setCurrentPage,
  isDownloading,
  currentPage,
}: PageToggleProps) {
  return (
    <div className="flex gap-2">
      {ITEMS.map(({ page, icon: Icon, label }) => (
        <Button
          key={page}
          className="px-2"
          disabled={isDownloading || currentPage === page}
          onClick={() => setCurrentPage(page)}
        >
          <span className="sr-only">{label}</span>
          <Icon aria-hidden />
        </Button>
      ))}
    </div>
  );
}
