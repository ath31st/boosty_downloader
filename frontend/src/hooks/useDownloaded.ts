import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'sonner';
import type { BlogSnapshot, DownloadPostsResult } from '@/types/downloaded';
import type { DownloadSession } from '@/hooks/useDownloadingContent';
import type { Page } from '@/constants/pages';
import { useDownloadedStore } from '@/store/downloaded';
import { confirmAction } from '@/utils/confirmAction';

export function useDownloaded(
  session: DownloadSession,
  setCurrentPage: (page: Page) => void,
  active: boolean,
) {
  const { setDownloading, downloadOptions, resetDownloadUi, resetProgress } =
    session;
  const blogs = useDownloadedStore((state) => state.blogs);
  const loading = useDownloadedStore((state) => state.loading);
  const initialized = useDownloadedStore((state) => state.initialized);
  const mergeScan = useDownloadedStore((state) => state.mergeScan);
  const replaceBlog = useDownloadedStore((state) => state.replaceBlog);
  const removePostFromStore = useDownloadedStore((state) => state.removePost);
  const removeBlogFromStore = useDownloadedStore((state) => state.removeBlog);
  const setLoading = useDownloadedStore((state) => state.setLoading);
  const setInitialized = useDownloadedStore((state) => state.setInitialized);
  const [checking, setChecking] = useState(false);

  const load = useCallback(async () => {
    if (!useDownloadedStore.getState().initialized) setLoading(true);
    try {
      const list = (await invoke('list_downloaded')) as BlogSnapshot[];
      mergeScan(list);
    } catch (e) {
      console.error(e);
      toast.error('Не удалось прочитать скачанное');
    } finally {
      setLoading(false);
      setInitialized(true);
    }
  }, [mergeScan, setInitialized, setLoading]);

  useEffect(() => {
    if (!active) return;
    void load();
  }, [active, load]);

  const fetchRefresh = async (blog: string) => {
    const updated = (await invoke('refresh_downloaded_blog', {
      blog,
    })) as BlogSnapshot;
    replaceBlog(updated);
    return updated;
  };

  const toastDownloadResult = (
    result: DownloadPostsResult,
    okMessage: string,
  ) => {
    if (result.downloaded === 0 && result.skipped > 0) {
      toast.info(
        result.skipped === 1
          ? 'Пост пропущен: нет доступа'
          : `Пропущено постов: ${result.skipped} (нет доступа)`,
      );
      return;
    }
    if (result.skipped > 0) {
      toast.success(`${okMessage}. Пропущено: ${result.skipped}`);
      return;
    }
    toast.success(okMessage);
  };

  const refreshBlog = async (blog: string) => {
    setChecking(true);
    setDownloading(true);
    try {
      await fetchRefresh(blog);
      toast.success('Проверка завершена');
    } catch (e) {
      console.error(e);
      if (String(e) === 'Download cancelled by user') {
        toast.info('Проверка отменена');
      } else {
        toast.error('Не удалось проверить блог');
      }
    } finally {
      setChecking(false);
      setDownloading(false);
    }
  };

  const runDownload = async (
    blog: string,
    postIds: string[],
    force: boolean,
    okMessage: string,
  ) => {
    if (postIds.length === 0) {
      toast.info('Нет постов для загрузки');
      return;
    }
    if (downloadOptions.length === 0) {
      toast.error('Выберите типы контента');
      return;
    }

    resetDownloadUi();
    setDownloading(true);
    setCurrentPage('main');
    try {
      const result = (await invoke('download_downloaded_posts', {
        blog,
        postIds,
        downloadOptions,
        force,
      })) as DownloadPostsResult;
      toastDownloadResult(result, okMessage);
      try {
        await fetchRefresh(blog);
      } catch (refreshErr) {
        console.error(refreshErr);
        await load();
      }
    } catch (e) {
      console.error(e);
      if (String(e) === 'Download cancelled by user') {
        toast.info('Загрузка отменена');
      } else {
        toast.error('Не удалось загрузить');
      }
    } finally {
      setDownloading(false);
      resetProgress();
    }
  };

  const downloadNew = async (blog: string) => {
    const current = useDownloadedStore
      .getState()
      .blogs.find((item) => item.blog === blog);
    const ids =
      current?.posts.filter((p) => p.status === 'new').map((p) => p.post_id) ??
      [];
    await runDownload(blog, ids, false, 'Новые посты скачаны');
  };

  const downloadPost = async (blog: string, postId: string, force: boolean) => {
    const post = useDownloadedStore
      .getState()
      .blogs.find((item) => item.blog === blog)
      ?.posts.find((p) => p.post_id === postId);
    if (force) {
      const ok = await confirmAction(
        `Перекачать пост «${post?.title ?? postId}»? Текущие файлы будут удалены.`,
        'Перекачать пост',
      );
      if (!ok) return;
    }
    const okMessage = force
      ? 'Пост перекачан'
      : post?.status === 'new'
        ? 'Пост скачан'
        : 'Пост докачан';
    return runDownload(blog, [postId], force, okMessage);
  };

  const deletePost = async (blog: string, postId: string, title: string) => {
    const ok = await confirmAction(`Удалить пост «${title}»?`, 'Удалить пост');
    if (!ok) return;
    try {
      await invoke('delete_downloaded_post', { blog, postId });
      removePostFromStore(blog, postId);
      toast.success('Пост удалён');
    } catch (e) {
      console.error(e);
      toast.error('Не удалось удалить пост');
    }
  };

  const deleteBlog = async (blog: string) => {
    const ok = await confirmAction(
      `Удалить блог «${blog}» и все его посты?`,
      'Удалить блог',
    );
    if (!ok) return;
    try {
      await invoke('delete_downloaded_blog', { blog });
      removeBlogFromStore(blog);
      toast.success('Блог удалён');
    } catch (e) {
      console.error(e);
      toast.error('Не удалось удалить блог');
    }
  };

  return {
    blogs,
    loading: loading && !initialized,
    checking,
    refreshBlog,
    downloadNew,
    downloadPost,
    deletePost,
    deleteBlog,
  };
}
