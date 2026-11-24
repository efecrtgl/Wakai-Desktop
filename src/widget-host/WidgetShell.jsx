// Tüm widget'ların ortak çerçevesi: sürükleme şeridi, boyutlandırma tutamağı ve bağlam menüsü.
// Sürükleme/boyutlandırma yalnızca widget "kaldırılmış" (üzerine gelinmiş) iken mümkündür;
// bu durumda pencere normal bir üst düzey penceredir ve işletim sisteminin taşıma döngüsü kullanılır.
import { useCallback, useEffect, useRef, useState } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { call } from '../lib/ipc.js';
import { tr } from '../lib/i18n.js';

const MENU_WIDTH = 190;
const MENU_HEIGHT = 170;

function ContextMenu({ x, y, instance, onClose }) {
  const ref = useRef(null);
  const [confirmRemove, setConfirmRemove] = useState(false);

  useEffect(() => {
    const onDown = (e) => {
      if (!ref.current?.contains(e.target)) onClose();
    };
    const onKey = (e) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('pointerdown', onDown);
    window.addEventListener('keydown', onKey);
    window.addEventListener('blur', onClose);
    return () => {
      window.removeEventListener('pointerdown', onDown);
      window.removeEventListener('keydown', onKey);
      window.removeEventListener('blur', onClose);
    };
  }, [onClose]);

  const run = (fn) => async () => {
    onClose();
    try {
      await fn();
    } catch (e) {
      console.error(e);
    }
  };

  // Menünün pencere dışına taşmasını önle.
  const style = {
    left: Math.max(0, Math.min(x, window.innerWidth - MENU_WIDTH)),
    top: Math.max(0, Math.min(y, window.innerHeight - MENU_HEIGHT)),
  };
  const id = instance.id;

  return (
    <div ref={ref} className="widget-menu" style={style} role="menu">
      <button type="button" role="menuitem" onClick={run(() => call('open_control_center', { section: 'widgets', widgetId: id }))}>
        {tr.widget.settings}
      </button>
      <button type="button" role="menuitem" onClick={run(() => call('set_widget_click_through', { id, value: true }))}>
        {tr.widget.clickThrough}
      </button>
      <button type="button" role="menuitem" onClick={run(() => call('set_widget_visible', { id, visible: false }))}>
        {tr.widget.hide}
      </button>
      {confirmRemove ? (
        <button type="button" role="menuitem" className="danger" onClick={run(() => call('remove_widget', { id }))}>
          {tr.widget.confirmRemove}
        </button>
      ) : (
        <button type="button" role="menuitem" className="danger" onClick={() => setConfirmRemove(true)}>
          {tr.widget.remove}
        </button>
      )}
    </div>
  );
}

export function WidgetShell({ instance, definition, children }) {
  const [menu, setMenu] = useState(null);
  const closeMenu = useCallback(() => setMenu(null), []);

  const startDrag = (e) => {
    if (e.button !== 0) return;
    getCurrentWindow().startDragging().catch(console.error);
  };
  const startResize = (e) => {
    if (e.button !== 0) return;
    e.stopPropagation();
    getCurrentWindow().startResizeDragging('SouthEast').catch(console.error);
  };

  return (
    <div
      className="widget-frame"
      data-kind={instance.kind}
      onContextMenu={(e) => {
        e.preventDefault();
        setMenu({ x: e.clientX, y: e.clientY });
      }}
    >
      <div className="widget-drag" onPointerDown={startDrag} title={tr.widget.dragHint} aria-label={definition.name} />
      <div className="widget-content">{children}</div>
      <div className="widget-resize" onPointerDown={startResize} title={tr.widget.resizeHint} aria-hidden="true" />
      {menu ? <ContextMenu {...menu} instance={instance} onClose={closeMenu} /> : null}
    </div>
  );
}
