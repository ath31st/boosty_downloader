import type { ReactNode } from 'react';

interface LabelProps {
  htmlFor?: string;
  className?: string;
  children?: ReactNode;
}

export function Label({ children, className, htmlFor }: LabelProps) {
  return (
    <label
      htmlFor={htmlFor}
      className={`flex items-center text-(--meta-text) text-sm ${className ?? ''}`}
    >
      {children}
    </label>
  );
}
