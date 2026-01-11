import type { LogMessage } from '@/types/logMessage';

export function formatLog(msg: LogMessage) {
  let color = '';

  switch (msg.level) {
    case 'Error':
      color = 'text-red-500';
      break;
    case 'Warn':
      color = 'text-yellow-500';
      break;
    case 'Info':
      color = 'text-blue-500';
      break;
    default:
      color = 'text-blue-500';
      break;
  }

  return (
    <span>
      <span className={color}>[{msg.level}]</span> <span>{msg.message}</span>
    </span>
  );
}
