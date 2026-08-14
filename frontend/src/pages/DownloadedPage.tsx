import { Button } from '@/components/Button';
import { DownloadOptionsPanel } from '@/components/DownloadOptionsPanel';
import { useDownloaded } from '@/hooks/useDownloaded';
import type { DownloadSession } from '@/hooks/useDownloadingContent';
import type { Page } from '@/constants/pages';
import { useDownloadedStore } from '@/store/downloaded';
import { STATUS_LABEL, type PostSyncStatus } from '@/types/downloaded';
import { openPath } from '@tauri-apps/plugin-opener';
import {
  Banknote,
  DownloadIcon,
  FolderOpen,
  RefreshCw,
  Square,
  Trash2,
} from 'lucide-react';

interface DownloadedPageProps {
  session: DownloadSession;
  setCurrentPage: (page: Page) => void;
  active: boolean;
}

function statusClass(status: PostSyncStatus): string {
  switch (status) {
    case 'up_to_date':
      return 'text-(--success)';
    case 'new':
    case 'updated':
    case 'partial':
      return 'text-(--warning)';
    case 'unavailable':
    case 'gone':
      return 'text-(--error)';
    default:
      return 'text-(--meta-text)';
  }
}

export default function DownloadedPage({
  session,
  setCurrentPage,
  active,
}: DownloadedPageProps) {
  const { isDownloading, downloadOptions, setDownloadOptions, cancelDownload } =
    session;
  const expandedBlogs = useDownloadedStore((state) => state.expandedBlogs);
  const setBlogExpanded = useDownloadedStore((state) => state.setBlogExpanded);
  const {
    blogs,
    loading,
    checking,
    refreshBlog,
    downloadNew,
    downloadPost,
    deletePost,
    deleteBlog,
  } = useDownloaded(session, setCurrentPage, active);

  if (loading) {
    return (
      <div className="rounded-lg border border-(--border) bg-(--background) p-4 text-(--meta-text)">
        Загрузка списка...
      </div>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 rounded-lg border border-(--border) bg-(--background) p-4 text-(--text)">
      <div className="shrink-0">
        <DownloadOptionsPanel
          value={downloadOptions}
          onChange={setDownloadOptions}
          disabled={isDownloading}
        />
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto">
        {blogs.length === 0 ? (
          <div className="flex min-h-0 flex-1 items-center justify-center">
            <p className="max-w-xl text-center text-(--meta-text) text-xl">
              Пока ничего не скачано. После загрузки блога или поста он появится
              здесь.
            </p>
          </div>
        ) : (
          blogs.map((blog) => (
            <details
              key={blog.blog}
              className="rounded-lg border border-(--border) bg-(--secondary-bg) p-3"
              open={expandedBlogs.includes(blog.blog)}
              onToggle={(event) => {
                const open = event.currentTarget.open;
                if (open !== expandedBlogs.includes(blog.blog)) {
                  setBlogExpanded(blog.blog, open);
                }
              }}
            >
              <summary className="cursor-pointer font-medium">
                {blog.blog}
              </summary>
              <div className="mt-3 flex flex-wrap gap-2">
                <Button
                  disabled={isDownloading}
                  onClick={() => refreshBlog(blog.blog)}
                >
                  <div className="flex items-center gap-2">
                    <RefreshCw size={16} />
                    Проверить
                  </div>
                </Button>
                {blog.posts.some((p) => p.status === 'new') && (
                  <Button
                    disabled={isDownloading || downloadOptions.length === 0}
                    onClick={() => downloadNew(blog.blog)}
                  >
                    <div className="flex items-center gap-2">
                      <DownloadIcon size={16} />
                      Скачать новое
                    </div>
                  </Button>
                )}
                <Button
                  disabled={isDownloading}
                  onClick={() => deleteBlog(blog.blog)}
                >
                  <div className="flex items-center gap-2">
                    <Trash2 size={16} />
                    Удалить блог
                  </div>
                </Button>
              </div>

              <ul className="mt-3 flex flex-col gap-2">
                {blog.posts.length === 0 && (
                  <li className="text-(--meta-text) text-sm">Нет постов</li>
                )}
                {blog.posts.map((post) => (
                  <li
                    key={post.post_id}
                    className="flex flex-col gap-2 rounded-md border border-(--border) p-2 sm:flex-row sm:items-center sm:justify-between"
                  >
                    <div className="min-w-0">
                      <p className="flex items-center gap-1.5 truncate">
                        <span className="truncate">{post.title}</span>
                        {post.is_paid && (
                          <span title="Платный пост" className="shrink-0">
                            <Banknote
                              size={16}
                              className="text-(--warning)"
                              aria-hidden
                            />
                            <span className="sr-only">Платный пост</span>
                          </span>
                        )}
                      </p>
                      <p className={`text-sm ${statusClass(post.status)}`}>
                        {STATUS_LABEL[post.status]}
                      </p>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {post.status === 'partial' && (
                        <Button
                          className="px-2 py-1 text-sm"
                          disabled={
                            isDownloading || downloadOptions.length === 0
                          }
                          onClick={() =>
                            downloadPost(blog.blog, post.post_id, false)
                          }
                        >
                          Докачать
                        </Button>
                      )}
                      {post.folder_path && (
                        <Button
                          className="px-2 py-1 text-sm"
                          disabled={isDownloading}
                          onClick={() =>
                            downloadPost(blog.blog, post.post_id, true)
                          }
                        >
                          Перекачать
                        </Button>
                      )}
                      {post.folder_path && (
                        <Button
                          className="px-2"
                          disabled={isDownloading}
                          onClick={() => openPath(post.folder_path)}
                        >
                          <FolderOpen size={16} />
                        </Button>
                      )}
                      {post.folder_path && (
                        <Button
                          className="px-2"
                          disabled={isDownloading}
                          onClick={() =>
                            deletePost(blog.blog, post.post_id, post.title)
                          }
                        >
                          <Trash2 size={16} />
                        </Button>
                      )}
                      {post.status === 'new' && (
                        <Button
                          className="px-2 py-1 text-sm"
                          disabled={
                            isDownloading || downloadOptions.length === 0
                          }
                          onClick={() =>
                            downloadPost(blog.blog, post.post_id, false)
                          }
                        >
                          Скачать
                        </Button>
                      )}
                    </div>
                  </li>
                ))}
              </ul>
            </details>
          ))
        )}
      </div>

      {checking && (
        <div className="flex shrink-0 justify-center">
          <Button className="w-60" onClick={cancelDownload} aria-label="Stop">
            <div className="flex items-center justify-center gap-2">
              <Square className="fill-current" />
              Остановить проверку
            </div>
          </Button>
        </div>
      )}
    </div>
  );
}
