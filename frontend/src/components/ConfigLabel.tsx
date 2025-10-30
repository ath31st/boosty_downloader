interface ConfigLabelProps {
  label: string;
  className?: string;
}

export function ConfigLabel({ label, className }: ConfigLabelProps) {
  return (
    <span className={`w-40 text-(--meta-text) text-lg ${className ?? ''}`}>
      {label}
    </span>
  );
}
