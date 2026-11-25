// Kontrol Merkezi bölüm görünümleri (gezinme yapısı nav.js'dedir).
import { Overview } from './Overview.jsx';
import { WidgetsHub, PlannerHub, PetHub, PersonalizeHub } from './hubs.jsx';
import { ThemeLibrary } from './ThemeLibrary.jsx';
import { SettingsSection } from './Settings.jsx';

export const SECTION_VIEWS = {
  overview: Overview,
  widgets: WidgetsHub,
  planner: PlannerHub,
  pethome: PetHub,
  themes: ThemeLibrary,
  personalize: PersonalizeHub,
  settings: SettingsSection,
};
