import { useState } from 'react';
import type { VpdaAppState } from '../data/useVpdaApp';

type Props = { app: VpdaAppState };

export default function ActivationGate({ app }: Props) {
  const [key, setKey] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [alreadyActive, setAlreadyActive] = useState(false);

  if (!app.startup?.activationRequired) return null;

  async function handleActivate() {
    const trimmed = key.trim().toUpperCase();
    if (!trimmed) { setError('Enter your license key.'); return; }
    setBusy(true);
    setError(null);
    try {
      const result = await app.submitActivation(trimmed);
      if (result.success) {
        if (result.message.toLowerCase().includes('already')) {
          setAlreadyActive(true);
        }
      } else {
        setError(result.message);
      }
    } catch {
      setError('Unexpected error. Check your connection and try again.');
    } finally {
      setBusy(false);
    }
  }

  const machineId = app.account?.machineId ?? '…';

  return (
    <div style={{
      position: 'fixed', inset: 0, zIndex: 99999,
      background: 'var(--bg0)',
      display: 'flex', alignItems: 'center', justifyContent: 'center',
    }}>
      <div style={{
        width: 420, background: 'var(--bg2)',
        border: '1px solid var(--border2)', borderRadius: 8,
        padding: '32px 28px', display: 'flex', flexDirection: 'column', gap: 20,
      }}>
        {alreadyActive ? (
          <>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              <span style={{ fontSize: 18, fontWeight: 700, color: 'var(--text)' }}>Already Activated</span>
              <span style={{ fontSize: 12, color: 'var(--text2)', lineHeight: 1.5 }}>
                This machine is already activated with that license key. You're all set.
              </span>
            </div>
            <div style={{
              background: '#3dcf6e18', border: '1px solid #3dcf6e44',
              borderRadius: 6, padding: '10px 14px',
              fontSize: 12, color: '#3dcf6e',
            }}>
              License is active on this machine.
            </div>
          </>
        ) : (
          <>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              <span style={{ fontSize: 18, fontWeight: 700, color: 'var(--text)' }}>Activate VPDA</span>
              <span style={{ fontSize: 12, color: 'var(--text2)', lineHeight: 1.5 }}>
                {app.startup.activationMessage ?? 'This machine is not activated. Enter your license key to continue.'}
              </span>
            </div>

            <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
              <label style={{ fontSize: 11, color: 'var(--text3)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>
                License Key
              </label>
              <input
                type="text"
                placeholder="XXXX-XXXX-XXXX"
                value={key}
                onChange={e => { setKey(e.target.value); setError(null); }}
                onKeyDown={e => e.key === 'Enter' && !busy && handleActivate()}
                disabled={busy}
                style={{
                  background: 'var(--input-bg)', border: '1px solid var(--border2)',
                  borderRadius: 4, padding: '8px 10px', color: 'var(--text)',
                  fontSize: 13, fontFamily: 'monospace', letterSpacing: '0.08em',
                  outline: 'none', width: '100%', boxSizing: 'border-box',
                }}
              />
              {error && (
                <span style={{ fontSize: 12, color: '#e05050' }}>{error}</span>
              )}
            </div>

            <button
              onClick={handleActivate}
              disabled={busy}
              style={{
                background: busy ? 'var(--bg3)' : 'var(--accent)',
                color: busy ? 'var(--text3)' : '#000',
                border: 'none', borderRadius: 4, padding: '9px 0',
                fontWeight: 700, fontSize: 13, cursor: busy ? 'not-allowed' : 'pointer',
              }}
            >
              {busy ? 'Activating…' : 'Activate'}
            </button>

            <div style={{
              borderTop: '1px solid var(--border1)', paddingTop: 14,
              display: 'flex', flexDirection: 'column', gap: 4,
            }}>
              <span style={{ fontSize: 10, color: 'var(--text3)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>
                Machine ID (for support)
              </span>
              <span style={{
                fontSize: 11, fontFamily: 'monospace', color: 'var(--text2)',
                wordBreak: 'break-all', userSelect: 'all',
              }}>
                {machineId}
              </span>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
