import { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'sonner';
import { useUrlValidation } from '@/hooks/useUrlValidation';
import { isBlogUrl } from '@/utils/isBlogUrl';
import { isSameBlogUrl } from '@/utils/isSameBlogUrl';
import type { DownloadSession } from '@/hooks/useDownloadingContent';

export function useDownloadProcess(session: DownloadSession) {
  const {
    setDownloading,
    downloadOptions,
    resetDownloadUi,
    resetProgress,
    cancelDownload,
  } = session;

  const [url, setUrl] = useState(() => {
    if (typeof window === 'undefined') return '';
    return sessionStorage.getItem('url') ?? '';
  });

  const [offsetUrl, setOffsetUrl] = useState(() => {
    if (typeof window === 'undefined') return '';
    return sessionStorage.getItem('offsetUrl') ?? '';
  });

  const { urlError, validateUrl } = useUrlValidation();

  useEffect(() => {
    if (url) sessionStorage.setItem('url', url);
    else sessionStorage.removeItem('url');
  }, [url]);

  useEffect(() => {
    if (offsetUrl) sessionStorage.setItem('offsetUrl', offsetUrl);
    else sessionStorage.removeItem('offsetUrl');
  }, [offsetUrl]);

  const startDownload = async () => {
    if (!url) return;
    if (!validateUrl(url)) {
      toast.error('Введите корректный URL');
      return;
    }

    resetDownloadUi();
    setDownloading(true);

    try {
      await invoke('download_content', {
        url: url.trim(),
        offsetUrl: offsetUrl.trim() !== '' ? offsetUrl : undefined,
        downloadOptions,
      });
      toast.success('Загрузка завершена');
    } catch (e) {
      console.error(e);
      if (String(e) === 'Download cancelled by user') {
        toast.info('Загрузка отменена');
      } else {
        toast.error('Не удалось произвести загрузку');
      }
    } finally {
      setDownloading(false);
      resetProgress();
    }
  };

  const isOffsetUrlDisabled = useMemo(() => {
    if (!url) return true;
    if (!isBlogUrl(url)) return true;
    return false;
  }, [url]);

  const isDifferentBlogs = useMemo(() => {
    if (!url || !offsetUrl) return false;
    return !isSameBlogUrl(url, offsetUrl);
  }, [url, offsetUrl]);

  useEffect(() => {
    if (isDifferentBlogs) toast.error('Введены разные блоги');
  }, [isDifferentBlogs]);

  return {
    url,
    offsetUrl,
    setUrl,
    setOffsetUrl,
    urlError,
    startDownload,
    cancelDownload,
    isOffsetUrlDisabled,
    isDifferentBlogs,
  };
}
