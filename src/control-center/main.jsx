import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import '../ui/tokens.css';
import '../ui/base.css';
import '../ui/components.css';
import './control-center.css';
import './sections.css';
import App from './App.jsx';

createRoot(document.getElementById('root')).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
