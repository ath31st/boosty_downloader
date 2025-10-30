import { revealItemInDir } from '@tauri-apps/plugin-opener';

import { Button } from '@/components/Button';
import { invoke } from '@tauri-apps/api/core';
import { resolve } from '@tauri-apps/api/path';

export function OpenFolderButton() {
  const handleOpenFolder = async () => {
    const exePathString = (await invoke('get_exe_path', {})) as string;
    const exePath = await resolve(exePathString);
    await revealItemInDir(exePath);
  };

  return <Button onClick={handleOpenFolder}>Открыть папку</Button>;
}
