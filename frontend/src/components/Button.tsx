import type { ReactNode } from 'react';

interface ButtonProps {
  children: ReactNode;
  onClick?: () => void;
}

export function Button({ children, onClick }: ButtonProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="rounded-lg bg-(--button-bg) px-4 py-2 text-(--button-text) transition-colors hover:bg-(--button-hover-bg)"
    >
      {children}
    </button>
  );
}
