import { createRoot } from 'react-dom/client';
import '@fontsource/pixelify-sans/400.css';
import '@fontsource/pixelify-sans/600.css';
import '../ui/tokens.css';
import '../ui/base.css';
import '../ui/components.css';
import '../ui/theme-styles.css';
import './widget-host.css';
import { WidgetHost } from './WidgetHost.jsx';

const id = new URLSearchParams(location.search).get('id');
createRoot(document.getElementById('root')).render(<WidgetHost id={id} />);
