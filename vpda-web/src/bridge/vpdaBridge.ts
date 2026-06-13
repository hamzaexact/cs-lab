export type Workspace = 'market' | 'forecast' | 'sessionForecast' | 'shell' | 'inventory' | 'vchart';

export type ScenarioStep = {
  sequence: number;
  day: string;
  action: string;
  price: number;
  pdRef: string;
  condition: string;
  invalidation: number | null;
};
export type ForecastBias = 'AUTO' | 'BULLISH' | 'BEARISH';
export type ForecastTarget = 'TODAY' | 'NEXT' | 'CUSTOM';
export type DrawTool = 'cursor' | 'trend' | 'hline' | 'vline' | 'rect' | 'text' | 'fib' | 'rds';

export type Candle = {
  time: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
};

export type WatchItem = {
  symbol: string;
  description: string;
  price: number;
  changePct: number;
};

export type MarketSnapshot = {
  symbol: string;
  timeframe: string;
  candles: Candle[];
  last: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume24h: number;
  heatmap: Array<{ price: number; intensity: number }>;
};

export type ChartOverlay = {
  id: string;
  kind: 'zone' | 'line';
  label: string;
  role: 'manipulation' | 'target' | 'invalidation' | 'confluence' | 'extension';
  lower?: number;
  upper?: number;
  price?: number;
  colorHint: 'bullish' | 'bearish' | 'ifvg' | 'target' | 'invalidation' | 'extension';
};

export type ForecastScenario = {
  name: string;
  probability: number;
  direction: 'Bullish' | 'Bearish';
  sourceTag?: 'Search' | 'Legacy' | 'VM';
  detail: string;
  contextScore: number;
  pdConfluence: string[];
  pathSteps: ScenarioStep[];
  manipulationTarget: { name: string; price: number; status: string } | null;
  weeklyTarget: { name: string; price: number; status: string } | null;
  chartOverlays?: ChartOverlay[];
};

export type ThesisDefense = {
  label: string;
  role: string;
  state: string;
};

export type Thesis = {
  direction: string;
  confidence: number;
  invalidated: boolean;
  narrative: string[];
  defenses: ThesisDefense[];
};

export type ForecastResult = {
  symbol: string;
  target: string;
  bias: string;
  confidence: number;
  thesis?: Thesis | null;
  forecastedDay: string;
  weeklyProfile: string;
  currentPrice: number;
  summary: string;
  weeklyProfiles: Array<{ name: string; score: number }>;
  levels: Array<{ name: string; price: number; status: 'taken' | 'untouched' }>;
  vpdaScenarios: ForecastScenario[];
  searchScenarios: ForecastScenario[];
  vmScenarios: ForecastScenario[];
  warnings: string[];
  context: string[];
  sections: Array<{ title: string; lines: string[] }>;
};

export type SessionScenario = {
  rank: number;
  probability: number;
  description: string;
  targetPrice: number;
  targetLabel: string;
  roadmap: string[];
};

export type SessionLevel = {
  price: number;
  label: string;
  levelType: string;
  significance: number;
};

export type TargetLevel = {
  price: number;
  levelType: string;
  distancePips: number;
  sessionOrigin: string;
  significance: number;
};

export type ContextField = {
  field: string;
  value: string;
};

export type SessionLiquidityEntry = {
  session: string;
  high: number;
  low: number;
  highTaken: string;
  lowTaken: string;
};

export type IntradayMapEntry = {
  timeframe: string;
  gapType: string;
  status: string;
  rangeLower: number;
  rangeUpper: number;
  mid: number;
};

export type SessionForecastResult = {
  symbol: string;
  session: string;
  forecastTime: string;
  currentSession: string;
  sessionProgress: number;
  bias: string;
  confidence: number;
  currentPrice: number;
  summary: string;
  scenarios: SessionScenario[];
  levels: SessionLevel[];
  warnings: string[];
  context: string[];
  // Comprehensive CLI data fields
  upsideTargets: TargetLevel[];
  downsideTargets: TargetLevel[];
  longTermUpside: TargetLevel[];
  longTermDownside: TargetLevel[];
  contextFields: ContextField[];
  intradayMap: IntradayMapEntry[];
  sessionLiquidity: SessionLiquidityEntry[];
  sessionStats: string[];
  hitRateAnalytics: string[];
  criticalAlerts: string[];
  dynamicUpdates: string[];
};

export type SessionScenarioVisualizationLevel = {
  price: number;
  label: string;
  type: 'manipulation' | 'target' | 'invalidation' | 'support' | 'resistance';
  color: string;
};

export type SessionScenarioPathSegment = {
  description: string;
  startPrice: number;
  endPrice: number;
  candleCount: number;
  direction: 'bullish' | 'bearish';
  volatility: number;
};

export type SessionScenarioVisualization = {
  scenario: SessionScenario;
  symbol: string;
  timeframe: string;
  currentPrice: number;
  levels: SessionScenarioVisualizationLevel[];
  pathSegments: SessionScenarioPathSegment[];
};

export type ForecastDrawingRequest =
  | {
      id: number;
      kind: 'scenario';
      symbol: string;
      scenario: ForecastScenario;
    }
  | {
      id: number;
      kind: 'levels';
      symbol: string;
      levels: ForecastResult['levels'];
    }
  | {
      id: number;
      kind: 'clear';
      symbol: string;
    };

export type ShellLine = {
  level: 'info' | 'ok' | 'warn' | 'err';
  text: string;
};

export type StartupState = {
  activationRequired: boolean;
  activationMessage?: string;
  isAdmin?: boolean;
  warning?: { title: string; body: string; tone: 'info' | 'warning' | 'error' };
};

const isTauri = () => '__TAURI_INTERNALS__' in window;
const WEB_API_BASE = 'http://127.0.0.1:3017/api';

// In-memory cache for market data (session-only, cleared on restart)
const marketDataCache = new Map<string, MarketSnapshot>();

async function invokeOrNull<T>(command: string, args?: Record<string, unknown>): Promise<T | null> {
  if (!isTauri()) return null;
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<T>(command, args);
  } catch {
    return null;
  }
}

async function fetchOrNull<T>(path: string, init?: RequestInit): Promise<T | null> {
  try {
    // Add 60-second timeout for long-running forecasts
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), 60000);

    const response = await fetch(`${WEB_API_BASE}${path}`, {
      headers: {
        'Content-Type': 'application/json',
        ...(init?.headers ?? {}),
      },
      ...init,
      signal: controller.signal,
    });

    clearTimeout(timeoutId);

    if (!response.ok) {
      const payload = (await response.json().catch(() => null)) as { error?: string } | null;
      throw new Error(payload?.error ?? `Request failed: ${response.status}`);
    }

    return (await response.json()) as T;
  } catch (error) {
    if (error instanceof TypeError || (error instanceof Error && error.name === 'AbortError')) {
      return null;  // Network error or timeout - return null
    }
    throw error;
  }
}

export async function loadStartupState(): Promise<StartupState> {
  const local = await fetchOrNull<StartupState>('/startup-state');
  if (local) return local;

  return (await invokeOrNull<StartupState>('vpda_startup_state')) ?? {
    activationRequired: false,
    isAdmin: false,
    warning: {
      title: 'Bridge Offline',
      body: 'The local Rust web API is not running yet, so the shell is using fallback frontend data.',
      tone: 'info',
    },
  };
}

export async function loadWatchlist(): Promise<WatchItem[]> {
  const local = await fetchOrNull<WatchItem[]>('/watchlist');
  if (local) return local;

  const fallback: Array<[string, string, number, number]> = [
    ['EURUSD', 'Euro / US Dollar', 1.08421, 0.18],
    ['USDJPY', 'US Dollar / Japanese Yen', 151.41, -0.11],
    ['GBPUSD', 'British Pound / US Dollar', 1.27314, -0.22],
    ['USDCHF', 'US Dollar / Swiss Franc', 0.9021, 0.07],
    ['NQ', 'Nasdaq Futures', 18442.5, 0.62],
    ['ES', 'S&P Futures', 5288.25, 0.31],
  ];

  return fallback.map(([symbol, description, price, changePct]) => ({
    symbol,
    description,
    price,
    changePct,
  }));
}

function seeded(symbol: string, timeframe: string) {
  let seed = `${symbol}:${timeframe}`.split('').reduce((acc, ch) => acc + ch.charCodeAt(0), 13);
  return () => {
    seed = (seed * 9301 + 49297) % 233280;
    return seed / 233280;
  };
}

function basePrice(symbol: string) {
  switch (symbol) {
    case 'EURUSD': return 1.084;
    case 'GBPUSD': return 1.273;
    case 'XAUUSD': return 2192;
    case 'NQ': return 18420;
    case 'ES': return 5280;
    case 'BTCUSD': return 84200;
    default: return 100;
  }
}

function digits(symbol: string) {
  return symbol.length === 6 && symbol.endsWith('USD') ? 5 : symbol === 'XAUUSD' ? 2 : 2;
}

export async function loadMarket(symbol: string, timeframe: string): Promise<MarketSnapshot> {
  // Check cache first for instant switching
  const cacheKey = `${symbol}-${timeframe}`;
  const cached = marketDataCache.get(cacheKey);
  if (cached) return cached;

  const local = await fetchOrNull<MarketSnapshot>(
    `/market?symbol=${encodeURIComponent(symbol)}&timeframe=${encodeURIComponent(timeframe)}`,
  );
  if (local) {
    marketDataCache.set(cacheKey, local);
    return local;
  }

  const remote = await invokeOrNull<MarketSnapshot>('vpda_load_market_snapshot', { symbol, timeframe });
  if (remote) {
    marketDataCache.set(cacheKey, remote);
    return remote;
  }

  console.log('[DEBUG loadMarket] Generating fallback random data for:', timeframe);

  const rand = seeded(symbol, timeframe);
  const base = basePrice(symbol);
  const prec = digits(symbol);
  const step = timeframe === '1W' ? 86400 * 7 : timeframe === '4H' ? 14400 : timeframe === '1H' ? 3600 : timeframe === '15M' ? 900 : 86400;
  let price = base;
  const now = Math.floor(Date.now() / 1000);
  const candles: Candle[] = [];
  for (let i = 0; i < 260; i += 1) {
    const open = price;
    const drift = (rand() - 0.49) * base * 0.007;
    const close = Math.max(0.0001, open + drift);
    const high = Math.max(open, close) + rand() * base * 0.0035;
    const low = Math.min(open, close) - rand() * base * 0.0035;
    candles.push({
      time: now - (259 - i) * step,
      open: Number(open.toFixed(prec)),
      high: Number(high.toFixed(prec)),
      low: Number(low.toFixed(prec)),
      close: Number(close.toFixed(prec)),
      volume: Math.round(1000 + rand() * 12000),
    });
    price = close;
  }
  const last = candles[candles.length - 1];
  const high = Math.max(...candles.slice(-30).map((c) => c.high));
  const low = Math.min(...candles.slice(-30).map((c) => c.low));
  const result = {
    symbol,
    timeframe,
    candles,
    last: last.close,
    open: last.open,
    high: last.high,
    low: last.low,
    close: last.close,
    volume24h: candles.slice(-24).reduce((sum, c) => sum + c.volume, 0),
    heatmap: Array.from({ length: 18 }, (_, i) => ({
      price: Number((low + ((high - low) * i) / 17).toFixed(prec)),
      intensity: 0.25 + rand() * 0.75,
    })),
  };
  marketDataCache.set(cacheKey, result);
  return result;
}

export async function runForecast(request: {
  symbol: string;
  target: ForecastTarget;
  bias: ForecastBias;
  customDate: string | null;
}): Promise<ForecastResult> {
  const local = await fetchOrNull<ForecastResult>('/forecast', {
    method: 'POST',
    body: JSON.stringify(request),
  });
  if (local) return local;

  const remote = await invokeOrNull<ForecastResult>('vpda_run_forecast_daily', { request });
  if (remote) return remote;
  const market = await loadMarket(request.symbol, '1D');
  const bearish = request.bias === 'AUTO' ? market.close < market.open : request.bias === 'BEARISH';
  const prec = digits(request.symbol);
  return {
    symbol: request.symbol,
    target: request.target === 'CUSTOM' && request.customDate ? request.customDate : request.target,
    bias: bearish ? 'Bearish' : 'Bullish',
    confidence: bearish ? 76 : 68,
    forecastedDay:
      request.target === 'CUSTOM' && request.customDate
        ? request.customDate
        : request.target === 'NEXT'
          ? 'Next Session'
          : 'Today',
    weeklyProfile: bearish ? 'Wednesday Weekly Bearish Reversal' : 'Monday Low of the Week',
    currentPrice: market.last,
    summary: bearish
      ? 'VPDA reads current price action as a bearish delivery continuation after premium liquidity engineering.'
      : 'VPDA favors a bullish continuation path, but only after the expected manipulation phase resolves cleanly.',
    weeklyProfiles: [
      { name: bearish ? 'Wednesday Weekly Bearish Reversal' : 'Monday Low of the Week', score: 100 },
      { name: bearish ? 'Consolidation Thursday Bearish Reversal' : 'Thursday Bullish Expansion', score: 91 },
      { name: bearish ? 'Midweek Decline' : 'Wednesday Bullish Reversal', score: 74 },
      { name: bearish ? 'Monday Low of the Week' : 'Consolidation Rotation', score: 48 },
    ],
    levels: [
      { name: 'PDH', price: Number((market.high * 1.002).toFixed(prec)), status: bearish ? 'taken' : 'untouched' },
      { name: 'PDL', price: Number((market.low * 0.998).toFixed(prec)), status: bearish ? 'untouched' : 'taken' },
      { name: 'PWH', price: Number((market.high * 1.01).toFixed(prec)), status: 'untouched' },
      { name: 'PWL', price: Number((market.low * 0.99).toFixed(prec)), status: 'untouched' },
    ],
    vpdaScenarios: [
      {
        name: bearish ? 'Primary Delivery Continuation' : 'Manipulation Then Expansion',
        probability: bearish ? 78 : 63,
        direction: bearish ? 'Bearish' : 'Bullish',
        sourceTag: 'Legacy',
        detail: bearish
          ? 'Price accepts below local structure and delivers toward previous day low and weekly draw-on liquidity.'
          : 'Discount holds, liquidity is swept, and the market expands higher into the favored VPDA bias.',
        contextScore: 0.65,
        pdConfluence: bearish ? ['Bearish FVG above current price'] : ['Bullish FVG below current price'],
        pathSteps: bearish
          ? [
              { sequence: 1, day: 'Monday PM (setup)', action: 'HOLD', price: market.last, pdRef: 'Below premium', condition: 'Monday fails to reach premium array', invalidation: null },
              { sequence: 2, day: 'Tuesday London (02:00–05:00 NY)', action: 'SWEEP', price: Number((market.high * 1.001).toFixed(prec)), pdRef: 'BSL / Premium Array', condition: 'Tuesday sweeps premium forming HOW', invalidation: Number((market.high * 1.003).toFixed(prec)) },
              { sequence: 3, day: 'Tuesday NY (07:00–10:00 NY)', action: 'STRUCTURE', price: market.last, pdRef: '4H structure confirmation', condition: 'Price closes below Tuesday open confirming reversal', invalidation: null },
              { sequence: 4, day: 'Tuesday NY → Wednesday', action: 'DELIVER', price: Number((market.low * 0.998).toFixed(prec)), pdRef: 'SSL / Previous Week Low', condition: 'Bearish delivery to weekly draw', invalidation: null },
            ]
          : [
              { sequence: 1, day: 'Monday PM (setup)', action: 'HOLD', price: market.last, pdRef: 'Above discount', condition: 'Monday fails to reach discount array', invalidation: null },
              { sequence: 2, day: 'Tuesday London (02:00–05:00 NY)', action: 'SWEEP', price: Number((market.low * 0.999).toFixed(prec)), pdRef: 'SSL / Discount Array', condition: 'Tuesday sweeps discount forming LOW of Week', invalidation: Number((market.low * 0.997).toFixed(prec)) },
              { sequence: 3, day: 'Tuesday NY (07:00–10:00 NY)', action: 'STRUCTURE', price: market.last, pdRef: '4H structure confirmation', condition: 'Price closes above Tuesday open confirming reversal', invalidation: null },
              { sequence: 4, day: 'Tuesday NY → Thursday', action: 'DELIVER', price: Number((market.high * 1.002).toFixed(prec)), pdRef: 'BSL / Previous Week High', condition: 'Bullish delivery to weekly draw', invalidation: null },
            ],
        manipulationTarget: {
          name: bearish ? 'Tuesday HOW — BSL swept' : 'Tuesday LOW — SSL swept',
          price: bearish ? Number((market.high * 1.001).toFixed(prec)) : Number((market.low * 0.999).toFixed(prec)),
          status: 'untouched',
        },
        weeklyTarget: {
          name: bearish ? 'SSL / Previous Week Low' : 'BSL / Previous Week High',
          price: bearish ? Number((market.low * 0.998).toFixed(prec)) : Number((market.high * 1.002).toFixed(prec)),
          status: 'untouched',
        },
      },
      {
        name: bearish ? 'Short Squeeze Failure' : 'Premium Rejection',
        probability: bearish ? 22 : 37,
        direction: bearish ? 'Bullish' : 'Bearish',
        sourceTag: 'Legacy',
        detail: 'Secondary invalidation path kept visible for execution planning and scenario management.',
        contextScore: 0.25,
        pdConfluence: [],
        pathSteps: [],
        manipulationTarget: null,
        weeklyTarget: null,
      },
    ],
    searchScenarios: [
      {
        name: bearish ? 'Search Shadow Bearish Rebalance Path 1' : 'Search Shadow Bullish Rebalance Path 1',
        probability: bearish ? 71 : 69,
        direction: bearish ? 'Bearish' : 'Bullish',
        sourceTag: 'Search',
        detail: 'Shadow search preview branch exposed for engine diagnostics.',
        contextScore: 0.71,
        pdConfluence: bearish ? ['Bearish FVG above current price'] : ['Bullish FVG below current price'],
        pathSteps: [],
        manipulationTarget: null,
        weeklyTarget: null,
      },
    ],
    vmScenarios: [
      {
        name: bearish ? 'Historical Friday Bearish Close' : 'Historical Friday Bullish Close',
        probability: bearish ? 54.2 : 57.1,
        direction: bearish ? 'Bearish' : 'Bullish',
        sourceTag: 'VM',
        detail: 'Pattern matching engine output from the historical archive.',
        contextScore: 0.5,
        pdConfluence: [],
        pathSteps: [],
        manipulationTarget: null,
        weeklyTarget: null,
      },
      {
        name: bearish ? 'Historical Recovery Attempt' : 'Historical Mean Reversion',
        probability: bearish ? 45.8 : 42.9,
        direction: bearish ? 'Bullish' : 'Bearish',
        sourceTag: 'VM',
        detail: 'Lower-ranked pattern path from the VM engine.',
        contextScore: 0.3,
        pdConfluence: [],
        pathSteps: [],
        manipulationTarget: null,
        weeklyTarget: null,
      },
    ],
    warnings: [
      bearish
        ? `Price is trading near a bearish FVG above ${market.last.toFixed(prec)}; watch for downside reaction.`
        : `Price is trading near a bullish reaction zone; monitor for continuation after manipulation.`
    ],
    context: [
      'Forecast is a full workspace, not a side panel, and is ready for real Rust streaming output.',
      request.target === 'CUSTOM' ? `Custom date selected: ${request.customDate}.` : 'Today / Next mode is already modeled in the request bridge.',
    ],
    sections: [
      {
        title: 'ANALYZE DAILY SNAPSHOT',
        lines: [
          `Current Price | ${market.last.toFixed(prec)} | ${bearish ? 'bearish' : 'bullish'}`,
          `Orderflow | ${bearish ? 'Bearish continuation' : 'Bullish continuation'}`,
        ],
      },
      {
        title: 'WEEKLY PROFILE ANALYSIS',
        lines: ['Fallback mock profile output.'],
      },
      {
        title: 'HISTORICAL PATTERN MATCHING',
        lines: ['Fallback mock pattern output.'],
      },
    ],
  };
}

export async function runSessionForecast(request: {
  symbol: string;
  targetDate: string | null;
  targetTime: string | null;
  bias: ForecastBias;
}): Promise<SessionForecastResult> {
  const response = await fetchOrNull<SessionForecastResult>('/forecast/session', {
    method: 'POST',
    body: JSON.stringify(request),
  });

  if (response) return response;

  // Fallback mock data if API unavailable
  const market = await loadMarket(request.symbol, '1h');
  return {
    symbol: request.symbol,
    session: 'NewYorkAM',
    forecastTime: new Date().toISOString(),
    currentSession: 'NewYorkAM',
    sessionProgress: 0.45,
    bias: request.bias === 'AUTO' ? 'Bullish' : request.bias,
    confidence: 0.68,
    currentPrice: market.last,
    summary: `Session forecast for ${request.symbol} during New York AM session`,
    scenarios: [],
    levels: [],
    warnings: [],
    context: ['Session forecasting active', 'Monitoring intraday structure'],
    upsideTargets: [],
    downsideTargets: [],
    longTermUpside: [],
    longTermDownside: [],
    contextFields: [],
    intradayMap: [],
    sessionLiquidity: [],
    sessionStats: [],
    hitRateAnalytics: [],
    criticalAlerts: [],
    dynamicUpdates: [],
  };
}

export type AccountInfo = {
  machineId: string;
  licenseKey: string;
  status: string;
  dailyRemaining: number;
  activatedSince: string | null;
};

export type ActivateResponse = {
  success: boolean;
  message: string;
  code: number;
};

export async function activateLicense(license: string): Promise<ActivateResponse> {
  const local = await fetchOrNull<ActivateResponse>('/activate', {
    method: 'POST',
    body: JSON.stringify({ license }),
  });
  if (local) return local;
  return { success: false, message: 'Bridge offline — server not running.', code: 99 };
}

export async function loadAccount(): Promise<AccountInfo | null> {
  return fetchOrNull<AccountInfo>('/account');
}

export type DbLogEntry = {
  ts: string;
  db: string;
  status: string;
};

export async function loadDbLogs(): Promise<DbLogEntry[]> {
  const result = await fetchOrNull<DbLogEntry[]>('/db-logs');
  return result ?? [];
}

export type BreakpointUpdateRequest = {
  symbol: string;
  targetDate: string;
  mode: string;
};

export type BreakpointUpdateResponse = {
  success: boolean;
  message: string;
  forecastId: number | null;
  breakpointLabel: string | null;
  scenariosUpdated: number;
};

export async function runBreakpointUpdate(
  request: BreakpointUpdateRequest,
): Promise<BreakpointUpdateResponse> {
  // Match the Rust DTO (camelCase via serde rename) + pass an empty candles
  // array so the backend fetches fresh 1h candles server-side.
  const payload = {
    symbol: request.symbol,
    targetDate: request.targetDate,
    mode: request.mode,
    candles: [],
  };
  const local = await fetchOrNull<BreakpointUpdateResponse>('/breakpoints/update', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
  if (local) return local;

  return {
    success: false,
    message: 'Bridge offline: backend at 127.0.0.1:3017 is not running.',
    forecastId: null,
    breakpointLabel: null,
    scenariosUpdated: 0,
  };
}

export async function runShell(command: string): Promise<ShellLine[]> {
  const local = await fetchOrNull<ShellLine[]>('/shell', {
    method: 'POST',
    body: JSON.stringify({ command }),
  });
  if (local) return local;

  const remote = await invokeOrNull<ShellLine[]>('vpda_run_shell_command', { command });
  if (remote) return remote;
  return [
    { level: 'ok', text: `Executed: ${command}` },
    { level: 'info', text: 'Shell workspace is using the bridge shape that will later stream real Rust output.' },
  ];
}

export type FvgZone = {
  upper: number;
  lower: number;
  kind: string;
  status: string;
};

export type FvgScanResponse = {
  fvgs: FvgZone[];
  ifvgs: FvgZone[];
};

export async function scanFvgs(
  symbol: string,
  timeframe: string,
  currentPrice: number,
  candles?: any[],
): Promise<FvgScanResponse> {
  const payload = {
    symbol,
    timeframe,
    current_price: currentPrice,
    candles: candles || null
  };
  const result = await fetchOrNull<FvgScanResponse>('/scan-fvgs', {
    method: 'POST',
    body: JSON.stringify(payload),
  });
  return result || { fvgs: [], ifvgs: [] };
}
