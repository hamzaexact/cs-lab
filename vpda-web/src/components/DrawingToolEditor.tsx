import React, { useState } from 'react';
import type { Drawing, StrokeStyle } from './ChartViewport';
import ColorControl from './ColorControl';

type Props = {
  drawing: Drawing;
  onClose: () => void;
  onUpdate: (updates: Partial<Drawing>) => void;
  onSaveTemplate: (name: string) => void;
  onApplyTemplate: (name: string) => void;
  templates: string[];
};

export default function DrawingToolEditor({ drawing, onClose, onUpdate, onSaveTemplate, onApplyTemplate, templates }: Props) {
  const [templateName, setTemplateName] = useState('');

  return (
    <div className="object-editor" style={{ width: 280, position: 'absolute', top: 50, left: 50, zIndex: 1000 }}>
      <div className="object-editor-header" onPointerDown={(e) => e.stopPropagation()}>
        <span className="object-editor-title">{drawing.type.toUpperCase()} SETTINGS</span>
        <button className="object-editor-close" onClick={onClose}>×</button>
      </div>
      
      <div className="object-editor-body" onPointerDown={(e) => e.stopPropagation()}>
        <label className="object-editor-field">
          <span>Apply Template</span>
          <select onChange={(e) => onApplyTemplate(e.target.value)} value="">
            <option value="" disabled>Presets...</option>
            {templates.map(t => <option key={t} value={t}>{t}</option>)}
          </select>
        </label>

        <div className="object-editor-row-sep" />

        <label className="object-editor-field">
          <span>Save As Template</span>
          <div style={{ display: 'flex', gap: 4 }}>
            <input 
              type="text" 
              placeholder="Name..."
              value={templateName} 
              onChange={(e) => setTemplateName(e.target.value)} 
            />
            <button className="ue-btn small" onClick={() => { onSaveTemplate(templateName); setTemplateName(''); }}>Save</button>
          </div>
        </label>

        <div className="object-editor-row-sep" />

        <label className="object-editor-field">
          <span>Color</span>
          <ColorControl value={drawing.color} onChange={(c) => onUpdate({ color: c })} />
        </label>

        {'width' in drawing && (
          <label className="object-editor-field">
            <span>Width</span>
            <input type="number" min={0.5} max={10} step={0.5} value={drawing.width} onChange={(e) => onUpdate({ width: Number(e.target.value) })} />
          </label>
        )}

        {'strokeStyle' in drawing && (
          <label className="object-editor-field">
            <span>Style</span>
            <select value={drawing.strokeStyle} onChange={(e) => onUpdate({ strokeStyle: e.target.value as StrokeStyle })}>
              <option value="solid">Solid</option>
              <option value="dashed">Dashed</option>
              <option value="dotted">Dotted</option>
            </select>
          </label>
        )}

        {'labelFontSize' in drawing && (
          <label className="object-editor-field">
            <span>Text Size</span>
            <input type="number" min={8} max={32} value={drawing.labelFontSize} onChange={(e) => onUpdate({ labelFontSize: Number(e.target.value) })} />
          </label>
        )}
      </div>
    </div>
  );
}
