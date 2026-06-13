import type { VpdaAppState } from '../data/useVpdaApp';

type Props = {
  app: VpdaAppState;
};

function ModalCard({
  title,
  body,
  tone,
  onClose,
}: {
  title: string;
  body: string;
  tone: 'info' | 'warning' | 'error';
  onClose: () => void;
}) {
  return (
    <div className="modal-overlay">
      <div className={`modal-card ${tone}`}>
        <div className="modal-title-row">
          <h3>{title}</h3>
        </div>
        <p>{body}</p>
        <div className="modal-actions">
          <button className="ue-btn strong" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

export default function ModalHost({ app }: Props) {
  return (
    <>
      {app.startup?.warning && (
        <ModalCard
          title={app.startup.warning.title}
          body={app.startup.warning.body}
          tone={app.startup.warning.tone}
          onClose={app.dismissStartupWarning}
        />
      )}
      {/* Activation is handled by ActivationGate (full-screen, non-dismissable) */}
    </>
  );
}
