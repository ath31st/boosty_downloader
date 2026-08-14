import { create } from 'zustand';
import type { BlogSnapshot, PostSnapshot } from '@/types/downloaded';

type DownloadedState = {
  blogs: BlogSnapshot[];
  loading: boolean;
  initialized: boolean;
  expandedBlogs: string[];
  setLoading: (loading: boolean) => void;
  setInitialized: (initialized: boolean) => void;
  mergeScan: (scanned: BlogSnapshot[]) => void;
  replaceBlog: (updated: BlogSnapshot) => void;
  removePost: (blog: string, postId: string) => void;
  removeBlog: (blog: string) => void;
  setBlogExpanded: (blog: string, open: boolean) => void;
};

export function mergeDownloadedBlogs(
  scanned: BlogSnapshot[],
  previous: BlogSnapshot[],
): BlogSnapshot[] {
  const prevByName = new Map(previous.map((blog) => [blog.blog, blog]));
  return scanned
    .map((blog) => mergeBlog(blog, prevByName.get(blog.blog)))
    .sort((a, b) => a.blog.localeCompare(b.blog));
}

function mergeBlog(scanned: BlogSnapshot, prev?: BlogSnapshot): BlogSnapshot {
  if (!prev) return scanned;

  const scannedIds = new Set(scanned.posts.map((post) => post.post_id));
  const prevById = new Map(prev.posts.map((post) => [post.post_id, post]));
  const locals = scanned.posts.map((post) =>
    mergeLocalPost(post, prevById.get(post.post_id)),
  );
  const remoteOnly = prev.posts.filter((post) => !scannedIds.has(post.post_id));
  const posts = [...locals, ...remoteOnly].sort(
    (a, b) => b.created_at - a.created_at,
  );

  return {
    blog: scanned.blog,
    last_checked_at: prev.last_checked_at ?? scanned.last_checked_at,
    posts,
  };
}

function mergeLocalPost(
  scanned: PostSnapshot,
  prev?: PostSnapshot,
): PostSnapshot {
  if (!prev || prev.status === 'unchecked') return scanned;
  if (prev.status === 'new' || prev.status === 'unavailable') {
    return { ...scanned, is_paid: prev.is_paid || scanned.is_paid };
  }
  return {
    ...prev,
    title: scanned.title || prev.title,
    folder: scanned.folder,
    folder_path: scanned.folder_path,
    downloaded_options: scanned.downloaded_options,
    is_paid: scanned.is_paid || prev.is_paid,
    created_at: scanned.created_at || prev.created_at,
    updated_at: scanned.updated_at || prev.updated_at,
  };
}

export const useDownloadedStore = create<DownloadedState>((set) => ({
  blogs: [],
  loading: true,
  initialized: false,
  expandedBlogs: [],
  setLoading: (loading) => set({ loading }),
  setInitialized: (initialized) => set({ initialized }),
  mergeScan: (scanned) =>
    set((state) => ({
      blogs: mergeDownloadedBlogs(scanned, state.blogs),
    })),
  replaceBlog: (updated) =>
    set((state) => {
      const rest = state.blogs.filter((blog) => blog.blog !== updated.blog);
      return {
        blogs: [...rest, updated].sort((a, b) => a.blog.localeCompare(b.blog)),
      };
    }),
  removePost: (blog, postId) =>
    set((state) => ({
      blogs: state.blogs
        .map((item) =>
          item.blog === blog
            ? {
                ...item,
                posts: item.posts.filter((post) => post.post_id !== postId),
              }
            : item,
        )
        .filter((item) => item.posts.length > 0),
    })),
  removeBlog: (blog) =>
    set((state) => ({
      blogs: state.blogs.filter((item) => item.blog !== blog),
      expandedBlogs: state.expandedBlogs.filter((name) => name !== blog),
    })),
  setBlogExpanded: (blog, open) =>
    set((state) => ({
      expandedBlogs: open
        ? [...new Set([...state.expandedBlogs, blog])]
        : state.expandedBlogs.filter((name) => name !== blog),
    })),
}));
