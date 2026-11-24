// Kontrol Merkezi bölümleri. Yalnızca gerçekten çalışan bölümler listelenir.
import { tr } from '../../lib/i18n.js';
import { Overview } from './Overview.jsx';
import { SettingsSection } from './Settings.jsx';

export const SECTIONS = [
  { id: 'overview', label: tr.nav.overview, icon: '🏠', component: Overview },
  { id: 'settings', label: tr.nav.settings, icon: '⚙️', component: SettingsSection },
];
