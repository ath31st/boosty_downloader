import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './app/App';
import './global.css';
import { ThemeProvider } from './providers/ThemeProvider';
import { ToasterProvider } from './providers/ToasterProvider';

createRoot(document.getElementById('root') as HTMLElement).render(
  <StrictMode>
    <ThemeProvider>
      <ToasterProvider />
      <App />
    </ThemeProvider>
  </StrictMode>,
);
