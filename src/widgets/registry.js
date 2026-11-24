// Ön yüz widget kayıt defteri: tür kimliği → tembel yüklenen bileşen.
// `lazy` modül yüklenirken yalnızca bir sarmalayıcı oluşturur; asıl kod widget ilk
// çizildiğinde indirilir. Böylece her widget penceresi yalnızca kendi türünün kodunu çalıştırır.
import { lazy } from 'react';

export const WIDGET_COMPONENTS = {
  calendar: lazy(() => import('./calendar/CalendarWidget.jsx')),
  tasks: lazy(() => import('./tasks/TaskWidget.jsx')),
  note: lazy(() => import('./note/NoteWidget.jsx')),
  shopping: lazy(() => import('./shopping/ShoppingWidget.jsx')),
  clock: lazy(() => import('./clock/ClockWidget.jsx')),
  focus: lazy(() => import('./focus/FocusWidget.jsx')),
  weather: lazy(() => import('./weather/WeatherWidget.jsx')),
  music: lazy(() => import('./music/MusicWidget.jsx')),
  system: lazy(() => import('./system/SystemWidget.jsx')),
};

export const supportedKinds = Object.keys(WIDGET_COMPONENTS);
