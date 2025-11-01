import { useState } from 'react';

export function useUrlValidation() {
  const [urlError, setUrlError] = useState<string | null>(null);

  const validateUrl = (url: string): boolean => {
    if (!url) {
      setUrlError(null);
      return true;
    }

    try {
      const parsed = new URL(url);
      if (parsed.origin !== 'https://boosty.to') {
        setUrlError('URL должен начинаться с https://boosty.to/');
        return false;
      }
      setUrlError(null);
      return true;
    } catch {
      setUrlError('Некорректный URL');
      return false;
    }
  };

  return { urlError, validateUrl };
}
