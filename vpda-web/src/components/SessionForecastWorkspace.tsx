import { useState, useEffect } from 'react';
import type { VpdaAppState } from '../data/useVpdaApp';
import type { SessionForecastResult, SessionScenario, SessionScenarioPathSegment, SessionScenarioVisualizationLevel, SessionScenarioVisualization } from '../bridge/vpdaBridge';
import { runSessionForecast } from '../bridge/vpdaBridge';

type Props = {
  app: VpdaAppState;
};

export default function SessionForecastWorkspace({ app }: Props) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>('');
  const [targetDate, setTargetDate] = useState<string>('');
  const [targetTime, setTargetTime] = useState<string>('');

  // Phase 3: Progressive Disclosure - collapsible sections state
  const [expanded, setExpanded] = useState({
    contextFields: false,
    intradayMap: true,
    sessionLiquidity: false,
    sessionStats: false,
    hitRateAnalytics: false,
    dynamicUpdates: false,
  });

  // Toggle section expansion
  const toggleSection = (section: keyof typeof expanded) => {
    setExpanded(prev => ({ ...prev, [section]: !prev[section] }));
  };

  // Phase 6: Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Escape to clear error
      if (e.key === 'Escape' && error) {
        setError('');
      }
      // Ctrl+Enter or Cmd+Enter to run forecast
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter' && !loading && app.symbol) {
        e.preventDefault();
        handleRunForecast();
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [error, loading, app.symbol]);

  const result = app.sessionForecastResult;

  const handleRunForecast = async () => {
    if (!app.symbol) {
      setError('Please select a symbol first');
      return;
    }

    setLoading(true);
    setError('');
    app.setSessionForecastResult(null);

    try {
      const request = {
        symbol: app.symbol,
        targetDate: targetDate || null,
        targetTime: targetTime || null,
        bias: 'AUTO' as const,
      };

      const forecast = await runSessionForecast(request);

      if (forecast) {
        app.setSessionForecastResult(forecast);
      } else {
        setError('No data returned from forecast');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate forecast');
    } finally {
      setLoading(false);
    }
  };

  const handleDrawScenario = (scenario: SessionScenario) => {
    if (!result) return;

    // Parse timeframe from scenario description or targetLabel
    const detectTimeframe = (text: string): string => {
      if (text.includes('4H') || text.includes('4h')) return '4h';
      if (text.includes('1H') || text.includes('1h')) return '1h';
      if (text.includes('15M') || text.includes('15m')) return '15m';
      if (text.includes('5M') || text.includes('5m')) return '5m';
      return '1h'; // default
    };

    const timeframe = detectTimeframe(scenario.description + ' ' + scenario.targetLabel);

    // Create path segments for visualization
    const pathSegments: SessionScenarioPathSegment[] = [
      {
        description: 'Move to target',
        startPrice: result.currentPrice,
        endPrice: scenario.targetPrice,
        candleCount: Math.min(8, Math.max(6, Math.floor(Math.abs(scenario.targetPrice - result.currentPrice) / (result.currentPrice * 0.005)))),
        direction: scenario.targetPrice > result.currentPrice ? 'bullish' : 'bearish',
        volatility: 0.5,
      },
    ];

    // Create levels for visualization
    const levels: SessionScenarioVisualizationLevel[] = [
      {
        price: scenario.targetPrice,
        label: scenario.targetLabel,
        type: 'target',
        color: scenario.targetPrice > result.currentPrice ? '#10b981' : '#ef4444',
      },
    ];

    // Create visualization object
    const viz: SessionScenarioVisualization = {
      scenario,
      symbol: result.symbol,
      timeframe,
      currentPrice: result.currentPrice,
      levels,
      pathSegments,
    };

    // Pass custom date/time for admin truncation
    if (app.startup?.isAdmin && (targetDate || targetTime)) {
      (viz as any).customDate = targetDate;
      (viz as any).customTime = targetTime;
    }

    // Trigger visualization
    app.setSessionScenarioVisualization(viz);
    app.setTimeframe(timeframe);
    app.setWorkspace('vchart');
  };

  const formatPrice = (price: number) => {
    const symbol = app.symbol || '';
    if (symbol.includes('JPY')) return price.toFixed(3);
    if (symbol.includes('USD') || symbol.includes('EUR') || symbol.includes('GBP')) {
      return price.toFixed(5);
    }
    return price.toFixed(2);
  };

  const getScenarioColor = (prob: number) => {
    if (prob >= 0.7) return 'var(--chart-bull)';
    if (prob >= 0.5) return 'var(--warning)';
    return 'var(--text2)';
  };

  return (
    <div className="workspace-page" style={{
      display: 'flex',
      flexDirection: 'column',
      height: '100%',
      overflow: 'hidden',
      padding: '20px',
      background: 'var(--bg1)'
    }}>
      <div style={{
        marginBottom: '20px',
        fontSize: 'var(--fs-18)',
        fontWeight: 700,
        color: 'var(--text)'
      }}>
        SESSION FORECAST - {app.symbol || 'No Symbol'}
      </div>

      <button
        className="ue-btn small"
        onClick={handleRunForecast}
        disabled={loading || !app.symbol}
      >
        {loading ? 'Loading...' : 'Run Forecast'}
      </button>

      {app.startup?.isAdmin && (
        <div style={{ marginTop: '16px', display: 'flex', gap: '12px', alignItems: 'center' }}>
          <div>
            <label style={{ fontSize: 'var(--fs-12)', color: 'var(--text2)', marginBottom: '4px', display: 'block' }}>
              Target Date (Admin)
            </label>
            <input
              type="date"
              value={targetDate}
              onChange={(e) => setTargetDate(e.target.value)}
              style={{
                padding: '8px',
                fontSize: 'var(--fs-13)',
                background: 'var(--input-bg)',
                color: 'var(--text)',
                border: '1px solid var(--border1)',
                borderRadius: '4px',
              }}
            />
          </div>
          <div>
            <label style={{ fontSize: 'var(--fs-12)', color: 'var(--text2)', marginBottom: '4px', display: 'block' }}>
              Target Time (Admin)
            </label>
            <input
              type="time"
              value={targetTime}
              onChange={(e) => setTargetTime(e.target.value)}
              style={{
                padding: '8px',
                fontSize: 'var(--fs-13)',
                background: 'var(--input-bg)',
                color: 'var(--text)',
                border: '1px solid var(--border1)',
                borderRadius: '4px',
              }}
            />
          </div>
        </div>
      )}

      {loading && (
        <div style={{
          marginTop: '40px',
          textAlign: 'center',
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          gap: '16px'
        }}>
          <div style={{
            width: '40px',
            height: '40px',
            border: '3px solid var(--border2)',
            borderTop: '3px solid var(--accent)',
            borderRadius: '50%',
            animation: 'spin 0.8s linear infinite'
          }} />
          <div style={{ color: 'var(--text2)', fontSize: 'var(--fs-14)' }}>
            Generating session forecast...
          </div>
        </div>
      )}

      {error && (
        <div style={{
          marginTop: '40px',
          padding: '20px',
          background: 'linear-gradient(to right, rgba(239, 68, 68, 0.1), var(--bg-error))',
          border: '1px solid var(--border-error)',
          borderRadius: '6px',
          display: 'flex',
          flexDirection: 'column',
          gap: '12px'
        }}>
          <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
            <span style={{ fontSize: 'var(--fs-18)', color: 'var(--text-error)' }}>⚠</span>
            <strong style={{ color: 'var(--text-error)', fontSize: 'var(--fs-14)' }}>Error</strong>
          </div>
          <div style={{ color: 'var(--text-error)', fontSize: 'var(--fs-13)', lineHeight: '1.5' }}>
            {error}
          </div>
          <button
            className="ue-btn small"
            onClick={handleRunForecast}
            style={{ alignSelf: 'flex-start', marginTop: '4px' }}
          >
            Retry
          </button>
        </div>
      )}

      {result && !loading && (
        <div style={{ marginTop: '20px', display: 'flex', flexDirection: 'column', gap: '16px', flex: 1, overflow: 'auto' }}>
          {/* Summary Card */}
          <div style={{ padding: '16px', background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px' }}>
            <div style={{ fontSize: 'var(--fs-13)', fontWeight: 600, color: 'var(--text2)', marginBottom: '8px' }}>SUMMARY</div>
            <div style={{ fontSize: 'var(--fs-14)', color: 'var(--text)', marginBottom: '12px' }}>{result.summary}</div>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(3, 1fr)', gap: '12px' }}>
              <div>
                <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>Bias</div>
                <div style={{ fontSize: 'var(--fs-15)', fontWeight: 600, color: 'var(--text)' }}>{result.bias}</div>
              </div>
              <div>
                <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>Confidence</div>
                <div style={{ fontSize: 'var(--fs-15)', fontWeight: 600, color: 'var(--text)' }}>{(result.confidence * 100).toFixed(0)}%</div>
              </div>
              <div>
                <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>Current Price</div>
                <div style={{ fontSize: 'var(--fs-15)', fontWeight: 600, color: 'var(--text)' }}>{formatPrice(result.currentPrice)}</div>
              </div>
            </div>
          </div>

          {/* Scenarios */}
          {result.scenarios.length > 0 && (
            <div>
              <div style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase' }}>
                SCENARIOS ({result.scenarios.length})
              </div>
              {result.scenarios.map((scenario) => {
                const isBullish = scenario.description.toLowerCase().includes('bullish');
                const isBearish = scenario.description.toLowerCase().includes('bearish');
                const semanticBg = isBullish
                  ? 'linear-gradient(to right, rgba(34, 197, 94, 0.08), var(--bg2))'
                  : isBearish
                  ? 'linear-gradient(to right, rgba(239, 68, 68, 0.08), var(--bg2))'
                  : 'var(--bg2)';

                return (
                <div key={scenario.rank} style={{ marginBottom: '12px', padding: '16px', background: semanticBg, border: '1px solid var(--border1)', borderRadius: '6px' }}>
                  <div style={{ display: 'flex', alignItems: 'flex-start', gap: '12px', marginBottom: '12px' }}>
                    <div style={{ width: '28px', height: '28px', borderRadius: '50%', background: 'var(--bg3)', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: 'var(--fs-13)', fontWeight: 700, flexShrink: 0 }}>
                      {scenario.rank}
                    </div>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div style={{ fontSize: 'var(--fs-15)', fontWeight: 600, color: 'var(--text)', marginBottom: '4px' }}>
                        {scenario.description}
                      </div>
                      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text2)' }}>
                        Target: {formatPrice(scenario.targetPrice)} • {scenario.targetLabel}
                      </div>
                    </div>
                    <div style={{ fontSize: 'var(--fs-18)', fontWeight: 700, color: getScenarioColor(scenario.probability), flexShrink: 0 }}>
                      {(scenario.probability * 100).toFixed(0)}%
                    </div>
                  </div>

                  {/* Probability Bar */}
                  <div style={{ width: '100%', height: '4px', background: 'var(--bg3)', borderRadius: '2px', overflow: 'hidden', marginBottom: '12px' }}>
                    <div style={{ width: `${scenario.probability * 100}%`, height: '100%', background: getScenarioColor(scenario.probability), transition: 'width 0.3s' }} />
                  </div>

                  {/* Draw Button */}
                  <button
                    className="ue-btn small"
                    onClick={() => handleDrawScenario(scenario)}
                  >
                    Draw Scenario
                  </button>

                  {/* Roadmap */}
                  {scenario.roadmap.length > 0 && (
                    <div>
                      <div style={{ fontSize: 'var(--fs-11)', fontWeight: 600, color: 'var(--text2)', marginBottom: '6px', textTransform: 'uppercase' }}>
                        Roadmap
                      </div>
                      {scenario.roadmap.map((step, idx) => (
                        <div key={idx} style={{ fontSize: 'var(--fs-12)', color: 'var(--text)', marginBottom: '4px', display: 'flex', gap: '6px' }}>
                          <span style={{ color: 'var(--text2)', minWidth: '16px' }}>{idx + 1}.</span>
                          <span style={{ wordBreak: 'break-word' }}>{step}</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
              })}
            </div>
          )}

          {/* Levels */}
          {result.levels.length > 0 && (
            <div>
              <div style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase' }}>
                KEY LEVELS ({result.levels.length})
              </div>
              <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden' }}>
                {result.levels.map((level, idx) => {
                  const direction = level.price > result.currentPrice ? '↑' : '↓';
                  const directionColor = level.price > result.currentPrice ? 'var(--chart-bull)' : 'var(--chart-bear)';
                  const sigColor = level.significance >= 0.7 ? 'var(--chart-bull)' : level.significance >= 0.4 ? 'var(--warning)' : 'var(--text2)';

                  return (
                    <div
                      key={idx}
                      style={{
                        padding: '12px 16px',
                        borderBottom: idx < result.levels.length - 1 ? '1px solid var(--border1)' : 'none',
                        display: 'flex',
                        alignItems: 'center',
                        gap: '12px'
                      }}
                    >
                      <div style={{ fontSize: 'var(--fs-16)', color: directionColor, fontWeight: 700, minWidth: '20px' }}>
                        {direction}
                      </div>
                      <div style={{ flex: 1 }}>
                        <div style={{ fontSize: 'var(--fs-13)', fontWeight: 600, color: 'var(--text)', marginBottom: '2px' }}>
                          {level.label}
                        </div>
                        <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>
                          {level.levelType}
                        </div>
                      </div>
                      <div style={{ fontSize: 'var(--fs-14)', fontWeight: 600, color: 'var(--text)' }}>
                        {formatPrice(level.price)}
                      </div>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '6px', minWidth: '70px', justifyContent: 'flex-end' }}>
                        <div style={{ width: '6px', height: '6px', borderRadius: '50%', background: sigColor }} />
                        <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>
                          {(level.significance * 100).toFixed(0)}%
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Upside Targets */}
          {result.upsideTargets && result.upsideTargets.length > 0 && (
            <div>
              <div style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase' }}>
                UPSIDE TARGETS ({result.upsideTargets.length})
              </div>
              <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden' }}>
                {result.upsideTargets.map((target, idx) => {
                  const sigColor = target.significance >= 0.7 ? 'var(--chart-bull)' : target.significance >= 0.4 ? 'var(--warning)' : 'var(--text2)';

                  return (
                    <div
                      key={idx}
                      style={{
                        padding: '12px 16px',
                        borderBottom: idx < result.upsideTargets.length - 1 ? '1px solid var(--border1)' : 'none',
                        display: 'grid',
                        gridTemplateColumns: '20px 2fr 1fr 1fr 1fr 90px',
                        gap: '12px',
                        alignItems: 'center'
                      }}
                    >
                      <div style={{ fontSize: 'var(--fs-16)', color: 'var(--chart-bull)', fontWeight: 700 }}>↑</div>
                      <div>
                        <div style={{ fontSize: 'var(--fs-13)', fontWeight: 600, color: 'var(--text)' }}>
                          {target.levelType}
                        </div>
                      </div>
                      <div style={{ fontSize: 'var(--fs-13)', color: 'var(--text)' }}>
                        {formatPrice(target.price)}
                      </div>
                      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text2)' }}>
                        {target.distancePips.toFixed(1)} pips
                      </div>
                      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text2)' }}>
                        {target.sessionOrigin}
                      </div>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '6px', justifyContent: 'flex-end' }}>
                        <div style={{ width: '6px', height: '6px', borderRadius: '50%', background: sigColor }} />
                        <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>
                          {(target.significance * 100).toFixed(0)}%
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Downside Targets */}
          {result.downsideTargets && result.downsideTargets.length > 0 && (
            <div>
              <div style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase' }}>
                DOWNSIDE TARGETS ({result.downsideTargets.length})
              </div>
              <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden' }}>
                {result.downsideTargets.map((target, idx) => {
                  const sigColor = target.significance >= 0.7 ? 'var(--chart-bear)' : target.significance >= 0.4 ? 'var(--warning)' : 'var(--text2)';

                  return (
                    <div
                      key={idx}
                      style={{
                        padding: '12px 16px',
                        borderBottom: idx < result.downsideTargets.length - 1 ? '1px solid var(--border1)' : 'none',
                        display: 'grid',
                        gridTemplateColumns: '20px 2fr 1fr 1fr 1fr 90px',
                        gap: '12px',
                        alignItems: 'center'
                      }}
                    >
                      <div style={{ fontSize: 'var(--fs-16)', color: 'var(--chart-bear)', fontWeight: 700 }}>↓</div>
                      <div>
                        <div style={{ fontSize: 'var(--fs-13)', fontWeight: 600, color: 'var(--text)' }}>
                          {target.levelType}
                        </div>
                      </div>
                      <div style={{ fontSize: 'var(--fs-13)', color: 'var(--text)' }}>
                        {formatPrice(target.price)}
                      </div>
                      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text2)' }}>
                        {target.distancePips.toFixed(1)} pips
                      </div>
                      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text2)' }}>
                        {target.sessionOrigin}
                      </div>
                      <div style={{ display: 'flex', alignItems: 'center', gap: '6px', justifyContent: 'flex-end' }}>
                        <div style={{ width: '6px', height: '6px', borderRadius: '50%', background: sigColor }} />
                        <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>
                          {(target.significance * 100).toFixed(0)}%
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Long-term Targets */}
          {((result.longTermUpside && result.longTermUpside.length > 0) || (result.longTermDownside && result.longTermDownside.length > 0)) && (
            <div>
              <div style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase' }}>
                LONG-TERM TARGETS
              </div>
              {result.longTermUpside && result.longTermUpside.length > 0 && (
                <div style={{ marginBottom: '12px' }}>
                  <div style={{ fontSize: 'var(--fs-12)', fontWeight: 600, color: 'var(--text2)', marginBottom: '8px' }}>
                    LONG-TERM UPSIDE ({result.longTermUpside.length})
                  </div>
                  <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden' }}>
                    {result.longTermUpside.map((target, idx) => (
                      <div
                        key={idx}
                        style={{
                          padding: '10px 16px',
                          borderBottom: idx < result.longTermUpside.length - 1 ? '1px solid var(--border1)' : 'none',
                          display: 'grid',
                          gridTemplateColumns: '2fr 1fr 1fr 1fr 80px',
                          gap: '12px',
                          alignItems: 'center'
                        }}
                      >
                        <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text)' }}>{target.levelType}</div>
                        <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text)' }}>{formatPrice(target.price)}</div>
                        <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>{target.distancePips.toFixed(1)} pips</div>
                        <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>{target.sessionOrigin}</div>
                        <div style={{ fontSize: 'var(--fs-10)', color: 'var(--text2)', textAlign: 'right' }}>{(target.significance * 100).toFixed(0)}%</div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
              {result.longTermDownside && result.longTermDownside.length > 0 && (
                <div>
                  <div style={{ fontSize: 'var(--fs-12)', fontWeight: 600, color: 'var(--text2)', marginBottom: '8px' }}>
                    LONG-TERM DOWNSIDE ({result.longTermDownside.length})
                  </div>
                  <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden' }}>
                    {result.longTermDownside.map((target, idx) => (
                      <div
                        key={idx}
                        style={{
                          padding: '10px 16px',
                          borderBottom: idx < result.longTermDownside.length - 1 ? '1px solid var(--border1)' : 'none',
                          display: 'grid',
                          gridTemplateColumns: '2fr 1fr 1fr 1fr 80px',
                          gap: '12px',
                          alignItems: 'center'
                        }}
                      >
                        <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text)' }}>{target.levelType}</div>
                        <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text)' }}>{formatPrice(target.price)}</div>
                        <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>{target.distancePips.toFixed(1)} pips</div>
                        <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>{target.sessionOrigin}</div>
                        <div style={{ fontSize: 'var(--fs-10)', color: 'var(--text2)', textAlign: 'right' }}>{(target.significance * 100).toFixed(0)}%</div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Context Fields */}
          {result.contextFields && result.contextFields.length > 0 && (
            <div>
              <div
                style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase', cursor: 'pointer', userSelect: 'none', display: 'flex', alignItems: 'center', gap: '8px' }}
                onClick={() => toggleSection('contextFields')}
              >
                <span>{expanded.contextFields ? '▼' : '▶'}</span>
                <span>CONTEXT FIELDS ({result.contextFields.length})</span>
              </div>
              {expanded.contextFields && (
                <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden' }}>
                  {result.contextFields.map((field, idx) => (
                    <div
                      key={idx}
                      style={{
                        padding: '12px 16px',
                        borderBottom: idx < result.contextFields.length - 1 ? '1px solid var(--border1)' : 'none',
                        display: 'grid',
                        gridTemplateColumns: '1fr 2fr',
                        gap: '16px',
                        alignItems: 'center'
                      }}
                    >
                      <div style={{ fontSize: 'var(--fs-12)', fontWeight: 600, color: 'var(--text2)' }}>{field.field}</div>
                      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text)' }}>{field.value}</div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Intraday Map */}
          {result.intradayMap && result.intradayMap.length > 0 && (
            <div>
              <div
                style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase', cursor: 'pointer', userSelect: 'none', display: 'flex', alignItems: 'center', gap: '8px' }}
                onClick={() => toggleSection('intradayMap')}
              >
                <span>{expanded.intradayMap ? '▼' : '▶'}</span>
                <span>INTRADAY MAP ({result.intradayMap.length})</span>
              </div>
              {expanded.intradayMap && (
                <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden' }}>
                  <div style={{ padding: '10px 16px', borderBottom: '1px solid var(--border1)', background: 'var(--bg1)', display: 'grid', gridTemplateColumns: '80px 100px 100px 120px 120px 100px', gap: '12px', fontSize: 'var(--fs-11)', fontWeight: 600, color: 'var(--text2)', textTransform: 'uppercase' }}>
                    <div>TF</div>
                    <div>Type</div>
                    <div>Status</div>
                    <div>Lower</div>
                    <div>Upper</div>
                    <div>Mid</div>
                  </div>
                  {result.intradayMap.map((entry, idx) => (
                    <div
                      key={idx}
                      style={{
                        padding: '12px 16px',
                        borderBottom: idx < result.intradayMap.length - 1 ? '1px solid var(--border1)' : 'none',
                        display: 'grid',
                        gridTemplateColumns: '80px 100px 100px 120px 120px 100px',
                        gap: '12px',
                        alignItems: 'center'
                      }}
                    >
                      <div style={{ fontSize: 'var(--fs-12)', fontWeight: 600, color: 'var(--text)' }}>{entry.timeframe}</div>
                      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text)' }}>{entry.gapType}</div>
                      <div style={{ fontSize: 'var(--fs-12)', color: entry.status === 'Held' ? 'var(--green)' : entry.status === 'Inverted' ? 'var(--red)' : 'var(--yellow)' }}>{entry.status}</div>
                      <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>{formatPrice(entry.rangeLower)}</div>
                      <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>{formatPrice(entry.rangeUpper)}</div>
                      <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>{formatPrice(entry.mid)}</div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Session Liquidity */}
          {result.sessionLiquidity && result.sessionLiquidity.length > 0 && (
            <div>
              <div
                style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase', cursor: 'pointer', userSelect: 'none', display: 'flex', alignItems: 'center', gap: '8px' }}
                onClick={() => toggleSection('sessionLiquidity')}
              >
                <span>{expanded.sessionLiquidity ? '▼' : '▶'}</span>
                <span>SESSION LIQUIDITY ({result.sessionLiquidity.length})</span>
              </div>
              {expanded.sessionLiquidity && (
                <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden' }}>
                  <div style={{ padding: '10px 16px', borderBottom: '1px solid var(--border1)', background: 'var(--bg1)', display: 'grid', gridTemplateColumns: '100px 120px 120px 100px 100px', gap: '12px', fontSize: 'var(--fs-11)', fontWeight: 600, color: 'var(--text2)', textTransform: 'uppercase' }}>
                    <div>Session</div>
                    <div>High</div>
                    <div>Low</div>
                    <div>High Taken</div>
                    <div>Low Taken</div>
                  </div>
                  {result.sessionLiquidity.map((entry, idx) => (
                    <div
                      key={idx}
                      style={{
                        padding: '12px 16px',
                        borderBottom: idx < result.sessionLiquidity.length - 1 ? '1px solid var(--border1)' : 'none',
                        display: 'grid',
                        gridTemplateColumns: '100px 120px 120px 100px 100px',
                        gap: '12px',
                        alignItems: 'center'
                      }}
                    >
                      <div style={{ fontSize: 'var(--fs-12)', fontWeight: 600, color: 'var(--text)' }}>{entry.session}</div>
                      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text)' }}>{formatPrice(entry.high)}</div>
                      <div style={{ fontSize: 'var(--fs-12)', color: 'var(--text)' }}>{formatPrice(entry.low)}</div>
                      <div style={{ fontSize: 'var(--fs-11)', color: entry.highTaken === 'Yes' ? 'var(--red)' : 'var(--green)' }}>{entry.highTaken}</div>
                      <div style={{ fontSize: 'var(--fs-11)', color: entry.lowTaken === 'Yes' ? 'var(--red)' : 'var(--green)' }}>{entry.lowTaken}</div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Session Stats */}
          {result.sessionStats && result.sessionStats.length > 0 && (
            <div>
              <div
                style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase', cursor: 'pointer', userSelect: 'none', display: 'flex', alignItems: 'center', gap: '8px' }}
                onClick={() => toggleSection('sessionStats')}
              >
                <span>{expanded.sessionStats ? '▼' : '▶'}</span>
                <span>SESSION STATS</span>
              </div>
              {expanded.sessionStats && (
                <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden', padding: '16px' }}>
                  {result.sessionStats.map((stat, idx) => (
                    <div
                      key={idx}
                      style={{
                        fontSize: 'var(--fs-12)',
                        color: 'var(--text)',
                        marginBottom: idx < result.sessionStats.length - 1 ? '8px' : '0',
                        lineHeight: '1.6'
                      }}
                    >
                      {stat}
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Hit Rate Analytics */}
          {result.hitRateAnalytics && result.hitRateAnalytics.length > 0 && (
            <div>
              <div
                style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase', cursor: 'pointer', userSelect: 'none', display: 'flex', alignItems: 'center', gap: '8px' }}
                onClick={() => toggleSection('hitRateAnalytics')}
              >
                <span>{expanded.hitRateAnalytics ? '▼' : '▶'}</span>
                <span>HIT RATE ANALYTICS</span>
              </div>
              {expanded.hitRateAnalytics && (
                <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden', padding: '16px' }}>
                  {result.hitRateAnalytics.map((analytic, idx) => (
                    <div
                      key={idx}
                      style={{
                        fontSize: 'var(--fs-12)',
                        color: 'var(--text)',
                        marginBottom: idx < result.hitRateAnalytics.length - 1 ? '8px' : '0',
                        lineHeight: '1.6',
                        fontFamily: 'monospace'
                      }}
                    >
                      {analytic}
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Critical Alerts */}
          {result.criticalAlerts && result.criticalAlerts.length > 0 && (
            <div>
              <div style={{ fontSize: 'var(--fs-13)', fontWeight: 600, color: 'var(--red)', marginBottom: '12px', textTransform: 'uppercase' }}>
                ⚠️ CRITICAL ALERTS
              </div>
              <div style={{ background: 'rgba(255, 82, 82, 0.1)', border: '1px solid var(--red)', borderRadius: '6px', overflow: 'hidden', padding: '16px' }}>
                {result.criticalAlerts.map((alert, idx) => (
                  <div
                    key={idx}
                    style={{
                      fontSize: 'var(--fs-12)',
                      color: 'var(--red)',
                      marginBottom: idx < result.criticalAlerts.length - 1 ? '12px' : '0',
                      lineHeight: '1.6',
                      fontWeight: 600
                    }}
                  >
                    {alert}
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Dynamic Updates */}
          {result.dynamicUpdates && result.dynamicUpdates.length > 0 && (
            <div>
              <div
                style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase', cursor: 'pointer', userSelect: 'none', display: 'flex', alignItems: 'center', gap: '8px' }}
                onClick={() => toggleSection('dynamicUpdates')}
              >
                <span>{expanded.dynamicUpdates ? '▼' : '▶'}</span>
                <span>DYNAMIC UPDATES</span>
              </div>
              {expanded.dynamicUpdates && (
                <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden', padding: '16px' }}>
                  {result.dynamicUpdates.map((update, idx) => (
                    <div
                      key={idx}
                      style={{
                        fontSize: 'var(--fs-12)',
                        color: 'var(--text)',
                        marginBottom: idx < result.dynamicUpdates.length - 1 ? '8px' : '0',
                        lineHeight: '1.6'
                      }}
                    >
                      {update}
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Levels (original) */}
          {result && result.levels.length > 0 && false && (
            <div>
              <div style={{ fontSize: 'var(--fs-13)', fontWeight: 700, color: 'var(--text2)', marginBottom: '20px', textTransform: 'uppercase' }}>
                KEY LEVELS ({result?.levels.length ?? 0})
              </div>
              <div style={{ background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px', overflow: 'hidden' }}>
                {result?.levels.map((level, idx) => (
                  <div
                    key={idx}
                    style={{
                      padding: '12px 16px',
                      borderBottom: idx < (result?.levels.length ?? 0) - 1 ? '1px solid var(--border1)' : 'none',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '12px'
                    }}
                  >
                    <div style={{ flex: 1 }}>
                      <div style={{ fontSize: 'var(--fs-13)', fontWeight: 600, color: 'var(--text)', marginBottom: '2px' }}>
                        {level.label}
                      </div>
                      <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)' }}>
                        {level.levelType}
                      </div>
                    </div>
                    <div style={{ fontSize: 'var(--fs-14)', fontWeight: 600, color: 'var(--text)' }}>
                      {formatPrice(level.price)}
                    </div>
                    <div style={{ fontSize: 'var(--fs-11)', color: 'var(--text2)', minWidth: '50px', textAlign: 'right' }}>
                      {(level.significance * 100).toFixed(0)}%
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Warnings */}
          {result.warnings.length > 0 && (
            <div style={{ padding: '14px', background: 'var(--bg-warning)', border: '1px solid var(--border-warning)', borderRadius: '6px' }}>
              <div style={{ fontSize: 'var(--fs-12)', fontWeight: 600, color: 'var(--text-warning)', marginBottom: '8px', textTransform: 'uppercase' }}>
                ⚠ WARNINGS
              </div>
              {result.warnings.map((warning, idx) => (
                <div key={idx} style={{ fontSize: 'var(--fs-12)', color: 'var(--text-warning)', marginBottom: idx < result.warnings.length - 1 ? '4px' : 0 }}>
                  • {warning}
                </div>
              ))}
            </div>
          )}

          {/* Context */}
          {result.context.length > 0 && (
            <div style={{ padding: '14px', background: 'var(--bg2)', border: '1px solid var(--border1)', borderRadius: '6px' }}>
              <div style={{ fontSize: 'var(--fs-12)', fontWeight: 600, color: 'var(--text2)', marginBottom: '8px', textTransform: 'uppercase' }}>
                CONTEXT
              </div>
              {result.context.map((ctx, idx) => (
                <div key={idx} style={{ fontSize: 'var(--fs-12)', color: 'var(--text2)', marginBottom: idx < result.context.length - 1 ? '4px' : 0 }}>
                  • {ctx}
                </div>
              ))}
            </div>
          )}

        </div>
      )}
    </div>
  );
}
