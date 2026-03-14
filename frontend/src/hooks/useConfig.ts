import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { AppConfig } from '@/types/config';
import { toast } from 'sonner';

export function useConfig() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [isLoading, setLoading] = useState(true);
  const [isSaving, setSaving] = useState(false);
  const [downloadPath, setDownloadPath] = useState<string | null>(null);

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

  useEffect(() => {
    if (!isLoading && config) {
      invoke<string>('get_download_path')
        .then((path) => setDownloadPath(path))
        .catch((err) => {
          console.error('Failed to get download path:', err);
          toast.error('Не удалось получить путь сохранения');
          setDownloadPath('ERROR');
        });
    }
  }, [isLoading, config]);

  const handleChange = (key: keyof AppConfig, value: unknown) => {
    if (!config) return;
    setConfig({ ...config, [key]: value });
  };

  const handleSelectDirectory = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: config?.download_path || undefined,
      });

      if (selected && typeof selected === 'string') {
        handleChange('download_path', selected);
      }
    } catch (err) {
      console.error('Failed to open directory dialog:', err);
      toast.error('Не удалось открыть диалог выбора папки');
    }
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      const finalConfig = {
        ...config,
        download_path: config.download_path?.trim() || null
      };

      await invoke('update_config', { newConfig: finalConfig });
      setDownloadPath(finalConfig.download_path);
      console.log('Config updated');
      toast.success('Настройки сохранены');
    } catch (err) {
      toast.error('Не удалось сохранить настройки');
      console.error('Failed to update config:', err);
    } finally {
      setSaving(false);
    }
  };

  return {
    config,
    setConfig,
    handleChange,
    handleSave,
    isLoading,
    isSaving,
    downloadPath,
    handleSelectDirectory
  };
}
