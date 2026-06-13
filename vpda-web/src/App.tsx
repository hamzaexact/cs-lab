import { useEffect, useMemo } from 'react';
import { useVpdaApp } from './data/useVpdaApp';
import './styles/layout.css';
import MenuBar from './components/MenuBar';
import MarketWorkspace from './components/MarketWorkspace';
import ForecastWorkspace from './components/ForecastWorkspace';
import SessionForecastWorkspace from './components/SessionForecastWorkspace';
import ShellWorkspace from './components/ShellWorkspace';
import InventoryWorkspace from './components/InventoryWorkspace';
import VChartWorkspace from './components/VChartWorkspace';
import AccountModal from './components/AccountModal';
import ActivationGate from './components/ActivationGate';
import DbLogsModal from './components/DbLogsModal';
import ModalHost from './components/ModalHost';
import SettingsModal from './components/SettingsModal';
import { formatPrice } from './utils/format';

export default function App() {
  const app = useVpdaApp();

  const titlePrice = useMemo(() => {
    if (!app.market) return 'Loading';
    return formatPrice(app.market.last, app.symbol);
  }, [app.market, app.symbol]);

  useEffect(() => {
    document.title = `${app.symbol} · ${titlePrice}`;
  }, [app.symbol, titlePrice]);

  return (
    <div className="app-root">
      <MenuBar app={app} />
      <div className="workspace-root">
        {/* MarketWorkspace stays always mounted so ChartViewport never re-mounts on workspace switch */}
        <div style={{ display: app.workspace !== 'market' ? 'none' : undefined, width: '100%', height: '100%' }}>
          <MarketWorkspace app={app} />
        </div>
        {app.workspace === 'forecast' && <ForecastWorkspace app={app} />}
        {app.workspace === 'sessionForecast' && <SessionForecastWorkspace app={app} />}
        {app.workspace === 'shell' && <ShellWorkspace app={app} />}
        {app.workspace === 'inventory' && <InventoryWorkspace app={app} />}
        {app.workspace === 'vchart' && <VChartWorkspace app={app} />}
      </div>
      <AccountModal app={app} />
      <DbLogsModal app={app} />
      <SettingsModal app={app} />
      <ModalHost app={app} />
      <ActivationGate app={app} />
      
      {/* Toast Notification */}
      {app.toastMessage && (
        <div style={{
          position: 'fixed',
          bottom: 32,
          left: '50%',
          transform: 'translateX(-50%)',
          background: 'var(--bg1)',
          color: 'var(--text)',
          padding: '8px 16px',
          borderRadius: '4px',
          boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
          border: '1px solid var(--border2)',
          zIndex: 9999,
          fontSize: 'var(--fs-10)',
          fontWeight: 500,
          animation: 'toastFadeIn 0.15s ease-out',
        }}>
          {app.toastMessage.text}
        </div>
      )}
    </div>
  );
}
