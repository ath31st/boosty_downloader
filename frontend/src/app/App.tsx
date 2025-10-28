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
      <h1 className="mb-4 font-bold text-2xl">Welcome to Tauri + React</h1>

      {!clientReady && (
        <p className="text-(--meta-text)">Initializing client...</p>
      )}

      {clientReady && (
        <>
          <div className="mb-4 flex space-x-2">
            <Button onClick={() => setCurrentPage('main')}>Main</Button>
            <Button onClick={() => setCurrentPage('config')}>Config</Button>
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
