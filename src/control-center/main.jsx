import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import '@fontsource/pixelify-sans/400.css';
import '@fontsource/pixelify-sans/600.css';
import '../ui/tokens.css';
import '../ui/base.css';
import '../ui/components.css';
import '../ui/theme-styles.css';
import './control-center.css';
import './sections.css';
import './redesign.css';
import App from './App.jsx';

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
