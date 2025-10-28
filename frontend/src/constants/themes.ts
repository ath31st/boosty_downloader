export const THEMES = ['blue', 'dark'] as const;
export type Theme = (typeof THEMES)[number];
