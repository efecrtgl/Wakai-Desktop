import { Card, Toggle, ErrorNote } from '../../ui/components.jsx';
import { tr } from '../../lib/i18n.js';
import { PageHeader } from './PageHeader.jsx';

export function SettingsSection({ settingsApi }) {
  const { settings, update, error } = settingsApi;
  if (!settings) return <div className="cc-page">{tr.common.loading}</div>;

  return (
    <div className="cc-page">
      <PageHeader title={tr.settings.title} />
      <ErrorNote error={error} />
      <Card title={tr.settings.general}>
        <Toggle
          label={tr.settings.closeToTray}
          description={tr.settings.closeToTrayDesc}
          checked={settings.close_to_tray}
          onChange={(v) => update({ close_to_tray: v })}
        />
        <Toggle
          label={tr.settings.launchOnStartup}
          description={tr.settings.launchOnStartupDesc}
          checked={settings.launch_on_startup}
          onChange={(v) => update({ launch_on_startup: v })}
        />
      </Card>
    </div>
  );
}
