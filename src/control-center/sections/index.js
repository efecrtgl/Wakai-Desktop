// Kontrol Merkezi bölümleri. Yalnızca gerçekten çalışan bölümler listelenir.
import { tr } from '../../lib/i18n.js';
import { Overview } from './Overview.jsx';
import { WidgetsSection } from './Widgets.jsx';
import { SettingsSection } from './Settings.jsx';

export const SECTIONS = [
  { id: 'overview', label: tr.nav.overview, icon: 'home', component: Overview },
  { id: 'widgets', label: tr.nav.widgets, icon: 'widgets', component: WidgetsSection },
  { id: 'settings', label: tr.nav.settings, icon: 'gear', component: SettingsSection },
];
