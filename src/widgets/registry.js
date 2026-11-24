// Ön yüz widget kayıt defteri: tür kimliği → tembel yüklenen bileşen.
// `lazy` modül yüklenirken yalnızca bir sarmalayıcı oluşturur; asıl kod widget ilk
// çizildiğinde indirilir. Böylece her widget penceresi yalnızca kendi türünün kodunu çalıştırır.
import { lazy } from 'react';

export const WIDGET_COMPONENTS = {
  clock: lazy(() => import('./clock/ClockWidget.jsx')),
};

export const supportedKinds = Object.keys(WIDGET_COMPONENTS);
