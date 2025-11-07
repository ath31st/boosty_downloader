import type { ChangeEvent } from 'react';

interface InputProps {
  value: string | number;
  onChange: (value: string | number) => void;
  type?: 'text' | 'number' | 'password';
  disabled?: boolean;
  placeholder?: string;
  className?: string;
}

export function Input({
  value,
  onChange,
  type = 'text',
  disabled,
  placeholder,
  className,
}: InputProps) {
  return (
    <input
      type={type}
      value={value}
      disabled={disabled}
      placeholder={placeholder}
      onChange={(e: ChangeEvent<HTMLInputElement>) =>
        type === 'number'
          ? onChange(Number(e.target.value))
          : onChange(e.target.value)
      }
      className={`rounded-lg border border-(--border) p-2 text-(--text) focus:outline-none focus:ring-(--button-bg) focus:ring-2 ${
        disabled
          ? 'cursor-not-allowed bg-(--secondary-bg) text-(--meta-text) opacity-50'
          : 'bg-(--secondary-bg)'
      } ${className ?? ''}`}
    />
  );
}
