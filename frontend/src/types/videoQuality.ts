export type VideoQuality = 'ultra_hd' | 'full_hd' | 'high' | 'medium' | 'low';

export const VIDEO_QUALITY_OPTIONS: { value: VideoQuality; label: string }[] = [
  { value: 'ultra_hd', label: 'Ultra HD' },
  { value: 'full_hd', label: 'Full HD' },
  { value: 'high', label: 'Высокое' },
  { value: 'medium', label: 'Среднее' },
  { value: 'low', label: 'Низкое' },
];
