import type { VpdaAppState } from '../data/useVpdaApp';
import ChartViewport from './ChartViewport';
import ToolRail from './ToolRail';
import { formatPrice } from '../utils/format';

type Props = {
  app: VpdaAppState;
};

export default function MarketWorkspace({ app }: Props) {
  return (
    <div className="workspace-page">
      <div
        className="market-layout"
        style={{
          gridTemplateColumns: app.showWatchlist ? '188px minmax(0, 1fr)' : 'minmax(0, 1fr)',
        }}
      >
        {app.showWatchlist && (
          <aside className="watchlist-panel">
            <div className="panel-title">Watchlist</div>
            <div className="watchlist-scroll">
              {app.watchlist.map((item) => (
                <button
                  key={item.symbol}
                  className={`watch-item ${item.symbol === app.symbol ? 'active' : ''}`}
                  onClick={() => app.setSymbol(item.symbol)}
                >
                  <div>
                    <div className="watch-symbol">{item.symbol}</div>
                    <div className="watch-label">{item.description}</div>
                  </div>
                  <div className="watch-values">
                    <div>{formatPrice(item.price, item.symbol)}</div>
                    <div className={item.changePct >= 0 ? 'up' : 'down'}>
                      {item.changePct >= 0 ? '+' : ''}
                      {item.changePct.toFixed(2)}%
                    </div>
                  </div>
                </button>
              ))}
            </div>
          </aside>
        )}

        <section className="market-main">
          <div
            className="market-chart-area"
            style={{
              gridTemplateColumns: `50px minmax(0, 1fr) ${app.showInspector ? '176px' : ''} ${app.managerOpen ? '260px' : ''}`.trim(),
              gridTemplateRows: '1fr',
            }}
          >
            <ToolRail app={app} />
            <ChartViewport app={app} />

            {app.showInspector && (
              <aside className="inspector-panel">
                <div className="panel-title">Inspector</div>
                <div className="inspector-grid">
                  <div className="inspector-row">
                    <span>Last</span>
                    <strong>{app.market ? formatPrice(app.market.last, app.symbol) : '--'}</strong>
                  </div>
                  <div className="inspector-row">
                    <span>24H Vol</span>
                    <strong>{app.market ? app.market.volume24h.toLocaleString('en-US') : '--'}</strong>
                  </div>
                  <div className="inspector-row">
                    <span>Crosshair</span>
                    <strong>{app.showCrosshair ? 'Enabled' : 'Hidden'}</strong>
                  </div>
                  <div className="inspector-row">
                    <span>Heatmap</span>
                    <strong>{app.showHeatmap ? 'Enabled' : 'Hidden'}</strong>
                  </div>
                  <div className="inspector-row">
                    <span>Magnet</span>
                    <strong>{app.showMagnet ? 'Enabled' : 'Hidden'}</strong>
                  </div>
                  <div className="inspector-row">
                    <span>Tool</span>
                    <strong>{app.drawTool.toUpperCase()}</strong>
                  </div>
                  <div className="inspector-row">
                    <span>Status</span>
                    <strong>{app.loadingMarket ? 'Loading' : 'Ready'}</strong>
                  </div>
                </div>
              </aside>
            )}

            {app.managerOpen && (
              <div id="manager-slot" style={{ minHeight: 0, overflow: 'hidden', display: 'flex', flexDirection: 'column' }} />
            )}
          </div>

          <div className="status-strip">
            <span>{app.symbol} · {app.timeframe}</span>
            <div style={{ flex: 1 }} />
            <span>
              Forecast Bridge: {app.forecastDrawingDebug.status.toUpperCase()} · {app.forecastDrawingDebug.message}
            </span>
            <div style={{ width: 12 }} />
            <span>{app.loadingMarket ? 'Refreshing chart...' : 'Ready.'}</span>
          </div>
        </section>
      </div>
    </div>
  );
}
