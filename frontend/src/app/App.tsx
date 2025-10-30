import MainPage from '../pages/MainPage';
import ConfigPage from '../pages/ConfigPage';
import { Button } from '../components/Button';
import { useInitApp } from '@/hooks/useInitApp';
import { useDownloadingContent } from '@/hooks/useDownloadingContent';

export default function App() {
  const { currentPage, clientReady, setCurrentPage } = useInitApp();
  const { isDownloading, setDownloading } = useDownloadingContent();

  return (
    <main className="container mx-auto p-2">
      <h1 className="mb-4 text-center font-bold text-2xl">Boosty Downloader</h1>

      {!clientReady && (
        <p className="text-(--meta-text)">Инициализация клиента...</p>
      )}

      {clientReady && (
        <>
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
              Настройки
            </Button>
          </div>

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
