import { Moon, Sun } from 'lucide-react';
import { useTheme } from '@/providers/ThemeProvider';
import { Switch } from './Switch';

export function ThemeSwitch() {
  const { theme, setTheme } = useTheme();
  const isDark = theme === 'dark';

  return (
    <div className="flex items-center gap-1.5">
      <Sun
        size={16}
        aria-hidden
        className={isDark ? 'text-(--meta-text)' : 'text-(--accent)'}
      />
      <Switch
        checked={isDark}
        onCheckedChange={(checked) => setTheme(checked ? 'dark' : 'light')}
        aria-label="Тёмная тема"
      />
      <Moon
        size={16}
        aria-hidden
        className={isDark ? 'text-(--accent)' : 'text-(--meta-text)'}
      />
    </div>
  );
}
