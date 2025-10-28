import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { Label } from '../components/Label';
import { ConfigLabel } from '../components/ConfigLabel';
import { useConfig } from '@/hooks/useConfig';

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
          <ConfigLabel label="Posts limit:" />
          <Input
            type="number"
            value={config.posts_limit}
            onChange={(e) => handleChange('posts_limit', e)}
            className="ml-2 flex-1"
          />
        </Label>

        <Label>
          <ConfigLabel label="Access token:" />
          <Input
            type="text"
            value={config.access_token}
            onChange={(e) => handleChange('access_token', e)}
            className="ml-2 flex-1"
          />
        </Label>

        <Label>
          <ConfigLabel label="Refresh token:" />
          <Input
            type="text"
            value={config.refresh_token}
            onChange={(e) => handleChange('refresh_token', e)}
            className="ml-2 flex-1"
          />
        </Label>

        <Label>
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
