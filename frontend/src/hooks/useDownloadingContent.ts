import { useState } from 'react';

export function useDownloadingContent() {
  const [isDownloading, setDownloading] = useState(false);

  return { isDownloading, setDownloading };
}
