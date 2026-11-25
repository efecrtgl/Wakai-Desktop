import { useCallback, useEffect, useState } from 'react';
import { Sidebar } from './Sidebar.jsx';
import { SECTION_VIEWS } from './sections/index.js';
import { NAV, resolveRoute } from './nav.js';
import { useSettings } from '../lib/settings.js';
import { useMotionAttribute } from '../lib/motion.js';
import { useTauriEvent } from '../lib/ipc.js';
import { UpdateDialog } from './UpdateDialog.jsx';
import { BrandNotice } from './BrandNotice.jsx';
import { CommandPalette } from './CommandPalette.jsx';
import { PreviewBar } from './PreviewBar.jsx';
import { useThemeEngine } from '../theme/useTheme.js';
import { ThemeContext } from '../theme/context.js';
import { ThemeBoundary } from '../theme/ThemeBoundary.jsx';

/** Arama sonucundaki öğeye kaydırır ve kısa bir vurgu uygular. */
function flashAnchor(anchor) {
  if (!anchor) return;
  requestAnimationFrame(() =>
    requestAnimationFrame(() => {
      const el = document.getElementById(anchor);
      if (!el) return;
      el.scrollIntoView({ block: 'center', behavior: 'smooth' });
      el.classList.remove('cc-flash');
      void el.offsetWidth;
      el.classList.add('cc-flash');
      el.querySelector('input, button, select, [role="switch"]')?.focus({
        preventScroll: true,
      });
    }),
  );
}

export default function App() {
  const [route, setRoute] = useState({ section: 'overview', sub: null });
  const [focusWidget, setFocusWidget] = useState(null);
  const [searchOpen, setSearchOpen] = useState(false);
  const settingsApi = useSettings();
  useMotionAttribute(settingsApi.settings);
  const themeApi = useThemeEngine(settingsApi.settings);

  const navigate = useCallback((section, sub = null, anchor = null) => {
    setRoute(resolveRoute(section, sub));
    document.querySelector('.cc__main')?.scrollTo({ top: 0 });
    if (anchor) flashAnchor(anchor);
  }, []);

  // Widget menüsü, tepsi ve kedi menüsü buraya yönlendirir (1.1.0 bölüm kimlikleri de geçerli).
  useTauriEvent('navigate', ({ section, sub, widget_id }) => {
    if (section) navigate(section, sub ?? null);
    setFocusWidget(widget_id ?? null);
  });

  // Klavye: Ctrl+K arama, Ctrl+1…7 bölümler.
  useEffect(() => {
    const onKey = (e) => {
      if (!e.ctrlKey || e.altKey || e.metaKey) return;
      if (e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setSearchOpen(true);
      } else if (/^[1-9]$/.test(e.key) && NAV[Number(e.key) - 1]) {
        e.preventDefault();
        navigate(NAV[Number(e.key) - 1].id);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [navigate]);

  const nav = NAV.find((n) => n.id === route.section) ?? NAV[0];
  const View = SECTION_VIEWS[nav.id];

  return (
    <ThemeContext.Provider value={themeApi}>
      <ThemeBoundary onFallback={() => themeApi.setPreview(null)}>
        <div className="cc">
          <a
            className="cc-skip"
            href="#cc-main"
            onClick={(e) => {
              e.preventDefault();
              document.getElementById('cc-main')?.focus();
            }}
          >
            İçeriğe geç
          </a>
          <Sidebar active={nav.id} onSelect={navigate} onSearch={() => setSearchOpen(true)} />
          <main className="cc__main" id="cc-main" tabIndex={-1} aria-label={nav.label}>
            <PreviewBar settingsApi={settingsApi} />
            <BrandNotice />
            <View key={nav.id} settingsApi={settingsApi} route={route} onNavigate={navigate} focusWidget={focusWidget} />
          </main>
          <UpdateDialog />
          <CommandPalette open={searchOpen} onClose={() => setSearchOpen(false)} onPick={(it) => navigate(it.section, it.sub ?? null, it.anchor ?? null)} />
        </div>
      </ThemeBoundary>
    </ThemeContext.Provider>
  );
}
