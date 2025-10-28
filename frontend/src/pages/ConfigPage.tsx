import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig } from '../types/config';
import { Button } from '../components/Button';
import { Input } from '../components/Input';
import { Label } from '../components/Label';
import { ConfigLabel } from '../components/ConfigLabel';

export default function ConfigPage() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    const fetchConfig = async () => {
      try {
        const cfg = await invoke<AppConfig>('get_config');
        setConfig(cfg);
      } catch (err) {
        console.error('Failed to fetch config:', err);
      }
    };
    fetchConfig();
  }, []);

  const handleChange = (key: keyof AppConfig, value: unknown) => {
    if (!config) return;
    setConfig({ ...config, [key]: value });
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      await invoke('update_config', { newConfig: config });
      console.log('Config updated');
    } catch (err) {
      console.error('Failed to update config:', err);
    } finally {
      setSaving(false);
    }
  };

  if (!config) {
    return (
      <div className="rounded-lg bg-(--background) p-4">
        <p>Загрузка конфигурации...</p>
      </div>
    );
  }

  return (
    <div className="space-y-6 rounded-lg border border-(--border) bg-(--background) p-4">
      <div className="space-y-4">
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

      <Button onClick={handleSave} disabled={saving}>
        {saving ? 'Сохраняем...' : 'Сохранить'}
      </Button>
    </div>
  );
}
