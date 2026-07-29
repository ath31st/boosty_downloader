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

  const refreshDownloadPath = async () => {
    try {
      const path = await invoke<string>('get_download_path');
      setDownloadPath(path);
    } catch (err) {
      console.error('Failed to get download path:', err);
      toast.error('Не удалось получить путь сохранения');
      setDownloadPath('ERROR');
    }
  };

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

  // biome-ignore lint/correctness/useExhaustiveDependencies: crying linter with red text
  useEffect(() => {
    if (!isLoading && config) {
      void refreshDownloadPath();
    }
  }, [isLoading, config?.download_path]);

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
        download_path: config.download_path?.trim() || null,
      };

      await invoke('update_config', { newConfig: finalConfig });
      const updated = await invoke<AppConfig>('get_config');
      setConfig(updated);
      await refreshDownloadPath();
      toast.success('Настройки сохранены');
    } catch (err) {
      toast.error('Не удалось сохранить настройки');
      console.error('Failed to update config:', err);
    } finally {
      setSaving(false);
    }
  };

  const handleClearAuth = async () => {
    if (!config) return;
    setSaving(true);
    try {
      const cleared: AppConfig = {
        ...config,
        access_token: '',
        refresh_token: '',
        device_id: '',
      };
      await invoke('update_config', { newConfig: cleared });
      const updated = await invoke<AppConfig>('get_config');
      setConfig(updated);
      toast.success('Токены очищены');
    } catch (err) {
      toast.error('Не удалось очистить токены');
      console.error('Failed to clear auth:', err);
    } finally {
      setSaving(false);
    }
  };

  return {
    config,
    setConfig,
    handleChange,
    handleSave,
    handleClearAuth,
    isLoading,
    isSaving,
    downloadPath,
    handleSelectDirectory,
  };
}
