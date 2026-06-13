import { useEffect, useMemo, useRef, useState } from 'react';
import { createPortal } from 'react-dom';
import {
  CandlestickSeries,
  ColorType,
  CrosshairMode,
  createChart,
} from 'lightweight-charts';
import ColorControl from './ColorControl';
import type { VpdaAppState } from '../data/useVpdaApp';
import { formatChartAnchorTime, formatChartTickMark, formatPrice, precisionForSymbol } from '../utils/format';
import { captureAppLayout } from '../utils/captureLayout';

type Props = {
  app: VpdaAppState;
};

type AnchorPoint = {
  time: number;
  price: number;
};

type TimeframeScope = '15M' | '1H' | '4H' | '1D' | '1W';

export type StrokeStyle = 'solid' | 'dashed' | 'dotted';
type LabelPosition = 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right' | 'center';
type PriceLabelAlign = 'left' | 'right';
type DrawingFilter = 'all' | 'trend' | 'rect' | 'hline' | 'vline' | 'fib' | 'text';
type ManagerTab = 'drawings' | 'snapshots' | 'replay';
type HandleKey = 'start' | 'end' | 'top' | 'bottom' | 'point' | 'price' | 'time';

type FibLevel = {
  id: string;
  value: number;
  color: string;
  visible: boolean;
  label: string;
};

type DrawingBase = {
  id: string;
  name: string;
  color: string;
  width: number;
  strokeStyle: StrokeStyle;
  labelFontSize: number;
  visible: boolean;
  locked: boolean;
  zLayer?: 'back' | 'front';
};

export type Drawing =
  | (DrawingBase & {
      type: 'trend';
      start: AnchorPoint;
      end: AnchorPoint;
      label: string;
      labelPosition: LabelPosition;
    })
  | (DrawingBase & {
      type: 'rect';
      start: AnchorPoint;
      end: AnchorPoint;
      label: string;
      labelPosition: LabelPosition;
      fillColor: string;
      fillAlpha: number;
      borderVisible: boolean;
    })
  | (DrawingBase & {
      type: 'hline';
      price: number;
      label: string;
    })
  | (DrawingBase & {
      type: 'vline';
      time: number;
      label: string;
    })
  | (DrawingBase & {
      type: 'fib';
      start: AnchorPoint;
      end: AnchorPoint;
      levels: FibLevel[];
      labelPosition: LabelPosition;
      priceLabelAlign: PriceLabelAlign;
      extendRight: boolean;
    })
  | {
      id: string;
      name: string;
      type: 'text';
      point: AnchorPoint;
      text: string;
      color: string;
      fontSize: number;
      visible: boolean;
      locked: boolean;
      zLayer?: 'back' | 'front';
    };

type StoredDrawing = Drawing & {
  sourceTimeframe: TimeframeScope;
};

type SelectionHit = { id: string };

type DragState = {
  origin: AnchorPoint;
  originLogical: number;
  snapshot: StoredDrawing[];
};

type HandleState = {
  drawingId: string;
  handle: HandleKey;
  snapshot: StoredDrawing;
};

type DrawingSnapshot = {
  id: string;
  note: string;
  createdAt: number;
  replayIndex: number | null;
  drawings: StoredDrawing[];
  thumbnail?: string;
};

type DrawingDefaults = {
  color: string;
  fillColor: string;
  fillAlpha: number;
  width: number;
  strokeStyle: StrokeStyle;
  fontSize: number;
  labelFontSize: number;
  labelPosition: LabelPosition;
  borderVisible: boolean;
  trendLabel: string;
  rectLabel: string;
  hlineLabel: string;
  vlineLabel: string;
  textValue: string;
  fibLevels: FibLevel[];
  fibLabelAlign: PriceLabelAlign;
  fibExtendRight: boolean;
};

type CachedDrawingDefaults = {
  color: string;
  fillColor: string;
  fillAlpha: number;
  width: number;
  strokeStyle: StrokeStyle;
  fontSize: number;
  labelFontSize: number;
  labelPosition: LabelPosition;
  borderVisible: boolean;
  fibLevels: FibLevel[];
  fibLabelAlign: PriceLabelAlign;
  fibExtendRight: boolean;
};

type HeatmapRow = {
  price: number;
  intensity: number;
  isMax: boolean;
  side: 'buy' | 'sell';
};

const STORAGE_NS = 'vpda-web-drawings-v3';
const DRAWING_DEFAULTS_KEY = 'vpda-web-drawing-defaults-v2';
const SNAPSHOTS_KEY = 'vpda-web-chart-snapshots-v1';
const MANAGER_OPEN_KEY = 'vpda-web-chart-manager-open-v1';
const TYPE_COLORS_KEY = 'vpda-web-type-colors-v1';
const FORECAST_DRAWING_PREFIX = '[FC]';

function getTypeColor(type: string, fallback: string): string {
  try {
    const raw = window.localStorage.getItem(TYPE_COLORS_KEY);
    if (!raw) return fallback;
    return (JSON.parse(raw) as Record<string, string>)[type] ?? fallback;
  } catch { return fallback; }
}

function makeId() {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
}

function clamp(value: number, min: number, max: number) {
  return Math.max(min, Math.min(max, value));
}

function withAlpha(hex: string, alpha: number): string {
  const a = Math.round(clamp(alpha, 0, 1) * 255)
    .toString(16)
    .padStart(2, '0');
  if (/^#[0-9a-fA-F]{6}$/.test(hex)) return `${hex}${a}`;
  return hex;
}

function isForecastDrawing(drawing: StoredDrawing): boolean {
  return drawing.name.startsWith(FORECAST_DRAWING_PREFIX);
}

function parseRangeFromText(text: string): [number, number] | null {
  const match = text.match(/(\d+(?:\.\d+)?)\s*[–-]\s*(\d+(?:\.\d+)?)/);
  if (!match) return null;
  const lower = Number(match[1]);
  const upper = Number(match[2]);
  if (!Number.isFinite(lower) || !Number.isFinite(upper)) return null;
  return lower <= upper ? [lower, upper] : [upper, lower];
}

function heatmapPalette(
  theme: VpdaAppState['theme'],
  side: HeatmapRow['side'],
  isMax: boolean,
) {
  if (isMax) {
    return {
      bar: theme.mode === 'light' ? '#111111' : '#f3f3f3',
      tick: theme.mode === 'light' ? '#000000' : '#ffffff',
      label: theme.mode === 'light' ? '#111111' : '#f3f3f3',
    };
  }

  if (side === 'buy') {
    return {
      bar: theme.mode === 'light' ? '#1f6f46' : '#4fd48a',
      tick: theme.mode === 'light' ? '#184f34' : '#7df0ab',
      label: theme.mode === 'light' ? '#1f6f46' : '#8cf3b7',
    };
  }

  return {
    bar: theme.mode === 'light' ? '#9a3939' : '#ff6b6b',
    tick: theme.mode === 'light' ? '#7c2d2d' : '#ff9a9a',
    label: theme.mode === 'light' ? '#9a3939' : '#ffb0b0',
  };
}

// ── Volume Profile heatmap ────────────────────────────────────────────────────
function buildVolumeProfile(
  candles: NonNullable<VpdaAppState['market']>['candles'],
  currentClose: number,
  bins = 48,
): HeatmapRow[] {
  const lookback = candles.slice(-Math.min(250, candles.length));
  if (lookback.length < 8) return [];

  const rangeTop = Math.max(...lookback.map((c) => c.high));
  const rangeBottom = Math.min(...lookback.map((c) => c.low));
  const range = rangeTop - rangeBottom;
  if (range === 0) return [];

  const step = range / bins;
  const volumes = new Array(bins).fill(0) as number[];

  for (const candle of lookback) {
    const vol = candle.volume || 1;
    const loIdx = clamp(Math.floor((candle.low - rangeBottom) / step), 0, bins - 1);
    const hiIdx = clamp(Math.ceil((candle.high - rangeBottom) / step), 0, bins - 1);
    const spanBins = Math.max(1, hiIdx - loIdx + 1);
    const volPerBin = vol / spanBins;
    for (let i = loIdx; i <= hiIdx && i < bins; i++) {
      volumes[i] += volPerBin;
    }
  }

  const maxVol = Math.max(...volumes, 1);
  const pocIdx = volumes.indexOf(maxVol);

  return volumes
    .map((vol, i) => ({
      price: rangeBottom + step * i + step / 2,
      intensity: vol / maxVol,
      isMax: i === pocIdx,
      side: rangeBottom + step * i + step / 2 <= currentClose
        ? ('buy' as const)
        : ('sell' as const),
    }))
    .filter((r) => r.intensity > 0.04);
}

// ── FVG formation-candle finder ───────────────────────────────────────────────
// Scans candle data to find which candle formed this exact FVG (lower/upper come
// from the same candle OHLC used to build the chart, so prices match exactly).
// Returns the Unix timestamp (seconds) of the impulse candle, or null if not found.
function findFvgFormationTime(
  candles: NonNullable<VpdaAppState['market']>['candles'],
  lower: number,
  upper: number,
): number | null {
  // Tolerance: 10% of FVG size to handle f64 → JSON → JS float conversions.
  const tol = Math.max(1e-6, Math.abs(upper - lower) * 0.1);

  for (let i = 0; i < candles.length; i++) {
    const c = candles[i];
    // 3-candle pattern: c = prev, c+1 = impulse, c+2 = post
    if (i + 2 < candles.length) {
      const post = candles[i + 2];
      // Bullish 3-candle FVG: prev.high ≈ lower, post.low ≈ upper
      if (Math.abs(c.high - lower) <= tol && Math.abs(post.low - upper) <= tol) {
        return candles[i + 1].time;
      }
      // Bearish 3-candle FVG: prev.low ≈ upper, post.high ≈ lower
      if (Math.abs(c.low - upper) <= tol && Math.abs(post.high - lower) <= tol) {
        return candles[i + 1].time;
      }
    }
    // 2-candle intraweek gap: c.high ≈ lower, next.low ≈ upper (bullish)
    if (i + 1 < candles.length) {
      const next = candles[i + 1];
      if (Math.abs(c.high - lower) <= tol && Math.abs(next.low - upper) <= tol) {
        return next.time;
      }
      // Bearish: next.high ≈ lower, c.low ≈ upper
      if (Math.abs(next.high - lower) <= tol && Math.abs(c.low - upper) <= tol) {
        return next.time;
      }
    }
  }
  return null;
}

// ── Coordinate helpers ────────────────────────────────────────────────────────
function averageStepSeconds(
  candles: NonNullable<VpdaAppState['market']>['candles'] | null | undefined,
): number {
  if (!candles || candles.length < 2) return 3600;
  let total = 0;
  for (let i = 1; i < candles.length; i++) {
    total += Math.max(1, candles[i].time - candles[i - 1].time);
  }
  return total / Math.max(1, candles.length - 1);
}

function shiftCandleTime(
  time: number,
  candles: NonNullable<VpdaAppState['market']>['candles'] | null | undefined,
  direction: -1 | 1,
  bars = 1,
): number {
  if (!candles || candles.length < 2) return time;
  let nearest = 0;
  for (let i = 1; i < candles.length; i++) {
    if (Math.abs(candles[i].time - time) < Math.abs(candles[nearest].time - time)) nearest = i;
  }
  return candles[Math.max(0, Math.min(candles.length - 1, nearest + direction * bars))].time;
}

function logicalFromTimeContinuous(
  targetTime: number,
  candles: NonNullable<VpdaAppState['market']>['candles'] | null | undefined,
): number | null {
  if (!candles?.length) return null;
  if (candles.length === 1) return 0;

  if (targetTime <= candles[0].time) {
    const step = Math.max(1, candles[1].time - candles[0].time);
    return (targetTime - candles[0].time) / step;
  }

  for (let i = 1; i < candles.length; i++) {
    const prev = candles[i - 1];
    const next = candles[i];
    if (targetTime <= next.time) {
      const span = Math.max(1, next.time - prev.time);
      return (i - 1) + (targetTime - prev.time) / span;
    }
  }

  const lastIndex = candles.length - 1;
  const step = Math.max(1, candles[lastIndex].time - candles[lastIndex - 1].time);
  return lastIndex + (targetTime - candles[lastIndex].time) / step;
}

function timeFromLogicalContinuous(
  logical: number,
  candles: NonNullable<VpdaAppState['market']>['candles'] | null | undefined,
): number | null {
  if (!candles?.length) return null;
  if (candles.length === 1) return candles[0].time;

  if (logical <= 0) {
    const step = Math.max(1, candles[1].time - candles[0].time);
    return Math.round(candles[0].time + logical * step);
  }

  const lastIndex = candles.length - 1;
  if (logical >= lastIndex) {
    const step = Math.max(1, candles[lastIndex].time - candles[lastIndex - 1].time);
    return Math.round(candles[lastIndex].time + (logical - lastIndex) * step);
  }

  const leftIndex = Math.floor(logical);
  const rightIndex = Math.min(lastIndex, leftIndex + 1);
  const left = candles[leftIndex];
  const right = candles[rightIndex];
  const frac = logical - leftIndex;
  return Math.round(left.time + (right.time - left.time) * frac);
}

function logicalToX(
  chart: ReturnType<typeof createChart>,
  logical: number,
): number | null {
  const x = chart.timeScale().logicalToCoordinate(logical as never);
  return x == null ? null : Number(x);
}

function xToLogical(
  chart: ReturnType<typeof createChart>,
  x: number,
): number | null {
  const logical = chart.timeScale().coordinateToLogical(x);
  return logical == null ? null : Number(logical);
}

function xFromTimeContinuous(
  chart: ReturnType<typeof createChart>,
  targetTime: number,
  candles: NonNullable<VpdaAppState['market']>['candles'] | null | undefined,
): number | null {
  const logical = logicalFromTimeContinuous(targetTime, candles);
  if (logical == null) return null;
  return logicalToX(chart, logical);
}

function toPoint(
  chart: ReturnType<typeof createChart>,
  series: any,
  x: number,
  y: number,
  candles?: NonNullable<VpdaAppState['market']>['candles'] | null,
): AnchorPoint | null {
  const logical = xToLogical(chart, x);
  const price = series.coordinateToPrice(y);
  if (logical == null || price == null) return null;
  const time = timeFromLogicalContinuous(logical, candles);
  if (time == null) return null;
  return { time, price: Number(price) };
}

function toCanvasPoint(
  chart: ReturnType<typeof createChart>,
  series: any,
  point: AnchorPoint,
  candles?: NonNullable<VpdaAppState['market']>['candles'] | null,
): { x: number; y: number } | null {
  const x = xFromTimeContinuous(chart, point.time, candles);
  const y = series.priceToCoordinate(point.price);
  if (x == null || y == null) return null;
  return { x, y };
}

function snapToMagnet(
  point: AnchorPoint,
  candles: NonNullable<VpdaAppState['market']>['candles'],
): AnchorPoint {
  if (!candles.length) return point;
  const nearest = candles.reduce((best, c) =>
    Math.abs(c.time - point.time) < Math.abs(best.time - point.time) ? c : best,
  candles[0]);
  const levels = [nearest.open, nearest.high, nearest.low, nearest.close];
  const price = levels.reduce((best, l) =>
    Math.abs(l - point.price) < Math.abs(best - point.price) ? l : best,
  levels[0]);
  return { ...point, price };
}

function storageKey(symbol: string, timeframe: string) {
  return `${STORAGE_NS}:${symbol}:${timeframe}`;
}

function normalizeTimeframeScope(timeframe: string): TimeframeScope {
  if (timeframe === '15M' || timeframe === '1H' || timeframe === '4H' || timeframe === '1D' || timeframe === '1W') return timeframe;
  return '1D';
}

function visibleTimeframes(timeframe: TimeframeScope): TimeframeScope[] {
  if (timeframe === '15M') return ['15M', '1H', '4H', '1D', '1W'];
  if (timeframe === '1H') return ['1H', '4H', '1D', '1W'];
  if (timeframe === '4H') return ['4H', '1D', '1W'];
  if (timeframe === '1D') return ['1D', '1W'];
  if (timeframe === '1W') return ['1W'];
  return ['1D'];
}

function stripStoredMeta(drawing: StoredDrawing): Drawing {
  const { sourceTimeframe: _sourceTimeframe, ...plain } = drawing;
  return plain;
}

function attachSourceTimeframe(drawings: Drawing[], sourceTimeframe: TimeframeScope): StoredDrawing[] {
  return drawings.map((drawing) => ({ ...drawing, sourceTimeframe }));
}

function loadScopedDrawings(symbol: string, timeframe: TimeframeScope): StoredDrawing[] {
  return visibleTimeframes(timeframe).flatMap((scope) => {
    try {
      const raw = window.localStorage.getItem(storageKey(symbol, scope));
      const parsed = raw ? (JSON.parse(raw) as Drawing[]) : [];
      return attachSourceTimeframe(parsed, scope);
    } catch {
      return [];
    }
  });
}

function saveScopedDrawings(
  symbol: string,
  timeframe: TimeframeScope,
  drawings: StoredDrawing[],
) {
  for (const scope of visibleTimeframes(timeframe)) {
    const scoped = drawings
      .filter((drawing) => drawing.sourceTimeframe === scope)
      .map(stripStoredMeta);
    window.localStorage.setItem(storageKey(symbol, scope), JSON.stringify(scoped));
  }
}

function normalizeStoredDrawing(drawing: StoredDrawing, index: number): StoredDrawing {
  const baseName = `${drawing.type.toUpperCase()} ${index + 1}`;
  if (drawing.type === 'text') {
    return {
      ...drawing,
      name: 'name' in drawing && drawing.name ? drawing.name : baseName,
      visible: 'visible' in drawing ? drawing.visible : true,
      locked: 'locked' in drawing ? drawing.locked : false,
    };
  }

  return {
    ...drawing,
    name: drawing.name || baseName,
    visible: drawing.visible ?? true,
    locked: drawing.locked ?? false,
    ...(drawing.type === 'fib'
      ? {
          priceLabelAlign: drawing.priceLabelAlign ?? 'right',
          extendRight: drawing.extendRight ?? false,
          levels: drawing.levels.map((level) => ({
            ...level,
            label: level.label ?? String(level.value),
          })),
        }
      : {}),
  };
}

function loadSnapshots(symbol: string, timeframe: TimeframeScope): DrawingSnapshot[] {
  try {
    const raw = window.localStorage.getItem(`${SNAPSHOTS_KEY}:${symbol}:${timeframe}`);
    return raw ? (JSON.parse(raw) as DrawingSnapshot[]) : [];
  } catch {
    return [];
  }
}

function saveSnapshots(symbol: string, timeframe: TimeframeScope, snapshots: DrawingSnapshot[]) {
  window.localStorage.setItem(
    `${SNAPSHOTS_KEY}:${symbol}:${timeframe}`,
    JSON.stringify(snapshots.slice(0, 12)),
  );
}

function makeDefaultFibLevels(color: string): FibLevel[] {
  return [
    { id: 'fib-0', value: 0, color, visible: true, label: '0' },
    { id: 'fib-236', value: 0.236, color, visible: true, label: '0.236' },
    { id: 'fib-382', value: 0.382, color, visible: true, label: '0.382' },
    { id: 'fib-500', value: 0.5, color, visible: true, label: '0.5' },
    { id: 'fib-618', value: 0.618, color, visible: true, label: '0.618' },
    { id: 'fib-786', value: 0.786, color, visible: true, label: '0.786' },
    { id: 'fib-1000', value: 1, color, visible: true, label: '1.0' },
  ];
}

function makeDefaultDrawingDefaults(accentColor: string): DrawingDefaults {
  return {
    color: accentColor,
    fillColor: accentColor,
    fillAlpha: 0.12,
    width: 1.5,
    strokeStyle: 'solid',
    fontSize: 12,
    labelFontSize: 11,
    labelPosition: 'top-right',
    borderVisible: true,
    trendLabel: '',
    rectLabel: '',
    hlineLabel: '',
    vlineLabel: '',
    textValue: 'Label',
    fibLevels: makeDefaultFibLevels(accentColor),
    fibLabelAlign: 'right',
    fibExtendRight: false,
  };
}

function cloneDrawingDefaults(defaults: DrawingDefaults): DrawingDefaults {
  return {
    ...defaults,
    fibLevels: defaults.fibLevels.map((level) => ({ ...level })),
  };
}

function sanitizeCachedDrawingDefaults(defaults: DrawingDefaults): CachedDrawingDefaults {
  return {
    color: defaults.color,
    fillColor: defaults.fillColor,
    fillAlpha: defaults.fillAlpha,
    width: defaults.width,
    strokeStyle: defaults.strokeStyle,
    fontSize: defaults.fontSize,
    labelFontSize: defaults.labelFontSize,
    labelPosition: defaults.labelPosition,
    borderVisible: defaults.borderVisible,
    fibLevels: defaults.fibLevels.map((level) => ({ ...level })),
    fibLabelAlign: defaults.fibLabelAlign,
    fibExtendRight: defaults.fibExtendRight,
  };
}

function applyCachedDefaults(
  current: DrawingDefaults,
  cached: Partial<CachedDrawingDefaults>,
): DrawingDefaults {
  return {
    ...current,
    ...cached,
    fibLevels:
      cached.fibLevels?.map((level) => ({
        ...level,
        label: level.label ?? String(level.value),
      })) ?? current.fibLevels.map((level) => ({ ...level })),
    fibLabelAlign: cached.fibLabelAlign ?? current.fibLabelAlign,
    fibExtendRight: cached.fibExtendRight ?? current.fibExtendRight,
    trendLabel: '',
    rectLabel: '',
    hlineLabel: '',
    vlineLabel: '',
    textValue: 'Label',
  };
}

function loadDrawingDefaults(accentColor: string): DrawingDefaults {
  const fallback = makeDefaultDrawingDefaults(accentColor);
  try {
    const raw = window.localStorage.getItem(DRAWING_DEFAULTS_KEY);
    if (!raw) return fallback;
    return applyCachedDefaults(fallback, JSON.parse(raw) as Partial<CachedDrawingDefaults>);
  } catch {
    return fallback;
  }
}

function saveDrawingDefaults(defaults: DrawingDefaults) {
  window.localStorage.setItem(
    DRAWING_DEFAULTS_KEY,
    JSON.stringify(sanitizeCachedDrawingDefaults(defaults)),
  );
}

function defaultsFromDrawing(drawing: StoredDrawing): DrawingDefaults {
  const base = makeDefaultDrawingDefaults('color' in drawing ? drawing.color : '#f0a030');

  if (drawing.type === 'text') {
    return {
      ...base,
      color: drawing.color,
      fontSize: drawing.fontSize,
      textValue: drawing.text,
    };
  }

  const withShared = {
    ...base,
    color: drawing.color,
    width: drawing.width,
    strokeStyle: drawing.strokeStyle,
    labelFontSize: drawing.labelFontSize,
  };

  if (drawing.type === 'trend') {
    return {
      ...withShared,
      labelPosition: drawing.labelPosition,
      trendLabel: drawing.label,
    };
  }

  if (drawing.type === 'rect') {
    return {
      ...withShared,
      labelPosition: drawing.labelPosition,
      rectLabel: drawing.label,
      fillColor: drawing.fillColor,
      fillAlpha: drawing.fillAlpha,
      borderVisible: drawing.borderVisible,
    };
  }

  if (drawing.type === 'hline') {
    return {
      ...withShared,
      hlineLabel: drawing.label,
    };
  }

  if (drawing.type === 'vline') {
    return {
      ...withShared,
      vlineLabel: drawing.label,
    };
  }

  return {
    ...withShared,
    labelPosition: drawing.labelPosition,
    fibLevels: drawing.levels.map((level) => ({ ...level })),
    fibLabelAlign: drawing.priceLabelAlign ?? 'right',
    fibExtendRight: drawing.extendRight ?? false,
  };
}

function setStrokeDash(ctx: CanvasRenderingContext2D, style: StrokeStyle, width: number) {
  if (style === 'dashed') ctx.setLineDash([8 * width, 5 * width]);
  else if (style === 'dotted') ctx.setLineDash([2 * width, 4 * width]);
  else ctx.setLineDash([]);
}

function distanceToSegment(
  pt: { x: number; y: number },
  a: { x: number; y: number },
  b: { x: number; y: number },
) {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  if (dx === 0 && dy === 0) return Math.hypot(pt.x - a.x, pt.y - a.y);
  const t = clamp(((pt.x - a.x) * dx + (pt.y - a.y) * dy) / (dx * dx + dy * dy), 0, 1);
  return Math.hypot(pt.x - (a.x + t * dx), pt.y - (a.y + t * dy));
}

function labelPoint(
  s: { x: number; y: number },
  e: { x: number; y: number },
  pos: LabelPosition,
) {
  const left = Math.min(s.x, e.x);
  const right = Math.max(s.x, e.x);
  const top = Math.min(s.y, e.y);
  const bottom = Math.max(s.y, e.y);
  const pad = 6;
  switch (pos) {
    case 'top-left':     return { x: left + pad,  y: top + 13,    align: 'left'   as CanvasTextAlign };
    case 'top-right':    return { x: right - pad, y: top + 13,    align: 'right'  as CanvasTextAlign };
    case 'bottom-left':  return { x: left + pad,  y: bottom - 5,  align: 'left'   as CanvasTextAlign };
    case 'bottom-right': return { x: right - pad, y: bottom - 5,  align: 'right'  as CanvasTextAlign };
    default:             return { x: (left + right) / 2, y: (top + bottom) / 2, align: 'center' as CanvasTextAlign };
  }
}

function hexAlphaToRgba(hex: string, alpha: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return `rgba(${r},${g},${b},${clamp(alpha, 0, 1).toFixed(2)})`;
}

function isValidHex(color: string): boolean {
  return /^#[0-9a-fA-F]{6}$/.test(color);
}

// ── Component ─────────────────────────────────────────────────────────────────
export type MgrPanelProps = {
  app: VpdaAppState;
  drawings: StoredDrawing[];
  setDrawings: React.Dispatch<React.SetStateAction<StoredDrawing[]>>;
  selectedIds: string[];
  setSelectedIds: React.Dispatch<React.SetStateAction<string[]>>;
  setEditorOpen: (open: boolean) => void;
  replayIndex: number | null;
  setReplayIndex: React.Dispatch<React.SetStateAction<number | null>>;
  setReplayEnabled: (enabled: boolean) => void;
  setReplayPlaying: (playing: boolean) => void;
  snapshots: DrawingSnapshot[];
  setSnapshotsState: React.Dispatch<React.SetStateAction<DrawingSnapshot[]>>;
  snapshotNote: string;
  setSnapshotNote: (note: string) => void;
  currentScope: any;
  undoRef: React.MutableRefObject<StoredDrawing[][]>;
  redoRef: React.MutableRefObject<StoredDrawing[][]>;
  drawingsRef: React.MutableRefObject<StoredDrawing[]>;
};

export function MgrPanel({
  app,
  drawings,
  setDrawings,
  selectedIds,
  setSelectedIds,
  setEditorOpen,
  replayIndex,
  setReplayIndex,
  setReplayEnabled,
  setReplayPlaying,
  snapshots,
  setSnapshotsState,
  snapshotNote,
  setSnapshotNote,
  currentScope,
  undoRef,
  redoRef,
  drawingsRef,
}: MgrPanelProps) {
  const [managerTab, setManagerTab] = useState<ManagerTab>('drawings');
  const [drawingFilter, setDrawingFilter] = useState<DrawingFilter>('all');

  const filteredDrawings = useMemo(() => {
    if (drawingFilter === 'all') return drawings;
    return drawings.filter((drawing) => drawing.type === drawingFilter);
  }, [drawings, drawingFilter]);

  return (
    <aside className="drawings-manager-panel" style={{ 
      width: 260, 
      borderLeft: '1px solid var(--border1)', 
      background: 'var(--bg1)',
      display: 'flex',
      flexDirection: 'column'
    }}>
      <div className="panel-title" style={{ padding: 'var(--sp-2) var(--sp-3)', fontSize: 'var(--fs-10)', fontWeight: 600, textTransform: 'uppercase', color: 'var(--text2)', background: 'var(--bg2)', borderBottom: '1px solid var(--border1)' }}>Chart Manager</div>
      <div className="drawings-manager-tabs" style={{ padding: 'var(--sp-2)', gap: 'var(--sp-1)', display: 'flex', background: 'var(--bg2)' }}>
        <button
          className={`ue-btn small ${managerTab === 'drawings' ? 'active' : ''}`}
          style={{ flex: 1 }}
          onClick={() => setManagerTab('drawings')}
        >
          DRAW
        </button>
        <button
          className={`ue-btn small ${managerTab === 'snapshots' ? 'active' : ''}`}
          style={{ flex: 1 }}
          onClick={() => setManagerTab('snapshots')}
        >
          SNAP
        </button>
        <button
          className={`ue-btn small ${managerTab === 'replay' ? 'active' : ''}`}
          style={{ flex: 1 }}
          onClick={() => setManagerTab('replay')}
        >
          PLY
        </button>
      </div>

      {managerTab === 'drawings' && (
        <div className="drawings-manager-body" style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          <div style={{ padding: 'var(--sp-2) var(--sp-3)', borderBottom: '1px solid var(--border1)' }}>
            <label className="object-editor-field" style={{ gridTemplateColumns: '50px 1fr' }}>
              <span style={{ fontSize: 'var(--fs-8)' }}>Filter</span>
              <select
                style={{ height: 22, fontSize: 'var(--fs-9)' }}
                value={drawingFilter}
                onChange={(e) => setDrawingFilter(e.target.value as DrawingFilter)}
              >
                <option value="all">ALL TYPES</option>
                <option value="trend">TREND LINE</option>
                <option value="rect">RECTANGLE</option>
                <option value="hline">HORIZON LINE</option>
                <option value="vline">VERT LINE</option>
                <option value="fib">FIBONACCI</option>
                <option value="text">TEXT LABEL</option>
              </select>
            </label>
          </div>

          <div className="drawing-list" style={{ flex: 1, overflowY: 'auto', padding: 'var(--sp-2)' }}>
            {filteredDrawings.length === 0 ? (
              <div style={{ padding: 'var(--sp-6)', textAlign: 'center', opacity: 0.4, fontSize: 'var(--fs-9)' }}>No drawings found</div>
            ) : (
              filteredDrawings.map((drawing, index) => (
                <div
                  key={drawing.id}
                  className={`drawing-row ${selectedIds.includes(drawing.id) ? 'active' : ''} ${!drawing.visible ? 'muted' : ''}`}
                  onClick={(e) => {
                    if (e.ctrlKey || e.metaKey) {
                      setSelectedIds(prev => 
                        prev.includes(drawing.id) ? prev.filter(id => id !== drawing.id) : [...prev, drawing.id]
                      );
                    } else {
                      setSelectedIds([drawing.id]);
                    }
                  }}
                >
                  <div className="drawing-row-main">
                    <div className="drawing-row-name">{drawing.name}</div>
                    <div className="drawing-row-meta">{drawing.type.toUpperCase()}</div>
                  </div>
                  <div className="drawing-manager-actions" style={{ display: 'flex', gap: '2px' }}>
                  <button
                    className="ue-btn small"
                    title="Move Up"
                    disabled={index === 0}
                    onClick={(e) => {
                      e.stopPropagation();
                      setDrawings((prev) => {
                        const idx = prev.findIndex(d => d.id === drawing.id);
                        if (idx <= 0) return prev;
                        const next = [...prev];
                        [next[idx - 1], next[idx]] = [next[idx], next[idx - 1]];
                        return next;
                      });
                    }}
                  >
                    <ChevronUpIcon />
                  </button>
                  <button
                    className="ue-btn small"
                    title="Move Down"
                    disabled={index === filteredDrawings.length - 1}
                    onClick={(e) => {
                      e.stopPropagation();
                      setDrawings((prev) => {
                        const idx = prev.findIndex(d => d.id === drawing.id);
                        if (idx === -1 || idx === prev.length - 1) return prev;
                        const next = [...prev];
                        [next[idx], next[idx + 1]] = [next[idx + 1], next[idx]];
                        return next;
                      });
                    }}
                  >
                    <ChevronDownIcon />
                  </button>
                  <button
                    className={`ue-btn small ${!drawing.visible ? 'active' : ''}`}
                    title={drawing.visible ? 'Hide' : 'Show'}
                    onClick={(e) => {
                      e.stopPropagation();
                      setDrawings((prev) =>
                        prev.map((d) => (d.id === drawing.id ? { ...d, visible: !d.visible } : d)),
                      );
                    }}
                  >
                    {drawing.visible ? <EyeIcon /> : <EyeOffIcon />}
                  </button>
                  <button
                    className={`ue-btn small ${drawing.locked ? 'active' : ''}`}
                    title={drawing.locked ? 'Unlock' : 'Lock'}
                    onClick={(e) => {
                      e.stopPropagation();
                      setDrawings((prev) =>
                        prev.map((d) => (d.id === drawing.id ? { ...d, locked: !d.locked } : d)),
                      );
                    }}
                  >
                    {drawing.locked ? <LockIcon /> : <UnlockIcon />}
                  </button>
                  <button
                    className="ue-btn small danger"
                    title="Delete"
                    onClick={(e) => {
                      e.stopPropagation();
                      undoRef.current.push(drawingsRef.current.map((item) => ({ ...item })));
                      redoRef.current = [];
                      setDrawings((prev) => prev.filter((item) => item.id !== drawing.id));
                      setSelectedIds((prev) => prev.filter((id) => id !== drawing.id));
                    }}
                  >
                    <TrashIcon />
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
      )}

      {managerTab === 'snapshots' && (
        <div className="drawings-manager-body" style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
          <div style={{ padding: 'var(--sp-2) var(--sp-3)', borderBottom: '1px solid var(--border1)' }}>
            <label className="object-editor-field" style={{ gridTemplateColumns: '50px 1fr' }}>
              <span style={{ fontSize: 'var(--fs-8)' }}>Note</span>
              <div style={{ display: 'flex', gap: '2px' }}>
                <input
                  style={{ height: 22, fontSize: 'var(--fs-9)', flex: 1 }}
                  value={snapshotNote}
                  onChange={(e) => setSnapshotNote(e.target.value)}
                  placeholder="Enter note..."
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && snapshotNote.trim()) {
                      const nextSnapshots = [
                        ...snapshots,
                        {
                          id: makeId(),
                          note: snapshotNote,
                          createdAt: Date.now(),
                          drawings: drawingsRef.current.map((d) => structuredClone(d)),
                          replayIndex,
                        },
                      ];
                      setSnapshotsState(nextSnapshots);
                      saveSnapshots(app.symbol, app.timeframe as any, nextSnapshots);
                      setSnapshotNote('');
                      app.showToast('Snapshot saved');
                    }
                  }}
                />
                <button 
                  className="ue-btn small strong"
                  disabled={!snapshotNote.trim()}
                  onClick={() => {
                    const nextSnapshots = [
                      ...snapshots,
                      {
                        id: makeId(),
                        note: snapshotNote,
                        createdAt: Date.now(),
                        drawings: drawingsRef.current.map((d) => structuredClone(d)),
                        replayIndex,
                      },
                    ];
                    setSnapshotsState(nextSnapshots);
                    saveSnapshots(app.symbol, app.timeframe as any, nextSnapshots);
                    setSnapshotNote('');
                    app.showToast('Snapshot saved');
                  }}
                >
                  SAVE
                </button>
              </div>
            </label>
          </div>
          <div className="snapshot-list" style={{ flex: 1, overflowY: 'auto', padding: 'var(--sp-2)' }}>
            {snapshots.length === 0 ? (
              <div style={{ padding: 'var(--sp-6)', textAlign: 'center', opacity: 0.4, fontSize: 'var(--fs-9)' }}>No snapshots yet</div>
            ) : (
              snapshots.map((snapshot) => (
                <div key={snapshot.id} className="drawing-row">
                  <div className="drawing-row-main">
                    <div className="drawing-row-name">{snapshot.note}</div>
                    <div className="drawing-row-meta" style={{ fontSize: 'var(--fs-7)' }}>
                      {new Date(snapshot.createdAt).toLocaleString([], { dateStyle: 'short', timeStyle: 'short' })}
                    </div>
                  </div>
                  <div className="drawing-manager-actions" style={{ display: 'flex', gap: '2px' }}>
                    <button
                      className="ue-btn small"
                      title="Restore"
                      onClick={() => {
                        undoRef.current.push(drawingsRef.current.map((drawing) => ({ ...drawing })));
                        redoRef.current = [];
                        setDrawings(snapshot.drawings.map((drawing) => structuredClone(drawing)));
                        setReplayIndex(snapshot.replayIndex);
                        setReplayEnabled(snapshot.replayIndex != null);
                        setSelectedIds([]);
                        setEditorOpen(false);
                        app.showToast('Snapshot restored');
                      }}
                    >
                      RESTORE
                    </button>
                    <button
                      className="ue-btn small danger"
                      title="Delete"
                      onClick={() => {
                        const nextSnapshots = snapshots.filter((item) => item.id !== snapshot.id);
                        setSnapshotsState(nextSnapshots);
                        saveSnapshots(app.symbol, app.timeframe as any, nextSnapshots);
                      }}
                    >
                      <TrashIcon />
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {managerTab === 'replay' && (
        <div className="drawings-manager-body" style={{ padding: 'var(--sp-3)' }}>
          <label className="object-editor-field">
            <span>Enabled</span>
            <input
              type="checkbox"
              checked={app.replayEnabled}
              onChange={(e) => {
                setReplayEnabled(e.target.checked);
                if (e.target.checked && replayIndex == null) {
                  setReplayIndex(Math.max(0, (app.market?.candles.length ?? 0) - 20));
                }
              }}
            />
          </label>
          <label className="object-editor-field" style={{ marginTop: 'var(--sp-2)' }}>
            <span>Index</span>
            <input
              type="number"
              style={{ width: 60 }}
              value={replayIndex ?? 0}
              onChange={(e) => {
                setReplayEnabled(true);
                setReplayIndex(Number(e.target.value));
              }}
            />
          </label>
        </div>
      )}
    </aside>
  );
}

export default function ChartViewport({ app }: Props) {
  // DOM refs
  const hostRef       = useRef<HTMLDivElement | null>(null);
  const surfaceRef    = useRef<HTMLDivElement | null>(null);
  const overlayRef    = useRef<HTMLCanvasElement | null>(null);
  const backCanvasRef = useRef<HTMLCanvasElement | null>(null);

  // Chart refs
  const chartRef  = useRef<ReturnType<typeof createChart> | null>(null);
  const seriesRef = useRef<any>(null);

  // History
  const undoRef = useRef<StoredDrawing[][]>([]);
  const redoRef = useRef<StoredDrawing[][]>([]);
  const clipboardRef = useRef<StoredDrawing | null>(null);

  // Save/load race prevention: suppresses the save effect right after a load
  const suppressSaveRef  = useRef(true);
  const hydratedKeyRef   = useRef<string | null>(null);

  // Drawing property memory – persists last-used settings across drawings
  const defaultsRef = useRef<DrawingDefaults>(loadDrawingDefaults(app.theme.accentColor));

  // State
  const [drawings,     setDrawings]     = useState<StoredDrawing[]>([]);
  const [selectedIds,  setSelectedIds]  = useState<string[]>([]);
  const [editorOpen,   setEditorOpen]   = useState(false);
  const [templateNameInput, setTemplateNameInput] = useState('');
  const { managerOpen, setManagerOpen } = app;
  const [managerTab,   setManagerTab]   = useState<ManagerTab>('drawings');
  const [drawingFilter, setDrawingFilter] = useState<DrawingFilter>('all');
  const [snapshots, setSnapshotsState] = useState<DrawingSnapshot[]>([]);
  const [snapshotNote, setSnapshotNote] = useState('');
  const [draftStart,   setDraftStart]   = useState<AnchorPoint | null>(null);
  const [draftCurrent, setDraftCurrent] = useState<AnchorPoint | null>(null);
  const [dragState,    setDragState]    = useState<DragState | null>(null);
  const [handleState, setHandleState] = useState<HandleState | null>(null);
  const [replayIndex, setReplayIndex] = useState<number | null>(null);
  const [replayPlaying, setReplayPlaying] = useState(false);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const { replayEnabled, setReplayEnabled } = app;
  const [panelDrag, setPanelDrag] = useState<{
    kind: 'editor';
    pointerId: number;
    offsetX: number;
    offsetY: number;
  } | null>(null);
  const [editorPanelPos, setEditorPanelPos] = useState({ x: 52, y: 8 });
  const [editorPanelSize, setEditorPanelSize] = useState({ width: 220, height: 400 });
  const [panelResize, setPanelResize] = useState<{
    pointerId: number;
    startWidth: number;
    startHeight: number;
    startX: number;
    startY: number;
  } | null>(null);
  const [managerPortalTarget, setManagerPortalTarget] = useState<HTMLElement | null>(null);
  const [contextMenu, setContextMenu] = useState<{ x: number, y: number, drawingId: string | null } | null>(null);
  const [floatingMenu, setFloatingMenu] = useState<{ type: 'templates', x: number, y: number } | null>(null);

  // ── RDS (Red Shift) tool state ─────────────────────────────────────────────
  const [rdsStep, setRdsStep]             = useState<'pick-x' | 'pick-y' | 'pick-dir' | null>(null);
  const [rdsX,    setRdsX]               = useState<number | null>(null);
  const [rdsY,    setRdsY]               = useState<number | null>(null);
  const [rdsCursorPrice, setRdsCursorPrice] = useState<number | null>(null);
  const lastForecastRequestIdRef = useRef<number | null>(null);


  const onSaveTemplate = (name: string, drawing: StoredDrawing) => {
    if (!name.trim()) return;
    const templateKey = `${drawing.type}:${name.trim()}`;
    const style: any = {
      color: drawing.color,
      width: 'width' in drawing ? (drawing as any).width : 1,
      strokeStyle: 'strokeStyle' in drawing ? (drawing as any).strokeStyle : 'solid',
      labelFontSize: 'labelFontSize' in drawing ? (drawing as any).labelFontSize : 11,
    };
    
    // Save text/label content
    if ('label' in drawing) style.label = (drawing as any).label;
    if ('text' in drawing) style.text = (drawing as any).text;

    if (drawing.type === 'rect') {
      style.fillColor = drawing.fillColor;
      style.fillAlpha = drawing.fillAlpha;
      style.borderVisible = drawing.borderVisible;
      style.labelPosition = drawing.labelPosition;
    } else if (drawing.type === 'trend') {
      style.labelPosition = drawing.labelPosition;
    } else if (drawing.type === 'fib') {
      style.levels = drawing.levels;
      style.priceLabelAlign = drawing.priceLabelAlign;
      style.extendRight = drawing.extendRight;
      style.labelPosition = drawing.labelPosition;
    } else if (drawing.type === 'text') {
      style.fontSize = drawing.fontSize;
    }
    app.setDrawingTemplates((prev: Record<string, any>) => ({ ...prev, [templateKey]: style }));
    setTemplateNameInput('');
    app.showToast(`Saved template "${name.trim()}"`);
  };

  const onApplyTemplate = (templateKey: string, drawingId: string) => {
    const style = app.drawingTemplates?.[templateKey];
    if (!style) return;
    setDrawings((prev) =>
      prev.map((d) => (d.id === drawingId ? { ...d, ...style } : d)),
    );
    app.showToast(`Applied template "${templateKey.split(':')[1]}"`);
  };

  const getTemplatesForType = (type: string) => {
    return Object.keys(app.drawingTemplates ?? {}).filter(k => k.startsWith(`${type}:`));
  };

  useEffect(() => {
    window.localStorage.setItem(MANAGER_OPEN_KEY, String(managerOpen));
  }, [managerOpen]);

  useEffect(() => {
    if (!managerOpen) {
      setManagerPortalTarget(null);
      return;
    }
    const syncTarget = () => {
      setManagerPortalTarget(document.getElementById('manager-slot'));
    };
    syncTarget();
    const rafId = window.requestAnimationFrame(syncTarget);
    return () => window.cancelAnimationFrame(rafId);
  }, [managerOpen, app.showInspector, app.showWatchlist]);

  // Ref so the keyboard-handler useEffect (empty deps) can always call latest setter
  const setEditorOpenRef = useRef(setEditorOpen);
  setEditorOpenRef.current = setEditorOpen;

  useEffect(() => {
    app.setCanUndo(undoRef.current.length > 0);
    app.setCanRedo(redoRef.current.length > 0);
  }, [drawings, app]);

  useEffect(() => {
    const handleUndo = () => {
      const prev = undoRef.current.pop();
      if (!prev) return;
      redoRef.current.push(drawingsRef.current.map((drawing) => ({ ...drawing })));
      setDrawings(prev.map((drawing) => ({ ...drawing })));
      setSelectedIds([]);
    };
    const handleRedo = () => {
      const next = redoRef.current.pop();
      if (!next) return;
      undoRef.current.push(drawingsRef.current.map((drawing) => ({ ...drawing })));
      setDrawings(next.map((drawing) => ({ ...drawing })));
      setSelectedIds([]);
    };

    let undoBtn: HTMLElement | null = null;
    let redoBtn: HTMLElement | null = null;
    let saveBtn: HTMLElement | null = null;

    // Use a small timeout to ensure buttons are in DOM if they are part of MenuBar
    const timer = setTimeout(() => {
      undoBtn = document.getElementById('global-undo-btn');
      redoBtn = document.getElementById('global-redo-btn');
      saveBtn = document.getElementById('global-save-btn');
      undoBtn?.addEventListener('click', handleUndo);
      redoBtn?.addEventListener('click', handleRedo);
      saveBtn?.addEventListener('click', () => {
        const chartCanvas = hostRef.current?.querySelector('canvas') as HTMLCanvasElement | null;
        const thumbnail = chartCanvas ? (() => { try { return chartCanvas.toDataURL('image/jpeg', 0.35); } catch { return undefined; } })() : undefined;
        const snapshot: DrawingSnapshot = {
          id: makeId(),
          note: snapshotNote || `${app.symbol} ${app.timeframe}`,
          createdAt: Date.now(),
          replayIndex,
          drawings: drawingsRef.current.map((drawing) => structuredClone(drawing)),
          thumbnail,
        };
        const nextSnapshots = [snapshot, ...snapshots];
        setSnapshotsState(nextSnapshots);
        saveSnapshots(app.symbol, currentScope, nextSnapshots);
        setSnapshotNote('');
        setManagerTab('snapshots');
        app.setManagerOpen(true);
        app.showToast('Snapshot saved');
      });
    }, 100);

    return () => {
      clearTimeout(timer);
      undoBtn?.removeEventListener('click', handleUndo);
      redoBtn?.removeEventListener('click', handleRedo);
      saveBtn?.removeEventListener('click', () => {});
    };
  }, [app]);

  // Fullscreen toggle with correct API
  const toggleFullscreen = () => {
    if (!hostRef.current) return;

    if (!document.fullscreenElement) {
      // Request fullscreen on the host element directly
      const element = hostRef.current;
      if (element.requestFullscreen) {
        element.requestFullscreen()
          .then(() => setIsFullscreen(true))
          .catch(err => console.error('Fullscreen request failed:', err));
      } else {
        console.warn('Fullscreen API not supported');
      }
    } else {
      // Exit fullscreen
      document.exitFullscreen()
        .then(() => setIsFullscreen(false))
        .catch(err => console.error('Exit fullscreen failed:', err));
    }
  };

  // Keyboard shortcuts for fullscreen
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Exit fullscreen on ESC
      if (e.key === 'Escape' && document.fullscreenElement) {
        document.exitFullscreen();
        return;
      }

      // Timeframe shortcuts (1-5) only when in fullscreen
      if (isFullscreen && !e.ctrlKey && !e.metaKey && !e.altKey) {
        const timeframeMap: { [key: string]: any } = {
          '1': '15M',
          '2': '1H',
          '3': '4H',
          '4': '1D',
          '5': '1W',
        };

        if (e.key in timeframeMap) {
          e.preventDefault();
          app.setTimeframe(timeframeMap[e.key]);
        }
      }
    };

    // Track fullscreen changes
    const handleFullscreenChange = () => {
      setIsFullscreen(!!document.fullscreenElement);
    };

    window.addEventListener('keydown', handleKeyDown);
    document.addEventListener('fullscreenchange', handleFullscreenChange);

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('fullscreenchange', handleFullscreenChange);
    };
  }, [isFullscreen, app]);

  // Render-loop refs (avoid stale closures in the RAF loop)
  const drawingsRef     = useRef(drawings);    drawingsRef.current    = drawings;
  const selectedIdsRef  = useRef(selectedIds); selectedIdsRef.current = selectedIds;
  const draftStartRef   = useRef(draftStart);  draftStartRef.current  = draftStart;
  const draftCurrentRef = useRef(draftCurrent);draftCurrentRef.current= draftCurrent;
  const dragStateRef    = useRef(dragState);   dragStateRef.current   = dragState;
  const handleStateRef  = useRef(handleState); handleStateRef.current = handleState;
  const rdsStepRef      = useRef(rdsStep);      rdsStepRef.current      = rdsStep;
  const rdsXRef         = useRef(rdsX);          rdsXRef.current         = rdsX;
  const rdsCursorPriceRef = useRef(rdsCursorPrice); rdsCursorPriceRef.current = rdsCursorPrice;
  const showHeatmapRef  = useRef(app.showHeatmap); showHeatmapRef.current = app.showHeatmap;
  const themeRef        = useRef(app.theme);       themeRef.current       = app.theme;
  const symbolRef       = useRef(app.symbol);      symbolRef.current      = app.symbol;
  const drawToolRef     = useRef(app.drawTool);    drawToolRef.current    = app.drawTool;
  const allCandlesRef = useRef<NonNullable<VpdaAppState['market']>['candles']>([]);
  allCandlesRef.current = app.market?.candles ?? [];
  const marketCandlesRef = useRef<NonNullable<VpdaAppState['market']>['candles']>([]);
  marketCandlesRef.current =
    replayEnabled && replayIndex != null && app.market?.candles?.length
      ? app.market.candles.slice(0, Math.max(1, replayIndex + 1))
      : app.market?.candles ?? [];
  const previousAccentRef = useRef(defaultsRef.current.color);
  const heatmapRowsRef  = useRef<HeatmapRow[]>([]);

  // Computed
  const precision = useMemo(() => precisionForSymbol(app.symbol), [app.symbol]);
  const precisionRef = useRef(precision); precisionRef.current = precision;
  const currentScope = useMemo(
    () => normalizeTimeframeScope(app.timeframe),
    [app.timeframe],
  );

  const selectedDrawing = drawings.find((d) => d.id === selectedIds[0]) ?? null;
  const filteredDrawings = useMemo(() => {
    if (drawingFilter === 'all') return drawings;
    return drawings.filter((drawing) => drawing.type === drawingFilter);
  }, [drawings, drawingFilter]);

  const replayCandles = useMemo(() => {
    if (!app.market) return [];
    if (!replayEnabled || replayIndex == null) return app.market.candles;
    return app.market.candles.slice(0, Math.max(1, replayIndex + 1));
  }, [app.market, replayEnabled, replayIndex]);

  useEffect(() => {
    if (!replayEnabled || !app.market?.candles.length || replayIndex == null) return;
    const lastIndex = app.market.candles.length - 1;
    if (replayIndex > lastIndex) {
      setReplayIndex(lastIndex);
    }
  }, [app.market, replayEnabled, replayIndex]);

  useEffect(() => {
    if (!replayEnabled || !replayPlaying || !app.market?.candles.length) return;
    const timer = window.setInterval(() => {
      setReplayIndex((prev) => {
        if (prev == null) return 1;
        const next = prev + 1;
        const lastIndex = app.market!.candles.length - 1;
        if (next >= lastIndex) {
          window.clearInterval(timer);
          setReplayPlaying(false);
          return lastIndex;
        }
        return next;
      });
    }, 450);

    return () => window.clearInterval(timer);
  }, [app.market, replayEnabled, replayPlaying]);

  // Update heatmap ref when market data changes
  const heatmapRows = useMemo(() => {
    if (!replayCandles.length) return [];
    return buildVolumeProfile(replayCandles, replayCandles[replayCandles.length - 1].close);
  }, [replayCandles]);

  useEffect(() => {
    heatmapRowsRef.current = heatmapRows;
  }, [heatmapRows]);



  // Sync accent color to defaultsRef when theme changes only if the user has not customized it
  useEffect(() => {
    if (defaultsRef.current.color === previousAccentRef.current) {
      defaultsRef.current.color = app.theme.accentColor;
    }
    if (defaultsRef.current.fillColor === previousAccentRef.current) {
      defaultsRef.current.fillColor = app.theme.accentColor;
    }
    previousAccentRef.current = app.theme.accentColor;
    saveDrawingDefaults(defaultsRef.current);
  }, [app.theme.accentColor]);

  // ── Load drawings per symbol/timeframe ───────────────────────────────────
  useEffect(() => {
    suppressSaveRef.current = true;
    hydratedKeyRef.current = `${app.symbol}:${currentScope}`;
    undoRef.current = [];
    redoRef.current = [];
    setDrawings(loadScopedDrawings(app.symbol, currentScope).map(normalizeStoredDrawing).filter(d => !isForecastDrawing(d)));
    setSnapshotsState(loadSnapshots(app.symbol, currentScope));
    setSelectedIds([]);
    setDraftStart(null);
    setDraftCurrent(null);
    setDragState(null);
    setHandleState(null);
    setReplayEnabled(false);
    setReplayIndex(null);
    setReplayPlaying(false);
  }, [app.symbol, currentScope]);

  // ── Save drawings across timeframe scopes ────────────────────────────────
  useEffect(() => {
    if (suppressSaveRef.current) {
      suppressSaveRef.current = false;
      return;
    }
    if (hydratedKeyRef.current !== `${app.symbol}:${currentScope}`) return;
    saveScopedDrawings(app.symbol, currentScope, drawings.filter(d => !isForecastDrawing(d)));
  }, [app.symbol, currentScope, drawings]);

  useEffect(() => {
    const request = app.forecastDrawingRequest;
    const candles = marketCandlesRef.current;
    if (!request || request.symbol !== app.symbol) return;
    if (lastForecastRequestIdRef.current === request.id) return;

    if (request.kind === 'clear') {
      lastForecastRequestIdRef.current = request.id;
      commitDrawings((prev) => prev.filter((drawing) => !isForecastDrawing(drawing)));
      setSelectedIds([]);
      setEditorOpen(false);
      app.setManagerOpen(true);
      app.setForecastDrawingDebug({
        requestId: request.id,
        status: 'cleared',
        message: 'Forecast drawings cleared from chart',
      });
      return;
    }

    // ── Condition check ───────────────────────────────────────────────────────
    const hasCandles = candles.length > 0;
    const symbolReady = !app.market || app.market.symbol === request.symbol;

    console.log('[FC-Draw]', {
      kind: request.kind,
      symbol: request.symbol,
      appSymbol: app.symbol,
      appTimeframe: app.timeframe,
      marketSymbol: app.market?.symbol,
      candles: candles.length,
      hasCandles,
      symbolReady,
      overlays: request.kind === 'scenario' ? (request.scenario.chartOverlays?.length ?? 'none') : 'n/a',
      weeklyTarget: request.kind === 'scenario' ? request.scenario.weeklyTarget?.price : 'n/a',
    });

    // No timeframe / market-readiness gate — chart is always mounted.
    // Lines (hlines) need no candle data. Rects use candles if available, else fallback timestamp.

    // Compute time bounds for zones — fallback to daily step if no candles yet.
    const nowSec = Math.floor(Date.now() / 1000);
    const stepSecs = hasCandles ? averageStepSeconds(candles) : 86400;
    const lastCandle = hasCandles ? candles[candles.length - 1] : null;
    const baseTime   = lastCandle?.time ?? nowSec;
    // Zone left anchor = formation candle (looked up from candle data).
    // Zone right edge extends 500 bars forward — visually "infinite" on any chart.
    const zoneStartTime = baseTime - stepSecs * 1;
    const futureTime    = baseTime + stepSecs * 10000; // HACK: Extended to 10000 bars for FVG lines

    // ── Color helpers ──────────────────────────────────────────────────────────
    const overlayColor = (hint: string): string => {
      switch (hint) {
        case 'bullish':     return app.theme.bullColor;
        case 'bearish':     return app.theme.bearColor;
        case 'ifvg':        return app.theme.mode === 'light' ? '#303030' : '#e8e8e8';
        case 'target':      return app.theme.mode === 'light' ? '#111111' : '#ffffff';
        case 'extension':   return app.theme.mode === 'light' ? '#4a4a4a' : '#b0b0b0';
        case 'invalidation':return app.theme.mode === 'light' ? '#5f5f5f' : '#b8b8b8';
        default:            return app.theme.mode === 'light' ? '#111111' : '#ffffff';
      }
    };

    const forecastRect = (
      name: string,
      label: string,
      lower: number,
      upper: number,
      colorHint: string,
    ): StoredDrawing => {
      const color = overlayColor(colorHint);
      const fillAlpha = colorHint === 'ifvg' ? 0.08 : 0.18;
      return {
        id: makeId(),
        type: 'rect',
        name: `${FORECAST_DRAWING_PREFIX} ${name}`,
        visible: true,
        locked: false,
        color,
        width: 1.5,
        strokeStyle: 'solid',
        labelFontSize: 11,
        sourceTimeframe: currentScope,
        // Start at current time, extend to the right (future)
        start: { time: baseTime, price: lower },
        end:   { time: futureTime, price: upper },
        label,
        labelPosition: 'top-right',
        fillColor: color,
        fillAlpha,
        borderVisible: false,
        zLayer: 'front',
      };
    };

    const forecastLine = (
      name: string,
      price: number,
      label: string,
      colorHint: string,
      strokeStyle: StrokeStyle = 'dashed',
    ): StoredDrawing => ({
      id: makeId(),
      type: 'hline',
      name: `${FORECAST_DRAWING_PREFIX} ${name}`,
      visible: true,
      locked: false,
      price,
      label,
      color: overlayColor(colorHint),
      width: colorHint === 'target' ? 1.5 : 1,
      strokeStyle,
      labelFontSize: 11,
      sourceTimeframe: currentScope,
      zLayer: 'front',
    });

    const nextDrawings: StoredDrawing[] = [];
    if (request.kind === 'scenario') {
      const scenario = request.scenario;
      const overlays = scenario.chartOverlays ?? [];

      if (overlays.length > 0) {
        // ── Use structured backend overlays ────────────────────────────────────
        for (const ov of overlays) {
          if (ov.kind === 'zone' && ov.lower != null && ov.upper != null) {
            nextDrawings.push(forecastRect(
              `${scenario.name} ${ov.role}`,
              ov.label,
              ov.lower,
              ov.upper,
              ov.colorHint,
            ));
          } else if (ov.kind === 'line' && ov.price != null) {
            const strokeStyle: StrokeStyle =
              ov.role === 'invalidation' ? 'dotted' :
              ov.role === 'extension'    ? 'dashed'  : 'dashed';
            nextDrawings.push(forecastLine(
              `${scenario.name} ${ov.role}`,
              ov.price,
              ov.label,
              ov.colorHint,
              strokeStyle,
            ));
          }
        }
      } else {
        // ── Fallback: derive from legacy fields (VM / old scenarios) ───────────
        if (scenario.weeklyTarget) {
          nextDrawings.push(forecastLine(
            `${scenario.name} draw`,
            scenario.weeklyTarget.price,
            scenario.weeklyTarget.name,
            'target',
            'dashed',
          ));
        }
        const invalidation = [...(scenario.pathSteps ?? [])]
          .sort((a, b) => a.sequence - b.sequence)
          .find((step) => step.invalidation !== null)?.invalidation;
        if (invalidation != null) {
          nextDrawings.push(forecastLine(
            `${scenario.name} invalidation`,
            invalidation,
            `Invalidation ${formatPrice(invalidation, app.symbol)}`,
            'invalidation',
            'dotted',
          ));
        }
      }
    } else if (request.kind === 'levels') {
      request.levels.forEach((level) => {
        if (level.status === 'taken') return;
        nextDrawings.push(
          forecastLine(
            `Level ${level.name}`,
            level.price,
            level.name,
            'target',
            'dashed',
          ),
        );
      });
    }

    if (nextDrawings.length === 0) {
      lastForecastRequestIdRef.current = request.id;
      if (request.kind === 'scenario') {
        console.warn('[FC-Draw] EMPTY — scenario dump:', JSON.stringify({
          name: request.scenario.name,
          chartOverlays: request.scenario.chartOverlays,
          weeklyTarget: request.scenario.weeklyTarget,
          pathStepsCount: request.scenario.pathSteps?.length,
          pdConfluence: request.scenario.pdConfluence,
          manipulationTarget: request.scenario.manipulationTarget,
        }, null, 2));
      }
      app.setForecastDrawingDebug({
        requestId: request.id,
        status: 'empty',
        message: 'Chart request reached renderer, but no drawable forecast overlays were produced — check console',
      });
      app.showToast('No drawable forecast levels found — check browser console');
      return;
    }

    lastForecastRequestIdRef.current = request.id;
    commitDrawings((prev) => {
      const manual = prev.filter((drawing) => !isForecastDrawing(drawing));
      return [...manual, ...nextDrawings];
    });
    setSelectedIds(nextDrawings.map((drawing) => drawing.id));
    setEditorOpen(false);
    app.setManagerOpen(true);
    app.setForecastDrawingDebug({
      requestId: request.id,
      status: 'applied',
      message: `Applied ${nextDrawings.length} forecast drawing(s) to chart`,
    });
    app.showToast(`Applied ${nextDrawings.length} forecast drawing${nextDrawings.length === 1 ? '' : 's'}`);
  }, [app, app.forecastDrawingRequest, app.symbol, app.timeframe, app.market, currentScope]);

  // ── Create / destroy chart ────────────────────────────────────────────────
  useEffect(() => {
    if (!hostRef.current || !app.market) return undefined;

    // Insert back-layer canvas as first child of chart-host so it paints BEHIND the LW Charts canvas
    const backCanvas = document.createElement('canvas');
    backCanvas.style.cssText = 'position:absolute;inset:0;width:100%;height:100%;pointer-events:none;';
    hostRef.current.insertBefore(backCanvas, hostRef.current.firstChild ?? null);
    backCanvasRef.current = backCanvas;

    const chart = createChart(hostRef.current, {
      autoSize: true,
      layout: {
        background: { type: ColorType.Solid, color: 'rgba(0,0,0,0)' },
        textColor:  app.theme.chartText,
        fontFamily: "'Share Tech Mono', 'Consolas', monospace",
        fontSize:   Math.max(10, app.theme.fontSize),
        attributionLogo: false,
      },
      grid: {
        vertLines: { color: app.showGrid ? app.theme.gridColor : 'transparent' },
        horzLines: { color: app.showGrid ? app.theme.gridColor : 'transparent' },
      },
      crosshair: {
        mode: app.showCrosshair ? CrosshairMode.Normal : CrosshairMode.Hidden,
        vertLine: {
          color: app.theme.crosshairColor,
          width: 1,
          style: 0,
          labelBackgroundColor: app.theme.chartBackground,
        },
        horzLine: {
          color: app.theme.crosshairColor,
          width: 1,
          style: 0,
          labelBackgroundColor: app.theme.accentColor,
          labelVisible: true,
        },
      },
      rightPriceScale: {
        borderColor: app.theme.gridColor,
        scaleMargins: { top: 0.06, bottom: 0.18 },
      },
      timeScale: {
        borderColor:    app.theme.gridColor,
        timeVisible:    true,
        secondsVisible: false,
        barSpacing:     12,
        minBarSpacing:  2,
        rightOffset:    150,
        fixRightEdge:   false,
        tickMarkFormatter: (time: number, type: number) =>
          formatChartTickMark(time, type, app.timeframe),
      },
      localization: {
        priceFormatter: (price: number) => formatPrice(price, app.symbol),
        timeFormatter: (time: number) => formatChartAnchorTime(time),
      },
      handleScroll: { mouseWheel: true, pressedMouseMove: true },
      handleScale:  { mouseWheel: true, pinch: true, axisPressedMouseMove: true },
    });

    const series = chart.addSeries(CandlestickSeries, {
      upColor:         app.theme.bullColor,
      downColor:       app.theme.bearColor,
      borderUpColor:   app.theme.bullWickColor,
      borderDownColor: app.theme.bearWickColor,
      wickUpColor:     app.theme.bullWickColor,
      wickDownColor:   app.theme.bearWickColor,
      borderVisible:   true,
      wickVisible:     true,
      priceLineVisible:false,
      lastValueVisible:true,
      priceFormat: {
        type:     'price',
        precision,
        minMove: precision === 5 ? 0.00001 : precision === 3 ? 0.001 : 0.25,
      },
    });

    series.setData(
      replayCandles.map((c) => ({
        time:  c.time as never,
        open:  c.open,
        high:  c.high,
        low:   c.low,
        close: c.close,
      })),
    );

    chart.subscribeCrosshairMove((param) => {
      if (!param.time || !param.point) {
        app.setHoveredCandle(null);
        return;
      }
      const data = param.seriesData.get(series);
      if (data) app.setHoveredCandle(data);
    });

    // Show last ~60 candles (≈1 month on daily, ~2.5 days on 1H)
    const total = replayCandles.length;
    if (total > 0) {
      const showBars = Math.min(60, total);
      chart.timeScale().setVisibleLogicalRange({
        from: total - showBars - 1,
        to:   total + 24,
      });
    }

    chartRef.current  = chart;
    seriesRef.current = series;

    return () => {
      chartRef.current  = null;
      seriesRef.current = null;
      chart.remove();
      backCanvasRef.current?.remove();
      backCanvasRef.current = null;
    };
  }, [
    app.market,
    app.showGrid,
    app.showCrosshair,
    app.theme,
    app.symbol,
    app.timeframe,
    precision,
    replayCandles,
  ]);

  // ── Keyboard shortcuts ────────────────────────────────────────────────────
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const typing =
        !!target &&
        (target.tagName === 'INPUT' ||
          target.tagName === 'TEXTAREA' ||
          target.tagName === 'SELECT' ||
          target.isContentEditable);
      if (typing) return;

      const commandKey = e.metaKey || e.ctrlKey;

      if (commandKey && e.key.toLowerCase() === 'c' && selectedIdsRef.current.length) {
        const selected = drawingsRef.current.find(d => d.id === selectedIdsRef.current[0]);
        if (selected) {
          clipboardRef.current = structuredClone(selected);
          app.showToast('Copied');
        }
        return;
      }

      if (commandKey && e.key.toLowerCase() === 'v' && clipboardRef.current) {
        e.preventDefault();
        undoRef.current.push(drawingsRef.current.map(d => ({ ...d })));
        redoRef.current = [];
        const clone = structuredClone(clipboardRef.current);
        clone.id = makeId();
        clone.name = `${clone.name} Copy`;
        
        setDrawings(prev => [...prev, clone]);
        setSelectedIds([clone.id]);
        app.showToast('Pasted');
        return;
      }

      if (commandKey && e.key.toLowerCase() === 'z' && !e.shiftKey) {
        const prev = undoRef.current.pop();
        if (prev) {
          e.preventDefault();
          redoRef.current.push(drawingsRef.current.map((drawing) => ({ ...drawing })));
          setDrawings(prev.map((drawing) => ({ ...drawing })));
          setSelectedIds([]);
        }
        return;
      }

      if (
        (commandKey && e.key.toLowerCase() === 'z' && e.shiftKey) ||
        (commandKey && e.key.toLowerCase() === 'y')
      ) {
        const next = redoRef.current.pop();
        if (next) {
          e.preventDefault();
          undoRef.current.push(drawingsRef.current.map((drawing) => ({ ...drawing })));
          setDrawings(next.map((drawing) => ({ ...drawing })));
          setSelectedIds([]);
        }
        return;
      }

      if ((e.key === 'Delete' || e.key === 'Backspace') && selectedIdsRef.current.length) {
        e.preventDefault();
        const ids = [...selectedIdsRef.current];
        undoRef.current.push(drawingsRef.current.map((drawing) => ({ ...drawing })));
        redoRef.current = [];
        setDrawings((prev) => prev.filter((d) => !ids.includes(d.id)));
        setSelectedIds([]);
        setEditorOpenRef.current(false);
        return;
      }

      if (commandKey && e.key.toLowerCase() === 'd' && selectedIdsRef.current.length) {
        e.preventDefault();
        undoRef.current.push(drawingsRef.current.map((drawing) => ({ ...drawing })));
        redoRef.current = [];
        setDrawings((prev) => {
          const selected = prev.filter((drawing) => selectedIdsRef.current.includes(drawing.id));
          const copies = selected.map((drawing, index) => {
            const clone = structuredClone(drawing);
            clone.id = makeId();
            clone.name = `${drawing.name} Copy`;
            if (clone.type === 'text') {
              clone.point = { ...clone.point, price: clone.point.price + (index + 1) * 0.5 };
            } else if (clone.type === 'hline') {
              clone.price += (index + 1) * 0.5;
            } else if (clone.type === 'vline') {
              clone.time += averageStepSeconds(allCandlesRef.current);
            } else {
              clone.start = { ...clone.start, price: clone.start.price + (index + 1) * 0.5 };
              clone.end = { ...clone.end, price: clone.end.price + (index + 1) * 0.5 };
            }
            return clone;
          });
          setSelectedIds(copies.map((copy) => copy.id));
          return [...prev, ...copies];
        });
        return;
      }

      if (selectedIdsRef.current.length && ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(e.key)) {
        e.preventDefault();
        const prec = precisionRef.current;
        const deltaPrice = e.shiftKey
          ? (prec >= 5 ? 0.001 : prec >= 3 ? 0.05 : 1)
          : (prec >= 5 ? 0.0001 : prec >= 3 ? 0.01 : 0.25);
        const deltaBars = e.shiftKey ? 5 : 1;
        undoRef.current.push(drawingsRef.current.map((drawing) => ({ ...drawing })));
        redoRef.current = [];
        setDrawings((prev) =>
          prev.map((drawing) => {
            if (!selectedIdsRef.current.includes(drawing.id) || drawing.locked) return drawing;

            if (drawing.type === 'text') {
              return {
                ...drawing,
                point: {
                  time:
                    e.key === 'ArrowLeft'
                      ? shiftCandleTime(drawing.point.time, allCandlesRef.current, -1, deltaBars)
                      : e.key === 'ArrowRight'
                        ? shiftCandleTime(drawing.point.time, allCandlesRef.current, 1, deltaBars)
                        : drawing.point.time,
                  price:
                    e.key === 'ArrowUp'
                      ? drawing.point.price + deltaPrice
                      : e.key === 'ArrowDown'
                        ? drawing.point.price - deltaPrice
                        : drawing.point.price,
                },
              };
            }

            if (drawing.type === 'hline') {
              return {
                ...drawing,
                price:
                  e.key === 'ArrowUp'
                    ? drawing.price + deltaPrice
                    : e.key === 'ArrowDown'
                      ? drawing.price - deltaPrice
                      : drawing.price,
              };
            }

            if (drawing.type === 'vline') {
              return {
                ...drawing,
                time:
                  e.key === 'ArrowLeft'
                    ? shiftCandleTime(drawing.time, allCandlesRef.current, -1, deltaBars)
                    : e.key === 'ArrowRight'
                      ? shiftCandleTime(drawing.time, allCandlesRef.current, 1, deltaBars)
                      : drawing.time,
              };
            }

            return {
              ...drawing,
              start: {
                time:
                  e.key === 'ArrowLeft'
                    ? shiftCandleTime(drawing.start.time, allCandlesRef.current, -1, deltaBars)
                    : e.key === 'ArrowRight'
                      ? shiftCandleTime(drawing.start.time, allCandlesRef.current, 1, deltaBars)
                      : drawing.start.time,
                price:
                  e.key === 'ArrowUp'
                    ? drawing.start.price + deltaPrice
                    : e.key === 'ArrowDown'
                      ? drawing.start.price - deltaPrice
                      : drawing.start.price,
              },
              end: {
                time:
                  e.key === 'ArrowLeft'
                    ? shiftCandleTime(drawing.end.time, allCandlesRef.current, -1, deltaBars)
                    : e.key === 'ArrowRight'
                      ? shiftCandleTime(drawing.end.time, allCandlesRef.current, 1, deltaBars)
                      : drawing.end.time,
                price:
                  e.key === 'ArrowUp'
                    ? drawing.end.price + deltaPrice
                    : e.key === 'ArrowDown'
                      ? drawing.end.price - deltaPrice
                      : drawing.end.price,
              },
            };
          }),
        );
        return;
      }

      if (e.key === 'Escape') {
        if (rdsStepRef.current) {
          setRdsStep(null); setRdsX(null); setRdsY(null); setRdsCursorPrice(null);
        }
        setDraftStart(null);
        setDraftCurrent(null);
        setSelectedIds([]);
        setEditorOpenRef.current(false);
      }
    };

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  // ── RAF render loop (runs once; reads from refs for fresh data) ───────────
  useEffect(() => {
    let frame = 0;

    const render = () => {
      const canvas  = overlayRef.current;
      const surface = surfaceRef.current;
      const chart   = chartRef.current;
      const series  = seriesRef.current;

      if (!canvas || !surface || !chart || !series) {
        frame = requestAnimationFrame(render);
        return;
      }

      const rect = surface.getBoundingClientRect();
      const dpr  = window.devicePixelRatio || 1;
      const cw   = Math.max(1, Math.round(rect.width  * dpr));
      const ch   = Math.max(1, Math.round(rect.height * dpr));

      if (canvas.width !== cw || canvas.height !== ch) {
        canvas.width        = cw;
        canvas.height       = ch;
        canvas.style.width  = `${rect.width}px`;
        canvas.style.height = `${rect.height}px`;
      }

      const ctx = canvas.getContext('2d');
      if (!ctx) { frame = requestAnimationFrame(render); return; }

      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.imageSmoothingEnabled = true;
      ctx.imageSmoothingQuality = 'high';
      ctx.textBaseline = 'alphabetic';
      ctx.lineJoin = 'round';
      ctx.lineCap = 'round';
      ctx.clearRect(0, 0, rect.width, rect.height);

      const theme        = themeRef.current;
      const drawings     = drawingsRef.current;
      const selectedIds  = selectedIdsRef.current;
      const draftStart   = draftStartRef.current;
      const draftCurrent = draftCurrentRef.current;
      const drawTool     = drawToolRef.current;
      const sym          = symbolRef.current;
      const prec         = precisionRef.current;
      const heatmapRows  = heatmapRowsRef.current;
      const candles      = marketCandlesRef.current;

      const plotBottom = rect.height - 24;

      // ── Volume profile (heatmap) ────────────────────────────────────────
      if (showHeatmapRef.current && heatmapRows.length > 0) {
        // Estimate price scale width based on precision
        const priceScaleWidth = prec >= 5 ? 76 : prec >= 3 ? 64 : 60;
        const plotRight = rect.width - priceScaleWidth;
        const maxBarWidth = Math.min(plotRight * 0.28, 120);

        for (const row of heatmapRows) {
          const y = series.priceToCoordinate(row.price);
          if (y == null || y < 0 || y > plotBottom) continue;

          const normalized  = Math.pow(clamp(row.intensity, 0.04, 1), 0.65);
          const barWidth    = clamp(normalized * maxBarWidth, 2, maxBarWidth);
          const barX        = plotRight - barWidth;
          const barHeight   = Math.max(2, Math.min(8, (rect.height / heatmapRows.length) * 0.7));

          const palette = heatmapPalette(theme, row.side, row.isMax);
          const baseColor = palette.bar;

          const grad = ctx.createLinearGradient(barX, 0, plotRight, 0);
          grad.addColorStop(0,    withAlpha(baseColor, 0));
          grad.addColorStop(0.18, withAlpha(baseColor, row.isMax ? 0.18 : 0.12));
          grad.addColorStop(0.62, withAlpha(baseColor, row.isMax ? 0.72 : 0.42));
          grad.addColorStop(1,    withAlpha(baseColor, row.isMax ? 0.94 : 0.68));

          ctx.fillStyle = grad;
          ctx.fillRect(barX, y - barHeight / 2, barWidth, barHeight);

          // Right-edge tick
          ctx.fillStyle = withAlpha(palette.tick, row.isMax ? 1 : 0.82);
          ctx.fillRect(plotRight - (row.isMax ? 3 : 2), y - barHeight / 2, row.isMax ? 3 : 2, barHeight);

          // Price label for POC + top-2 intensity rows
          if (row.isMax) {
            ctx.save();
            ctx.fillStyle  = palette.label;
            ctx.font       = `${Math.max(9, theme.fontSize - 1)}px 'Share Tech Mono',Consolas,monospace`;
            ctx.textAlign  = 'right';
            ctx.fillText(`● ${formatPrice(row.price, sym)}`, plotRight - 6, y + 3);
            ctx.restore();
          }
        }
      }

      // ── Helper: draw label text ─────────────────────────────────────────
      const drawLabel = (
        text: string,
        x: number,
        y: number,
        color: string,
        align: CanvasTextAlign = 'left',
        targetCtx: CanvasRenderingContext2D = ctx,
      ) => {
        if (!text.trim()) return;
        targetCtx.save();
        targetCtx.fillStyle  = withAlpha(color, 0.9);
        targetCtx.textAlign  = align;
        targetCtx.font       = `${Math.max(9, theme.fontSize)}px 'Share Tech Mono',Consolas,monospace`;
        targetCtx.fillText(text, x, y);
        targetCtx.restore();
      };

      // ── Helper: draw selection handle dot ──────────────────────────────
      const drawHandle = (p: { x: number, y: number }, targetCtx: CanvasRenderingContext2D = ctx) => {
        targetCtx.save();
        targetCtx.fillStyle = theme.accentColor;
        targetCtx.strokeStyle = '#fff';
        targetCtx.lineWidth = 1.5;
        targetCtx.beginPath();
        targetCtx.arc(p.x, p.y, 4, 0, Math.PI * 2);
        targetCtx.fill();
        targetCtx.stroke();
        targetCtx.restore();
      };

      // ── Size back canvas to match front overlay ─────────────────────────
      const backCanvas = backCanvasRef.current;
      let backCtx: CanvasRenderingContext2D | null = null;
      if (backCanvas) {
        if (backCanvas.width !== cw || backCanvas.height !== ch) {
          backCanvas.width  = cw;
          backCanvas.height = ch;
        }
        backCtx = backCanvas.getContext('2d');
        if (backCtx) {
          backCtx.setTransform(dpr, 0, 0, dpr, 0, 0);
          backCtx.clearRect(0, 0, rect.width, rect.height);
          // Paint chart background so it shows through the transparent LW canvas
          backCtx.fillStyle = theme.chartBackground;
          backCtx.fillRect(0, 0, rect.width, rect.height);
        }
      }

      // ── Render all committed drawings ───────────────────────────────────
      for (const drawing of drawings) {
        if (!drawing.visible) continue;
        const selected = selectedIds.includes(drawing.id);
        // Default is back canvas; only 'front' explicitly uses the overlay canvas
        const dc: CanvasRenderingContext2D = (drawing.zLayer !== 'front' && backCtx) ? backCtx : ctx;
        dc.save();

        if (drawing.type === 'text') {
          const pt = toCanvasPoint(chart, series, drawing.point, candles);
          if (pt) {
            dc.font      = `${drawing.fontSize}px 'Share Tech Mono',Consolas,monospace`;
            dc.fillStyle = drawing.color;
            dc.fillText(drawing.text, pt.x, pt.y);
          }
          dc.restore();
          continue;
        }

        dc.lineWidth   = drawing.width;
        dc.strokeStyle = drawing.color;
        setStrokeDash(dc, drawing.strokeStyle, drawing.width);

        if (drawing.type === 'trend') {
          const s = toCanvasPoint(chart, series, drawing.start, candles);
          const e = toCanvasPoint(chart, series, drawing.end, candles);
          if (s && e) {
            dc.beginPath();
            dc.moveTo(s.x, s.y);
            dc.lineTo(e.x, e.y);
            dc.stroke();

            if (drawing.label) {
              const lp = labelPoint(s, e, drawing.labelPosition);
              const lfs = drawing.labelFontSize ?? theme.fontSize;
              dc.save();
              dc.fillStyle = withAlpha(drawing.color, 0.9);
              dc.textAlign = lp.align;
              dc.font = `${Math.max(9, lfs)}px 'Share Tech Mono',Consolas,monospace`;
              dc.fillText(drawing.label, lp.x, lp.y);
              dc.restore();
            }

            if (selected) {
              drawHandle(s, dc);
              drawHandle(e, dc);
            }
          }
        } else if (drawing.type === 'rect') {
          const s = toCanvasPoint(chart, series, drawing.start, candles);
          const eRaw = toCanvasPoint(chart, series, drawing.end, candles);
          // If end time is off the right edge, extend to canvas right boundary.
          const e = eRaw ?? (() => {
            const ey = series.priceToCoordinate(drawing.end.price);
            return (s && ey != null) ? { x: rect.width + 20, y: ey } : null;
          })();
          if (s && e) {
            const left   = Math.min(s.x, e.x);
            const top    = Math.min(s.y, e.y);
            const width  = Math.abs(e.x - s.x);
            const height = Math.abs(e.y - s.y);

            dc.fillStyle = isValidHex(drawing.fillColor)
              ? hexAlphaToRgba(drawing.fillColor, drawing.fillAlpha)
              : withAlpha(drawing.fillColor, drawing.fillAlpha);
            dc.fillRect(left, top, width, height);

            if (drawing.borderVisible) {
              dc.setLineDash([]);
              setStrokeDash(dc, drawing.strokeStyle, drawing.width);
              dc.strokeRect(left, top, width, height);
            }

            if (drawing.label) {
              const lp = labelPoint(s, e, drawing.labelPosition);
              const lfs = drawing.labelFontSize ?? theme.fontSize;
              dc.save();
              dc.fillStyle = withAlpha(drawing.color, 0.9);
              dc.textAlign = lp.align;
              dc.font = `${Math.max(9, lfs)}px 'Share Tech Mono',Consolas,monospace`;
              dc.fillText(drawing.label, lp.x, lp.y);
              dc.restore();
            }

            if (selected) {
              drawHandle(s, dc);
              drawHandle(e, dc);
            }
          }
        } else if (drawing.type === 'hline') {
          const y = series.priceToCoordinate(drawing.price);
          if (y != null && y >= 0 && y <= rect.height) {
            dc.beginPath();
            dc.moveTo(0, y);
            dc.lineTo(rect.width, y);
            dc.stroke();

            // Use time-scale plot width (stable, never changes during scroll).
            const plotRight = chart.timeScale().width();
            drawLabel(drawing.label || formatPrice(drawing.price, sym), plotRight - 6, y - 5, drawing.color, 'right', dc);
          }
        } else if (drawing.type === 'vline') {
          const x = xFromTimeContinuous(chart, drawing.time, candles);
          if (x != null && x >= 0 && x <= rect.width) {
            dc.beginPath();
            dc.moveTo(x, 0);
            dc.lineTo(x, plotBottom);
            dc.stroke();

            if (drawing.label) {
              const lfs = drawing.labelFontSize ?? theme.fontSize;
              dc.save();
              dc.fillStyle = withAlpha(drawing.color, 0.85);
              dc.textAlign = 'left';
              dc.font = `${Math.max(9, lfs)}px 'Share Tech Mono',Consolas,monospace`;
              dc.fillText(drawing.label, x + 5, 14);
              dc.restore();
            }
          }
        } else if (drawing.type === 'fib') {
          const s = toCanvasPoint(chart, series, drawing.start, candles);
          const e = toCanvasPoint(chart, series, drawing.end, candles);
          if (s && e) {
            const left = Math.min(s.x, e.x);
            const right = Math.max(s.x, e.x);
            const topPrice = drawing.start.price;
            const bottomPrice = drawing.end.price;
            const range = bottomPrice - topPrice;

            for (const level of drawing.levels) {
              if (!level.visible) continue;
              const price = topPrice + range * level.value;
              const y = series.priceToCoordinate(price);
              if (y == null) continue;

              dc.save();
              dc.strokeStyle = level.color;
              dc.lineWidth = Math.max(1, drawing.width);
              dc.setLineDash([]);
              dc.beginPath();
              dc.moveTo(left, y);
              dc.lineTo(right, y);
              dc.stroke();

              const lfs = drawing.labelFontSize ?? theme.fontSize;
              dc.fillStyle = withAlpha(level.color, 0.95);
              dc.textAlign = 'left';
              dc.font = `${Math.max(9, lfs)}px 'Share Tech Mono',Consolas,monospace`;
              dc.fillText(`${level.value.toFixed(3)} · ${formatPrice(price, sym)}`, right + 6, y - 3);
              dc.restore();
            }

            if (selected) {
              drawHandle(s, dc);
              drawHandle(e, dc);
            }
          }
        }

        dc.restore();
      }

      // ── RDS guide lines ─────────────────────────────────────────────────
      const rdsStepNow   = rdsStepRef.current;
      const rdsXNow      = rdsXRef.current;
      const rdsCursorNow = rdsCursorPriceRef.current;
      const rdsActive = rdsStepNow === 'pick-x' || rdsStepNow === 'pick-y' || (rdsStepNow === null && drawToolRef.current === 'rds');
      const effectiveStep = rdsStepNow ?? 'pick-x';
      if (rdsActive && rdsCursorNow != null) {
        const cy = series.priceToCoordinate(rdsCursorNow);
        if (cy != null) {
          ctx.save();
          ctx.strokeStyle = effectiveStep === 'pick-x' ? '#3a8fd0' : '#f0a030';
          ctx.lineWidth = 1;
          ctx.setLineDash([6, 4]);
          ctx.beginPath(); ctx.moveTo(0, cy); ctx.lineTo(rect.width, cy); ctx.stroke();
          ctx.setLineDash([]);
          ctx.fillStyle = effectiveStep === 'pick-x' ? '#3a8fd0' : '#f0a030';
          ctx.font = `bold 11px 'Share Tech Mono',Consolas,monospace`;
          ctx.fillText(
            `${effectiveStep === 'pick-x' ? '→ Pick X' : '→ Pick Y'}  ${formatPrice(rdsCursorNow, sym)}`,
            8, cy - 6,
          );
          ctx.restore();
        }
        // If X is already anchored, draw a solid thin line for it
        if (rdsStepNow === 'pick-y' && rdsXNow != null) {
          const xy = series.priceToCoordinate(rdsXNow);
          if (xy != null) {
            ctx.save();
            ctx.strokeStyle = '#3a8fd0';
            ctx.lineWidth = 1;
            ctx.setLineDash([]);
            ctx.globalAlpha = 0.6;
            ctx.beginPath(); ctx.moveTo(0, xy); ctx.lineTo(rect.width, xy); ctx.stroke();
            ctx.globalAlpha = 1;
            ctx.fillStyle = '#3a8fd0';
            ctx.font = `11px 'Share Tech Mono',Consolas,monospace`;
            ctx.fillText(`X  ${formatPrice(rdsXNow, sym)}`, 8, xy - 6);
            ctx.restore();
          }
        }
      }

      // ── Draft preview ───────────────────────────────────────────────────
      if (draftStart && draftCurrent && (drawTool === 'trend' || drawTool === 'rect' || drawTool === 'fib')) {
        const s = toCanvasPoint(chart, series, draftStart, candles);
        const e = toCanvasPoint(chart, series, draftCurrent, candles);
        if (s && e) {
          ctx.save();
          ctx.strokeStyle = theme.accentColor;
          ctx.lineWidth   = 1.5;
          ctx.setLineDash([4, 4]);

          if (drawTool === 'trend') {
            ctx.beginPath();
            ctx.moveTo(s.x, s.y);
            ctx.lineTo(e.x, e.y);
            ctx.stroke();
          } else {
            const left   = Math.min(s.x, e.x);
            const top    = Math.min(s.y, e.y);
            const width  = Math.abs(e.x - s.x);
            const height = Math.abs(e.y - s.y);
            ctx.fillStyle = isValidHex(theme.accentColor)
              ? hexAlphaToRgba(theme.accentColor, drawTool === 'fib' ? 0.03 : 0.08)
              : 'rgba(240,160,48,0.08)';
            ctx.fillRect(left, top, width, height);
            ctx.strokeRect(left, top, width, height);
          }

          ctx.restore();
        }
      }


      frame = requestAnimationFrame(render);
    };

    frame = requestAnimationFrame(render);
    return () => cancelAnimationFrame(frame);
  }, []); // runs once; uses refs throughout

  // ── Helpers ───────────────────────────────────────────────────────────────
  function commitDrawings(
    next: StoredDrawing[] | ((prev: StoredDrawing[]) => StoredDrawing[]),
    resetSelection = false,
  ) {
    setDrawings((prev) => {
      const resolved = typeof next === 'function' ? next(prev) : next;
      undoRef.current.push(prev.map((drawing) => ({ ...drawing })));
      if (undoRef.current.length > 100) undoRef.current.shift();
      redoRef.current = [];
      if (resetSelection) setSelectedIds([]);
      return resolved;
    });
  }


  function findHit(pt: { x: number; y: number }): SelectionHit | null {
    const chart  = chartRef.current;
    const series = seriesRef.current;
    const candles = marketCandlesRef.current;
    if (!chart || !series) return null;

    for (let i = drawingsRef.current.length - 1; i >= 0; i--) {
      const d = drawingsRef.current[i];
      if (!d.visible) continue;
      if (d.type === 'trend') {
        const s = toCanvasPoint(chart, series, d.start, candles);
        const e = toCanvasPoint(chart, series, d.end, candles);
        if (s && e && distanceToSegment(pt, s, e) < 14) return { id: d.id };
      } else if (d.type === 'rect' || d.type === 'fib') {
        const s = toCanvasPoint(chart, series, d.start, candles);
        const e = toCanvasPoint(chart, series, d.end, candles);
        if (s && e) {
          const pad = 10;
          const left = Math.min(s.x, e.x) - pad, right  = Math.max(s.x, e.x) + pad;
          const top  = Math.min(s.y, e.y) - pad, bottom = Math.max(s.y, e.y) + pad;
          if (pt.x >= left && pt.x <= right && pt.y >= top && pt.y <= bottom) return { id: d.id };
        }
      } else if (d.type === 'hline') {
        const y = series.priceToCoordinate(d.price);
        if (y != null && Math.abs(pt.y - y) < 12) return { id: d.id };
      } else if (d.type === 'vline') {
        const x = xFromTimeContinuous(chart, d.time, candles);
        if (x != null && Math.abs(pt.x - x) < 12) return { id: d.id };
      } else if (d.type === 'text') {
        const tp = toCanvasPoint(chart, series, d.point, candles);
        if (tp && Math.abs(pt.x - tp.x) < 70 && Math.abs(pt.y - tp.y) < 28) return { id: d.id };
      }
    }
    return null;
  }

  function findHandleHit(pt: { x: number; y: number }): HandleState | null {
    const chart = chartRef.current;
    const series = seriesRef.current;
    const candles = marketCandlesRef.current;
    if (!chart || !series) return null;

    for (const id of selectedIdsRef.current) {
      const d = drawingsRef.current.find((item) => item.id === id);
      if (!d || d.locked) continue;

      const check = (p: AnchorPoint, key: HandleKey): HandleState | null => {
        const cp = toCanvasPoint(chart, series, p, candles);
        if (cp && Math.hypot(pt.x - cp.x, pt.y - cp.y) < 12) {
          return { drawingId: d.id, handle: key, snapshot: structuredClone(d) };
        }
        return null;
      };

      if (d.type === 'trend' || d.type === 'rect' || d.type === 'fib') {
        const h1 = check(d.start, 'start'); if (h1) return h1;
        const h2 = check(d.end,   'end');   if (h2) return h2;
      } else if (d.type === 'text') {
        const h = check(d.point, 'point'); if (h) return h;
      }
    }
    return null;
  }

  function eventPoint(
    event: React.PointerEvent<HTMLElement>,
    magnet = false,
  ) {
    if (!surfaceRef.current || !chartRef.current || !seriesRef.current) return null;
    const rect  = surfaceRef.current.getBoundingClientRect();
    const candles = marketCandlesRef.current;
    const point = toPoint(
      chartRef.current,
      seriesRef.current,
      event.clientX - rect.left,
      event.clientY - rect.top,
      candles,
    );
    if (!point) return null;
    const anchor =
      magnet && app.showMagnet && candles.length
        ? snapToMagnet(point, candles)
        : point;
    return {
      anchor,
      screen: { x: event.clientX - rect.left, y: event.clientY - rect.top },
    };
  }

  function updateSelected(updater: (d: StoredDrawing) => StoredDrawing) {
    setDrawings((prev) => {
      const next = prev.map((d) => {
        if (!selectedIdsRef.current.includes(d.id)) return d;
        let updated = updater(d);
        
        // Custom logic for Fibonacci main color
        if (d.type === 'fib' && updated.type === 'fib' && updated.color !== d.color) {
          updated.levels = updated.levels.map(l => ({ ...l, color: updated.color }));
        }
        
        const nextDefaults = defaultsFromDrawing(updated);
        defaultsRef.current = applyCachedDefaults(
          {
            ...defaultsRef.current,
            ...nextDefaults,
          },
          sanitizeCachedDrawingDefaults(nextDefaults),
        );
        saveDrawingDefaults(defaultsRef.current);
        return updated;
      });
      undoRef.current.push(prev.map((drawing) => ({ ...drawing })));
      if (undoRef.current.length > 100) undoRef.current.shift();
      redoRef.current = [];
      return next;
    });
  }

  function deleteSelected() {
    const ids = [...selectedIdsRef.current];
    if (!ids.length) return;
    undoRef.current.push(drawingsRef.current.map((drawing) => ({ ...drawing })));
    redoRef.current = [];
    setDrawings((prev) => prev.filter((d) => !ids.includes(d.id)));
    setSelectedIds([]);
    setEditorOpen(false);
  }

  // ── Constraint: shift-snap trend to 45° angles ────────────────────────────
  function constrainTrend(
    screen: { x: number; y: number },
    fallback: AnchorPoint,
  ): AnchorPoint {
    if (!draftStart || !chartRef.current || !seriesRef.current) return fallback;
    const candles = marketCandlesRef.current;
    const sp = toCanvasPoint(
      chartRef.current,
      seriesRef.current,
      draftStart,
      candles,
    );
    if (!sp) return fallback;
    const dx = screen.x - sp.x;
    const dy = screen.y - sp.y;
    const dist = Math.hypot(dx, dy);
    if (!dist) return fallback;
    const angle    = Math.atan2(dy, dx);
    const snapped  = Math.round(angle / (Math.PI / 4)) * (Math.PI / 4);
    const resolved = toPoint(
      chartRef.current,
      seriesRef.current,
      sp.x + Math.cos(snapped) * dist,
      sp.y + Math.sin(snapped) * dist,
      candles,
    );
    return resolved ?? fallback;
  }

  function moveDrawingLogical(
    d: StoredDrawing,
    dLogical: number,
    dp: number,
    candles: NonNullable<VpdaAppState['market']>['candles'],
  ): StoredDrawing {
    const shiftTime = (time: number) => {
      const logical = logicalFromTimeContinuous(time, candles);
      if (logical == null) return time;
      const shifted = timeFromLogicalContinuous(logical + dLogical, candles);
      return shifted ?? time;
    };

    if (d.type === 'trend' || d.type === 'rect' || d.type === 'fib') {
      return {
        ...d,
        start: { time: shiftTime(d.start.time), price: d.start.price + dp },
        end: { time: shiftTime(d.end.time), price: d.end.price + dp },
      };
    }
    if (d.type === 'hline') return { ...d, price: d.price + dp };
    if (d.type === 'vline') return { ...d, time: shiftTime(d.time) };
    return {
      ...d,
      point: { time: shiftTime(d.point.time), price: d.point.price + dp },
    };
  }

  // ── Surface: cursor-mode drag ─────────────────────────────────────────────
  function onSurfacePointerDownCapture(e: React.PointerEvent<HTMLDivElement>) {
    if (e.button === 2) return; // Ignore right-click
    const inMenu = !!(e.target as HTMLElement).closest?.('.context-menu, .floating-toolbar, .object-editor');
    if (contextMenu && !inMenu) setContextMenu(null);
    if (floatingMenu && !inMenu) setFloatingMenu(null);
    if (app.drawTool !== 'cursor') return;
    if (!chartRef.current) return;

    if ((e.target as HTMLElement).closest?.('.object-editor')) return;
    if ((e.target as HTMLElement).closest?.('.floating-toolbar')) return;
    if ((e.target as HTMLElement).closest?.('.drawings-manager-panel')) return;
    if ((e.target as HTMLElement).closest?.('.context-menu')) return;

    const point = eventPoint(e, false);
    if (!point) return;
    const originLogical = xToLogical(chartRef.current, point.screen.x);
    if (originLogical == null) return;

    const handleHit = findHandleHit(point.screen);
    if (handleHit) {
      e.preventDefault();
      e.currentTarget.setPointerCapture(e.pointerId);
      chartRef.current?.applyOptions({ handleScroll: false as any, handleScale: false as any });
      undoRef.current.push(drawingsRef.current.map((d) => ({ ...d })));
      if (undoRef.current.length > 100) undoRef.current.shift();
      redoRef.current = [];
      setHandleState(handleHit);
      return;
    }

    const hit = findHit(point.screen);
    if (!hit) {
      if (!e.ctrlKey && !e.metaKey && !e.shiftKey) {
        setSelectedIds([]);
        setEditorOpen(false);
      }
      return;
    }

    const hitDrawing = drawingsRef.current.find((drawing) => drawing.id === hit.id);
    if (!hitDrawing) return;

    if (e.metaKey || e.ctrlKey || e.shiftKey) {
      e.preventDefault();
      setSelectedIds((prev) =>
        prev.includes(hit.id) ? prev.filter((id) => id !== hit.id) : [...prev, hit.id],
      );
      return;
    }

    e.preventDefault();
    e.currentTarget.setPointerCapture(e.pointerId);

    chartRef.current?.applyOptions({
      handleScroll: false as any,
      handleScale:  false as any,
    });

    if (!selectedIdsRef.current.includes(hit.id)) {
      setEditorOpen(false);
    }

    if (hitDrawing.locked) {
      setSelectedIds([hit.id]);
      setDragState(null);
      return;
    }

    const active = selectedIdsRef.current.includes(hit.id)
      ? selectedIdsRef.current
      : [hit.id];
    undoRef.current.push(drawingsRef.current.map((d) => ({ ...d })));
    if (undoRef.current.length > 100) undoRef.current.shift();
    redoRef.current = [];
    setSelectedIds(active);
    setDragState({
      origin: point.anchor,
      originLogical,
      snapshot: drawingsRef.current.filter((d) => active.includes(d.id)),
    });
  }

  function onSurfacePointerMoveCapture(e: React.PointerEvent<HTMLDivElement>) {
    const drag   = dragStateRef.current;
    const handle = handleStateRef.current;
    if (!chartRef.current || (!drag && !handle)) return;

    const point = eventPoint(e, app.showMagnet);
    if (!point) return;
    const candles = marketCandlesRef.current;
    if (!candles.length) return;

    if (handle) {
      setDrawings((prev) =>
        prev.map((d) => {
          if (d.id !== handle.drawingId) return d;
          const next = { ...d };
          if (next.type === 'trend' || next.type === 'rect' || next.type === 'fib') {
            if (handle.handle === 'start') next.start = point.anchor;
            if (handle.handle === 'end')   next.end   = point.anchor;
          } else if (next.type === 'text') {
            next.point = point.anchor;
          }
          return next;
        }),
      );
      return;
    }

    if (drag) {
      const currentLogical = xToLogical(chartRef.current, point.screen.x);
      if (currentLogical == null) return;
      const dLogical = currentLogical - drag.originLogical;
      const dp = point.anchor.price - drag.origin.price;
      const snapshot = drag.snapshot;
      if (!snapshot.length) return;
      setDrawings((prev) =>
        prev.map((d) => {
          const snap = snapshot.find((s) => s.id === d.id);
          if (!snap || d.locked) return d;
          return moveDrawingLogical(snap, dLogical, dp, candles);
        }),
      );
    }
  }

  function onSurfacePointerUpCapture(e: React.PointerEvent<HTMLDivElement>) {
    if (dragStateRef.current || handleStateRef.current) {
      if (e.currentTarget.hasPointerCapture?.(e.pointerId)) {
        e.currentTarget.releasePointerCapture(e.pointerId);
      }
      setDragState(null);
      setHandleState(null);
      chartRef.current?.applyOptions({
        handleScroll: { mouseWheel: true, pressedMouseMove: true },
        handleScale:  { mouseWheel: true, pinch: true, axisPressedMouseMove: true },
      });
    }
  }

  // ── RDS: activate / reset when draw tool changes ────────────────────────
  useEffect(() => {
    if (app.drawTool === 'rds') {
      setRdsStep('pick-x');
      setRdsX(null);
      setRdsY(null);
      setRdsCursorPrice(null);
    } else {
      setRdsStep(null);
      setRdsX(null);
      setRdsY(null);
      setRdsCursorPrice(null);
    }
  }, [app.drawTool]);

  function onRdsDirectionChosen(direction: 'bull' | 'bear') {
    if (rdsX == null || rdsY == null) return;
    const v      = direction === 'bull' ? 0.54 : 0.39;
    const level  = rdsX + v * (rdsY - rdsX);
    const d      = defaultsRef.current;
    const n      = drawingsRef.current.filter(dr => dr.type === 'hline').length + 1;
    const color  = direction === 'bull' ? '#26a69a' : '#ef5350';
    const drawing: StoredDrawing = {
      id: makeId(), type: 'hline',
      name:  `RDS-${direction === 'bull' ? 'Bull' : 'Bear'} ${n}`,
      visible: true, locked: false,
      price: level,
      label: `RDS ${direction === 'bull' ? '▲' : '▼'}  ${formatPrice(level, app.symbol)}`,
      color, width: d.width, strokeStyle: 'solid',
      labelFontSize: d.labelFontSize,
      sourceTimeframe: currentScope,
      zLayer: 'front',
    };
    commitDrawings(prev => [...prev, drawing]);
    setSelectedIds([drawing.id]);
    setEditorOpen(true);
    setRdsStep(null); setRdsX(null); setRdsY(null); setRdsCursorPrice(null);
    app.setDrawTool('cursor');
  }

  // ── Interaction layer: drawing tools ─────────────────────────────────────
  function onPointerDown(e: React.PointerEvent<HTMLDivElement>) {
    if (app.drawTool === 'cursor') return;
    const point = eventPoint(e, app.showMagnet);
    if (!point) return;

    // RDS pick steps (treat null as 'pick-x' in case effect hasn't fired yet)
    if (app.drawTool === 'rds') {
      const step = rdsStep ?? 'pick-x';
      if (step === 'pick-x') {
        setRdsX(point.anchor.price);
        setRdsStep('pick-y');
      } else if (step === 'pick-y') {
        setRdsY(point.anchor.price);
        setRdsStep('pick-dir');
      }
      return;
    }

    // Instant drawings (hline, vline, text)
    const d = defaultsRef.current;
    if (app.drawTool === 'hline') {
      const drawing: StoredDrawing = {
        id: makeId(), type: 'hline',
        name: `HLINE ${drawingsRef.current.filter((item) => item.type === 'hline').length + 1}`,
        visible: true,
        locked: false,
        price: point.anchor.price, label: d.hlineLabel,
        color: getTypeColor('hline', d.color), width: d.width, strokeStyle: d.strokeStyle,
        labelFontSize: d.labelFontSize,
        sourceTimeframe: currentScope,
      };
      saveDrawingDefaults(d);
      commitDrawings((prev) => [...prev, drawing]);
      setSelectedIds([drawing.id]);
      setEditorOpen(false);
      app.setDrawTool('cursor');
      return;
    }
    if (app.drawTool === 'vline') {
      const drawing: StoredDrawing = {
        id: makeId(), type: 'vline',
        name: `VLINE ${drawingsRef.current.filter((item) => item.type === 'vline').length + 1}`,
        visible: true,
        locked: false,
        time: point.anchor.time, label: d.vlineLabel,
        color: getTypeColor('vline', d.color), width: d.width, strokeStyle: d.strokeStyle,
        labelFontSize: d.labelFontSize,
        sourceTimeframe: currentScope,
      };
      saveDrawingDefaults(d);
      commitDrawings((prev) => [...prev, drawing]);
      setSelectedIds([drawing.id]);
      setEditorOpen(false);
      app.setDrawTool('cursor');
      return;
    }
    if (app.drawTool === 'text') {
      const drawing: StoredDrawing = {
        id: makeId(), type: 'text',
        name: `TEXT ${drawingsRef.current.filter((item) => item.type === 'text').length + 1}`,
        visible: true,
        locked: false,
        point: point.anchor,
        text: d.textValue, color: getTypeColor('text', d.color), fontSize: d.fontSize,
        sourceTimeframe: currentScope,
      };
      saveDrawingDefaults(d);
      commitDrawings((prev) => [...prev, drawing]);
      setSelectedIds([drawing.id]);
      app.setDrawTool('cursor');
      return;
    }

    // Drag drawings: start draft
    setDraftStart(point.anchor);
    setDraftCurrent(point.anchor);
  }

  function onPointerMove(e: React.PointerEvent<HTMLDivElement>) {
    // Track cursor price for RDS guide lines
    if (app.drawTool === 'rds') {
      const point = eventPoint(e, false);
      if (point) setRdsCursorPrice(point.anchor.price);
    }
    if (!draftStart) return;
    const point = eventPoint(e, false);
    if (!point) return;
    setDraftCurrent(
      app.drawTool === 'trend' && e.shiftKey
        ? constrainTrend(point.screen, point.anchor)
        : point.anchor,
    );
  }

  function onPointerUp(e: React.PointerEvent<HTMLDivElement>) {
    if (!draftStart) return;

    const point = eventPoint(e, false);
    setDraftStart(null);
    setDraftCurrent(null);

    if (!point) return;
    const endAnchor =
      app.drawTool === 'trend' && e.shiftKey
        ? constrainTrend(point.screen, point.anchor)
        : point.anchor;

    const d = defaultsRef.current;
    if (app.drawTool === 'trend') {
      const drawing: StoredDrawing = {
        id: makeId(), type: 'trend',
        name: `LINE ${drawingsRef.current.filter((item) => item.type === 'trend').length + 1}`,
        visible: true,
        locked: false,
        start: draftStart, end: endAnchor,
        label: d.trendLabel, labelPosition: d.labelPosition,
        color: getTypeColor('trend', d.color), width: d.width, strokeStyle: d.strokeStyle,
        labelFontSize: d.labelFontSize,
        sourceTimeframe: currentScope,
      };
      saveDrawingDefaults(d);
      commitDrawings((prev) => [...prev, drawing]);
      setSelectedIds([drawing.id]);
      setEditorOpen(false);
    } else if (app.drawTool === 'rect') {
      const drawing: StoredDrawing = {
        id: makeId(), type: 'rect',
        name: `BOX ${drawingsRef.current.filter((item) => item.type === 'rect').length + 1}`,
        visible: true,
        locked: false,
        start: draftStart, end: endAnchor,
        label: d.rectLabel, labelPosition: d.labelPosition,
        color: getTypeColor('rect', d.color), width: d.width, strokeStyle: d.strokeStyle,
        labelFontSize: d.labelFontSize,
        fillColor: getTypeColor('rect', d.fillColor), fillAlpha: d.fillAlpha,
        borderVisible: d.borderVisible,
        sourceTimeframe: currentScope,
      };
      saveDrawingDefaults(d);
      commitDrawings((prev) => [...prev, drawing]);
      setSelectedIds([drawing.id]);
      setEditorOpen(false);
    } else if (app.drawTool === 'fib') {
      const drawing: StoredDrawing = {
        id: makeId(),
        type: 'fib',
        name: `FIB ${drawingsRef.current.filter((item) => item.type === 'fib').length + 1}`,
        visible: true,
        locked: false,
        start: draftStart,
        end: endAnchor,
        levels: d.fibLevels.map((level) => ({ ...level })),
        labelPosition: d.labelPosition,
        priceLabelAlign: d.fibLabelAlign,
        extendRight: d.fibExtendRight,
        color: getTypeColor('fib', d.color),
        width: d.width,
        strokeStyle: d.strokeStyle,
        labelFontSize: d.labelFontSize,
        sourceTimeframe: currentScope,
      };
      saveDrawingDefaults(d);
      commitDrawings((prev) => [...prev, drawing]);
      setSelectedIds([drawing.id]);
      setEditorOpen(false);
    }

    app.setDrawTool('cursor');
  }

  // ── Render ────────────────────────────────────────────────────────────────
  return (
    <div
      className="chart-surface"
      ref={surfaceRef}
      onPointerDownCapture={onSurfacePointerDownCapture}
      onPointerMoveCapture={onSurfacePointerMoveCapture}
      onPointerUpCapture={onSurfacePointerUpCapture}
      onContextMenu={(e) => {
        e.preventDefault();
        const rect = surfaceRef.current!.getBoundingClientRect();
        const rawX = e.clientX - rect.left;
        const rawY = e.clientY - rect.top;
        const hit = findHit({ x: rawX, y: rawY });
        setContextMenu({
          x: Math.min(rawX, rect.width - 160),
          y: Math.min(rawY, rect.height - (hit?.id ? 220 : 140)),
          drawingId: hit?.id ?? null,
        });
        if (hit) setSelectedIds([hit.id]);
      }}
      onDoubleClick={(e) => {
        if (app.drawTool !== 'cursor') return;
        if ((e.target as HTMLElement).closest?.('.object-editor')) return;
        const rect = surfaceRef.current!.getBoundingClientRect();
        const hit = findHit({ x: e.clientX - rect.left, y: e.clientY - rect.top });
        if (hit) {
          setSelectedIds([hit.id]);
          setEditorOpen(true);
        }
      }}
    >
      {app.loadingMarket && (
        <div style={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          background: 'rgba(0, 0, 0, 0.4)',
          zIndex: 9999,
          pointerEvents: 'none',
        }}>
          <div style={{
            width: 40,
            height: 40,
            border: '4px solid rgba(255, 255, 255, 0.3)',
            borderTop: '4px solid white',
            borderRadius: '50%',
            animation: 'spin 1s linear infinite',
          }} />
        </div>
      )}
      {managerOpen && managerPortalTarget && createPortal(
        <aside className="drawings-manager-panel" style={{ 
          borderLeft: '1px solid var(--border1)',
          background: 'var(--bg1)',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
          minHeight: 0,
        }}>
          <div className="panel-title" style={{ padding: 'var(--sp-2) var(--sp-3)', fontSize: 'var(--fs-10)', fontWeight: 400, textTransform: 'uppercase', color: 'var(--text2)', background: 'var(--bg2)', borderBottom: '1px solid var(--border1)' }}>Chart Manager</div>
          <div className="drawings-manager-tabs" style={{ padding: 'var(--sp-2)', gap: 'var(--sp-1)', display: 'flex', background: 'var(--bg2)' }}>
            <button
              className={`ue-btn small ${managerTab === 'drawings' ? 'active' : ''}`}
              style={{ flex: 1 }}
              onClick={() => setManagerTab('drawings')}
            >
              DRAW
            </button>
            <button
              className={`ue-btn small ${managerTab === 'snapshots' ? 'active' : ''}`}
              style={{ flex: 1 }}
              onClick={() => setManagerTab('snapshots')}
            >
              SNAP
            </button>
            <button
              className={`ue-btn small ${managerTab === 'replay' ? 'active' : ''}`}
              style={{ flex: 1 }}
              onClick={() => setManagerTab('replay')}
            >
              PLY
            </button>
          </div>

          {managerTab === 'drawings' && (
            <div className="drawings-manager-body" style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
              <div style={{ padding: 'var(--sp-2) var(--sp-3)', borderBottom: '1px solid var(--border1)' }}>
                <label className="object-editor-field" style={{ gridTemplateColumns: '50px 1fr' }}>
                  <span style={{ fontSize: 'var(--fs-8)' }}>Filter</span>
                  <select
                    style={{ height: 22, fontSize: 'var(--fs-9)' }}
                    value={drawingFilter}
                    onChange={(e) => setDrawingFilter(e.target.value as DrawingFilter)}
                  >
                    <option value="all">ALL TYPES</option>
                    <option value="trend">TREND LINE</option>
                    <option value="rect">RECTANGLE</option>
                    <option value="hline">HORIZON LINE</option>
                    <option value="vline">VERT LINE</option>
                    <option value="fib">FIBONACCI</option>
                    <option value="text">TEXT LABEL</option>
                  </select>
                </label>
              </div>

              <div className="drawing-list" style={{ flex: 1, overflowY: 'auto', padding: 'var(--sp-2)' }}>
                {filteredDrawings.length === 0 ? (
                  <div style={{ padding: 'var(--sp-6)', textAlign: 'center', opacity: 0.4, fontSize: 'var(--fs-9)' }}>No drawings found</div>
                ) : (
                  filteredDrawings.map((drawing, index) => (
                    <div
                      key={drawing.id}
                      className={`drawing-row ${selectedIds.includes(drawing.id) ? 'active' : ''} ${!drawing.visible ? 'muted' : ''}`}
                      onClick={(e) => {
                        if (e.ctrlKey || e.metaKey) {
                          setSelectedIds(prev => 
                            prev.includes(drawing.id) ? prev.filter(id => id !== drawing.id) : [...prev, drawing.id]
                          );
                        } else {
                          setSelectedIds([drawing.id]);
                        }
                      }}
                    >
                      <div className="drawing-row-main">
                        <div className="drawing-row-name">{drawing.name}</div>
                        <div className="drawing-row-meta">{drawing.type.toUpperCase()}</div>
                      </div>
                      <div className="drawing-manager-actions" style={{ display: 'flex', gap: '2px' }}>
                      <button
                        className="ue-btn small"
                        title="Move Up"
                        disabled={index === 0}
                        onClick={(e) => {
                          e.stopPropagation();
                          setDrawings((prev) => {
                            const idx = prev.findIndex(d => d.id === drawing.id);
                            if (idx <= 0) return prev;
                            const next = [...prev];
                            [next[idx - 1], next[idx]] = [next[idx], next[idx - 1]];
                            return next;
                          });
                        }}
                      >
                        <ChevronUpIcon />
                      </button>
                      <button
                        className="ue-btn small"
                        title="Move Down"
                        disabled={index === filteredDrawings.length - 1}
                        onClick={(e) => {
                          e.stopPropagation();
                          setDrawings((prev) => {
                            const idx = prev.findIndex(d => d.id === drawing.id);
                            if (idx === -1 || idx === prev.length - 1) return prev;
                            const next = [...prev];
                            [next[idx], next[idx + 1]] = [next[idx + 1], next[idx]];
                            return next;
                          });
                        }}
                      >
                        <ChevronDownIcon />
                      </button>
                      <button
                        className={`ue-btn small ${!drawing.visible ? 'active' : ''}`}
                        title={drawing.visible ? 'Hide' : 'Show'}
                        onClick={(e) => {
                          e.stopPropagation();
                          setDrawings((prev) =>
                            prev.map((d) => (d.id === drawing.id ? { ...d, visible: !d.visible } : d)),
                          );
                        }}
                      >
                        {drawing.visible ? <EyeIcon /> : <EyeOffIcon />}
                      </button>
                      <button
                        className={`ue-btn small ${drawing.locked ? 'active' : ''}`}
                        title={drawing.locked ? 'Unlock' : 'Lock'}
                        onClick={(e) => {
                          e.stopPropagation();
                          setDrawings((prev) =>
                            prev.map((d) => (d.id === drawing.id ? { ...d, locked: !d.locked } : d)),
                          );
                        }}
                      >
                        {drawing.locked ? <LockIcon /> : <UnlockIcon />}
                      </button>
                      <button
                        className="ue-btn small danger"
                        title="Delete"
                        onClick={(e) => {
                          e.stopPropagation();
                          undoRef.current.push(drawingsRef.current.map((item) => ({ ...item })));
                          redoRef.current = [];
                          setDrawings((prev) => prev.filter((item) => item.id !== drawing.id));
                          setSelectedIds((prev) => prev.filter((id) => id !== drawing.id));
                        }}
                      >
                        <TrashIcon />
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
          )}

          {managerTab === 'snapshots' && (
            <div className="drawings-manager-body" style={{ flex: 1, display: 'flex', flexDirection: 'column', overflow: 'hidden' }}>
              <div style={{ padding: 'var(--sp-2) var(--sp-3)', borderBottom: '1px solid var(--border1)' }}>
                <label className="object-editor-field" style={{ gridTemplateColumns: '50px 1fr' }}>
                  <span style={{ fontSize: 'var(--fs-8)' }}>Note</span>
                  <div style={{ display: 'flex', gap: '2px' }}>
                    <input
                      style={{ height: 22, fontSize: 'var(--fs-9)', flex: 1 }}
                      value={snapshotNote}
                      onChange={(e) => setSnapshotNote(e.target.value)}
                      placeholder="Enter note..."
                      onKeyDown={(e) => {
                        if (e.key === 'Enter' && snapshotNote.trim()) {
                          const nextSnapshots = [
                            ...snapshots,
                            {
                              id: makeId(),
                              note: snapshotNote,
                              createdAt: Date.now(),
                              drawings: drawingsRef.current.map((d) => structuredClone(d)),
                              replayIndex,
                            },
                          ];
                          setSnapshotsState(nextSnapshots);
                          saveSnapshots(app.symbol, currentScope, nextSnapshots);
                          setSnapshotNote('');
                          app.showToast('Snapshot saved');
                        }
                      }}
                    />
                    <button 
                      className="ue-btn small strong"
                      disabled={!snapshotNote.trim()}
                      onClick={() => {
                        const nextSnapshots = [
                          ...snapshots,
                          {
                            id: makeId(),
                            note: snapshotNote,
                            createdAt: Date.now(),
                            drawings: drawingsRef.current.map((d) => structuredClone(d)),
                            replayIndex,
                          },
                        ];
                        setSnapshotsState(nextSnapshots);
                        saveSnapshots(app.symbol, currentScope, nextSnapshots);
                        setSnapshotNote('');
                        app.showToast('Snapshot saved');
                      }}
                    >
                      SAVE
                    </button>
                  </div>
                </label>
              </div>
              <div className="snapshot-list" style={{ flex: 1, overflowY: 'auto', padding: 'var(--sp-2)' }}>
                {snapshots.length === 0 ? (
                  <div style={{ padding: 'var(--sp-6)', textAlign: 'center', opacity: 0.4, fontSize: 'var(--fs-9)' }}>No snapshots yet</div>
                ) : (
                  snapshots.map((snapshot) => (
                    <div key={snapshot.id} className="drawing-row" style={{ flexDirection: 'column', alignItems: 'stretch', gap: 0 }}>
                      {snapshot.thumbnail && (
                        <img src={snapshot.thumbnail} alt="" style={{ width: '100%', height: 56, objectFit: 'cover', borderBottom: '1px solid var(--border1)', opacity: 0.85 }} />
                      )}
                      <div style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                      <div className="drawing-row-main">
                        <div className="drawing-row-name">{snapshot.note}</div>
                        <div className="drawing-row-meta" style={{ fontSize: 'var(--fs-7)' }}>
                          {new Date(snapshot.createdAt).toLocaleString([], { dateStyle: 'short', timeStyle: 'short' })}
                        </div>
                      </div>
                      <div className="drawing-manager-actions" style={{ display: 'flex', gap: '2px' }}>
                        <button
                          className="ue-btn small"
                          title="Restore"
                          onClick={() => {
                            undoRef.current.push(drawingsRef.current.map((drawing) => ({ ...drawing })));
                            redoRef.current = [];
                            setDrawings(snapshot.drawings.map((drawing) => structuredClone(drawing)));
                            setReplayIndex(snapshot.replayIndex);
                            setReplayEnabled(snapshot.replayIndex != null);
                            setSelectedIds([]);
                            setEditorOpen(false);
                            app.showToast('Snapshot restored');
                          }}
                        >
                          RESTORE
                        </button>
                        <button
                          className="ue-btn small danger"
                          title="Delete"
                          onClick={() => {
                            const nextSnapshots = snapshots.filter((item) => item.id !== snapshot.id);
                            setSnapshotsState(nextSnapshots);
                            saveSnapshots(app.symbol, currentScope, nextSnapshots);
                          }}
                        >
                          <TrashIcon />
                        </button>
                      </div>
                      </div>
                    </div>
                  ))
                )}
              </div>
            </div>
          )}

          {managerTab === 'replay' && (
            <div className="drawings-manager-body">
              <div className="drawings-manager-replay">
                <button
                  className="ue-btn small"
                  disabled={!replayEnabled || replayIndex == null || replayIndex <= 1}
                  onClick={() => {
                    setReplayEnabled(true);
                    setReplayIndex((prev) => Math.max(1, (prev ?? 1) - 1));
                    setReplayPlaying(false);
                  }}
                >
                  Prev
                </button>
                <button
                  className={`ue-btn small ${replayPlaying ? 'active' : ''}`}
                  disabled={!replayEnabled}
                  onClick={() => setReplayPlaying((prev) => !prev)}
                >
                  {replayPlaying ? 'Pause' : 'Play'}
                </button>
                <button
                  className="ue-btn small"
                  disabled={!replayEnabled || replayIndex == null || replayIndex >= (app.market?.candles.length ?? 0) - 1}
                  onClick={() => {
                    setReplayEnabled(true);
                    setReplayIndex((prev) => Math.min((app.market?.candles.length ?? 1) - 1, (prev ?? 0) + 1));
                    setReplayPlaying(false);
                  }}
                >
                  Next
                </button>
              </div>
              <label className="object-editor-field">
                <span>Bars</span>
                <input
                  type="range"
                  min={1}
                  max={Math.max(1, (app.market?.candles.length ?? 1) - 1)}
                  value={replayIndex ?? Math.max(1, (app.market?.candles.length ?? 1) - 1)}
                  onChange={(e) => {
                    setReplayEnabled(true);
                    setReplayPlaying(false);
                    setReplayIndex(Number(e.target.value));
                  }}
                />
              </label>
            </div>
          )}
        </aside>,
        managerPortalTarget
      )}

      <div ref={hostRef} className="chart-host" />
      <canvas ref={overlayRef} className="drawing-overlay" />
      <div
        className={`chart-interaction-layer ${app.drawTool === 'cursor' ? '' : 'active'}`}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
      />

      {/* RDS status banner */}
      {app.drawTool === 'rds' && rdsStep !== 'pick-dir' && (
        <div style={{
          position: 'absolute', top: 8, left: '50%', transform: 'translateX(-50%)',
          background: 'var(--bg3)', border: '1px solid var(--border3)',
          borderRadius: 3, padding: '5px 14px', zIndex: 1100,
          fontSize: 'var(--fs-9)', color: 'var(--text)', pointerEvents: 'none',
          display: 'flex', alignItems: 'center', gap: 10,
        }}>
          <span style={{ color: '#3a8fd0', fontWeight: 600 }}>RDS</span>
          <span>{(rdsStep === 'pick-x' || rdsStep === null) ? '① Click to set X level' : `① X = ${formatPrice(rdsX!, app.symbol)}  →  ② Click to set Y level`}</span>
          <span style={{ color: 'var(--text3)', fontSize: 'var(--fs-10)' }}>ESC to cancel</span>
        </div>
      )}

      {/* RDS direction chooser */}
      {rdsStep === 'pick-dir' && (
        <div style={{
          position: 'absolute', inset: 0, display: 'flex', alignItems: 'center',
          justifyContent: 'center', zIndex: 1200, background: 'rgba(0,0,0,0.35)',
        }}>
          <div style={{
            background: 'var(--bg2)', border: '1px solid var(--border3)',
            borderRadius: 4, padding: '20px 28px', display: 'flex', flexDirection: 'column',
            gap: 14, minWidth: 240,
          }}>
            <div style={{ fontSize: 'var(--fs-8)', color: 'var(--text)', fontWeight: 600, letterSpacing: '0.05em' }}>
              RDS — Choose Direction
            </div>
            <div style={{ fontSize: 'var(--fs-10)', color: 'var(--text3)', lineHeight: 1.5 }}>
              X = <strong style={{ color: '#3a8fd0' }}>{formatPrice(rdsX!, app.symbol)}</strong>
              &nbsp;&nbsp;Y = <strong style={{ color: '#f0a030' }}>{formatPrice(rdsY!, app.symbol)}</strong>
            </div>
            <div style={{ display: 'flex', gap: 10 }}>
              <button
                className="ue-btn strong"
                style={{ flex: 1, background: '#26a69a', borderColor: '#26a69a', color: '#fff', height: 34, fontSize: 'var(--fs-8)', fontWeight: 600 }}
                onClick={() => onRdsDirectionChosen('bull')}
              >▲ Bullish</button>
              <button
                className="ue-btn danger"
                style={{ flex: 1, background: '#ef5350', borderColor: '#ef5350', color: '#fff', height: 34, fontSize: 'var(--fs-8)', fontWeight: 600 }}
                onClick={() => onRdsDirectionChosen('bear')}
              >▼ Bearish</button>
            </div>
            <button
              className="ue-btn small"
              style={{ alignSelf: 'center' }}
              onClick={() => { setRdsStep(null); setRdsX(null); setRdsY(null); app.setDrawTool('cursor'); }}
            >Cancel</button>
          </div>
        </div>
      )}

      {selectedDrawing && !editorOpen && selectedIds.length === 1 && (
        <div
          className="floating-toolbar"
          onPointerDown={(e) => e.stopPropagation()}
          style={{
            position: 'absolute',
            top: 20,
            left: '50%',
            transform: 'translateX(-50%)',
            display: 'flex',
            alignItems: 'center',
            gap: '8px',
            padding: '6px 12px',
            background: 'var(--bg1)',
            border: '1px solid var(--border2)',
            boxShadow: '0 4px 16px rgba(0,0,0,0.4)',
            zIndex: 900,
            borderRadius: 0,
          }}
        >
          <span style={{ fontSize: 'var(--fs-9)', fontWeight: 600, color: 'var(--text)', textTransform: 'uppercase', marginRight: '8px' }}>
            {selectedDrawing.type}
          </span>
          <ColorControl
            value={selectedDrawing.type === 'rect' ? (selectedDrawing as any).fillColor : selectedDrawing.color}
            onChange={(c) => updateSelected((d) => 
              d.type === 'rect' ? { ...d, fillColor: c } : { ...d, color: c }
            )}
          />
          
          <div style={{ width: 1, height: 16, background: 'var(--border2)' }} />
          
          {/* Quick Template Button */}
          <button
            className={`ue-btn small ${floatingMenu?.type === 'templates' ? 'active' : ''}`}
            title="Templates"
            onClick={(e) => {
              const rect = e.currentTarget.getBoundingClientRect();
              setFloatingMenu(prev => prev ? null : { type: 'templates', x: rect.left, y: rect.bottom + 8 });
            }}
          >
            &lt;T&gt;
          </button>

          <div style={{ width: 1, height: 16, background: 'var(--border2)' }} />

          <button
            className="ue-btn small"
            title="Duplicate"
            onClick={() => {
              undoRef.current.push(drawingsRef.current.map(d => ({ ...d })));
              redoRef.current = [];
              const clone = structuredClone(selectedDrawing);
              clone.id = makeId();
              clone.name = `${selectedDrawing.name} Copy`;
              setDrawings(prev => [...prev, clone]);
              setSelectedIds([clone.id]);
              app.showToast('Duplicated');
            }}
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
          </button>

          <button
            className={`ue-btn small ${selectedDrawing.locked ? 'active' : ''}`}
            title={selectedDrawing.locked ? 'Unlock' : 'Lock'}
            onClick={() => updateSelected((d) => ({ ...d, locked: !d.locked }))}
          >
            {selectedDrawing.locked ? <LockIcon /> : <UnlockIcon />}
          </button>
          <button
            className={`ue-btn small ${!selectedDrawing.visible ? 'active' : ''}`}
            title={selectedDrawing.visible ? 'Hide' : 'Show'}
            onClick={() => updateSelected((d) => ({ ...d, visible: !d.visible }))}
          >
            {selectedDrawing.visible ? <EyeIcon /> : <EyeOffIcon />}
          </button>
          <button
            className="ue-btn small"
            title="Settings"
            onClick={() => setEditorOpen(true)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
              <path d="M8 11.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z" stroke="currentColor" strokeWidth="1.2"/>
              <path d="M2.5 8h1M12.5 8h1M8 2.5v1M8 12.5v1M4 4l.7.7M11.3 11.3l.7.7M4 12l.7-.7M11.3 4.7l.7-.7" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round"/>
            </svg>
          </button>
          <button
            className="ue-btn small danger"
            title="Delete"
            onClick={() => deleteSelected()}
          >
            <TrashIcon />
          </button>
        </div>
      )}

      {/* Multi-selection toolbar */}
      {selectedIds.length > 1 && (
        <div
          className="floating-toolbar"
          onPointerDown={(e) => e.stopPropagation()}
          style={{
            position: 'absolute', top: 20, left: '50%', transform: 'translateX(-50%)',
            display: 'flex', alignItems: 'center', gap: '8px', padding: '6px 12px',
            background: 'var(--bg1)', border: '1px solid var(--border2)',
            boxShadow: '0 4px 16px rgba(0,0,0,0.4)', zIndex: 900, borderRadius: 0,
          }}
        >
          <span style={{ fontSize: 'var(--fs-9)', color: 'var(--text3)', textTransform: 'uppercase', marginRight: 4 }}>
            {selectedIds.length} selected
          </span>
          <div style={{ width: 1, height: 16, background: 'var(--border2)' }} />
          <ColorControl
            value={drawings.find(d => selectedIds.includes(d.id))?.color ?? '#888'}
            onChange={(c) => setDrawings(prev => prev.map(d => selectedIds.includes(d.id) ? { ...d, color: c, ...(d.type === 'rect' ? { fillColor: c } : {}) } : d))}
          />
          <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text3)' }}>Opacity</span>
          <input type="range" min={0} max={1} step={0.05}
            style={{ width: 64 }}
            defaultValue={0.8}
            onChange={(e) => setDrawings(prev => prev.map(d => selectedIds.includes(d.id) && d.type === 'rect' ? { ...d, fillAlpha: Number(e.target.value) } : d))}
          />
          <div style={{ width: 1, height: 16, background: 'var(--border2)' }} />
          <button className="ue-btn small" onClick={() => { setDrawings(prev => prev.map(d => selectedIds.includes(d.id) ? { ...d, visible: false } : d)); }}>Hide</button>
          <button className="ue-btn small" onClick={() => { setDrawings(prev => prev.map(d => selectedIds.includes(d.id) ? { ...d, locked: true } : d)); }}>Lock</button>
          <button className="ue-btn small danger" onClick={() => {
            undoRef.current.push(drawingsRef.current.map(d => ({ ...d })));
            redoRef.current = [];
            setDrawings(prev => prev.filter(d => !selectedIds.includes(d.id)));
            setSelectedIds([]);
          }}>Delete all</button>
        </div>
      )}

      {/* Floating Menu (Templates) */}
      {floatingMenu?.type === 'templates' && selectedDrawing && (
        <div
          className="context-menu"
          style={{
            position: 'absolute',
            left: floatingMenu.x,
            top: floatingMenu.y,
            background: 'var(--bg1)',
            border: '1px solid var(--border3)',
            borderRadius: 0,
            boxShadow: '0 8px 24px rgba(0,0,0,0.5)',
            zIndex: 2000,
            minWidth: 140,
            padding: '4px',
          }}
          onPointerDown={(e) => e.stopPropagation()}
        >
          <div style={{ fontSize: 'var(--fs-8)', padding: '4px 8px', opacity: 0.5, textTransform: 'uppercase' }}>Templates</div>
          {getTemplatesForType(selectedDrawing.type).length === 0 ? (
            <div style={{ fontSize: 'var(--fs-9)', padding: '8px', opacity: 0.4 }}>No templates</div>
          ) : (
            getTemplatesForType(selectedDrawing.type).map(k => (
              <button
                key={k}
                className="context-menu-item"
                onClick={() => {
                  onApplyTemplate(k, selectedDrawing.id);
                  setFloatingMenu(null);
                }}
              >
                {k.split(':')[1]}
              </button>
            ))
          )}
        </div>
      )}

      {/* Context Menu */}
      {contextMenu && (
        <div
          className="context-menu"
          style={{
            position: 'absolute',
            left: contextMenu.x,
            top: contextMenu.y,
            background: 'var(--bg1)',
            border: '1px solid var(--border3)',
            borderRadius: 0,
            boxShadow: '0 8px 24px rgba(0,0,0,0.5)',
            zIndex: 2000,
            minWidth: 140,
            padding: '4px',
          }}
          onPointerDown={(e) => e.stopPropagation()}
        >
          {contextMenu.drawingId ? (
            <>
              <button className="context-menu-item" onClick={() => { setEditorOpen(true); setContextMenu(null); }}>Settings</button>
              <button className="context-menu-item" onClick={() => { 
                const d = drawings.find(item => item.id === contextMenu.drawingId);
                if (d) {
                  const clone = structuredClone(d);
                  clone.id = makeId();
                  clone.name = `${d.name} Copy`;
                  setDrawings(prev => [...prev, clone]);
                  setSelectedIds([clone.id]);
                  app.showToast('Duplicated');
                }
                setContextMenu(null); 
              }}>Duplicate</button>
              <div style={{ height: 1, background: 'var(--border1)', margin: '4px 0' }} />
              <button className="context-menu-item" onClick={() => {
                setDrawings(prev => prev.map(d =>
                  d.id === contextMenu.drawingId ? { ...d, zLayer: 'front' as const } : d
                ).sort((a, b) => {
                  if (a.id === contextMenu.drawingId) return 1;
                  if (b.id === contextMenu.drawingId) return -1;
                  return 0;
                }));
                setContextMenu(null);
              }}>Bring to Front</button>
              <button className="context-menu-item" onClick={() => {
                setDrawings(prev => prev.map(d =>
                  d.id === contextMenu.drawingId ? { ...d, zLayer: 'back' as const } : d
                ).sort((a, b) => {
                  if (a.id === contextMenu.drawingId) return -1;
                  if (b.id === contextMenu.drawingId) return 1;
                  return 0;
                }));
                setContextMenu(null);
              }}>Send to Back</button>
              <div style={{ height: 1, background: 'var(--border1)', margin: '4px 0' }} />
              <button className="context-menu-item danger" onClick={() => { deleteSelected(); setContextMenu(null); }}>Delete</button>
            </>
          ) : (
            <>
              <button className="context-menu-item" onClick={() => { app.setDrawTool('trend'); setContextMenu(null); }}>Add Line</button>
              <button className="context-menu-item" onClick={() => { app.setDrawTool('rect'); setContextMenu(null); }}>Add Box</button>
              <button className="context-menu-item" onClick={() => { app.setDrawTool('text'); setContextMenu(null); }}>Add Text</button>
              <div style={{ height: 1, background: 'var(--border1)', margin: '4px 0' }} />
              <button className="context-menu-item" onClick={() => { app.setSettingsOpen(true); setContextMenu(null); }}>Chart Settings</button>
            </>
          )}
        </div>
      )}

      {selectedDrawing && editorOpen && (
        <div
          className="object-editor"
          style={{ 
            left: editorPanelPos.x, 
            top: editorPanelPos.y,
            width: editorPanelSize.width,
            height: editorPanelSize.height,
            maxHeight: 'calc(100% - 100px)',
            overflow: 'hidden',
            display: 'flex',
            flexDirection: 'column',
            zIndex: 1000,
            boxShadow: '0 8px 32px rgba(0,0,0,0.5)',
            border: '1px solid var(--border3)',
            borderRadius: 0,
            background: 'var(--bg1)',
            position: 'absolute'
          }}
          onPointerMove={(e) => {
            if (panelDrag?.kind === 'editor' && panelDrag.pointerId === e.pointerId && surfaceRef.current) {
              const rect = surfaceRef.current.getBoundingClientRect();
              const nextX = Math.max(0, Math.min(rect.width - 100, e.clientX - rect.left - panelDrag.offsetX));
              const nextY = Math.max(0, Math.min(rect.height - 40, e.clientY - rect.top - panelDrag.offsetY));
              setEditorPanelPos({ x: nextX, y: nextY });
            } else if (panelResize && panelResize.pointerId === e.pointerId) {
              const deltaX = e.clientX - panelResize.startX;
              const deltaY = e.clientY - panelResize.startY;
              setEditorPanelSize({
                width: Math.max(180, panelResize.startWidth + deltaX),
                height: Math.max(100, panelResize.startHeight + deltaY),
              });
            }
          }}
          onPointerUp={(e) => {
            if (panelDrag?.kind === 'editor' && panelDrag.pointerId === e.pointerId) {
              setPanelDrag(null);
            }
            if (panelResize && panelResize.pointerId === e.pointerId) {
              setPanelResize(null);
            }
          }}
        >
          <div
            className="object-editor-header"
            style={{ cursor: 'move', userSelect: 'none', flexShrink: 0 }}
            onPointerDown={(e) => {
              const panel = e.currentTarget.parentElement;
              if (!panel) return;
              const panelRect = panel.getBoundingClientRect();
              e.currentTarget.setPointerCapture(e.pointerId);
              setPanelDrag({
                kind: 'editor',
                pointerId: e.pointerId,
                offsetX: e.clientX - panelRect.left,
                offsetY: e.clientY - panelRect.top,
              });
            }}
          >
            <span className="object-editor-title" style={{ fontWeight: 600, fontSize: 'var(--fs-10)', textTransform: 'uppercase' }}>
              {selectedDrawing.type} SETTINGS
            </span>
            <button 
              className="object-editor-close" 
              onPointerDown={(e) => e.stopPropagation()}
              onClick={(e) => { 
                e.stopPropagation();
                setSelectedIds([]); 
                setEditorOpen(false); 
              }}
            >
              ×
            </button>
          </div>

          <div className="object-editor-body" style={{ gap: 0, padding: 0 }}>

            {/* ── Name ─────────────────────────────────────────────────── */}
            <div style={{ padding: '7px 10px', borderBottom: '1px solid var(--border1)', display: 'flex', alignItems: 'center', gap: 6 }}>
              <span style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', textTransform: 'uppercase', letterSpacing: '0.08em', flexShrink: 0 }}>Name</span>
              <input
                value={selectedDrawing.name}
                onChange={(e) => updateSelected((d) => ({ ...d, name: e.target.value }))}
                style={{ flex: 1, fontSize: 'var(--fs-9)' }}
              />
            </div>

            {/* ── Appearance ───────────────────────────────────────────── */}
            <div style={{ padding: '8px 10px', borderBottom: '1px solid var(--border1)' }}>
              <div style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 7 }}>Appearance</div>

              {'color' in selectedDrawing && (
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
                  <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text2)', width: 38 }}>Color</span>
                  <ColorControl value={selectedDrawing.color} ariaLabel="Drawing color"
                    onChange={(v) => updateSelected((d) => 'color' in d ? { ...d, color: v } : d)} />
                </div>
              )}

              {'width' in selectedDrawing && (
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
                  <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text2)', width: 38 }}>Width</span>
                  <div style={{ display: 'flex', gap: 2 }}>
                    {[1, 2, 3, 4, 5].map((w) => (
                      <button key={w}
                        className={`ue-btn small${(selectedDrawing as any).width === w ? ' active' : ''}`}
                        style={{ padding: '3px 7px', minWidth: 0 }}
                        onClick={() => updateSelected((d) => 'width' in d ? { ...d, width: w } : d)}
                      >
                        <svg width="14" height={w * 2 + 2} style={{ display: 'block' }}>
                          <line x1="0" y1={w} x2="14" y2={w} stroke="currentColor" strokeWidth={w} />
                        </svg>
                      </button>
                    ))}
                  </div>
                </div>
              )}

              {'strokeStyle' in selectedDrawing && (
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text2)', width: 38 }}>Style</span>
                  <div style={{ display: 'flex', gap: 2 }}>
                    {(['solid', 'dashed', 'dotted'] as const).map((s) => (
                      <button key={s}
                        className={`ue-btn small${(selectedDrawing as any).strokeStyle === s ? ' active' : ''}`}
                        style={{ padding: '2px 8px', minWidth: 0, fontSize: 11, letterSpacing: s === 'solid' ? 0 : 1 }}
                        onClick={() => updateSelected((d) => 'strokeStyle' in d ? { ...d, strokeStyle: s } : d)}
                      >
                        {s === 'solid' ? '━━' : s === 'dashed' ? '╌╌' : '···'}
                      </button>
                    ))}
                  </div>
                </div>
              )}
            </div>

            {/* ── Rectangle fill ───────────────────────────────────────── */}
            {selectedDrawing.type === 'rect' && (
              <div style={{ padding: '8px 10px', borderBottom: '1px solid var(--border1)' }}>
                <div style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 7 }}>Fill</div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
                  <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text2)', width: 38 }}>Color</span>
                  <ColorControl value={selectedDrawing.fillColor} ariaLabel="Fill color"
                    onChange={(v) => updateSelected((d) => d.type === 'rect' ? { ...d, fillColor: v } : d)} />
                  <label style={{ display: 'flex', alignItems: 'center', gap: 4, marginLeft: 8 }}>
                    <input type="checkbox" checked={selectedDrawing.borderVisible}
                      onChange={(e) => updateSelected((d) => d.type === 'rect' ? { ...d, borderVisible: e.target.checked } : d)} />
                    <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text2)' }}>Border</span>
                  </label>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text2)', width: 38 }}>Alpha</span>
                  <input type="range" min={0} max={1} step={0.05} value={selectedDrawing.fillAlpha} style={{ flex: 1 }}
                    onChange={(e) => updateSelected((d) => d.type === 'rect' ? { ...d, fillAlpha: Number(e.target.value) } : d)} />
                  <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text3)', minWidth: 30, textAlign: 'right' }}>
                    {Math.round(selectedDrawing.fillAlpha * 100)}%
                  </span>
                </div>
              </div>
            )}

            {/* ── Position / Price ─────────────────────────────────────── */}
            {(selectedDrawing.type === 'trend' || selectedDrawing.type === 'rect' || selectedDrawing.type === 'fib' || selectedDrawing.type === 'hline') && (
              <div style={{ padding: '8px 10px', borderBottom: '1px solid var(--border1)' }}>
                <div style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 7 }}>Position</div>
                {selectedDrawing.type === 'hline' && (
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text2)', width: 38 }}>Price</span>
                    <input type="number" step="any" value={selectedDrawing.price} style={{ flex: 1 }}
                      onChange={(e) => updateSelected((d) => d.type === 'hline' ? { ...d, price: Number(e.target.value) } : d)} />
                  </div>
                )}
                {(selectedDrawing.type === 'trend' || selectedDrawing.type === 'fib') && (
                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 6 }}>
                    <div>
                      <div style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', marginBottom: 2 }}>Start px</div>
                      <input type="number" step="any" value={selectedDrawing.start.price} style={{ width: '100%' }}
                        onChange={(e) => updateSelected((d) => (d.type === 'trend' || d.type === 'fib') ? { ...d, start: { ...d.start, price: Number(e.target.value) } } : d)} />
                    </div>
                    <div>
                      <div style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', marginBottom: 2 }}>End px</div>
                      <input type="number" step="any" value={selectedDrawing.end.price} style={{ width: '100%' }}
                        onChange={(e) => updateSelected((d) => (d.type === 'trend' || d.type === 'fib') ? { ...d, end: { ...d.end, price: Number(e.target.value) } } : d)} />
                    </div>
                  </div>
                )}
                {selectedDrawing.type === 'rect' && (
                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 6 }}>
                    <div>
                      <div style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', marginBottom: 2 }}>High</div>
                      <input type="number" step="any" value={Math.max(selectedDrawing.start.price, selectedDrawing.end.price)} style={{ width: '100%' }}
                        onChange={(e) => updateSelected((d) => {
                          if (d.type !== 'rect') return d;
                          const n = Number(e.target.value);
                          return d.start.price >= d.end.price ? { ...d, start: { ...d.start, price: n } } : { ...d, end: { ...d.end, price: n } };
                        })} />
                    </div>
                    <div>
                      <div style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', marginBottom: 2 }}>Low</div>
                      <input type="number" step="any" value={Math.min(selectedDrawing.start.price, selectedDrawing.end.price)} style={{ width: '100%' }}
                        onChange={(e) => updateSelected((d) => {
                          if (d.type !== 'rect') return d;
                          const n = Number(e.target.value);
                          return d.start.price <= d.end.price ? { ...d, start: { ...d.start, price: n } } : { ...d, end: { ...d.end, price: n } };
                        })} />
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* ── Label ────────────────────────────────────────────────── */}
            {(selectedDrawing.type === 'trend' || selectedDrawing.type === 'rect' || selectedDrawing.type === 'hline' || selectedDrawing.type === 'vline') && (
              <div style={{ padding: '8px 10px', borderBottom: '1px solid var(--border1)' }}>
                <div style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 7 }}>Label</div>
                <input value={selectedDrawing.label} placeholder="optional" style={{ width: '100%', marginBottom: 6 }}
                  onChange={(e) => updateSelected((d) => d.type !== 'text' && d.type !== 'fib' ? { ...d, label: e.target.value } : d)} />
                <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6 }}>
                  <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text2)', width: 38 }}>Size</span>
                  <input type="range" min={9} max={28} step={1} value={(selectedDrawing as any).labelFontSize ?? 11} style={{ flex: 1 }}
                    onChange={(e) => updateSelected((d) => d.type !== 'text' ? { ...d, labelFontSize: Number(e.target.value) } : d)} />
                  <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text3)', minWidth: 22, textAlign: 'right' }}>
                    {(selectedDrawing as any).labelFontSize ?? 11}
                  </span>
                </div>
                {(selectedDrawing.type === 'trend' || selectedDrawing.type === 'rect') && (
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text2)', width: 38 }}>Pos</span>
                    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4,1fr)', gap: 2, flex: 1 }}>
                      {(['top-left','top-right','bottom-left','bottom-right'] as const).map((p) => (
                        <button key={p}
                          className={`ue-btn small${selectedDrawing.labelPosition === p ? ' active' : ''}`}
                          style={{ fontSize: 'var(--fs-7)', padding: '2px 3px', textAlign: 'center' }}
                          onClick={() => updateSelected((d) => (d.type==='trend'||d.type==='rect'||d.type==='fib') ? { ...d, labelPosition: p } : d)}
                        >
                          {p.replace('top-','T').replace('bottom-','B').replace('left','L').replace('right','R')}
                        </button>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            )}

            {/* ── Text content ─────────────────────────────────────────── */}
            {selectedDrawing.type === 'text' && (
              <div style={{ padding: '8px 10px', borderBottom: '1px solid var(--border1)' }}>
                <div style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 7 }}>Content</div>
                <input value={selectedDrawing.text} style={{ width: '100%', marginBottom: 6 }}
                  onChange={(e) => updateSelected((d) => d.type === 'text' ? { ...d, text: e.target.value } : d)} />
                <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text2)', width: 38 }}>Size</span>
                  <input type="range" min={9} max={32} step={1} value={selectedDrawing.fontSize} style={{ flex: 1 }}
                    onChange={(e) => updateSelected((d) => d.type === 'text' ? { ...d, fontSize: Number(e.target.value) } : d)} />
                  <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text3)', minWidth: 22, textAlign: 'right' }}>
                    {selectedDrawing.fontSize}
                  </span>
                </div>
              </div>
            )}

            {/* ── Fibonacci levels ─────────────────────────────────────── */}
            {selectedDrawing.type === 'fib' && (
              <div style={{ padding: '8px 10px', borderBottom: '1px solid var(--border1)' }}>
                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 7 }}>
                  <div style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', textTransform: 'uppercase', letterSpacing: '0.08em' }}>Levels</div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    <label style={{ display: 'flex', alignItems: 'center', gap: 3 }}>
                      <input type="checkbox" checked={selectedDrawing.extendRight}
                        onChange={(e) => updateSelected((d) => d.type === 'fib' ? { ...d, extendRight: e.target.checked } : d)} />
                      <span style={{ fontSize: 'var(--fs-8)', color: 'var(--text2)' }}>Extend</span>
                    </label>
                    <select value={selectedDrawing.priceLabelAlign}
                      onChange={(e) => updateSelected((d) => d.type === 'fib' ? { ...d, priceLabelAlign: e.target.value as PriceLabelAlign } : d)}
                      style={{ fontSize: 'var(--fs-8)' }}
                    >
                      <option value="right">Right</option>
                      <option value="left">Left</option>
                    </select>
                  </div>
                </div>
                <div style={{ display: 'grid', gridTemplateColumns: '14px 1fr 44px 16px 18px', gap: 3, alignItems: 'center', marginBottom: 3 }}>
                  <span style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)' }}> </span>
                  <span style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)' }}>Value</span>
                  <span style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)' }}>Label</span>
                  <span style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)' }}>Clr</span>
                  <span> </span>
                </div>
                {selectedDrawing.levels.map((level) => (
                  <div key={level.id} style={{ display: 'grid', gridTemplateColumns: '14px 1fr 44px 16px 18px', gap: 3, alignItems: 'center', marginBottom: 3 }}>
                    <input type="checkbox" checked={level.visible} style={{ width: 12, margin: 0 }}
                      onChange={(e) => updateSelected((d) => d.type === 'fib' ? { ...d, levels: d.levels.map((l) => l.id === level.id ? { ...l, visible: e.target.checked } : l) } : d)} />
                    <input type="number" step="any" value={level.value}
                      onChange={(e) => updateSelected((d) => d.type === 'fib' ? { ...d, levels: d.levels.map((l) => l.id === level.id ? { ...l, value: Number(e.target.value) } : l) } : d)} />
                    <input value={level.label}
                      onChange={(e) => updateSelected((d) => d.type === 'fib' ? { ...d, levels: d.levels.map((l) => l.id === level.id ? { ...l, label: e.target.value } : l) } : d)} />
                    <ColorControl value={level.color} ariaLabel="Fib level color"
                      onChange={(v) => updateSelected((d) => d.type === 'fib' ? { ...d, levels: d.levels.map((l) => l.id === level.id ? { ...l, color: v } : l) } : d)} />
                    <button className="ue-btn small danger" style={{ padding: '1px 3px', minWidth: 0 }}
                      onClick={() => updateSelected((d) => d.type === 'fib' ? { ...d, levels: d.levels.filter((l) => l.id !== level.id) } : d)}
                    >×</button>
                  </div>
                ))}
                <button className="ue-btn small" style={{ marginTop: 5 }}
                  onClick={() => updateSelected((d) => d.type === 'fib' ? { ...d, levels: [...d.levels, { id: makeId(), value: 0.5, color: d.color, visible: true, label: '0.5' }] } : d)}
                >+ Level</button>
              </div>
            )}

            {/* ── Templates ────────────────────────────────────────────── */}
            <div style={{ padding: '8px 10px', borderBottom: '1px solid var(--border1)' }}>
              <div style={{ fontSize: 'var(--fs-7)', color: 'var(--text3)', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 7 }}>Templates</div>
              <div style={{ display: 'flex', gap: 4, marginBottom: 4 }}>
                <select style={{ flex: 1 }} value="" onChange={(e) => onApplyTemplate(e.target.value, selectedDrawing.id)}>
                  <option value="" disabled>Apply template…</option>
                  {getTemplatesForType(selectedDrawing.type).map((k) => (
                    <option key={k} value={k}>{k.split(':')[1]}</option>
                  ))}
                </select>
              </div>
              <div style={{ display: 'flex', gap: 4 }}>
                <input placeholder="Save as…" value={templateNameInput}
                  onChange={(e) => setTemplateNameInput(e.target.value)}
                  style={{ flex: 1 }} />
                <button className="ue-btn small strong" onClick={() => onSaveTemplate(templateNameInput, selectedDrawing)}>Save</button>
              </div>
            </div>

          </div>

          <div className="object-editor-actions">
            <button
              className="ue-btn small"
              onPointerDown={(e) => e.stopPropagation()}
              onClick={(e) => {
                e.stopPropagation();
                const ids = [...selectedIdsRef.current];
                if (!ids.length) return;
                undoRef.current.push(drawingsRef.current.map((drawing) => ({ ...drawing })));
                redoRef.current = [];
                setDrawings((prev) => {
                  const selected = prev.filter((drawing) => ids.includes(drawing.id));
                  const copies = selected.map((drawing, index) => {
                    const clone = structuredClone(drawing);
                    clone.id = makeId();
                    clone.name = `${drawing.name} Copy`;
                    if (clone.type === 'text') {
                      clone.point = { ...clone.point, price: clone.point.price + (index + 1) * 0.5 };
                    } else if (clone.type === 'hline') {
                      clone.price += (index + 1) * 0.5;
                    } else if (clone.type === 'vline') {
                      clone.time += averageStepSeconds(allCandlesRef.current);
                    } else {
                      clone.start = { ...clone.start, price: clone.start.price + (index + 1) * 0.5 };
                      clone.end = { ...clone.end, price: clone.end.price + (index + 1) * 0.5 };
                    }
                    return clone;
                  });
                  setSelectedIds(copies.map((copy) => copy.id));
                  return [...prev, ...copies];
                });
              }}
            >
              Duplicate
            </button>
            <button
              className="ue-btn small danger"
              onPointerDown={(e) => e.stopPropagation()}
              onClick={(e) => { e.stopPropagation(); deleteSelected(); }}
            >
              Delete
            </button>
          </div>

          {/* Real Resize Handle */}
          <div
            style={{
              position: 'absolute',
              bottom: 0,
              right: 0,
              width: 14,
              height: 14,
              cursor: 'nwse-resize',
              zIndex: 1001,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
            onPointerDown={(e) => {
              e.stopPropagation();
              e.currentTarget.setPointerCapture(e.pointerId);
              setPanelResize({
                pointerId: e.pointerId,
                startWidth: editorPanelSize.width,
                startHeight: editorPanelSize.height,
                startX: e.clientX,
                startY: e.clientY,
              });
            }}
          >
            <svg width="8" height="8" viewBox="0 0 8 8">
              <path d="M7 1 L1 7 M7 4 L4 7 M7 7 L7 7" stroke="var(--text3)" strokeWidth="1" strokeLinecap="round" />
            </svg>
          </div>
        </div>
      )}

      {!app.market && (
        <div className="chart-loading">
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', gap: '12px' }}>
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="var(--accent)" strokeWidth="1.5" strokeLinecap="round" style={{ animation: 'spin 1s linear infinite' }}>
              <circle cx="12" cy="12" r="10" strokeOpacity="0.2" />
              <path d="M12 2a10 10 0 0 1 10 10" />
            </svg>
            <span style={{ fontSize: 'var(--fs-9)', letterSpacing: '0.1em', textTransform: 'uppercase', color: 'var(--text3)' }}>
              {app.loadingMarket ? 'Loading chart…' : 'No data'}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}

function EyeIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>
  );
}

function EyeOffIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/><line x1="1" y1="1" x2="23" y2="23"/></svg>
  );
}

function LockIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></svg>
  );
}

function UnlockIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 9.9-1"/></svg>
  );
}

function TrashIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></svg>
  );
}

function ChevronUpIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="18 15 12 9 6 15"/></svg>
  );
}

function ChevronDownIcon() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><polyline points="6 9 12 15 18 9"/></svg>
  );
}
