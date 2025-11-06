import { checkForUpdate } from '@/utils/checkForLatestVersion';
import { getVersion } from '@tauri-apps/api/app';
import { useEffect, useState } from 'react';
import { HintIcon } from './HintIcon';
import { isNewerVersion } from '@/utils/compareVersions';
import type { Page } from '@/constants/pages';
import { PageToggle } from './PageToggle';

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
    <header className="relative mb-4 flex items-center justify-center">
      {currentVersion && (
        <a
          href="https://github.com/ath31st/boosty_downloader/releases"
          target="_blank"
          rel="noopener noreferrer"
          className="absolute left-4 text-(--meta-text) text-sm hover:underline"
        >
          v{currentVersion}
        </a>
      )}
      {hasUpdate && (
        <div className="absolute left-18">
          <HintIcon
            text={`Доступна новая версия приложения: ${latestVersion}`}
          />
        </div>
      )}

      <h1 className="flex-1 text-center font-bold text-2xl">
        Boosty Downloader
      </h1>

      <div className="absolute top-0 right-4">
        <PageToggle
          currentPage={currentPage}
          setCurrentPage={setCurrentPage}
          isDownloading={isDownloading}
        />
      </div>
    </header>
  );
}
