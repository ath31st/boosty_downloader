import type { Page } from '@/constants/pages';
import { Button } from './Button';

interface PageTabsProps {
  setCurrentPage: (page: Page) => void;
  isDownloading: boolean;
}

export function PageTabs({ setCurrentPage, isDownloading }: PageTabsProps) {
  return (
    <div className="mb-6 flex gap-4">
      <Button
        className="flex-1"
        disabled={isDownloading}
        onClick={() => setCurrentPage('main')}
      >
        Главная
      </Button>
      <Button
        className="flex-1"
        disabled={isDownloading}
        onClick={() => setCurrentPage('config')}
      >
        Настройки
      </Button>
    </div>
  );
}
