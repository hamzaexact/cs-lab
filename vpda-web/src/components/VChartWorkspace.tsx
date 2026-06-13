import { useEffect, useMemo, useRef, useState } from 'react';
import {
  CandlestickSeries,
  ColorType,
  CrosshairMode,
  createChart,
} from 'lightweight-charts';
import type { VpdaAppState } from '../data/useVpdaApp';
import type { Candle } from '../bridge/vpdaBridge';
import { scanFvgs } from '../bridge/vpdaBridge';
import { formatChartAnchorTime, formatChartTickMark, formatPrice, precisionForSymbol } from '../utils/format';

type Props = {
  app: VpdaAppState;
};

type VirtualCandle = Candle & {
  isVirtual: true;
};

// Utility: Adjust color brightness/opacity for virtual candles
function adjustColorForVirtual(color: string, opacity: number = 0.5): string {
  // If color is already rgba, adjust its opacity
  if (color.startsWith('rgba')) {
    return color.replace(/[\d.]+\)$/g, `${opacity})`);
  }

  // If color is rgb, convert to rgba
  if (color.startsWith('rgb(')) {
    return color.replace('rgb(', 'rgba(').replace(')', `, ${opacity})`);
  }

  // If hex color, convert to rgba
  if (color.startsWith('#')) {
    const hex = color.replace('#', '');
    const r = parseInt(hex.substring(0, 2), 16);
    const g = parseInt(hex.substring(2, 4), 16);
    const b = parseInt(hex.substring(4, 6), 16);
    return `rgba(${r}, ${g}, ${b}, ${opacity})`;
  }

  // Fallback: wrap in rgba with opacity
  return color;
}

export default function VChartWorkspace({ app }: Props) {
  const hostRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<ReturnType<typeof createChart> | null>(null);
  const seriesRef = useRef<any>(null);

  const [virtualCandles, setVirtualCandles] = useState<VirtualCandle[]>([]);
  const [hoveredCandle, setHoveredCandle] = useState<any>(null);
  const [fvgZones, setFvgZones] = useState<any>(null);
  const [isFullscreen, setIsFullscreen] = useState(false);
  const fvgOverlaysRef = useRef<HTMLDivElement[]>([]);

  const precision = useMemo(() => precisionForSymbol(app.symbol), [app.symbol]);

  // Combine real + virtual candles
  const allCandles = useMemo(() => {
    const real = app.market?.candles ?? [];
    return [...real, ...virtualCandles];
  }, [app.market?.candles, virtualCandles]);

  // Generate a realistic bull candle
  const generateBullCandle = () => {
    if (!app.market?.candles.length && virtualCandles.length === 0) {
      alert('No candles to base virtual candle on. Load a symbol first.');
      return;
    }

    const lastCandle = virtualCandles.length > 0
      ? virtualCandles[virtualCandles.length - 1]
      : app.market!.candles[app.market!.candles.length - 1];

    const open = lastCandle.close;

    // Bull candle: close > open
    // Body size: 0.3% to 1.2% move
    const bodyPercent = 0.003 + Math.random() * 0.009; // 0.3% to 1.2%
    const close = open * (1 + bodyPercent);

    // Add wicks (10% to 40% of body size)
    const bodySize = close - open;
    const upperWickSize = bodySize * (0.1 + Math.random() * 0.3);
    const lowerWickSize = bodySize * (0.1 + Math.random() * 0.3);

    const high = close + upperWickSize;
    const low = open - lowerWickSize;

    // Next timestamp (increment by timeframe)
    const timeIncrement = app.timeframe === '1H' ? 3600 :
                          app.timeframe === '4H' ? 14400 :
                          86400; // 1D default
    const time = lastCandle.time + timeIncrement;

    const newCandle: VirtualCandle = {
      time,
      open: Number(open.toFixed(precision)),
      high: Number(high.toFixed(precision)),
      low: Number(low.toFixed(precision)),
      close: Number(close.toFixed(precision)),
      volume: Math.round(lastCandle.volume * (0.8 + Math.random() * 0.4)),
      isVirtual: true,
    };

    setVirtualCandles(prev => [...prev, newCandle]);
  };

  // Generate a realistic bear candle
  const generateBearCandle = () => {
    if (!app.market?.candles.length && virtualCandles.length === 0) {
      alert('No candles to base virtual candle on. Load a symbol first.');
      return;
    }

    const lastCandle = virtualCandles.length > 0
      ? virtualCandles[virtualCandles.length - 1]
      : app.market!.candles[app.market!.candles.length - 1];

    const open = lastCandle.close;

    // Bear candle: close < open
    // Body size: 0.3% to 1.2% move
    const bodyPercent = 0.003 + Math.random() * 0.009; // 0.3% to 1.2%
    const close = open * (1 - bodyPercent);

    // Add wicks (10% to 40% of body size)
    const bodySize = open - close;
    const upperWickSize = bodySize * (0.1 + Math.random() * 0.3);
    const lowerWickSize = bodySize * (0.1 + Math.random() * 0.3);

    const high = open + upperWickSize;
    const low = close - lowerWickSize;

    // Next timestamp (increment by timeframe)
    const timeIncrement = app.timeframe === '1H' ? 3600 :
                          app.timeframe === '4H' ? 14400 :
                          86400; // 1D default
    const time = lastCandle.time + timeIncrement;

    const newCandle: VirtualCandle = {
      time,
      open: Number(open.toFixed(precision)),
      high: Number(high.toFixed(precision)),
      low: Number(low.toFixed(precision)),
      close: Number(close.toFixed(precision)),
      volume: Math.round(lastCandle.volume * (0.8 + Math.random() * 0.4)),
      isVirtual: true,
    };

    setVirtualCandles(prev => [...prev, newCandle]);
  };

  const clearVirtualCandles = () => {
    setVirtualCandles([]);
  };

  const removeLastVirtual = () => {
    setVirtualCandles(prev => prev.slice(0, -1));
  };

  // Fullscreen toggle
  const toggleFullscreen = () => {
    if (!hostRef.current) return;

    if (!document.fullscreenElement) {
      const element = hostRef.current;
      if (element.requestFullscreen) {
        element.requestFullscreen()
          .then(() => setIsFullscreen(true))
          .catch(err => console.error('Fullscreen request failed:', err));
      } else {
        console.warn('Fullscreen API not supported');
      }
    } else {
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
        const timeframeMap: { [key: string]: '15M' | '1H' | '4H' | '1D' | '1W' } = {
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

  // Initialize chart
  useEffect(() => {
    if (!hostRef.current || !app.market) return;

    const chart = createChart(hostRef.current, {
      autoSize: true,
      layout: {
        background: { type: ColorType.Solid, color: app.theme.chartBackground },
        textColor: app.theme.chartText,
        fontFamily: "'Share Tech Mono', 'Consolas', monospace",
        fontSize: Math.max(10, app.theme.fontSize),
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
        borderColor: app.theme.gridColor,
        timeVisible: true,
        secondsVisible: false,
        barSpacing: 12,
        minBarSpacing: 2,
        rightOffset: 150,
        fixRightEdge: false,
        tickMarkFormatter: (time: number, type: number) =>
          formatChartTickMark(time, type, app.timeframe),
      },
      localization: {
        priceFormatter: (price: number) => formatPrice(price, app.symbol),
        timeFormatter: (time: number) => formatChartAnchorTime(time),
      },
      handleScroll: { mouseWheel: true, pressedMouseMove: true },
      handleScale: { mouseWheel: true, pinch: true, axisPressedMouseMove: true },
    });

    const series = chart.addSeries(CandlestickSeries, {
      upColor: app.theme.bullColor,
      downColor: app.theme.bearColor,
      borderUpColor: app.theme.bullWickColor,
      borderDownColor: app.theme.bearWickColor,
      wickUpColor: app.theme.bullWickColor,
      wickDownColor: app.theme.bearWickColor,
      borderVisible: true,
      wickVisible: true,
      priceLineVisible: false,
      lastValueVisible: true,
      priceFormat: {
        type: 'price',
        precision,
        minMove: precision === 5 ? 0.00001 : precision === 3 ? 0.001 : 0.25,
      },
    });

    chartRef.current = chart;
    seriesRef.current = series as any;

    chart.subscribeCrosshairMove((param) => {
      if (!param.time || !param.point) {
        setHoveredCandle(null);
        return;
      }
      const data = param.seriesData.get(series as any);
      if (data) setHoveredCandle(data);
    });

    return () => {
      chart.remove();
      chartRef.current = null;
      seriesRef.current = null;
    };
  }, [app.market, app.theme, app.showGrid, app.showCrosshair, app.symbol, precision]);

  // Update chart data when candles change
  useEffect(() => {
    if (!seriesRef.current || !allCandles.length) return;

    // Check if we need to truncate candles for admin custom date/time
    let displayCandles = allCandles;
    const viz = app.sessionScenarioVisualization;
    if (viz && ((viz as any).customDate || (viz as any).customTime)) {
      const customDate = (viz as any).customDate as string;
      const customTime = (viz as any).customTime as string;

      if (customDate && customTime) {
        // Parse custom date+time to timestamp
        const customDateTime = new Date(`${customDate}T${customTime}`).getTime() / 1000;
        // Truncate candles to only show up to custom date/time
        displayCandles = allCandles.filter(c => c.time <= customDateTime);
      }
    }

    seriesRef.current.setData(
      displayCandles.map((c) => {
        const isBull = c.close >= c.open;
        const isVirtual = 'isVirtual' in c && c.isVirtual;

        // Base candle data
        const candleData: any = {
          time: c.time as never,
          open: c.open,
          high: c.high,
          low: c.low,
          close: c.close,
        };

        // Apply custom colors for virtual candles (50% opacity)
        if (isVirtual) {
          if (isBull) {
            candleData.color = adjustColorForVirtual(app.theme.bullColor, 0.5);
            candleData.borderColor = adjustColorForVirtual(app.theme.bullWickColor, 0.5);
            candleData.wickColor = adjustColorForVirtual(app.theme.bullWickColor, 0.5);
          } else {
            candleData.color = adjustColorForVirtual(app.theme.bearColor, 0.5);
            candleData.borderColor = adjustColorForVirtual(app.theme.bearWickColor, 0.5);
            candleData.wickColor = adjustColorForVirtual(app.theme.bearWickColor, 0.5);
          }
        }

        return candleData;
      })
    );

    // Auto-scroll to show latest candles
    if (chartRef.current && displayCandles.length > 0) {
      const total = displayCandles.length;
      chartRef.current.timeScale().setVisibleLogicalRange({
        from: Math.max(0, total - 60),
        to: total + 10,
      });
    }
  }, [allCandles, app.theme, app.sessionScenarioVisualization]);

  // FVG zone rendering effect with dynamic position updates
  useEffect(() => {
    // Clear existing FVG overlays
    fvgOverlaysRef.current.forEach((overlay) => {
      if (overlay && overlay.parentNode) {
        overlay.parentNode.removeChild(overlay);
      }
    });
    fvgOverlaysRef.current = [];

    // If no FVG zones or no chart/series, nothing to render
    if (!fvgZones || !chartRef.current || !seriesRef.current || !hostRef.current) return;

    const chart = chartRef.current;
    const series = seriesRef.current;

    // Store FVG data with overlay elements for updates
    const fvgData: Array<{ fvg: any; overlay: HTMLDivElement; isInverted: boolean }> = [];

    const createFvgRectangle = (fvg: any, isInverted: boolean) => {
      const color = isInverted
        ? 'rgba(158, 158, 158, 0.3)' // Gray for inverted
        : fvg.kind === 'Bullish'
          ? 'rgba(0, 200, 83, 0.3)'  // Green for bullish
          : 'rgba(239, 83, 80, 0.3)'; // Red for bearish

      // Create overlay div
      const overlay = document.createElement('div');
      overlay.style.position = 'absolute';
      overlay.style.left = '0';
      overlay.style.right = '0';
      overlay.style.backgroundColor = color;
      overlay.style.pointerEvents = 'none';
      overlay.style.zIndex = '1';

      // Append to chart container
      hostRef.current!.appendChild(overlay);
      fvgOverlaysRef.current.push(overlay);

      // Store data for position updates
      fvgData.push({ fvg, overlay, isInverted });

      return overlay;
    };

    // Function to update all FVG rectangle positions
    const updateFvgPositions = () => {
      fvgData.forEach(({ fvg, overlay }) => {
        // Convert prices to current pixel coordinates
        const upperY = series.priceToCoordinate(fvg.upper);
        const lowerY = series.priceToCoordinate(fvg.lower);

        if (upperY === null || lowerY === null) {
          overlay.style.display = 'none';
          return;
        }

        // Update position
        overlay.style.display = 'block';
        overlay.style.top = `${upperY}px`;
        overlay.style.height = `${lowerY - upperY}px`;
      });
    };

    // Create initial rectangles
    fvgZones.fvgs.forEach((fvg: any) => createFvgRectangle(fvg, false));
    fvgZones.ifvgs.forEach((ifvg: any) => createFvgRectangle(ifvg, true));

    // Initial position update
    updateFvgPositions();

    // Subscribe to chart events for position updates
    chart.timeScale().subscribeVisibleTimeRangeChange(updateFvgPositions);
    chart.timeScale().subscribeVisibleLogicalRangeChange(updateFvgPositions);

    // Cleanup function
    return () => {
      chart.timeScale().unsubscribeVisibleTimeRangeChange(updateFvgPositions);
      chart.timeScale().unsubscribeVisibleLogicalRangeChange(updateFvgPositions);
    };
  }, [fvgZones, chartRef.current, seriesRef.current]);

  // Scenario visualization effect - generates virtual candles from scenario path
  useEffect(() => {
    if (!app.sessionScenarioVisualization) return;

    const viz = app.sessionScenarioVisualization;

    // Handle admin custom date/time truncation
    let displayCandles = allCandles;
    if ((viz as any).customDate || (viz as any).customTime) {
      const customDate = (viz as any).customDate as string;
      const customTime = (viz as any).customTime as string;

      if (customDate && customTime) {
        // Parse custom date+time to timestamp
        const customDateTime = new Date(`${customDate}T${customTime}`).getTime() / 1000;
        // Truncate candles to only show up to custom date/time
        displayCandles = allCandles.filter(c => c.time <= customDateTime);
      }
    }

    // Generate virtual candles showing the predicted price path
    const scenarioCandles: VirtualCandle[] = [];
    const lastCandle = displayCandles[displayCandles.length - 1];
    if (!lastCandle) return;

    // Start AFTER last real candle to avoid overlap
    let currentPrice = viz.currentPrice;
    const timeIncrement = viz.timeframe === '1H' || viz.timeframe === '1h' ? 3600 :
                          viz.timeframe === '4H' || viz.timeframe === '4h' ? 14400 :
                          86400; // 1D default
    let currentTime = lastCandle.time + timeIncrement;

    // Generate candles for each path segment
    for (const segment of viz.pathSegments) {
      const candleCount = Math.min(segment.candleCount, 8); // Cap at 8 candles for better visibility
      const priceChange = segment.endPrice - segment.startPrice;
      const isBull = segment.direction === 'bullish';

      for (let i = 0; i < candleCount; i++) {
        const progress = (i + 1) / candleCount;

        // Non-linear progression - ease-in-out for realistic movement
        const easeProgress = progress < 0.5
          ? 2 * progress * progress
          : 1 - Math.pow(-2 * progress + 2, 2) / 2;

        const targetPrice = viz.currentPrice + (priceChange * easeProgress);
        const prevPrice = i === 0 ? currentPrice : (viz.currentPrice + (priceChange * ((i / candleCount) < 0.5 ? 2 * Math.pow(i / candleCount, 2) : 1 - Math.pow(-2 * (i / candleCount) + 2, 2) / 2)));

        // Create realistic OHLC
        const bodySize = Math.abs(targetPrice - prevPrice);
        const volatility = segment.volatility || 0.3;

        let open, close, high, low;

        if (isBull) {
          open = prevPrice;
          close = targetPrice;
          // Wicks: upper wick smaller for bull candles - INCREASED for visibility
          const upperWickSize = bodySize * volatility * (1.5 + Math.random() * 1.5);
          const lowerWickSize = bodySize * volatility * (2.0 + Math.random() * 2.0);
          high = close + upperWickSize;
          low = open - lowerWickSize;
        } else {
          open = prevPrice;
          close = targetPrice;
          // Wicks: lower wick smaller for bear candles - INCREASED for visibility
          const upperWickSize = bodySize * volatility * (2.0 + Math.random() * 2.0);
          const lowerWickSize = bodySize * volatility * (1.5 + Math.random() * 1.5);
          high = open + upperWickSize;
          low = close - lowerWickSize;
        }

        scenarioCandles.push({
          time: currentTime,
          open: Number(open.toFixed(precision)),
          high: Number(high.toFixed(precision)),
          low: Number(low.toFixed(precision)),
          close: Number(close.toFixed(precision)),
          volume: Math.round(lastCandle.volume * (0.7 + Math.random() * 0.6)),
          isVirtual: true,
        });

        currentTime += timeIncrement;
        currentPrice = targetPrice;
      }
    }

    // Set generated candles
    setVirtualCandles(scenarioCandles);

    // Add level highlighting
    if (seriesRef.current && chartRef.current) {
      // Remove old price lines if any
      const series = seriesRef.current;

      // Add horizontal lines for each level in visualization
      viz.levels.forEach(level => {
        const lineColor = level.color || (level.type === 'target' ? '#10b981' : '#ef4444');
        const lineWidth = level.type === 'target' ? 2 : 1;
        const lineStyle = level.type === 'target' ? 0 : 2; // 0=solid, 2=dashed

        // Create price line without title to avoid label clutter
        series.createPriceLine({
          price: level.price,
          color: lineColor,
          lineWidth: lineWidth,
          lineStyle: lineStyle,
          axisLabelVisible: true,
          title: '', // Remove title to eliminate repeated labels
        });
      });
    }

    // Clear visualization state after processing
    setTimeout(() => {
      app.clearSessionScenarioVisualization();
    }, 100);
  }, [app.sessionScenarioVisualization, allCandles, precision, app.timeframe]);

  return (
    <div className="workspace-page">
      <div className="vchart-layout" style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
        {/* Control Bar */}
        <div className="vchart-controls" style={{
          display: 'flex',
          gap: '8px',
          padding: '12px',
          background: 'var(--bg2)',
          borderBottom: '1px solid var(--border1)',
          alignItems: 'center'
        }}>
          <span style={{ fontWeight: 600, marginRight: '8px', color: 'var(--text)' }}>
            V-CHART: {app.symbol} · {app.timeframe}
          </span>
          <button
            className="ue-btn small"
            onClick={generateBullCandle}
          >
            + BULL Candle
          </button>
          <button
            className="ue-btn small"
            onClick={generateBearCandle}
          >
            + BEAR Candle
          </button>
          <button
            className="ue-btn small"
            onClick={removeLastVirtual}
            disabled={virtualCandles.length === 0}
          >
            Remove Last
          </button>
          <button
            className="ue-btn small"
            onClick={clearVirtualCandles}
            disabled={virtualCandles.length === 0}
          >
            Clear All
          </button>
            <button
              className="ue-btn small"
              onClick={async () => {
                try {
                  const currentPrice = app.market?.candles[app.market.candles.length - 1]?.close || 0;
                  const allCandles = app.market?.candles || [];
                  const recentCandles = allCandles.slice(-100); // Last 100 candles only for FVG scan
                  const response = await scanFvgs(app.symbol, app.timeframe, currentPrice, recentCandles);
                  console.log("FVG scan response:", response);
                  setFvgZones(response);
                  app.showToast(`Found ${response.fvgs.length} FVGs, ${response.ifvgs.length} iFVGs`);
                } catch (err) {
                  app.showToast(`FVG scan failed: ${err}`);
                }
              }}
              style={{ marginLeft: "8px" }}
            >
              F
            </button>
          <div style={{ flex: 1 }} />
          <span style={{ color: 'var(--text2)', fontSize: 'var(--fs-13)' }}>
            Real: {app.market?.candles.length ?? 0} | Virtual: {virtualCandles.length} | Total: {allCandles.length}
          </span>
        </div>

        {/* Chart Container */}
        <div style={{ flex: 1, display: 'flex', flexDirection: 'row', minHeight: 0 }}>
          <div style={{ flex: 1, position: 'relative', minWidth: 0 }}>
            <div
              ref={hostRef}
              style={{
                position: 'absolute',
                inset: 0,
                width: '100%',
                height: '100%'
              }}
            />
          </div>

          {/* Inspector Panel */}
          {app.showInspector && (
            <aside className="inspector-panel" style={{
              width: '176px',
              background: 'var(--bg2)',
              borderLeft: '1px solid var(--border1)',
              padding: '12px',
              display: 'flex',
              flexDirection: 'column',
              gap: '8px',
              fontSize: 'var(--fs-13)'
            }}>
              <div style={{ fontWeight: 600, marginBottom: '8px', color: 'var(--text)' }}>Inspector</div>
              <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--text2)' }}>
                <span>Last</span>
                <strong style={{ color: 'var(--text)' }}>
                  {app.market ? formatPrice(app.market.last, app.symbol) : '--'}
                </strong>
              </div>
              <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--text2)' }}>
                <span>24H Vol</span>
                <strong style={{ color: 'var(--text)' }}>
                  {app.market ? app.market.volume24h.toLocaleString('en-US') : '--'}
                </strong>
              </div>
              {hoveredCandle && (
                <>
                  <div style={{ height: '1px', background: 'var(--border1)', margin: '8px 0' }} />
                  <div style={{ fontWeight: 600, color: 'var(--text)' }}>Hovered</div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--text2)' }}>
                    <span>O</span>
                    <strong style={{ color: 'var(--text)' }}>{formatPrice(hoveredCandle.open, app.symbol)}</strong>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--text2)' }}>
                    <span>H</span>
                    <strong style={{ color: 'var(--text)' }}>{formatPrice(hoveredCandle.high, app.symbol)}</strong>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--text2)' }}>
                    <span>L</span>
                    <strong style={{ color: 'var(--text)' }}>{formatPrice(hoveredCandle.low, app.symbol)}</strong>
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', color: 'var(--text2)' }}>
                    <span>C</span>
                    <strong style={{ color: 'var(--text)' }}>{formatPrice(hoveredCandle.close, app.symbol)}</strong>
                  </div>
                </>
              )}
            </aside>
          )}
        </div>

        {/* Status Strip */}
        <div className="status-strip" style={{
          padding: '8px 12px',
          background: 'var(--bg2)',
          borderTop: '1px solid var(--border1)',
          display: 'flex',
          alignItems: 'center',
          fontSize: 'var(--fs-12)',
          color: 'var(--text2)',
          gap: '12px'
        }}>
          <span>{app.symbol} · {app.timeframe}</span>
          (
            <button
              className="ue-btn small"
              onClick={async () => {
                try {
                  const currentPrice = app.market?.candles[app.market.candles.length - 1]?.close || 0;
                  const allCandles = app.market?.candles || [];
                  const recentCandles = allCandles.slice(-100); // Last 100 candles only for FVG scan
                  const response = await scanFvgs(app.symbol, app.timeframe, currentPrice, recentCandles);
                  console.log("FVG scan response:", response);
                  setFvgZones(response);
                  app.showToast(`Found ${response.fvgs.length} FVGs, ${response.ifvgs.length} iFVGs`);
                } catch (err) {
                  app.showToast(`FVG scan failed: ${err}`);
                }
              }}
              style={{ marginLeft: "8px" }}
            >
              F
            </button>
          <div style={{ flex: 1 }} />
          <span>V-CHART: Virtual Candle Simulator</span>
          <div style={{ width: 12 }} />
          <span>{app.loadingMarket ? 'Loading...' : 'Ready'}</span>
        </div>
      </div>
    </div>
  );
}
