import { Toaster } from 'sonner';

export function ToasterProvider() {
  return (
    <Toaster
      position="bottom-right"
      richColors
      duration={3000}
      toastOptions={{
        style: {
          background: 'var(--secondary-bg)',
          color: 'var(--text)',
          border: '1px solid var(--border)',
        },
      }}
    />
  );
}
