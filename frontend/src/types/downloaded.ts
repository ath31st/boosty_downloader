import type { DownloadOption } from './downloadOptions';

export type PostSyncStatus =
  | 'up_to_date'
  | 'new'
  | 'updated'
  | 'partial'
  | 'unavailable'
  | 'gone'
  | 'unchecked';

export interface PostSnapshot {
  post_id: string;
  title: string;
  folder: string;
  folder_path: string;
  created_at: number;
  updated_at: number;
  downloaded_options: DownloadOption[];
  status: PostSyncStatus;
  is_paid: boolean;
}

export interface BlogSnapshot {
  blog: string;
  last_checked_at: number | null;
  posts: PostSnapshot[];
}

export interface DownloadPostsResult {
  downloaded: number;
  skipped: number;
}

export const STATUS_LABEL: Record<PostSyncStatus, string> = {
  up_to_date: 'актуально',
  new: 'новый',
  updated: 'есть изменения',
  partial: 'можно докачать',
  unavailable: 'недоступен',
  gone: 'больше нет',
  unchecked: 'не проверен',
};
