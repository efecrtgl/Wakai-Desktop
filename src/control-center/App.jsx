import { useState } from 'react';
import { Sidebar } from './Sidebar.jsx';
import { SECTIONS } from './sections/index.js';
import { useSettings } from '../lib/settings.js';
import { useMotionAttribute } from '../lib/motion.js';

export default function App() {
  const [active, setActive] = useState(SECTIONS[0].id);
  const settingsApi = useSettings();
  useMotionAttribute(settingsApi.settings);
  const section = SECTIONS.find((s) => s.id === active) ?? SECTIONS[0];
  const View = section.component;

  return (
    <div className="cc">
      <Sidebar sections={SECTIONS} active={section.id} onSelect={setActive} />
      <main className="cc__main">
        <View settingsApi={settingsApi} onNavigate={setActive} />
      </main>
    </div>
  );
}
