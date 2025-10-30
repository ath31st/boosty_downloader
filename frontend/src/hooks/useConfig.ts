import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig } from '@/types/config';

export function useConfig() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [isLoading, setLoading] = useState(true);
  const [isSaving, setSaving] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const cfg = await invoke<AppConfig>('get_config');
        setConfig(cfg);
      } catch (err) {
        console.error('Failed to fetch config:', err);
      } finally {
        setLoading(false);
      }
    })();
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

  return { config, setConfig, handleChange, handleSave, isLoading, isSaving };
}
