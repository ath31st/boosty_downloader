export interface CommentsConfig {
  enabled: boolean;
  reply_limit?: number;
  limit?: number;
  order?: string;
}

export interface AppConfig {
  posts_limit: number;
  access_token: string;
  refresh_token: string;
  device_id: string;
  comments: CommentsConfig;
  download_path: string | null;
}
