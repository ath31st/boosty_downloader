import MainPage from '../pages/MainPage';
import ConfigPage from '../pages/ConfigPage';
import DownloadedPage from '../pages/DownloadedPage';
import { useInitApp } from '@/hooks/useInitApp';
import { useDownloadingContent } from '@/hooks/useDownloadingContent';
import { Header } from '@/components/Header';
import { Button } from '@/components/Button';

export default function App() {
  const { currentPage, clientReady, setCurrentPage, initFailed, handleReload } =
    useInitApp();
  const session = useDownloadingContent();
  const { isDownloading } = session;

  return (
    <main className="flex h-full min-h-0 w-full flex-col overflow-hidden p-4">
      <Header
        currentPage={currentPage}
        setCurrentPage={setCurrentPage}
        isDownloading={isDownloading}
      />

      {!clientReady && (
        <p className="text-(--meta-text)">Инициализация клиента...</p>
      )}

      {initFailed && (
        <div className="mt-40 flex flex-col items-center gap-4">
          <p className="text-(--meta-text)">
            Не удалось инициализировать клиент
          </p>
          <Button onClick={handleReload}>Перезагрузить приложение</Button>
        </div>
      )}

      {clientReady && !initFailed && (
        <>
          <div
            className={
              currentPage === 'main' ? 'flex min-h-0 flex-1 flex-col' : 'hidden'
            }
          >
            <MainPage session={session} />
          </div>
          <div
            className={
              currentPage === 'downloaded'
                ? 'flex min-h-0 flex-1 flex-col'
                : 'hidden'
            }
          >
            <DownloadedPage
              session={session}
              setCurrentPage={setCurrentPage}
              active={currentPage === 'downloaded'}
            />
          </div>
          {currentPage === 'config' && (
            <div className="flex min-h-0 flex-1 flex-col">
              <ConfigPage />
            </div>
          )}
        </>
      )}
    </main>
  );
}
