import { useEffect, useMemo, useState } from 'react';
import type { VpdaAppState } from '../data/useVpdaApp';
import { captureAppLayout } from '../utils/captureLayout';
import { formatPrice } from '../utils/format';

function CameraIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none" aria-hidden="true">
      <path
        d="M5.2 3.2h1.1l.7-1.2h1.9l.7 1.2h1.2c1.5 0 2.3.8 2.3 2.3v5.3c0 1.5-.8 2.3-2.3 2.3H5.2c-1.5 0-2.3-.8-2.3-2.3V5.5c0-1.5.8-2.3 2.3-2.3Z"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinejoin="round"
      />
      <circle cx="8" cy="8.2" r="2.3" stroke="currentColor" strokeWidth="1.2" />
      <circle cx="11.8" cy="5.5" r=".7" fill="currentColor" />
    </svg>
  );
}

const WORKSPACES = [
  ['market',   'Market Watch'],
  ['forecast', 'Forecast'],
  ['sessionForecast', 'Session Forecast'],
  ['inventory', 'Inventory'],
  ['vchart',   'V-Chart'],
] as const;

type Props = { app: VpdaAppState };

function useClock() {
  const [time, setTime] = useState(() =>
    new Date().toLocaleTimeString('en-GB', {
      hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false,
      timeZone: 'America/New_York',
    }),
  );

  useEffect(() => {
    const id = setInterval(() => {
      setTime(
        new Date().toLocaleTimeString('en-GB', {
          hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false,
          timeZone: 'America/New_York',
        }),
      );
    }, 1000);
    return () => clearInterval(id);
  }, []);

  return time;
}

function AccountIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="5.5" r="2.5" stroke="currentColor" strokeWidth="1.2"/>
      <path d="M2.5 13.5c0-3 2.5-4.5 5.5-4.5s5.5 1.5 5.5 4.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
    </svg>
  );
}

function SettingsIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <path d="M8 11.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z" stroke="currentColor" strokeWidth="1.2"/>
      <path d="M2.5 8h1M12.5 8h1M8 2.5v1M8 12.5v1M4 4l.7.7M11.3 11.3l.7.7M4 12l.7-.7M11.3 4.7l.7-.7" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
    </svg>
  );
}

function ZoomInIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <path d="M8 4v8M4 8h8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
    </svg>
  );
}

function ZoomOutIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <path d="M4 8h8" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/>
    </svg>
  );
}

function UndoIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <path d="M4.5 7.5 2 10l2.5 2.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round"/>
      <path d="M2 10h7a4 4 0 0 1 0 8" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" opacity="0.5"/>
    </svg>
  );
}

function RedoIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <path d="m11.5 7.5 2.5 2.5-2.5 2.5" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" strokeLinejoin="round"/>
      <path d="M14 10H7a4 4 0 0 0 0 8" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" opacity="0.5"/>
    </svg>
  );
}

function GridIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <path d="M2 5h12M2 11h12M5 2v12M11 2v12" stroke="currentColor" strokeWidth="1.2" opacity="0.6"/>
    </svg>
  );
}

const TIMEFRAMES = ['15M', '1H', '4H', '1D', '1W'];

function MagnetIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <path d="M4 5v4a4 4 0 0 0 8 0V5M4 5h2v4a2 2 0 0 0 4 0V5h2" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
    </svg>
  );
}

function CrosshairIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <path d="M8 2v12M2 8h12M8 8m-2 0a2 2 0 1 0 4 0 2 2 0 1 0-4 0" stroke="currentColor" strokeWidth="1.2"/>
    </svg>
  );
}

function HeatmapIcon() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <path d="M2 12h12M2 8h12M2 4h12" stroke="currentColor" strokeWidth="2" opacity="0.4"/>
      <path d="M4 12h2M8 8h3M5 4h5" stroke="currentColor" strokeWidth="2"/>
    </svg>
  );
}

export default function MenuBar({ app }: Props) {
  const clock = useClock();
  
  // OHLC needs to come from the latest candle or the market snapshot
  const ohlc = useMemo(() => {
    if (!app.market?.candles.length) return null;
    return app.market.candles[app.market.candles.length - 1];
  }, [app.market?.candles]);

  async function captureFullPage() {
    try {
      await captureAppLayout(
        `vpda-web-${app.symbol}-${new Date().toISOString().replace(/[:.]/g, '-')}.png`,
      );
    } catch (error) {
      console.error('Full page screenshot failed', error);
    }
  }

  return (
    <header className="menu-bar">

      <nav className="workspace-tabs" style={{ gap: 'var(--sp-1)' }}>
        {WORKSPACES.map(([key, label]) => (
          <button
            key={key}
            className={`menu-tab ${app.workspace === key ? 'active' : ''}`}
            onClick={() => app.setWorkspace(key)}
          >
            {label.split(' ')[0]}
          </button>
        ))}
      </nav>

      <div className="toolbar-divider" style={{ height: 16, width: 1, background: 'var(--border2)', margin: '0 4px' }} />

      {/* Asset & Timeframe */}
      <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--sp-2)' }}>
        <select
          className="toolbar-select"
          value={app.symbol}
          onChange={(e) => app.setSymbol(e.target.value)}
          style={{ height: 24, padding: '0 4px', fontSize: 'var(--fs-10)', background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '4px' }}
        >
          {app.watchlist.map((item) => (
            <option key={item.symbol} value={item.symbol}>{item.symbol}</option>
          ))}
        </select>

        <div style={{ display: 'flex', gap: '2px' }}>
          {TIMEFRAMES.map((tf) => (
            <button
              key={tf}
              className={`ue-btn small ${app.timeframe === tf ? 'active' : ''}`}
              onClick={() => app.setTimeframe(tf)}
              style={{ padding: '0 8px', height: 24, minWidth: 32 }}
            >
              {tf}
            </button>
          ))}
        </div>
      </div>

      <div className="menu-meta" style={{ gap: 'var(--sp-1)', flex: 1, justifyContent: 'flex-end' }}>
        {/* Chart Management */}
        <div style={{ display: 'flex', gap: '2px', marginRight: 'var(--sp-2)' }}>
          <button 
            className={`ue-btn small ${app.managerOpen ? 'active' : ''}`} 
            onClick={() => app.setManagerOpen(!app.managerOpen)}
            title="Chart Manager"
          >
            MGR
          </button>
          <button 
            className="ue-btn small strong" 
            id="global-save-btn"
            title="Save Snapshot"
          >
            SAVE
          </button>
        </div>

        {/* History Controls */}
        <div style={{ display: 'flex', gap: '2px', marginRight: 'var(--sp-2)' }}>
          <button className="ue-btn small" title="Undo (Ctrl+Z)" disabled={!app.canUndo} id="global-undo-btn"><UndoIcon /></button>
          <button className="ue-btn small" title="Redo (Ctrl+Y)" disabled={!app.canRedo} id="global-redo-btn"><RedoIcon /></button>
        </div>

        {/* Visibility Toggles - All restored */}
        <div style={{ display: 'flex', gap: '2px', marginRight: 'var(--sp-2)' }}>
          <button className={`ue-btn small ${app.showGrid ? 'active' : ''}`} onClick={() => app.setShowGrid(!app.showGrid)} title="Toggle Grid"><GridIcon /></button>
          <button className={`ue-btn small ${app.showCrosshair ? 'active' : ''}`} onClick={() => app.setShowCrosshair(!app.showCrosshair)} title="Toggle Crosshair"><CrosshairIcon /></button>
          <button className={`ue-btn small ${app.showHeatmap ? 'active' : ''}`} onClick={() => app.setShowHeatmap(!app.showHeatmap)} title="Toggle Heatmap"><HeatmapIcon /></button>
          <button className={`ue-btn small ${app.showMagnet ? 'active' : ''}`} onClick={() => app.setShowMagnet(!app.showMagnet)} title="Magnet Mode"><MagnetIcon /></button>
          <button className={`ue-btn small ${app.showWatchlist ? 'active' : ''}`} onClick={() => app.setShowWatchlist(!app.showWatchlist)} title="Watchlist (W)">W</button>
          <button className={`ue-btn small ${app.showInspector ? 'active' : ''}`} onClick={() => app.setShowInspector(!app.showInspector)} title="Inspector (I)">I</button>
        </div>

        <button className="ue-btn small strong" title="Capture Screenshot" onClick={() => void captureFullPage()}><CameraIcon /></button>
        {app.startup?.isAdmin && (
          <button className="ue-btn small" title="DB Logs (Admin)" onClick={() => app.setDbLogsOpen(true)} style={{ fontSize: 10, letterSpacing: '0.04em' }}>LOGS</button>
        )}
        <button className="ue-btn small" title="Account" onClick={() => app.setAccountOpen(true)}><AccountIcon /></button>
        <button className="ue-btn small" title="Settings" onClick={() => app.setSettingsOpen(true)}><SettingsIcon /></button>

        <div className="menu-clock" style={{ fontSize: 'var(--fs-9)', opacity: 0.6, marginLeft: 'var(--sp-2)', minWidth: 60 }}>{clock}</div>
      </div>
    </header>
  );
}