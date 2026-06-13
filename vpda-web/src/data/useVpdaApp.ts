import { useEffect, useMemo, useState } from 'react';
import {
  type AccountInfo,
  type ActivateResponse,
  type BreakpointUpdateResponse,
  type DrawTool,
  type ForecastBias,
  type ForecastDrawingRequest,
  type ForecastScenario,
  type ForecastResult,
  type ForecastTarget,
  type MarketSnapshot,
  type SessionForecastResult,
  type SessionScenarioVisualization,
  type ShellLine,
  type StartupState,
  type WatchItem,
  type Workspace,
  activateLicense,
  loadAccount,
  loadMarket,
  loadStartupState,
  loadWatchlist,
  runBreakpointUpdate,
  runForecast,
  runShell,
} from '../bridge/vpdaBridge';

const FULL_WATCHLIST: WatchItem[] = [
  ['EURUSD', 'Euro / US Dollar'],
  ['USDJPY', 'US Dollar / Japanese Yen'],
  ['GBPUSD', 'British Pound / US Dollar'],
  ['USDCHF', 'US Dollar / Swiss Franc'],
  ['AUDUSD', 'Australian Dollar / US Dollar'],
  ['USDCAD', 'US Dollar / Canadian Dollar'],
  ['NZDUSD', 'New Zealand Dollar / US Dollar'],
  ['EURGBP', 'Euro / British Pound'],
  ['EURJPY', 'Euro / Japanese Yen'],
  ['EURCHF', 'Euro / Swiss Franc'],
  ['EURAUD', 'Euro / Australian Dollar'],
  ['EURCAD', 'Euro / Canadian Dollar'],
  ['GBPJPY', 'British Pound / Japanese Yen'],
  ['GBPAUD', 'British Pound / Australian Dollar'],
  ['AUDJPY', 'Australian Dollar / Japanese Yen'],
  ['AUDCAD', 'Australian Dollar / Canadian Dollar'],
  ['AUDNZD', 'Australian Dollar / New Zealand Dollar'],
  ['NZDJPY', 'New Zealand Dollar / Japanese Yen'],
  ['CADJPY', 'Canadian Dollar / Japanese Yen'],
  ['CHFJPY', 'Swiss Franc / Japanese Yen'],
  ['NQ', 'Nasdaq Futures'],
  ['ES', 'S&P Futures'],
].map(([symbol, description]) => ({
  symbol,
  description,
  price: 0,
  changePct: 0,
}));

export type ThemeSettings = {
  mode: 'dark' | 'light';
  fontSize: number;
  fontFamily: string;
  chartBackground: string;
  chartText: string;
  bullColor: string;
  bearColor: string;
  bullWickColor: string;
  bearWickColor: string;
  gridColor: string;
  crosshairColor: string;
  accentColor: string;
  heatmapColor: string;
  tzOffset: number;
};

const DARK_THEME: ThemeSettings = {
  mode: 'dark',
  fontSize: 13,
  fontFamily: 'Fira Code',
  chartBackground: '#0d0d0f',
  chartText: '#55555f',
  bullColor: '#3dcf6e',
  bearColor: '#e04545',
  bullWickColor: '#2d9f55',
  bearWickColor: '#b03030',
  gridColor: '#14141a',
  crosshairColor: '#3a3a44',
  accentColor: '#f0a030',
  heatmapColor: '#3a8fd0',
  tzOffset: 0,
};

const LIGHT_THEME: ThemeSettings = {
  mode: 'light',
  fontSize: 13,
  fontFamily: 'Fira Code',
  chartBackground: '#f5f1e8',
  chartText: '#61584c',
  bullColor: '#2b9a57',
  bearColor: '#cf4d4d',
  bullWickColor: '#1f7d45',
  bearWickColor: '#a33a3a',
  gridColor: '#d8d0c222',
  crosshairColor: '#9e927f',
  accentColor: '#b9771f',
  heatmapColor: '#5487c7',
  tzOffset: 0,
};

export const DEFAULT_THEME = DARK_THEME;

function themePreset(mode: 'dark' | 'light'): ThemeSettings {
  return mode === 'light' ? LIGHT_THEME : DARK_THEME;
}

type UiPrefs = {
  workspace: Workspace;
  symbol: string;
  timeframe: string;
  drawTool: DrawTool;
  showGrid: boolean;
  showCrosshair: boolean;
  showHeatmap: boolean;
  showMagnet: boolean;
  showWatchlist: boolean;
  showInspector: boolean;
  forecastSymbol: string;
  forecastTarget: ForecastTarget;
  forecastBias: ForecastBias;
  forecastDate: string;
  showEngineDebug?: boolean;
  drawingTemplates?: Record<string, any>;
};

const UI_PREFS_KEY = 'vpda-web-ui-prefs';

function loadTheme(): ThemeSettings {
  try {
    const raw = window.localStorage.getItem('vpda-web-theme');
    if (!raw) return DEFAULT_THEME;
    const parsed = JSON.parse(raw) as Partial<ThemeSettings>;
    const base = themePreset(parsed.mode ?? 'dark');
    return { ...base, ...parsed };
  } catch {
    return DEFAULT_THEME;
  }
}

function loadUiPrefs(): Partial<UiPrefs> {
  try {
    const raw = window.localStorage.getItem(UI_PREFS_KEY);
    return raw ? (JSON.parse(raw) as Partial<UiPrefs>) : {};
  } catch {
    return {};
  }
}

export function useVpdaApp() {
  const uiPrefs = loadUiPrefs();

  const [workspace, setWorkspace] = useState<Workspace>(
    uiPrefs.workspace === 'shell' ? 'market' : (uiPrefs.workspace ?? 'market'),
  );
  const [watchlist, setWatchlist] = useState<WatchItem[]>(FULL_WATCHLIST);
  const [symbol, setSymbol] = useState(uiPrefs.symbol ?? 'EURUSD');
  const [timeframe, setTimeframe] = useState(uiPrefs.timeframe ?? '1D');
  const [drawTool, setDrawTool] = useState<DrawTool>(uiPrefs.drawTool ?? 'cursor');
  const [showGrid, setShowGrid] = useState(uiPrefs.showGrid ?? true);
  const [showCrosshair, setShowCrosshair] = useState(uiPrefs.showCrosshair ?? true);
  const [showHeatmap, setShowHeatmap] = useState(uiPrefs.showHeatmap ?? true);
  const [showMagnet, setShowMagnet] = useState(uiPrefs.showMagnet ?? false);
  const [showWatchlist, setShowWatchlist] = useState(uiPrefs.showWatchlist ?? true);
  const [showInspector, setShowInspector] = useState(uiPrefs.showInspector ?? true);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [theme, setTheme] = useState<ThemeSettings>(loadTheme);
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);

  const [hoveredCandle, setHoveredCandle] = useState<any>(null);
  const [managerOpen, setManagerOpen] = useState(false);
  const [adminMode, setAdminMode] = useState(false);
  const [replayEnabled, setReplayEnabled] = useState(false);

  const [market, setMarket] = useState<MarketSnapshot | null>(null);
  const [loadingMarket, setLoadingMarket] = useState(true);

  const [drawingTemplates, setDrawingTemplates] = useState<Record<string, any>>(uiPrefs.drawingTemplates ?? {});

  const [forecastSymbol, setForecastSymbol] = useState(uiPrefs.forecastSymbol ?? 'EURUSD');
  const [forecastTarget, setForecastTarget] = useState<ForecastTarget>(uiPrefs.forecastTarget ?? 'TODAY');
  const [forecastBias, setForecastBias] = useState<ForecastBias>(uiPrefs.forecastBias ?? 'AUTO');
  const [forecastDate, setForecastDate] = useState(
    uiPrefs.forecastDate ?? new Date().toISOString().slice(0, 10),
  );
  const [forecastLoading, setForecastLoading] = useState(false);
  const [forecastResult, setForecastResult] = useState<ForecastResult | null>(null);
  const [forecastError, setForecastError] = useState<string | null>(null);
  const [forecastMarket, setForecastMarket] = useState<MarketSnapshot | null>(null);
  const [showEngineDebug, setShowEngineDebug] = useState(uiPrefs.showEngineDebug ?? true);
  const [forecastDrawingRequest, setForecastDrawingRequest] = useState<ForecastDrawingRequest | null>(null);
  const [forecastDrawingDebug, setForecastDrawingDebug] = useState<{
    requestId: number | null;
    status: 'idle' | 'queued' | 'waiting' | 'applied' | 'empty' | 'cleared';
    message: string;
  }>({
    requestId: null,
    status: 'idle',
    message: 'No forecast drawing request yet',
  });

  const [breakpointBusy, setBreakpointBusy] = useState(false);
  const [breakpointResult, setBreakpointResult] = useState<BreakpointUpdateResponse | null>(null);
  const [breakpointError, setBreakpointError] = useState<string | null>(null);

  const [shellInput, setShellInput] = useState('FORECAST DAILY EURUSD');
  const [shellBusy, setShellBusy] = useState(false);
  const [shellLines, setShellLines] = useState<ShellLine[]>([
    { level: 'ok', text: 'VPDA shell initialized.' },
    { level: 'info', text: 'Bridge layer is ready for Rust/Tauri commands and streamed output.' },
  ]);

  const [startup, setStartup] = useState<StartupState | null>(null);
  const [account, setAccount] = useState<AccountInfo | null>(null);
  const [accountLoading, setAccountLoading] = useState(false);
  const [accountOpen, setAccountOpen] = useState(false);
  const [dbLogsOpen, setDbLogsOpen] = useState(false);
  
  const [toastMessage, setToastMessage] = useState<{ id: number, text: string } | null>(null);
  const showToast = (text: string) => {
    const id = Date.now();
    setToastMessage({ id, text });
    setTimeout(() => {
      setToastMessage(prev => prev?.id === id ? null : prev);
    }, 2500);
  };

  useEffect(() => {
    loadStartupState().then(setStartup);
    loadAccount().then(setAccount);
    loadWatchlist().then((items) => {
      setWatchlist((prev) =>
        prev.map((existing) => items.find((item) => item.symbol === existing.symbol) ?? existing),
      );
    });
  }, []);

  useEffect(() => {
    window.localStorage.setItem('vpda-web-theme', JSON.stringify(theme));
    document.documentElement.style.setProperty('--ui', `"${theme.fontFamily}", system-ui, sans-serif`);
    document.documentElement.style.setProperty('--vpda-font-size', `${theme.fontSize}px`);
    document.documentElement.style.setProperty('--chart-bg', theme.chartBackground);
    document.documentElement.style.setProperty('--chart-text', theme.chartText);
    document.documentElement.style.setProperty('--chart-bull', theme.bullColor);
    document.documentElement.style.setProperty('--chart-bear', theme.bearColor);
    document.documentElement.style.setProperty('--chart-bull-wick', theme.bullWickColor);
    document.documentElement.style.setProperty('--chart-bear-wick', theme.bearWickColor);
    document.documentElement.style.setProperty('--chart-grid', theme.gridColor);
    document.documentElement.style.setProperty('--chart-crosshair', theme.crosshairColor);
    document.documentElement.style.setProperty('--accent', theme.accentColor);
    document.documentElement.style.setProperty('--heatmap-color', theme.heatmapColor);
    if (theme.mode === 'light') {
      document.documentElement.style.setProperty('--bg0', '#f1ede4');
      document.documentElement.style.setProperty('--bg1', '#f7f3eb');
      document.documentElement.style.setProperty('--bg2', '#ece6db');
      document.documentElement.style.setProperty('--bg3', '#dfd6c8');
      document.documentElement.style.setProperty('--bg4', '#d5cab9');
      document.documentElement.style.setProperty('--bg5', '#cfc2ae');
      document.documentElement.style.setProperty('--border0', '#d6cbbb');
      document.documentElement.style.setProperty('--border1', '#cbbda9');
      document.documentElement.style.setProperty('--border2', '#baa98f');
      document.documentElement.style.setProperty('--border3', '#a48f72');
      document.documentElement.style.setProperty('--text', '#2e2820');
      document.documentElement.style.setProperty('--text2', '#6f6558');
      document.documentElement.style.setProperty('--text3', '#9a8f82');
      document.documentElement.style.setProperty('--blue', '#3c6fa8');
      document.documentElement.style.setProperty('color-scheme', 'light');
      // reactive component vars
      document.documentElement.style.setProperty('--active-bg',     '#ede4ce');
      document.documentElement.style.setProperty('--active-border', '#c4954a');
      document.documentElement.style.setProperty('--watch-active1', '#e8dfce');
      document.documentElement.style.setProperty('--watch-active2', '#ddd5bc');
      document.documentElement.style.setProperty('--card-bg2',      '#ece8df');
      document.documentElement.style.setProperty('--page-bg2',      '#e8e3d8');
      document.documentElement.style.setProperty('--console-bg',    '#f8f4ec');
      document.documentElement.style.setProperty('--track-bg',      '#d8d0bc');
      document.documentElement.style.setProperty('--float-bg',      'rgba(247,243,235,0.97)');
      document.documentElement.style.setProperty('--input-bg',      '#ede8de');
      document.documentElement.style.setProperty('--hover-bg',      '#e8e3d8');
    } else {
      document.documentElement.style.setProperty('--bg0', '#0d0d0f');
      document.documentElement.style.setProperty('--bg1', '#111114');
      document.documentElement.style.setProperty('--bg2', '#161619');
      document.documentElement.style.setProperty('--bg3', '#1c1c21');
      document.documentElement.style.setProperty('--bg4', '#242429');
      document.documentElement.style.setProperty('--bg5', '#2c2c32');
      document.documentElement.style.setProperty('--border0', '#1e1e24');
      document.documentElement.style.setProperty('--border1', '#28282f');
      document.documentElement.style.setProperty('--border2', '#363640');
      document.documentElement.style.setProperty('--border3', '#454550');
      document.documentElement.style.setProperty('--text', '#c8c8d4');
      document.documentElement.style.setProperty('--text2', '#8f8f9d');
      document.documentElement.style.setProperty('--text3', '#5c5c69');
      document.documentElement.style.setProperty('--blue', '#3a8fd0');
      document.documentElement.style.setProperty('color-scheme', 'dark');
      // reactive component vars
      document.documentElement.style.setProperty('--active-bg',     '#1e1508');
      document.documentElement.style.setProperty('--active-border', '#5c3d0e');
      document.documentElement.style.setProperty('--watch-active1', '#24180b');
      document.documentElement.style.setProperty('--watch-active2', '#17120d');
      document.documentElement.style.setProperty('--card-bg2',      '#111114');
      document.documentElement.style.setProperty('--page-bg2',      '#101013');
      document.documentElement.style.setProperty('--console-bg',    '#080809');
      document.documentElement.style.setProperty('--track-bg',      '#141417');
      document.documentElement.style.setProperty('--float-bg',      'rgba(13,13,15,0.97)');
      document.documentElement.style.setProperty('--input-bg',      '#0a0a0c');
      document.documentElement.style.setProperty('--hover-bg',      '#1c1c21');
    }
  }, [theme]);

  useEffect(() => {
    const prefs: UiPrefs = {
      workspace,
      symbol,
      timeframe,
      drawTool,
      showGrid,
      showCrosshair,
      showHeatmap,
      showMagnet,
      showWatchlist,
      showInspector,
      forecastSymbol,
      forecastTarget,
      forecastBias,
      forecastDate,
      showEngineDebug,
      drawingTemplates,
    };
    window.localStorage.setItem(UI_PREFS_KEY, JSON.stringify(prefs));
  }, [
    workspace,
    symbol,
    timeframe,
    drawTool,
    showGrid,
    showCrosshair,
    showHeatmap,
    showMagnet,
    showWatchlist,
    showInspector,
    forecastSymbol,
    forecastTarget,
    forecastBias,
    forecastDate,
    showEngineDebug,
    drawingTemplates,
  ]);

  useEffect(() => {
    setLoadingMarket(true);
    loadMarket(symbol, timeframe)
      .then((snapshot) => setMarket(snapshot))
      .finally(() => setLoadingMarket(false));
  }, [symbol, timeframe]);

  useEffect(() => {
    loadMarket(forecastSymbol, '1D').then((snapshot) => setForecastMarket(snapshot));
  }, [forecastSymbol]);

  const changePct = useMemo(() => {
    if (!market || !watchlist.length) return 0;
    return watchlist.find((item) => item.symbol === symbol)?.changePct ?? 0;
  }, [market, symbol, watchlist]);

  async function executeForecast() {
    setForecastLoading(true);
    setForecastError(null);
    try {
      const result = await runForecast({
        symbol: forecastSymbol,
        target: forecastTarget,
        bias: forecastBias,
        customDate: forecastTarget === 'CUSTOM' ? forecastDate : null,
      });
      setForecastResult(result);
    } catch (error) {
      setForecastError(error instanceof Error ? error.message : String(error));
    } finally {
      setForecastLoading(false);
    }
  }

  async function executeBreakpointUpdate() {
    setBreakpointBusy(true);
    setBreakpointError(null);
    try {
      // Resolve the target date: CUSTOM → user-selected, NEXT → tomorrow, else today.
      let targetDate: string;
      if (forecastTarget === 'CUSTOM' && forecastDate) {
        targetDate = forecastDate;
      } else {
        const d = new Date();
        if (forecastTarget === 'NEXT') d.setDate(d.getDate() + 1);
        targetDate = d.toISOString().slice(0, 10);
      }
      const mode = forecastTarget === 'NEXT' ? 'next' : 'today';
      const response = await runBreakpointUpdate({
        symbol: forecastSymbol,
        targetDate,
        mode,
      });
      setBreakpointResult(response);
    } catch (error) {
      setBreakpointError(error instanceof Error ? error.message : String(error));
    } finally {
      setBreakpointBusy(false);
    }
  }

  async function executeShell() {
    if (!shellInput.trim()) return;
    setShellBusy(true);
    const command = shellInput.trim();
    setShellLines((prev) => [...prev, { level: 'info', text: `> ${command}` }]);
    try {
      const result = await runShell(command);
      setShellLines((prev) => [...prev, ...result]);
    } catch (error) {
      setShellLines((prev) => [...prev, { level: 'err', text: error instanceof Error ? error.message : String(error) }]);
    } finally {
      setShellBusy(false);
    }
  }

  function dismissStartupWarning() {
    setStartup((prev) => (prev ? { ...prev, warning: undefined } : prev));
  }

  function dismissActivationGate() {
    setStartup((prev) =>
      prev ? { ...prev, activationRequired: false, activationMessage: undefined } : prev,
    );
  }

  async function submitActivation(licenseKey: string): Promise<ActivateResponse> {
    const result = await activateLicense(licenseKey);
    if (result.success) {
      const fresh = await loadStartupState();
      setStartup(fresh);
      const acc = await loadAccount();
      setAccount(acc);
    }
    return result;
  }

  function refreshAccount() {
    setAccountLoading(true);
    loadAccount()
      .then(setAccount)
      .finally(() => setAccountLoading(false));
  }

  function requestForecastScenarioDraw(scenario: ForecastScenario) {
    const requestId = Date.now();
    setWorkspace('market');
    setSymbol(forecastSymbol);
    setTimeframe('1D');
    setManagerOpen(true);
    setForecastDrawingRequest({
      id: requestId,
      kind: 'scenario',
      symbol: forecastSymbol,
      scenario,
    });
    setForecastDrawingDebug({
      requestId,
      status: 'queued',
      message: `Queued scenario draw for ${scenario.name}`,
    });
    showToast(`Drawing ${scenario.name} on chart`);
  }

  function requestForecastLevelsDraw() {
    if (!forecastResult) return;
    const requestId = Date.now();
    setWorkspace('market');
    setSymbol(forecastSymbol);
    setTimeframe('1D');
    setManagerOpen(true);
    setForecastDrawingRequest({
      id: requestId,
      kind: 'levels',
      symbol: forecastSymbol,
      levels: forecastResult.levels,
    });
    setForecastDrawingDebug({
      requestId,
      status: 'queued',
      message: `Queued ${forecastResult.levels.length} key level(s) for drawing`,
    });
    showToast('Drawing forecast key levels on chart');
  }

  function clearForecastDrawings() {
    const requestId = Date.now();
    setWorkspace('market');
    setSymbol(forecastSymbol);
    setTimeframe('1D');
    setForecastDrawingRequest({
      id: requestId,
      kind: 'clear',
      symbol: forecastSymbol,
    });
    setForecastDrawingDebug({
      requestId,
      status: 'queued',
      message: 'Queued forecast drawing clear request',
    });
    showToast('Cleared forecast drawings');
  }


  // Session scenario visualization state
  const [sessionScenarioVisualization, setSessionScenarioVisualization] =
    useState<SessionScenarioVisualization | null>(null);

  const [sessionForecastResult, setSessionForecastResult] =
    useState<SessionForecastResult | null>(null);

  const clearSessionScenarioVisualization = () => {
    setSessionScenarioVisualization(null);
  };

  const toggleAdminMode = () => {
    setAdminMode(!adminMode);
  };

  return {
    workspace,
    setWorkspace,
    watchlist,
    symbol,
    setSymbol,
    timeframe,
    setTimeframe,
    drawTool,
    setDrawTool,
    showGrid,
    setShowGrid,
    showCrosshair,
    setShowCrosshair,
    showHeatmap,
    setShowHeatmap,
    showMagnet,
    setShowMagnet,
    showWatchlist,
    setShowWatchlist,
    showInspector,
    setShowInspector,
    settingsOpen,
    setSettingsOpen,
    theme,
    setTheme,
    canUndo,
    setCanUndo,
    canRedo,
    setCanRedo,
    hoveredCandle,
    setHoveredCandle,
    managerOpen,
    setManagerOpen,
    adminMode,
    toggleAdminMode,
    replayEnabled,
    setReplayEnabled,
    drawingTemplates,
    setDrawingTemplates,
    market,
    loadingMarket,
    changePct,
    forecastSymbol,
    setForecastSymbol,
    forecastTarget,
    setForecastTarget,
    forecastBias,
    setForecastBias,
    forecastDate,
    setForecastDate,
    forecastLoading,
    forecastResult,
    forecastError,
    forecastMarket,
    showEngineDebug,
    setShowEngineDebug,
    forecastDrawingRequest,
    forecastDrawingDebug,
    setForecastDrawingDebug,
    executeForecast,
    requestForecastScenarioDraw,
    requestForecastLevelsDraw,
    clearForecastDrawings,
    breakpointBusy,
    breakpointResult,
    breakpointError,
    executeBreakpointUpdate,
    shellInput,
    setShellInput,
    shellBusy,
    shellLines,
    executeShell,
    startup,
    account,
    accountLoading,
    accountOpen,
    setAccountOpen,
    dbLogsOpen,
    setDbLogsOpen,
    refreshAccount,
    submitActivation,
    dismissStartupWarning,
    dismissActivationGate,
    toastMessage,
    showToast,
    sessionForecastResult,
    setSessionForecastResult,
    sessionScenarioVisualization,
    setSessionScenarioVisualization,
    clearSessionScenarioVisualization,
  };
}

export type VpdaAppState = ReturnType<typeof useVpdaApp>;
