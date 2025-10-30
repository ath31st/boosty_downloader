import type { Page } from '@/constants/pages';
import { invoke } from '@tauri-apps/api/core';
import { useEffect, useState } from 'react';
import { toast } from 'sonner';

export function useInitApp() {
  const [clientReady, setClientReady] = useState(false);
  const [currentPage, setCurrentPage] = useState<Page>('main');
  const [isFailed, setFailed] = useState(false);

  useEffect(() => {
    const init = async () => {
      try {
        await invoke('init_client');
        console.log('Client initialized');
        setClientReady(true);
      } catch (err) {
        toast.error('Не удалось инициализировать клиент');
        console.error('Failed to init client:', err);
        setFailed(true);
      }
    };
    init();
  }, []);

  const handleReload = () => {
    window.location.reload();
  };

  return {
    currentPage,
    clientReady,
    setCurrentPage,
    initFailed: isFailed,
    handleReload,
  };
}
