import { openPath } from '@tauri-apps/plugin-opener';
import { Button } from '@/components/Button';
import { invoke } from '@tauri-apps/api/core';
import { FolderOpen } from 'lucide-react';

export function OpenFolderButton() {
  const handleOpenFolder = async () => {
    const downloadPath = (await invoke('get_download_path', {})) as string;
    await openPath(downloadPath);
  };

  return (
    <Button onClick={handleOpenFolder}>
      <FolderOpen />
    </Button>
  );
}
