import { useEffect, useRef, useState } from 'react';
import { type DbLogEntry, loadDbLogs } from '../bridge/vpdaBridge';
import type { VpdaAppState } from '../data/useVpdaApp';

type Props = { app: VpdaAppState };

function statusColor(status: string): string {
  if (status.includes('start')) return '#f0c060';
  if (status.includes('end')) return '#3dcf6e';
  if (status.includes('skip')) return '#888';
  if (status.includes('initialized')) return '#60b0f0';
  return 'var(--text2)';
}

export default function DbLogsModal({ app }: Props) {
  const [logs, setLogs] = useState<DbLogEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!app.dbLogsOpen) return;
    refresh();
    const id = setInterval(refresh, 3000);
    return () => clearInterval(id);
  }, [app.dbLogsOpen]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [logs]);

  async function refresh() {
    setLoading(true);
    try {
      const entries = await loadDbLogs();
      setLogs(entries);
    } finally {
      setLoading(false);
    }
  }

  if (!app.dbLogsOpen) return null;

  return (
    <div
      className="modal-overlay"
      onClick={(e) => { if (e.target === e.currentTarget) app.setDbLogsOpen(false); }}
    >
      <div className="modal-card info" style={{ width: 560, maxHeight: '70vh', padding: '24px 20px', display: 'flex', flexDirection: 'column', gap: 14 }}>

        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', flexShrink: 0 }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <span style={{ fontSize: 14, fontWeight: 700, color: 'var(--text)' }}>Database Logs</span>
            {loading && (
              <span style={{ fontSize: 10, color: 'var(--text3)', letterSpacing: '0.04em' }}>refreshing…</span>
            )}
          </div>
          <div style={{ display: 'flex', gap: 6 }}>
            <button className="ue-btn small" onClick={refresh}>Refresh</button>
            <button className="object-editor-close" onClick={() => app.setDbLogsOpen(false)}>×</button>
          </div>
        </div>

        <div style={{
          flex: 1, overflowY: 'auto',
          background: 'var(--bg1)', border: '1px solid var(--border1)',
          borderRadius: 4, padding: '10px 12px',
          fontFamily: 'monospace', fontSize: 11,
          display: 'flex', flexDirection: 'column', gap: 3,
          minHeight: 200,
        }}>
          {logs.length === 0 ? (
            <span style={{ color: 'var(--text3)' }}>No log entries yet. Updates start in the background after launch.</span>
          ) : (
            logs.map((entry, i) => (
              <div key={i} style={{ display: 'flex', gap: 10, lineHeight: 1.6 }}>
                <span style={{ color: 'var(--text3)', flexShrink: 0, minWidth: 56 }}>{entry.ts}</span>
                <span style={{ color: 'var(--text2)', flexShrink: 0, minWidth: 160 }}>DB: &quot;{entry.db}&quot;</span>
                <span style={{ color: statusColor(entry.status) }}>{entry.status}</span>
              </div>
            ))
          )}
          <div ref={bottomRef} />
        </div>

        <div style={{ fontSize: 10, color: 'var(--text3)', flexShrink: 0 }}>
          Auto-refreshes every 3 seconds while open. Session data updates every 6h, pattern DB every 24h.
        </div>
      </div>
    </div>
  );
}
