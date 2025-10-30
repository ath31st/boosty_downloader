import type { LogLevel } from './logLevel';

export interface LogMessage {
  level: LogLevel;
  message: string;
}
