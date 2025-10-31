import { checkForUpdate } from '@/utils/checkForLatestVersion';
import { getVersion } from '@tauri-apps/api/app';
import { useEffect, useState } from 'react';
import { HintIcon } from './HintIcon';

export const Header = () => {
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

  return (
    <header className="relative mb-4 flex items-center justify-center">
      {currentVersion && (
        <a
          href="https://github.com/ath31st/boosty_downloader/releases"
          target="_blank"
          rel="noopener noreferrer"
          className="absolute left-0 text-(--meta-text) text-sm hover:underline"
        >
          {currentVersion}
        </a>
      )}
      {latestVersion && latestVersion !== currentVersion && (
        <div className="absolute left-14">
          <HintIcon
            text={`Доступна новая версия приложения: ${latestVersion}`}
          />
        </div>
      )}
      <h1 className="flex-1 text-center font-bold text-2xl">
        Boosty Downloader
      </h1>
    </header>
  );
};
