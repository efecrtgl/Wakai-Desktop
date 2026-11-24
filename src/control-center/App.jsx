import { useState } from 'react';
import { Sidebar } from './Sidebar.jsx';
import { SECTIONS } from './sections/index.js';
import { useSettings } from '../lib/settings.js';
import { useMotionAttribute } from '../lib/motion.js';
import { useTauriEvent } from '../lib/ipc.js';

export default function App() {
  const [active, setActive] = useState(SECTIONS[0].id);
  const [focusWidget, setFocusWidget] = useState(null);
  const settingsApi = useSettings();
  useMotionAttribute(settingsApi.settings);

  // Widget'ın bağlam menüsünden "Widget ayarları" seçildiğinde ilgili bölüme git.
  useTauriEvent('navigate', ({ section, widget_id }) => {
    if (section && SECTIONS.some((s) => s.id === section)) setActive(section);
    setFocusWidget(widget_id ?? null);
  });

  const section = SECTIONS.find((s) => s.id === active) ?? SECTIONS[0];
  const View = section.component;

  return (
    <div className="cc">
      <Sidebar sections={SECTIONS} active={section.id} onSelect={setActive} />
      <main className="cc__main">
        <View settingsApi={settingsApi} onNavigate={setActive} focusWidget={focusWidget} />
      </main>
    </div>
  );
}
