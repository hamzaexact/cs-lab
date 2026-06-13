import type { VpdaAppState } from '../data/useVpdaApp';
import type { ForecastScenario, ScenarioStep } from '../bridge/vpdaBridge';
import { formatPrice } from '../utils/format';
import { useState } from 'react';

type Props = {
  app: VpdaAppState;
};

function directionClass(direction: 'Bullish' | 'Bearish') {
  return direction === 'Bullish' ? 'bull' : 'bear';
}

function actionClass(action: string) {
  switch (action) {
    case 'SWEEP':    return 'step-sweep';
    case 'TAP':      return 'step-tap';
    case 'DELIVER':  return 'step-deliver';
    case 'DISPLACE': return 'step-displace';
    case 'STRUCTURE':return 'step-structure';
    case 'REJECT':   return 'step-reject';
    case 'HOLD':     return 'step-hold';
    default:         return 'step-default';
  }
}

function formatPercent(value: number) {
  const rounded = Math.round(value * 10) / 10;
  return Number.isInteger(rounded) ? `${rounded.toFixed(0)}%` : `${rounded.toFixed(1)}%`;
}

function getConfidenceLevel(probability: number): { level: string; class: string; color: string } {
  if (probability >= 70) {
    return { level: 'HIGH', class: 'confidence-high', color: '#22c55e' };
  } else if (probability >= 50) {
    return { level: 'MEDIUM', class: 'confidence-medium', color: '#f59e0b' };
  } else {
    return { level: 'LOW', class: 'confidence-low', color: '#ef4444' };
  }
}

function formatScenarioDescription(detail: string): string[] {
  // Split long descriptions into readable paragraphs
  // Split on ". " to create sentence breaks for better readability
  const sentences = detail.split(/\.\s+/);
  const paragraphs: string[] = [];
  let current = '';

  for (let i = 0; i < sentences.length; i++) {
    const sentence = sentences[i];
    if (current.length + sentence.length > 200) {
      // Start new paragraph if current is getting too long
      if (current) paragraphs.push(current + '.');
      current = sentence;
    } else {
      current = current ? current + '. ' + sentence : sentence;
    }
  }

  if (current) {
    paragraphs.push(current.endsWith('.') ? current : current + '.');
  }

  return paragraphs.length > 0 ? paragraphs : [detail];
}

function BiasStrengthMeter({ confidence }: { confidence: number }) {
  const confidenceLevel = getConfidenceLevel(confidence);
  return (
    <div style={{ marginTop: '12px' }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '6px' }}>
        <span style={{ fontSize: '13px', fontWeight: 500, opacity: 0.8 }}>Bias Strength</span>
        <span style={{ fontSize: '13px', fontWeight: 600, color: confidenceLevel.color }}>
          {confidenceLevel.level} ({Math.round(confidence)}%)
        </span>
      </div>
      <div style={{
        height: '8px',
        backgroundColor: 'rgba(255,255,255,0.1)',
        borderRadius: '4px',
        overflow: 'hidden',
        position: 'relative'
      }}>
        <div style={{
          width: `${confidence}%`,
          height: '100%',
          backgroundColor: confidenceLevel.color,
          transition: 'width 0.3s ease',
          borderRadius: '4px'
        }} />
      </div>
    </div>
  );
}

function RoadmapStep({ step, symbol }: { step: ScenarioStep; symbol: string }) {
  return (
    <div className="roadmap-step">
      <div className="roadmap-step-seq">{step.sequence}</div>
      <div className="roadmap-step-body">
        <div className="roadmap-step-header">
          <span className={`roadmap-action ${actionClass(step.action)}`}>{step.action}</span>
          <span className="roadmap-step-day">{step.day}</span>
          <span className="roadmap-step-price">{formatPrice(step.price, symbol)}</span>
        </div>
        <div className="roadmap-step-pdref">{step.pdRef}</div>
        <div className="roadmap-step-condition">{step.condition}</div>
        {step.invalidation !== null && (
          <div className="roadmap-step-inv">
            Invalidation: {formatPrice(step.invalidation, symbol)}
          </div>
        )}
      </div>
    </div>
  );
}

function ScenarioCard({
  app,
  scenario,
  symbol,
  rank,
}: {
  app: VpdaAppState;
  scenario: ForecastScenario;
  symbol: string;
  rank: number;
}) {
  const rankLabel = rank === 0 ? 'PRIMARY' : rank === 1 ? 'SECONDARY' : 'ALT';
  const barLen = Math.round((scenario.probability / 100) * 20);
  const sourceLabel = scenario.sourceTag ?? 'Legacy';
  const confidence = getConfidenceLevel(scenario.probability);
  const [roadmapExpanded, setRoadmapExpanded] = useState(true);
  const [confluenceExpanded, setConfluenceExpanded] = useState(true);

  return (
    <div className={`scenario-full-card rank-${rankLabel.toLowerCase()}`}>
      <div className="scenario-full-header">
        <div className="scenario-rank-badge">{rankLabel}</div>
        <strong className="scenario-full-name">{scenario.name}</strong>
        <span className="scenario-source-tag">{sourceLabel}</span>
        <span className={`scenario-confidence-badge ${confidence.class}`}>
          {confidence.level}
        </span>
        <span className={directionClass(scenario.direction)}>
          {scenario.direction}
        </span>
        <div className="scenario-prob-bar-wrap">
          <div className="scenario-prob-bar">
            <div style={{ width: `${scenario.probability}%`, backgroundColor: confidence.color }} />
          </div>
          <span className="scenario-prob-pct" style={{ color: confidence.color, fontWeight: 600 }}>
            {Math.round(scenario.probability)}%
          </span>
        </div>
      </div>

      <div className="scenario-full-desc">
        {formatScenarioDescription(scenario.detail).map((para, i) => (
          <p key={i} style={{ marginBottom: i < formatScenarioDescription(scenario.detail).length - 1 ? '8px' : '0' }}>
            {para}
          </p>
        ))}
      </div>

      <div className="scenario-card-actions" style={{ display: 'flex', gap: '6px', marginBottom: '12px' }}>
        <button className="ue-btn small" onClick={() => app.requestForecastScenarioDraw(scenario)}>
          Draw on Chart
        </button>
      </div>

      {/* Roadmap */}
      {scenario.pathSteps && scenario.pathSteps.length > 0 && (
        <div className="roadmap-block">
          <div className="roadmap-label" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }} onClick={() => setRoadmapExpanded(!roadmapExpanded)}>
            <span>ROADMAP ({scenario.pathSteps.length} steps)</span>
            <span style={{ fontSize: '12px', userSelect: 'none' }}>{roadmapExpanded ? '▼' : '▶'}</span>
          </div>
          {roadmapExpanded && (
            <div className="roadmap-steps">
              {[...scenario.pathSteps]
                .sort((a, b) => a.sequence - b.sequence)
                .map((step) => (
                  <RoadmapStep key={step.sequence} step={step} symbol={symbol} />
                ))}
            </div>
          )}
        </div>
      )}

      {/* PD Confluence */}
      {scenario.pdConfluence && scenario.pdConfluence.filter(p => !p.startsWith('▶ Profile')).length > 0 && (
        <div className="scenario-confluence">
          <div className="confluence-label" style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', cursor: 'pointer' }} onClick={() => setConfluenceExpanded(!confluenceExpanded)}>
            <span>CONFLUENCE ({scenario.pdConfluence.filter(p => !p.startsWith('▶ Profile')).length} items)</span>
            <span style={{ fontSize: '12px', userSelect: 'none' }}>{confluenceExpanded ? '▼' : '▶'}</span>
          </div>
          {confluenceExpanded && scenario.pdConfluence
            .filter(p => !p.startsWith('▶ Profile'))
            .map((pd, i) => (
              <div key={i} className="confluence-item">· {pd}</div>
            ))}
        </div>
      )}

      {/* Targets */}
      <div className="scenario-targets" style={{ marginTop: '16px', display: 'grid', gap: '12px', gridTemplateColumns: scenario.manipulationTarget && scenario.weeklyTarget ? '1fr 1fr' : '1fr' }}>
        {scenario.manipulationTarget && (
          <div className="scenario-target manip" style={{
            padding: '12px',
            borderRadius: '6px',
            backgroundColor: 'rgba(251, 191, 36, 0.1)',
            border: '1px solid rgba(251, 191, 36, 0.3)'
          }}>
            <div style={{ fontSize: '11px', fontWeight: 600, color: '#fbbf24', marginBottom: '6px', textTransform: 'uppercase', letterSpacing: '0.5px' }}>
              Manipulation Target
            </div>
            <div style={{ fontSize: '13px', marginBottom: '4px', opacity: 0.9 }}>
              {scenario.manipulationTarget.name}
            </div>
            <div style={{ fontSize: '16px', fontWeight: 600, fontFamily: 'monospace', color: '#fbbf24' }}>
              {formatPrice(scenario.manipulationTarget.price, symbol)}
            </div>
          </div>
        )}
        {scenario.weeklyTarget && (
          <div className="scenario-target draw" style={{
            padding: '12px',
            borderRadius: '6px',
            backgroundColor: 'rgba(59, 130, 246, 0.1)',
            border: '1px solid rgba(59, 130, 246, 0.3)'
          }}>
            <div style={{ fontSize: '11px', fontWeight: 600, color: '#3b82f6', marginBottom: '6px', textTransform: 'uppercase', letterSpacing: '0.5px' }}>
              Weekly Draw
            </div>
            <div style={{ fontSize: '13px', marginBottom: '4px', opacity: 0.9 }}>
              {scenario.weeklyTarget.name}
            </div>
            <div style={{ fontSize: '16px', fontWeight: 600, fontFamily: 'monospace', color: '#3b82f6' }}>
              {formatPrice(scenario.weeklyTarget.price, symbol)}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

type SectionBlock =
  | { type: 'text'; lines: string[] }
  | { type: 'table'; rows: string[][] };

function isPipeRow(line: string) {
  return line.includes('|') && line.split('|').filter((cell) => cell.trim().length > 0).length >= 2;
}

function parsePipeRow(line: string) {
  return line
    .split('|')
    .map((cell) => cell.trim())
    .filter((cell) => cell.length > 0);
}

function sectionBlocks(lines: string[]): SectionBlock[] {
  const blocks: SectionBlock[] = [];
  let textBuffer: string[] = [];
  let tableBuffer: string[][] = [];

  const flushText = () => {
    if (textBuffer.length) {
      blocks.push({ type: 'text', lines: textBuffer });
      textBuffer = [];
    }
  };

  const flushTable = () => {
    if (tableBuffer.length) {
      blocks.push({ type: 'table', rows: tableBuffer });
      tableBuffer = [];
    }
  };

  for (const rawLine of lines) {
    const line = rawLine.trim();
    if (!line) {
      flushText();
      flushTable();
      continue;
    }

    if (isPipeRow(line)) {
      flushText();
      tableBuffer.push(parsePipeRow(line));
    } else {
      flushTable();
      textBuffer.push(line);
    }
  }

  flushText();
  flushTable();

  return blocks;
}

function renderSection(section: { title: string; lines: string[] }) {
  return sectionBlocks(section.lines).map((block, blockIndex) => {
    if (block.type === 'text') {
      return (
        <div key={`${section.title}-text-${blockIndex}`} className="section-lines">
          {block.lines.map((line, lineIndex) => (
            <div key={`${section.title}-text-${blockIndex}-${lineIndex}`} className="section-paragraph">
              {line}
            </div>
          ))}
        </div>
      );
    }

    const [header, ...rows] = block.rows;
    return (
      <div key={`${section.title}-table-${blockIndex}`} className="section-table-wrap">
        <table className="section-table">
          <thead>
            <tr>
              {header.map((cell, cellIndex) => (
                <th key={`${section.title}-head-${blockIndex}-${cellIndex}`}>{cell}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((row, rowIndex) => (
              <tr key={`${section.title}-row-${blockIndex}-${rowIndex}`}>
                {row.map((cell, cellIndex) => (
                  <td key={`${section.title}-cell-${blockIndex}-${rowIndex}-${cellIndex}`}>{cell}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    );
  });
}

export default function ForecastWorkspace({ app }: Props) {
  return (
    <div className="workspace-page forecast-page">
      <div className="action-toolbar forecast-toolbar">
        <div className="toolbar-group">
          <span className="toolbar-label">Asset</span>
          <select
            className="toolbar-select"
            value={app.forecastSymbol}
            onChange={(e) => app.setForecastSymbol(e.target.value)}
          >
            {app.watchlist.map((item) => (
              <option key={item.symbol} value={item.symbol}>
                {item.symbol}
              </option>
            ))}
          </select>
        </div>

        <div className="toolbar-group">
          <span className="toolbar-label">Target</span>
          <select
            className="toolbar-select"
            value={app.forecastTarget}
            onChange={(e) => app.setForecastTarget(e.target.value as typeof app.forecastTarget)}
          >
            <option value="TODAY">TODAY</option>
            <option value="NEXT">NEXT</option>
            <option value="CUSTOM">CUSTOM</option>
          </select>
        </div>

        {app.forecastTarget === 'CUSTOM' && (
          <div className="toolbar-group">
            <span className="toolbar-label">Date</span>
            <input
              className="toolbar-input"
              type="date"
              value={app.forecastDate}
              onChange={(e) => app.setForecastDate(e.target.value)}
            />
          </div>
        )}

        <div className="toolbar-group">
          <span className="toolbar-label">Bias</span>
          <select
            className="toolbar-select"
            value={app.forecastBias}
            onChange={(e) => app.setForecastBias(e.target.value as typeof app.forecastBias)}
          >
            <option value="AUTO">AUTO</option>
            <option value="BULLISH">BULLISH</option>
            <option value="BEARISH">BEARISH</option>
          </select>
        </div>

        <button className="ue-btn strong" onClick={() => app.executeForecast()} disabled={app.forecastLoading}>
          {app.forecastLoading ? 'Generating...' : 'Run Forecast Daily'}
        </button>

        <button
          className={`ue-btn small ${app.showEngineDebug ? 'active' : ''}`}
          onClick={() => app.setShowEngineDebug(!app.showEngineDebug)}
        >
          {app.showEngineDebug ? 'Hide Beta Tuning' : 'Beta Tuning'}
        </button>

        <button
          className="ue-btn small"
          onClick={() => app.executeBreakpointUpdate()}
          disabled={app.breakpointBusy}
          title="Detect breakpoint observations for the current forecast (runs inside the active ICT kill-zone window)."
        >
          {app.breakpointBusy ? 'Updating...' : 'Update Breakpoints'}
        </button>
      </div>

      {app.breakpointError && (
        <div className="forecast-banner error">Breakpoint update failed: {app.breakpointError}</div>
      )}
      {app.breakpointResult && (
        <div className={`forecast-banner ${app.breakpointResult.success ? 'info' : 'error'}`}>
          <strong>Breakpoints: </strong>
          {app.breakpointResult.message}
          {app.breakpointResult.breakpointLabel && (
            <> · <em>{app.breakpointResult.breakpointLabel}</em></>
          )}
          {app.breakpointResult.scenariosUpdated > 0 && (
            <> · <span>{app.breakpointResult.scenariosUpdated} scenario(s) updated</span></>
          )}
          {app.breakpointResult.forecastId !== null && (
            <> · <span className="muted">forecast #{app.breakpointResult.forecastId}</span></>
          )}
        </div>
      )}

      {app.forecastError && <div className="forecast-banner error">{app.forecastError}</div>}
      {!app.forecastResult && !app.forecastLoading && (
        <div className="workspace-empty">Forecast workspace ready. Run a forecast to populate the analysis cards.</div>
      )}
      {app.forecastLoading && <div className="workspace-empty">Building forecast analysis...</div>}

      {app.forecastResult && (
        <div className="forecast-grid">
          <section className="forecast-card hero-card">
            <div>
              <div className="card-eyebrow">Forecast Summary</div>
              <h2>
                {app.forecastResult.symbol} · {app.forecastResult.bias}
              </h2>
              <p>{app.forecastResult.summary}</p>
              <BiasStrengthMeter confidence={app.forecastResult.confidence} />
            </div>

            <div className="hero-metrics">
              <div className="metric-box">
                <span>Confidence</span>
                <strong>{formatPercent(app.forecastResult.confidence)}</strong>
              </div>
              <div className="metric-box">
                <span>Price</span>
                <strong>{formatPrice(app.forecastResult.currentPrice, app.forecastSymbol)}</strong>
              </div>
              <div className="metric-box">
                <span>Forecasted Day</span>
                <strong>{app.forecastResult.forecastedDay}</strong>
              </div>
              <div className="metric-box">
                <span>Weekly Profile</span>
                <strong>{app.forecastResult.weeklyProfile}</strong>
              </div>
              <div className="metric-box">
                <span>Target</span>
                <strong>{app.forecastResult.target}</strong>
              </div>
            </div>
          </section>

          {app.forecastResult.thesis && (
            <section className="forecast-card">
              <div
                className="card-title"
                style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}
              >
                <span>Weekly Thesis · {app.forecastResult.thesis.direction}</span>
                <span
                  style={{
                    fontSize: 'var(--fs-12)',
                    fontWeight: 600,
                    color: app.forecastResult.thesis.invalidated ? '#ef4444' : '#22c55e',
                    textTransform: 'uppercase',
                  }}
                >
                  {app.forecastResult.thesis.invalidated
                    ? 'Invalidated'
                    : `${Math.round(app.forecastResult.thesis.confidence * 100)}% intact`}
                </span>
              </div>

              {/* Defense stack: ordered lines of defense, nearest first */}
              {app.forecastResult.thesis.defenses.length > 0 && (
                <div style={{ marginTop: '8px' }}>
                  <div style={{ fontSize: 'var(--fs-11)', opacity: 0.7, marginBottom: '4px', textTransform: 'uppercase' }}>
                    Lines of Defense
                  </div>
                  {app.forecastResult.thesis.defenses.map((d, i) => {
                    const broken = d.state === 'Broken';
                    const testing = d.state === 'Testing';
                    const stateColor = broken ? '#ef4444' : testing ? '#f59e0b' : '#22c55e';
                    return (
                      <div
                        key={`${d.label}-${i}`}
                        style={{
                          display: 'flex',
                          justifyContent: 'space-between',
                          alignItems: 'center',
                          padding: '4px 0',
                          fontSize: 'var(--fs-13)',
                          opacity: broken ? 0.55 : 1,
                        }}
                      >
                        <span>
                          <span style={{ opacity: 0.6, marginRight: '6px' }}>{i + 1}.</span>
                          {d.label}
                          <span style={{ opacity: 0.55, marginLeft: '6px', fontSize: 'var(--fs-11)' }}>
                            ({d.role})
                          </span>
                        </span>
                        <span style={{ color: stateColor, fontSize: 'var(--fs-11)', fontWeight: 600, textTransform: 'uppercase' }}>
                          {d.state}
                        </span>
                      </div>
                    );
                  })}
                </div>
              )}

              {/* Narrative log: the running story, most recent last */}
              {app.forecastResult.thesis.narrative.length > 0 && (
                <div style={{ marginTop: '10px' }}>
                  <div style={{ fontSize: 'var(--fs-11)', opacity: 0.7, marginBottom: '4px', textTransform: 'uppercase' }}>
                    Story
                  </div>
                  {app.forecastResult.thesis.narrative.map((line, i) => (
                    <div
                      key={`thesis-narrative-${i}`}
                      style={{ fontSize: 'var(--fs-12)', opacity: 0.85, padding: '2px 0', lineHeight: 1.5 }}
                    >
                      {line}
                    </div>
                  ))}
                </div>
              )}
            </section>
          )}

          <section className="forecast-card">
            <div className="card-title">Weekly Profile Ranking</div>
            <div className="profile-stack">
              {app.forecastResult.weeklyProfiles.map((profile, index) => {
                const isTopProfile = index === 0;
                const profileConfidence = getConfidenceLevel(profile.score);
                return (
                  <div key={profile.name} className="profile-row">
                    <div className="profile-head">
                      <span style={{ fontWeight: isTopProfile ? 600 : 400, fontSize: isTopProfile ? '14px' : '13px' }}>
                        {profile.name}
                      </span>
                      <strong style={{ color: profileConfidence.color, fontSize: isTopProfile ? '14px' : '13px' }}>
                        {formatPercent(profile.score)}
                      </strong>
                    </div>
                    <div className="profile-bar">
                      <div style={{ width: `${profile.score}%`, backgroundColor: profileConfidence.color, opacity: isTopProfile ? 1 : 0.7 }} />
                    </div>
                  </div>
                );
              })}
            </div>
          </section>

          <section className="forecast-card">
            <div className="card-title">Key Levels</div>
            <table className="forecast-table">
              <thead>
                <tr>
                  <th>Level</th>
                  <th>Price</th>
                  <th>Status</th>
                </tr>
              </thead>
              <tbody>
            {app.forecastResult.levels.map((level) => {
              const isTaken = level.status === 'taken';
              return (
                  <tr key={level.name} style={{ opacity: isTaken ? 0.6 : 1 }}>
                    <td style={{ fontWeight: isTaken ? 400 : 500 }}>{level.name}</td>
                    <td style={{ fontFamily: 'monospace', fontSize: '13px' }}>{formatPrice(level.price, app.forecastSymbol)}</td>
                    <td>
                      <span style={{
                        display: 'inline-block',
                        padding: '2px 8px',
                        borderRadius: '4px',
                        fontSize: '11px',
                        fontWeight: 600,
                        backgroundColor: isTaken ? 'rgba(239, 68, 68, 0.2)' : 'rgba(34, 197, 94, 0.2)',
                        color: isTaken ? '#ef4444' : '#22c55e',
                        textTransform: 'uppercase'
                      }}>
                        {isTaken ? 'Taken' : 'Open'}
                      </span>
                    </td>
                  </tr>
              );
            })}
              </tbody>
            </table>
            <div className="scenario-card-actions" style={{ display: 'flex', gap: '6px', marginTop: '12px' }}>
              <button className="ue-btn small" onClick={() => app.requestForecastLevelsDraw()}>
                Draw Key Levels
              </button>
            </div>
          </section>

          <section className="forecast-card scenario-card full-span">
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '10px' }}>
              <div className="card-title" style={{ margin: 0 }}>Logic Scenarios</div>
              <div style={{ display: 'flex', gap: '5px', flexWrap: 'wrap' }}>
                {app.forecastResult.vpdaScenarios[0] && (
                  <button
                    className="ue-btn small"
                    title="Draw primary scenario on chart"
                    onClick={() => app.requestForecastScenarioDraw(app.forecastResult!.vpdaScenarios[0])}
                  >
                    Draw Primary
                  </button>
                )}
                {app.forecastResult.vpdaScenarios[1] && (
                  <button
                    className="ue-btn small"
                    title="Draw secondary scenario on chart"
                    onClick={() => app.requestForecastScenarioDraw(app.forecastResult!.vpdaScenarios[1])}
                  >
                    Draw Secondary
                  </button>
                )}
                <button className="ue-btn small" onClick={() => app.clearForecastDrawings()} title="Clear all forecast drawings">
                  Clear
                </button>
              </div>
            </div>
            {app.forecastResult.vpdaScenarios.map((scenario, i) => (
              <ScenarioCard
                app={app}
                key={scenario.name}
                scenario={scenario}
                symbol={app.forecastSymbol}
                rank={i}
              />
            ))}
          </section>

          {app.showEngineDebug && app.forecastResult.searchScenarios.length > 0 && (
            <section className="forecast-card scenario-card full-span">
              <div className="card-title">Beta Tuning — Beam Search</div>
              {app.forecastResult.searchScenarios.map((scenario, i) => (
                <ScenarioCard
                  app={app}
                  key={`${scenario.name}-${i}`}
                  scenario={scenario}
                  symbol={app.forecastSymbol}
                  rank={i}
                />
              ))}
            </section>
          )}

          {app.forecastResult.vmScenarios.length > 0 && (
            <section className="forecast-card scenario-card full-span">
              <div className="card-title">VM Pattern Match Scenarios</div>
              {app.forecastResult.vmScenarios.map((scenario, i) => (
                <ScenarioCard
                  app={app}
                  key={scenario.name}
                  scenario={scenario}
                  symbol={app.forecastSymbol}
                  rank={i}
                />
              ))}
            </section>
          )}

          {app.forecastResult.warnings.length > 0 && (
            <section className="forecast-card">
              <div className="card-title">Warnings</div>
              {renderSection({ title: 'Warnings', lines: app.forecastResult.warnings })}
            </section>
          )}

          {app.forecastResult.context.length > 0 && (
            <section className="forecast-card">
              <div className="card-title">Context</div>
              {renderSection({ title: 'Context', lines: app.forecastResult.context })}
            </section>
          )}

          {app.forecastResult.sections
            .filter((section) => {
              if (!app.showEngineDebug) {
                return section.title !== 'SEARCH TAKEOVER' && section.title !== 'SEARCH ENGINE PREVIEW';
              }
              return true;
            })
            .map((section) => (
            <section key={section.title} className="forecast-card full-span">
              <div className="card-title">{section.title === 'SEARCH ENGINE PREVIEW' ? 'Beta Tuning' : section.title}</div>
              {renderSection(section)}
            </section>
          ))}
        </div>
      )}
    </div>
  );
}
