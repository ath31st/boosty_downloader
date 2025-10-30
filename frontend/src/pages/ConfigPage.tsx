import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { Label } from '../components/Label';
import { ConfigLabel } from '../components/ConfigLabel';
import { useConfig } from '@/hooks/useConfig';
import { HintIcon } from '@/components/HintIcon';

export default function ConfigPage() {
  const { config, handleChange, handleSave, isLoading, isSaving } = useConfig();

  if (isLoading || !config) {
    return (
      <div className="rounded-lg bg-(--background) p-4">
        <p>Загрузка конфигурации...</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center gap-6 rounded-lg border border-(--border) bg-(--background) p-4">
      <div className="flex w-full flex-col gap-4">
        <Label>
          <HintIcon text="Максимальное количество постов для загрузки за одну сессию" />
          <ConfigLabel label="Лимит постов:" />
          <Input
            type="number"
            value={config.posts_limit}
            onChange={(e) => handleChange('posts_limit', e)}
            className="ml-2 flex-1"
          />
        </Label>

        <Label>
          <HintIcon
            text={
              <>
                Токен доступа, хранится в браузере.
                <br />
                &nbsp;
                <a
                  href="https://github.com/ath31st/boosty_downloader?tab=readme-ov-file#где-взять-токены"
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-(--button-text) underline hover:text-(--button-hover-bg)"
                >
                  Инструкция, как достать токен на GitHub
                </a>
              </>
            }
          />
          <ConfigLabel label="Access token:" />
          <Input
            type="text"
            value={config.access_token}
            onChange={(e) => handleChange('access_token', e)}
            className="ml-2 flex-1"
          />
        </Label>

        <Label>
          <HintIcon text="Токен для обновления токена доступа. Во время обновления токена доступа, обновляется и токен обновления" />
          <ConfigLabel label="Refresh token:" />
          <Input
            type="text"
            value={config.refresh_token}
            onChange={(e) => handleChange('refresh_token', e)}
            className="ml-2 flex-1"
          />
        </Label>

        <Label>
          <HintIcon text="Идентификатор клиента (вашего браузера). Обязательный компонент для использования токена обновления" />
          <ConfigLabel label="Device ID:" />
          <Input
            type="text"
            value={config.device_id}
            onChange={(e) => handleChange('device_id', e)}
            className="ml-2 flex-1"
          />
        </Label>
      </div>

      <Button className="w-50" onClick={handleSave} disabled={isSaving}>
        {isSaving ? 'Сохраняем...' : 'Сохранить'}
      </Button>
    </div>
  );
}
