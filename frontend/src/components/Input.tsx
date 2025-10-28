import type { ChangeEvent } from 'react';

interface InputProps {
  value: string | number;
  onChange: (value: string | number) => void;
  type?: 'text' | 'number' | 'password';
  disabled?: boolean;
  className?: string;
}

export function Input({
  value,
  onChange,
  type = 'text',
  disabled,
  className,
}: InputProps) {
  return (
    <input
      type={type}
      value={value}
      disabled={disabled}
      onChange={(e: ChangeEvent<HTMLInputElement>) =>
        type === 'number'
          ? onChange(Number(e.target.value))
          : onChange(e.target.value)
      }
      className={`rounded border border-(--border) bg-(--background) p-1 text-(--text) focus:outline-none focus:ring-(--button-bg) focus:ring-2 ${className ?? ''}`}
    />
  );
}
