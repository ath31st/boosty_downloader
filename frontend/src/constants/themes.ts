export const THEMES = ['dark', 'light'] as const;
export type Theme = (typeof THEMES)[number];
