// Sekmeli bölümler: mevcut sayfaları tek bir bilgi mimarisi altında toplar.
import { SectionTabs } from './SectionTabs.jsx';
import { NAV } from '../nav.js';
import { WidgetsSection } from './Widgets.jsx';
import { StoreSection } from './Store.jsx';
import { CalendarSection } from './Calendar.jsx';
import { TasksSection } from './Tasks.jsx';
import { NotesSection } from './Notes.jsx';
import { PetsSection } from './Pets.jsx';
import { PetWorldSection } from './PetWorld.jsx';
import { ProfilesSection } from './Profiles.jsx';
import { LookSection } from './Look.jsx';

const tabsOf = (id) => NAV.find((n) => n.id === id).tabs;

function Hub({ id, title, subtitle, views, route, onNavigate, ...props }) {
  const tabs = tabsOf(id);
  const sub = route?.sub ?? tabs[0].id;
  const View = views[sub] ?? views[tabs[0].id];
  return (
    <SectionTabs title={title} subtitle={subtitle} tabs={tabs} active={sub} onChange={(t) => onNavigate(id, t)}>
      <View {...props} route={route} onNavigate={onNavigate} />
    </SectionTabs>
  );
}

export const WidgetsHub = (p) => <Hub id="widgets" title="Widget'lar" subtitle="Masaüstünüzdeki widget'ları yönetin ya da yenisini ekleyin" views={{ mine: WidgetsSection, store: StoreSection }} {...p} />;
export const PlannerHub = (p) => <Hub id="planner" title="Planlayıcı" subtitle="Takvim, görevler ve notlar; widget'larla eş zamanlı" views={{ calendar: CalendarSection, tasks: TasksSection, notes: NotesSection }} {...p} />;
export const PetHub = (p) => <Hub id="pethome" title="Kedi Evi" subtitle="Kedilerinizin yaşadığı küçük dünya" views={{ world: PetWorldSection, cats: PetsSection }} {...p} />;
export const PersonalizeHub = (p) => <Hub id="personalize" title="Kişiselleştirme" subtitle="Görünümü kendinize göre ayarlayın, masaüstü profilleri oluşturun" views={{ look: LookSection, profiles: ProfilesSection }} {...p} />;
