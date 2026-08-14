export const PAGES = ['main', 'downloaded', 'config'] as const;
export type Page = (typeof PAGES)[number];
