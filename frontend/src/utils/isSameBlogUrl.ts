import { BASE_URL } from '@/constants/url';

export function isSameBlogUrl(url1: string, url2: string): boolean {
  try {
    const u1 = new URL(url1);
    const u2 = new URL(url2);

    if (u1.origin !== BASE_URL || u2.origin !== BASE_URL) {
      return false;
    }

    const blog1 = u1.pathname.split('/').filter(Boolean)[0];
    const blog2 = u2.pathname.split('/').filter(Boolean)[0];

    return blog1 !== undefined && blog1 === blog2;
  } catch {
    return false;
  }
}
