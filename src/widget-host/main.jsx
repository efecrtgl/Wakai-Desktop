import { createRoot } from 'react-dom/client';
import '../ui/tokens.css';
import '../ui/base.css';
import './widget-host.css';

function Probe() {
  return (
    <div className="widget-frame">
      <p style={{ fontWeight: 700 }}>Wakai katman testi</p>
      <input placeholder="Klavye testi" style={{ marginTop: 8 }} />
    </div>
  );
}

createRoot(document.getElementById('root')).render(<Probe />);
