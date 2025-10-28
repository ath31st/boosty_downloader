import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import MainPage from '../pages/MainPage';
import ConfigPage from '../pages/ConfigPage';
import { Button } from '../components/Button';
import type { Page } from '../constants/pages';

export default function App() {
  const [clientReady, setClientReady] = useState(false);
  const [currentPage, setCurrentPage] = useState<Page>('main');

  useEffect(() => {
    const init = async () => {
      try {
        await invoke('init_client');
        console.log('Client initialized');
        setClientReady(true);
      } catch (err) {
        console.error('Failed to init client:', err);
      }
    };
    init();
  }, []);

  return (
    <main className="container mx-auto p-2">
      <h1 className="mb-4 text-center font-bold text-2xl">Boosty Downloader</h1>

      {!clientReady && (
        <p className="text-(--meta-text)">Инициализация клиента...</p>
      )}

      {clientReady && (
        <>
          <div className="mb-6 flex gap-4">
            <Button className="flex-1" onClick={() => setCurrentPage('main')}>
              Главная
            </Button>
            <Button className="flex-1" onClick={() => setCurrentPage('config')}>
              Настройки
            </Button>
          </div>

          <div className="w-full">
            {currentPage === 'main' && <MainPage />}
            {currentPage === 'config' && <ConfigPage />}
          </div>
        </>
      )}
    </main>
  );
}
