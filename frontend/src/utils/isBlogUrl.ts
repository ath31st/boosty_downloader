import { BASE_URL } from '@/constants/url';

export function isBlogUrl(url: string): boolean {
  try {
    const parsed = new URL(url);

    if (parsed.origin !== BASE_URL) return false;

    const segments = parsed.pathname.split('/').filter(Boolean);

    return segments.length === 1;
  } catch {
    return false;
  }
}
