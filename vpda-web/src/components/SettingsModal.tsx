import { DEFAULT_THEME } from '../data/useVpdaApp';
import type { ThemeSettings, VpdaAppState } from '../data/useVpdaApp';
import ColorControl from './ColorControl';

type Props = { app: VpdaAppState };

const PRESETS: Array<{ label: string; theme: Partial<ThemeSettings> }> = [
  {
    label: 'Night',
    theme: {
      mode: 'dark',
      chartBackground: '#0D0D0F',
      chartText: '#55555F',
      bullColor: '#3DCF6E',
      bearColor: '#E04545',
      bullWickColor: '#2D9F55',
      bearWickColor: '#B03030',
      gridColor: '#14141A33',
      crosshairColor: '#3A3A44',
      accentColor: '#F0A030',
      heatmapColor: '#3A8FD0',
    },
  },
  {
    label: 'Void',
    theme: {
      mode: 'dark',
      chartBackground: '#050507',
      chartText: '#444450',
      bullColor: '#00D4AA',
      bearColor: '#FF4466',
      bullWickColor: '#009980',
      bearWickColor: '#CC2244',
      gridColor: '#0E0E1233',
      crosshairColor: '#2F3D46',
      accentColor: '#00D4AA',
      heatmapColor: '#6655DD',
    },
  },
  {
    label: 'Steel',
    theme: {
      mode: 'dark',
      chartBackground: '#0E1016',
      chartText: '#506070',
      bullColor: '#4DB8FF',
      bearColor: '#FF6633',
      bullWickColor: '#2288CC',
      bearWickColor: '#CC4411',
      gridColor: '#13182033',
      crosshairColor: '#3F5468',
      accentColor: '#4DB8FF',
      heatmapColor: '#8855FF',
    },
  },
  {
    label: 'Dawn',
    theme: {
      mode: 'light',
      chartBackground: '#F5F1E8',
      chartText: '#61584C',
      bullColor: '#2B9A57',
      bearColor: '#CF4D4D',
      bullWickColor: '#1F7D45',
      bearWickColor: '#A33A3A',
      gridColor: '#D8D0C233',
      crosshairColor: '#9E927F',
      accentColor: '#B9771F',
      heatmapColor: '#5487C7',
    },
  },
  {
    label: 'Paper',
    theme: {
      mode: 'light',
      chartBackground: '#FAF8F3',
      chartText: '#7A7060',
      bullColor: '#1E8A47',
      bearColor: '#C03535',
      bullWickColor: '#156634',
      bearWickColor: '#902020',
      gridColor: '#E8E2D533',
      crosshairColor: '#B2A48F',
      accentColor: '#C87C20',
      heatmapColor: '#4A7AB5',
    },
  },
];

function normalizeHexColor(value: string): string {
  if (/^#[0-9A-Fa-f]{6}$/.test(value)) return value.toUpperCase();
  if (/^#[0-9A-Fa-f]{8}$/.test(value)) return value.slice(0, 7).toUpperCase();
  return '#888888';
}

function withGridAlpha(hex: string): string {
  return `${normalizeHexColor(hex)}33`;
}

function RangeField({
  label,
  min,
  max,
  step,
  value,
  unit = '',
  onChange,
}: {
  label: string;
  min: number;
  max: number;
  step: number;
  value: number;
  unit?: string;
  onChange: (v: number) => void;
}) {
  return (
    <label className="settings-field">
      <span>{label}</span>
      <div className="range-row">
        <input
          type="range"
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={(e) => onChange(Number(e.target.value))}
        />
        <span className="range-value">
          {value}
          {unit}
        </span>
      </div>
    </label>
  );
}

function ColorField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <div className="settings-field">
      <span>{label}</span>
      <ColorControl
        value={value}
        onChange={onChange}
        ariaLabel={`${label} color`}
      />
    </div>
  );
}

const FONT_OPTIONS = [
  'Inter', 'Public Sans', 'IBM Plex Sans', 'Montserrat', 'Oswald', 
  'JetBrains Mono', 'IBM Plex Mono', 'Roboto Mono', 'Fira Code', 'Ubuntu Mono'
];

export default function SettingsModal({ app }: Props) {
  if (!app.settingsOpen) return null;

  const applyPreset = (preset: Partial<ThemeSettings>) => {
    const mode = preset.mode ?? app.theme.mode;
    const base = mode === 'light'
      ? { ...DEFAULT_THEME, mode: 'light' as const }
      : { ...DEFAULT_THEME, mode: 'dark' as const };

    app.setTheme({ ...base, ...preset });
  };

  return (
    <div
      className="modal-overlay"
      onClick={(e) => {
        if (e.target === e.currentTarget) app.setSettingsOpen(false);
      }}
    >
      <div className="modal-card settings-card info" style={{ padding: 'var(--sp-6)', maxWidth: 480 }}>
        <div className="modal-title-row" style={{ marginBottom: 'var(--sp-4)' }}>
          <h3 style={{ fontSize: 'var(--fs-14)', fontWeight: 600 }}>Preferences</h3>
          <button className="object-editor-close" onClick={() => app.setSettingsOpen(false)}>×</button>
        </div>

        <div className="settings-section">
          <div className="settings-section-title">Visual Themes</div>
          <div className="settings-preset-row" style={{ gap: 'var(--sp-2)', marginBottom: 'var(--sp-4)' }}>
            {PRESETS.map((preset) => (
              <button
                key={preset.label}
                className={`ue-btn small ${app.theme.mode === preset.theme.mode ? 'strong' : ''}`}
                onClick={() => applyPreset(preset.theme)}
                style={{ flex: 1 }}
              >
                {preset.label}
              </button>
            ))}
          </div>
        </div>

        <div className="settings-grid two-col" style={{ gap: 'var(--sp-4)', marginBottom: 'var(--sp-4)' }}>
          <label className="settings-field">
            <span>Interface Mode</span>
            <select
              value={app.theme.mode}
              onChange={(e) =>
                applyPreset(
                  e.target.value === 'light'
                    ? PRESETS.find((p) => p.label === 'Dawn')!.theme
                    : PRESETS.find((p) => p.label === 'Night')!.theme,
                )
              }
            >
              <option value="dark">Dark</option>
              <option value="light">Light</option>
            </select>
          </label>

          <RangeField
            label="Base Font Size"
            min={10}
            max={20}
            step={1}
            value={app.theme.fontSize}
            unit="px"
            onChange={(v) => app.setTheme((prev) => ({ ...prev, fontSize: v }))}
          />

          <label className="settings-field">
            <span>Font Family</span>
            <select
              value={app.theme.fontFamily}
              onChange={(e) => app.setTheme(prev => ({ ...prev, fontFamily: e.target.value }))}
            >
              {FONT_OPTIONS.map(f => <option key={f} value={f}>{f}</option>)}
            </select>
          </label>
        </div>

        <div className="settings-section-title">Color Configuration</div>
        <div className="settings-grid" style={{ gridTemplateColumns: 'repeat(2, 1fr)', gap: 'var(--sp-3)' }}>
          <div className="color-group">
            <div className="color-group-label" style={{ fontSize: 'var(--fs-8)', opacity: 0.6, marginBottom: 'var(--sp-1)' }}>Canvas</div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--sp-2)' }}>
              <ColorField
                label="Background"
                value={app.theme.chartBackground}
                onChange={(v) => app.setTheme((prev) => ({ ...prev, chartBackground: v }))}
              />
              <ColorField
                label="Text"
                value={app.theme.chartText}
                onChange={(v) => app.setTheme((prev) => ({ ...prev, chartText: v }))}
              />
              <ColorField
                label="Grid"
                value={app.theme.gridColor}
                onChange={(v) => app.setTheme((prev) => ({ ...prev, gridColor: withGridAlpha(v) }))}
              />
            </div>
          </div>

          <div className="color-group">
            <div className="color-group-label" style={{ fontSize: 'var(--fs-8)', opacity: 0.6, marginBottom: 'var(--sp-1)' }}>Candlesticks</div>
            <div style={{ display: 'flex', flexDirection: 'column', gap: 'var(--sp-2)' }}>
              <ColorField
                label="Bull body"
                value={app.theme.bullColor}
                onChange={(v) => app.setTheme((prev) => ({ ...prev, bullColor: v }))}
              />
              <ColorField
                label="Bull wick"
                value={app.theme.bullWickColor}
                onChange={(v) => app.setTheme((prev) => ({ ...prev, bullWickColor: v }))}
              />
              <ColorField
                label="Bear body"
                value={app.theme.bearColor}
                onChange={(v) => app.setTheme((prev) => ({ ...prev, bearColor: v }))}
              />
              <ColorField
                label="Bear wick"
                value={app.theme.bearWickColor}
                onChange={(v) => app.setTheme((prev) => ({ ...prev, bearWickColor: v }))}
              />
              <ColorField
                label="Accent"
                value={app.theme.accentColor}
                onChange={(v) => app.setTheme((prev) => ({ ...prev, accentColor: v }))}
              />
            </div>
          </div>
        </div>

        <div className="settings-section" style={{ marginTop: 'var(--sp-4)' }}>
          <div className="settings-section-title">Timezone</div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 'var(--sp-3)', marginTop: 'var(--sp-2)' }}>
            <span style={{ fontSize: 'var(--fs-9)', color: 'var(--text2)', minWidth: 64 }}>UTC offset</span>
            <select
              className="object-editor-select"
              value={app.theme.tzOffset ?? 0}
              onChange={(e) => app.setTheme(prev => ({ ...prev, tzOffset: Number(e.target.value) }))}
              style={{ flex: 1 }}
            >
              {[-12,-11,-10,-9,-8,-7,-6,-5,-4,-3,-2,-1,0,1,2,3,4,5,6,7,8,9,10,11,12,13,14].map(n => (
                <option key={n} value={n}>
                  {n === 0 ? 'UTC (0)' : n > 0 ? `UTC+${n}` : `UTC${n}`}
                  {n === 1 ? ' — London' : n === 2 ? ' — Frankfurt' : n === 3 ? ' — Moscow' : n === 5 ? ' — Karachi' : n === 8 ? ' — Singapore' : n === 9 ? ' — Tokyo' : n === -5 ? ' — New York' : n === -6 ? ' — Chicago' : n === -8 ? ' — Los Angeles' : ''}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="settings-section" style={{ marginTop: 'var(--sp-4)' }}>
          <div className="settings-section-title">Default Drawing Colors</div>
          <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 'var(--sp-2)', marginTop: 'var(--sp-2)' }}>
            {(['trend','rect','fib','hline','vline','text'] as const).map(type => {
              const stored = (() => { try { return JSON.parse(localStorage.getItem('vpda-web-type-colors-v1') ?? '{}'); } catch { return {}; } })();
              const cur = stored[type] ?? app.theme.accentColor;
              return (
                <div key={type} style={{ display: 'flex', alignItems: 'center', gap: 'var(--sp-2)' }}>
                  <span style={{ fontSize: 'var(--fs-9)', color: 'var(--text2)', flex: 1, textTransform: 'uppercase' }}>{type}</span>
                  <ColorControl value={cur} onChange={(v) => {
                    const next = { ...stored, [type]: v };
                    localStorage.setItem('vpda-web-type-colors-v1', JSON.stringify(next));
                  }} />
                </div>
              );
            })}
          </div>
        </div>

        <div className="settings-section" style={{ marginTop: 'var(--sp-4)' }}>
          <div className="settings-section-title">Keyboard Shortcuts</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: 'var(--sp-2)', fontSize: 'var(--fs-9)', color: 'var(--text2)' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}><span>Undo</span><kbd>Ctrl+Z</kbd></div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}><span>Redo</span><kbd>Ctrl+Y</kbd></div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}><span>Delete</span><kbd>Del / Backspace</kbd></div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}><span>Magnet Mode</span><kbd>Shift (Hold)</kbd></div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}><span>Multi Select</span><kbd>Ctrl / Cmd + Click</kbd></div>
            <div style={{ display: 'flex', justifyContent: 'space-between' }}><span>Deselect All</span><kbd>Esc</kbd></div>
          </div>
        </div>

        <div className="modal-actions" style={{ marginTop: 'var(--sp-6)', borderTop: '1px solid var(--border1)', paddingTop: 'var(--sp-4)' }}>
          <button
            className="ue-btn"
            onClick={() => app.setTheme(DEFAULT_THEME)}
          >
            Defaults
          </button>
          <div style={{ flex: 1 }} />
          <button
            className="ue-btn strong"
            onClick={() => app.setSettingsOpen(false)}
          >
            Apply & Close
          </button>
        </div>
      </div>
    </div>
  );
}
