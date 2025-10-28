import type { ReactNode } from 'react';

interface ButtonProps {
  children: ReactNode;
  disabled?: boolean;
  onClick?: () => void;
}

export function Button({ children, onClick, disabled }: ButtonProps) {
  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onClick}
      className="rounded-lg bg-(--button-bg) px-4 py-2 text-(--button-text) transition-colors hover:bg-(--button-hover-bg)"
    >
      {children}
    </button>
  );
}
