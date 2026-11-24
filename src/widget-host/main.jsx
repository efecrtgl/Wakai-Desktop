import { createRoot } from 'react-dom/client';
import '../ui/tokens.css';
import '../ui/base.css';
import '../ui/components.css';
import './widget-host.css';
import { WidgetHost } from './WidgetHost.jsx';

const id = new URLSearchParams(location.search).get('id');
createRoot(document.getElementById('root')).render(<WidgetHost id={id} />);
