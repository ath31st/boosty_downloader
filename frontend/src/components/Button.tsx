import type { ReactNode } from 'react';

interface ButtonProps {
  children: ReactNode;
  disabled?: boolean;
  className?: string;
  variant?: 'primary' | 'secondary';
  onClick?: () => void;
}

export function Button({
  children,
  onClick,
  disabled,
  className,
  variant = 'primary',
}: ButtonProps) {
  const variantClass =
    variant === 'secondary'
      ? 'border border-(--border) bg-(--button-secondary-bg) text-(--button-secondary-text) hover:bg-(--button-secondary-hover-bg) disabled:hover:bg-(--button-secondary-bg)'
      : 'bg-(--button-bg) text-(--button-text) hover:bg-(--button-hover-bg) disabled:hover:bg-(--button-bg)';

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onClick}
      className={`cursor-pointer rounded-lg px-4 py-2 transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${variantClass} ${className ?? ''}`}
    >
      {children}
    </button>
  );
}
