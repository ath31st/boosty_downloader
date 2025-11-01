import type { AppConfig } from '@/types/config';
import { useState } from 'react';

export function useConfigValidation() {
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = (config: AppConfig) => {
    if (!config) return false;

    const newErrors: Record<string, string> = {};

    const posts = Number(config.posts_limit);
    if (Number.isNaN(posts) || posts < 1 || posts > 5000) {
      newErrors.posts_limit = 'Лимит постов должен быть от 1 до 5000';
    }

    const checkToken = (key: string, label: string) => {
      const value = (config as unknown as Record<string, string>)[key]?.trim();
      if (!value) return;
      if (value.length < 10) newErrors[key] = `${label} слишком короткий`;
      else if (value.length > 255) newErrors[key] = `${label} слишком длинный`;
    };

    checkToken('access_token', 'Access token');
    checkToken('refresh_token', 'Refresh token');
    checkToken('device_id', 'Device ID');

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  return { errors, validate };
}
