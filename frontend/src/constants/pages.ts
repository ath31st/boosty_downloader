export const PAGES = ['main', 'config'] as const;
export type Page = (typeof PAGES)[number];
