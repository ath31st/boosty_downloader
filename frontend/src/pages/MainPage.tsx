import { Button } from '../components/Button';
import { DownloadProgress } from '@/components/DownloadProgress';
import { formatLog } from '@/components/FormatLog';
import { OpenFolderButton } from '@/components/OpenFolderButton';
import { DownloadIcon, Square } from 'lucide-react';
import { Input } from '@/components/Input';
import { useDownloadProcess } from '@/hooks/useDownloadProcess';
import { HintIcon } from '@/components/HintIcon';
import { DownloadOptionsPanel } from '@/components/DownloadOptionsPanel';
import type { DownloadSession } from '@/hooks/useDownloadingContent';

interface MainPageProps {
  session: DownloadSession;
}

export default function MainPage({ session }: MainPageProps) {
  const {
    isDownloading,
    downloadOptions,
    setDownloadOptions,
    logs,
    progress,
    logsEndRef,
  } = session;
  const {
    url,
    offsetUrl,
    setUrl,
    setOffsetUrl,
    startDownload,
    cancelDownload,
    isOffsetUrlDisabled,
    isDifferentBlogs,
    urlError,
  } = useDownloadProcess(session);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 rounded-lg border border-(--border) bg-(--background) p-4 text-(--text)">
      <div className="flex shrink-0 flex-col gap-2">
        <DownloadOptionsPanel
          value={downloadOptions}
          onChange={setDownloadOptions}
          disabled={isDownloading}
        />

        <div className="flex flex-row gap-4">
          <Input
            placeholder="URL адрес блога, поста или бандла"
            value={url}
            onChange={(value) => setUrl(String(value))}
            disabled={isDownloading}
            className="flex-1"
          />
          {isDownloading ? (
            <Button onClick={cancelDownload} aria-label="Stop">
              <Square className="fill-current" />
            </Button>
          ) : (
            <Button
              onClick={startDownload}
              disabled={
                !url || isDifferentBlogs || downloadOptions.length === 0
              }
            >
              <DownloadIcon />
            </Button>
          )}
        </div>
        {urlError && <p className="text-(--error) text-sm">{urlError}</p>}
        <div className="relative flex flex-row gap-4">
          <Input
            placeholder="URL адрес поста для отступа"
            value={offsetUrl}
            onChange={(value) => setOffsetUrl(String(value))}
            disabled={isDownloading || isOffsetUrlDisabled}
            className="flex-1"
          />

          <div className="absolute top-3 right-20">
            <HintIcon
              size={20}
              text={
                <div className="whitespace-pre-wrap">
                  ⚠️ Поле ввода разблокируется, если введен адрес блога, а не
                  поста.
                  {'\n'}В качестве отступа указывается ссылка на пост, ПОСЛЕ
                  которого "вниз" по ленте будут загружаться посты в количестве,
                  которое указано в настройках.
                </div>
              }
            />
          </div>

          <OpenFolderButton />
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto rounded-lg border border-(--border) bg-(--secondary-bg) p-2">
        {logs.map((msg, index) => (
          <p
            // biome-ignore lint/suspicious/noArrayIndexKey: normal index
            key={`${index}-${msg.level}-${msg.message}`}
            className="text-(--meta-text) text-sm"
          >
            {formatLog(msg)}
          </p>
        ))}
        <div ref={logsEndRef} />
      </div>

      <div className="shrink-0">
        <DownloadProgress
          filesDone={progress.files_done}
          filesTotal={progress.files_total}
          fileName={progress.file_name}
          current={progress.current}
          total={progress.total}
          isDownloading={isDownloading}
        />
      </div>
    </div>
  );
}
