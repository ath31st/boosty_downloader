export const LOG_LEVELS = ['Info', 'Warn', 'Error'] as const;
export type LogLevel = (typeof LOG_LEVELS)[number];
