// Kontrol Merkezi bölümleri.
import { tr } from '../../lib/i18n.js';
import { Overview } from './Overview.jsx';
import { WidgetsSection } from './Widgets.jsx';
import { StoreSection } from './Store.jsx';
import { CalendarSection } from './Calendar.jsx';
import { TasksSection } from './Tasks.jsx';
import { NotesSection } from './Notes.jsx';
import { PetsSection } from './Pets.jsx';
import { PetHomeSection } from './PetHome.jsx';
import { AppearanceSection } from './Appearance.jsx';
import { PerformanceSection } from './Performance.jsx';
import { NotificationsSection } from './Notifications.jsx';
import { SettingsSection } from './Settings.jsx';

export const SECTIONS = [
  { id: 'overview', label: tr.nav.overview, icon: 'home', component: Overview },
  { id: 'widgets', label: tr.nav.widgets, icon: 'widgets', component: WidgetsSection },
  { id: 'store', label: tr.nav.store, icon: 'store', component: StoreSection },
  { id: 'calendar', label: tr.nav.calendar, icon: 'calendar', component: CalendarSection },
  { id: 'tasks', label: tr.nav.tasks, icon: 'tasks', component: TasksSection },
  { id: 'notes', label: tr.nav.notes, icon: 'notes', component: NotesSection },
  { id: 'pets', label: 'Kediler', icon: 'paw', component: PetsSection },
  { id: 'pethome', label: 'Kedi Evi', icon: 'home', component: PetHomeSection },
  { id: 'appearance', label: tr.nav.appearance, icon: 'palette', component: AppearanceSection },
  { id: 'performance', label: tr.nav.performance, icon: 'bolt', component: PerformanceSection },
  { id: 'notifications', label: tr.nav.notifications, icon: 'bell', component: NotificationsSection },
  { id: 'settings', label: tr.nav.settings, icon: 'gear', component: SettingsSection },
];
