import { ask } from '@tauri-apps/plugin-dialog';

export function confirmAction(
  message: string,
  title = 'Подтверждение',
): Promise<boolean> {
  return ask(message, { title, kind: 'warning' });
}
