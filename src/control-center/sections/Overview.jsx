import { useEffect, useState } from 'react';
import { Card, Badge, ErrorNote } from '../../ui/components.jsx';
import { call } from '../../lib/ipc.js';
import { tr } from '../../lib/i18n.js';
import { PageHeader } from './PageHeader.jsx';
import { PixelCat } from '../../ui/cats/PixelCat.jsx';

export function Overview() {
  const [info, setInfo] = useState(null);
  const [error, setError] = useState(null);

  useEffect(() => {
    call('app_info').then(setInfo, (e) => setError(e.message));
  }, []);

  return (
    <div className="cc-page">
      <PageHeader title={tr.overview.greeting} subtitle={tr.app.tagline} actions={<PixelCat pose="happy" size={56} wag />} />
      <ErrorNote error={error} />
      <div className="cc-grid">
        <Card title={tr.overview.status} action={<Badge tone="mint">{tr.overview.running}</Badge>}>
          <dl className="cc-dl">
            <dt>{tr.overview.version}</dt>
            <dd>{info?.version ?? '—'}</dd>
            <dt>{tr.overview.schema}</dt>
            <dd>{info ? `v${info.schema_version}` : '—'}</dd>
            <dt>{tr.overview.database}</dt>
            <dd className="cc-dl__path">{info?.database_path ?? '—'}</dd>
          </dl>
        </Card>
      </div>
    </div>
  );
}
