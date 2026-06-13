import type { VpdaAppState } from '../data/useVpdaApp';

type Props = {
  app: VpdaAppState;
};

export default function ShellWorkspace({ app }: Props) {
  return (
    <div className="workspace-page shell-page">
      <div className="shell-header">
        <div>
          <div className="card-eyebrow">Command Workspace</div>
          <h2>Shell</h2>
        </div>
        <div className="shell-status">{app.shellBusy ? 'RUNNING' : 'READY'}</div>
      </div>

      <div className="shell-console">
        {app.shellLines.map((line, index) => (
          <div key={`${index}-${line.text}`} className={`shell-line ${line.level}`}>
            <span>{line.text}</span>
          </div>
        ))}
      </div>

      <div className="shell-input-row">
        <input
          className="shell-input"
          value={app.shellInput}
          onChange={(e) => app.setShellInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              app.executeShell();
            }
          }}
          placeholder="FORECAST DAILY EURUSD"
        />
        <button className="ue-btn strong" onClick={() => app.executeShell()} disabled={app.shellBusy}>
          {app.shellBusy ? 'Running...' : 'Run'}
        </button>
      </div>
    </div>
  );
}
