import { useEffect, useMemo, useRef, useState } from 'react';
import { HexColorPicker } from 'react-colorful';

type Props = {
  value: string;
  onChange: (value: string) => void;
  ariaLabel?: string;
  className?: string;
};

function normalizeHexColor(value: string): string {
  if (/^#[0-9a-fA-F]{6}$/.test(value)) return value.toUpperCase();
  if (/^#[0-9a-fA-F]{8}$/.test(value)) return value.slice(0, 7).toUpperCase();
  return '#888888';
}

export default function ColorControl({
  value,
  onChange,
  ariaLabel = 'Color control',
  className = '',
}: Props) {
  const normalized = useMemo(() => normalizeHexColor(value), [value]);
  const [open, setOpen] = useState(false);
  const [draftHex, setDraftHex] = useState(normalized);
  const rootRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    setDraftHex(normalized);
  }, [normalized]);

  useEffect(() => {
    if (!open) return;

    const onPointerDown = (event: MouseEvent | PointerEvent) => {
      const target = event.target as Node | null;
      if (!target) return;
      if (rootRef.current?.contains(target)) return;
      setOpen(false);
    };

    document.addEventListener('pointerdown', onPointerDown);
    return () => document.removeEventListener('pointerdown', onPointerDown);
  }, [open]);

  function commitHex(raw: string) {
    const next = raw.trim().toUpperCase();
    setDraftHex(next);
    if (/^#[0-9A-F]{6}$/.test(next)) {
      onChange(next);
    }
  }

  return (
    <div
      ref={rootRef}
      className={`color-control ${className}`.trim()}
      style={{
        position: 'relative',
        display: 'flex',
        alignItems: 'center',
        gap: '8px',
        minWidth: 0,
        width: '100%',
      }}
    >
      <button
        type="button"
        aria-label={ariaLabel}
        title={normalized}
        onClick={() => setOpen((prev) => !prev)}
        style={{
          width: '38px',
          height: '22px',
          flex: '0 0 38px',
          border: '1px solid var(--border2)',
          background: 'var(--input-bg)',
          padding: '2px',
          cursor: 'pointer',
        }}
      >
        <span
          style={{
            display: 'block',
            width: '100%',
            height: '100%',
            background: normalized,
            border: '1px solid rgba(0, 0, 0, 0.22)',
          }}
        />
      </button>

      <input
        type="text"
        value={draftHex}
        inputMode="text"
        spellCheck={false}
        maxLength={7}
        aria-label={`${ariaLabel} hex value`}
        onChange={(e) => setDraftHex(e.currentTarget.value.toUpperCase())}
        onBlur={(e) => commitHex(e.currentTarget.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            commitHex((e.currentTarget as HTMLInputElement).value);
            setOpen(false);
          }
          if (e.key === 'Escape') {
            setDraftHex(normalized);
            setOpen(false);
          }
        }}
        style={{
          flex: 1,
          minWidth: 0,
          height: '22px',
          padding: '0 6px',
          background: 'var(--input-bg)',
          color: 'var(--text)',
          border: '1px solid var(--border2)',
          fontFamily: 'var(--mono)',
          fontSize: 'var(--fs-10, var(--vpda-font-size))',
          outline: 'none',
          textTransform: 'uppercase',
          letterSpacing: '0.04em',
        }}
      />

      {open && (
        <div
          style={{
            position: 'absolute',
            top: 'calc(100% + 6px)',
            left: 0,
            zIndex: 2000,
            width: '220px',
            border: '1px solid var(--border2)',
            background: 'var(--float-bg)',
            backdropFilter: 'blur(6px)',
            WebkitBackdropFilter: 'blur(6px)',
            padding: '10px',
            display: 'flex',
            flexDirection: 'column',
            gap: '8px',
            boxShadow: '0 10px 30px rgba(0,0,0,0.28)',
          }}
        >
          <HexColorPicker
            color={normalized}
            onChange={(next) => {
              const upper = next.toUpperCase();
              setDraftHex(upper);
              onChange(upper);
            }}
            style={{
              width: '100%',
              height: '150px',
            }}
          />

          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '8px',
            }}
          >
            <div
              style={{
                width: '24px',
                height: '24px',
                flex: '0 0 24px',
                border: '1px solid var(--border2)',
                background: normalized,
              }}
            />
            <input
              type="text"
              value={draftHex}
              inputMode="text"
              spellCheck={false}
              maxLength={7}
              aria-label={`${ariaLabel} hex editor`}
              onChange={(e) => setDraftHex(e.currentTarget.value.toUpperCase())}
              onBlur={(e) => commitHex(e.currentTarget.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') {
                  commitHex((e.currentTarget as HTMLInputElement).value);
                }
              }}
              style={{
                flex: 1,
                minWidth: 0,
                height: '24px',
                padding: '0 6px',
                background: 'var(--input-bg)',
                color: 'var(--text)',
                border: '1px solid var(--border2)',
                fontFamily: 'var(--mono)',
                fontSize: 'var(--fs-10, var(--vpda-font-size))',
                outline: 'none',
                textTransform: 'uppercase',
                letterSpacing: '0.04em',
              }}
            />
          </div>
        </div>
      )}
    </div>
  );
}