import type { VpdaAppState } from '../data/useVpdaApp';
import type { DrawTool } from '../bridge/vpdaBridge';

function IconCursor() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z" />
      <path d="M13 13l6 6" />
    </svg>
  );
}

function IconLine() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <line x1="18" y1="6" x2="6" y2="18" />
    </svg>
  );
}

function IconHLine() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <line x1="3" y1="12" x2="21" y2="12" />
    </svg>
  );
}

function IconVLine() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <line x1="12" y1="3" x2="12" y2="21" />
    </svg>
  );
}

function IconRect() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
    </svg>
  );
}

function IconText() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <polyline points="4 7 4 4 20 4 20 7" />
      <line x1="12" y1="20" x2="12" y2="4" />
      <line x1="9" y1="20" x2="15" y2="20" />
    </svg>
  );
}

const TOOLS: Array<{ key: DrawTool; label: string; Icon: () => JSX.Element }> = [
  { key: 'cursor', label: 'Cursor', Icon: IconCursor },
];

function IconFib() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <line x1="4" y1="6" x2="20" y2="6" />
      <line x1="4" y1="12" x2="20" y2="12" />
      <line x1="4" y1="18" x2="20" y2="18" />
      <line x1="12" y1="3" x2="12" y2="21" opacity="0.3" />
    </svg>
  );
}

function IconRDS() {
  return (
    <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
      <circle cx="7" cy="7" r="2" />
      <circle cx="17" cy="17" r="2" />
      <line x1="4" y1="20" x2="20" y2="12" strokeWidth="2" />
      <text x="2" y="11" fontSize="6" fill="currentColor" stroke="none" fontFamily="monospace" fontWeight="bold">RDS</text>
    </svg>
  );
}

const DRAWING_TOOL_GROUPS = [
  [
    { key: 'trend',  label: 'Line',   Icon: IconLine  },
    { key: 'hline',  label: 'H-Line', Icon: IconHLine },
    { key: 'vline',  label: 'V-Line', Icon: IconVLine },
  ],
  [
    { key: 'rect',   label: 'Box',    Icon: IconRect  },
    { key: 'text',   label: 'Text',   Icon: IconText  },
  ],
  [
    { key: 'fib',    label: 'Fib',    Icon: IconFib   },
  ],
  [
    { key: 'rds',    label: 'RDS',    Icon: IconRDS   },
  ],
] as const;

type Props = {
  app: VpdaAppState;
};

export default function ToolRail({ app }: Props) {
  return (
    <aside className="tool-sidebar" style={{ gap: '2px', padding: '4px', width: 42 }}>
      {TOOLS.map(({ key, label, Icon }) => (
        <button
          key={key}
          className={`tool-btn ${app.drawTool === key ? 'active' : ''}`}
          title={label}
          onClick={() => app.setDrawTool(key)}
          style={{ width: 34, height: 34, padding: 0, display: 'flex', alignItems: 'center', justifyContent: 'center' }}
        >
          <span className="tool-icon" style={{ margin: 0 }}>
            <Icon />
          </span>
        </button>
      ))}

      {DRAWING_TOOL_GROUPS.map((group, groupIdx) => (
        <div key={groupIdx} style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}>
          <div className="tool-sep" style={{ margin: '4px 6px', height: 1, background: 'var(--border1)' }} />
          {group.map(({ key, label, Icon }) => (
            <button
              key={key}
              className={`tool-btn ${app.drawTool === key ? 'active' : ''}`}
              title={label}
              onClick={() => app.setDrawTool(key as DrawTool)}
              style={{ width: 34, height: 34, padding: 0, display: 'flex', alignItems: 'center', justifyContent: 'center' }}
            >
              <span className="tool-icon" style={{ margin: 0 }}>
                <Icon />
              </span>
            </button>
          ))}
        </div>
      ))}
    </aside>
  );
}