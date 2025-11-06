import type { Page } from '@/constants/pages';
import { Button } from './Button';
import { CogIcon, House } from 'lucide-react';

interface PageToggleProps {
  currentPage: Page;
  setCurrentPage: (page: Page) => void;
  isDownloading: boolean;
}

export function PageToggle({
  setCurrentPage,
  isDownloading,
  currentPage,
}: PageToggleProps) {
  const nextPage = currentPage === 'main' ? 'config' : 'main';
  const icon = currentPage === 'main' ? <CogIcon /> : <House />;

  return (
    <Button
      className="w-full"
      disabled={isDownloading}
      onClick={() => setCurrentPage(nextPage)}
    >
      {icon}
    </Button>
  );
}
