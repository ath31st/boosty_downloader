import type { Page } from '@/constants/pages';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';

export function useInitApp() {
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

  return { currentPage, clientReady, setCurrentPage };
}
