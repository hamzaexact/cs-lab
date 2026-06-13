import type { VpdaAppState } from '../data/useVpdaApp';

type Props = { app: VpdaAppState };

function Row({ label, value, mono = false }: { label: string; value: string; mono?: boolean }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
      <span style={{ fontSize: 10, color: 'var(--text3)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>
        {label}
      </span>
      <span style={{
        fontSize: 12,
        color: 'var(--text2)',
        fontFamily: mono ? 'monospace' : undefined,
        wordBreak: 'break-all',
        userSelect: 'all',
      }}>
        {value}
      </span>
    </div>
  );
}

export default function AccountModal({ app }: Props) {
  if (!app.accountOpen) return null;

  const acc = app.account;
  const isActive = acc?.status === 'Active';

  return (
    <div
      className="modal-overlay"
      onClick={(e) => { if (e.target === e.currentTarget) app.setAccountOpen(false); }}
    >
      <div className="modal-card info" style={{ width: 360, padding: '28px 24px', display: 'flex', flexDirection: 'column', gap: 20 }}>

        {/* Header */}
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <span style={{ fontSize: 14, fontWeight: 700, color: 'var(--text)' }}>Account</span>
          <button className="object-editor-close" onClick={() => app.setAccountOpen(false)}>×</button>
        </div>

        {/* License badge */}
        <div style={{
          display: 'flex', alignItems: 'center', gap: 10,
          background: isActive ? '#3dcf6e18' : '#e0505018',
          border: `1px solid ${isActive ? '#3dcf6e44' : '#e0505044'}`,
          borderRadius: 6, padding: '10px 14px',
        }}>
          <span style={{
            width: 8, height: 8, borderRadius: '50%',
            background: isActive ? '#3dcf6e' : '#e05050',
            flexShrink: 0,
          }} />
          <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <span style={{ fontSize: 11, fontWeight: 700, color: isActive ? '#3dcf6e' : '#e05050', letterSpacing: '0.04em' }}>
              {isActive ? 'ACTIVATED' : (acc?.status?.toUpperCase() ?? 'CHECKING…')}
            </span>
            <span style={{ fontSize: 11, color: 'var(--text3)' }}>Perpetual License</span>
          </div>
        </div>

        {/* Fields */}
        {acc ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
            {acc.activatedSince ? (
              <Row label="Activated On" value={acc.activatedSince} />
            ) : (
              <Row label="Activated On" value="—" />
            )}
            <Row label="Machine ID" value={acc.machineId} mono />
          </div>
        ) : (
          <span style={{ fontSize: 12, color: 'var(--text3)' }}>Loading…</span>
        )}

        {/* Footer */}
        <div style={{ borderTop: '1px solid var(--border1)', paddingTop: 14, display: 'flex', justifyContent: 'flex-end' }}>
          <button className="ue-btn small" onClick={() => app.setAccountOpen(false)}>Close</button>
        </div>
      </div>
    </div>
  );
}
