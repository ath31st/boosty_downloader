import { checkForUpdate } from '@/utils/checkForLatestVersion';
import { getVersion } from '@tauri-apps/api/app';
import { useEffect, useState } from 'react';
import { UpdateButton } from './UpdateButton';
import { isNewerVersion } from '@/utils/compareVersions';
import type { Page } from '@/constants/pages';
import { PageToggle } from './PageToggle';
import { ThemeSwitch } from './ThemeSwitch';

interface HeaderProps {
  currentPage: Page;
  setCurrentPage: (page: Page) => void;
  isDownloading: boolean;
}

export function Header({
  currentPage,
  setCurrentPage,
  isDownloading,
}: HeaderProps) {
  const [currentVersion, setCurrentVersion] = useState('');
  const [latestVersion, setLatestVersion] = useState<string | undefined>(
    undefined,
  );

  useEffect(() => {
    (async () => {
      const v = await getVersion();
      setCurrentVersion(v);

      const latest = await checkForUpdate();
      setLatestVersion(latest);
    })();
  }, []);

  const hasUpdate =
    latestVersion &&
    currentVersion &&
    isNewerVersion(latestVersion, currentVersion);

  return (
    <header className="relative mb-4 flex shrink-0 items-center justify-center">
      <div className="absolute left-4 flex items-center gap-2">
        {currentVersion && (
          <a
            href="https://github.com/ath31st/boosty_downloader/releases"
            target="_blank"
            rel="noopener noreferrer"
            className="text-(--meta-text) text-md hover:underline"
          >
            v{currentVersion}
          </a>
        )}
        {hasUpdate && latestVersion && (
          <UpdateButton latestVersion={latestVersion} />
        )}
      </div>

      <h1 className="flex-1 text-center font-bold text-2xl">
        Boosty Downloader
      </h1>

      <div className="absolute top-0 right-4 flex items-center gap-3">
        <ThemeSwitch />
        <PageToggle
          currentPage={currentPage}
          setCurrentPage={setCurrentPage}
          isDownloading={isDownloading}
        />
      </div>
    </header>
  );
}
