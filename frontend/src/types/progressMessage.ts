export interface ProgressMessage {
  files_done: number;
  files_total: number;
  file_name: string | null;
  current: number;
  total: number;
}
