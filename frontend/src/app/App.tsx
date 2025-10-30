import MainPage from '../pages/MainPage';
import ConfigPage from '../pages/ConfigPage';
import { useInitApp } from '@/hooks/useInitApp';
import { useDownloadingContent } from '@/hooks/useDownloadingContent';
import { Header } from '@/components/Header';
import { PageTabs } from '@/components/PageTabs';
import { Button } from '@/components/Button';

export default function App() {
  const { currentPage, clientReady, setCurrentPage, initFailed, handleReload } =
    useInitApp();
  const { isDownloading, setDownloading } = useDownloadingContent();

  return (
    <main className="container mx-auto p-2">
      <Header />

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
          <PageTabs
            setCurrentPage={setCurrentPage}
            isDownloading={isDownloading}
          />

          <div className="w-full">
            {currentPage === 'main' && (
              <MainPage
                setDownloading={setDownloading}
                isDownloading={isDownloading}
              />
            )}
            {currentPage === 'config' && <ConfigPage />}
          </div>
        </>
      )}
    </main>
  );
}
