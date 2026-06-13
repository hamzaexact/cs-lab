import { toBlob } from 'html-to-image';

function nextFrame() {
  return new Promise<void>((resolve) => {
    window.requestAnimationFrame(() => resolve());
  });
}

function downloadBlob(blob: Blob, fileName: string) {
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  link.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}

export async function captureAppLayout(fileName: string) {
  const root = document.querySelector('.app-root');
  if (!(root instanceof HTMLElement)) {
    throw new Error('App root not found');
  }

  await nextFrame();

  const rect = root.getBoundingClientRect();
  const width = Math.max(
    1,
    Math.ceil(rect.width),
    root.scrollWidth,
    document.documentElement.clientWidth,
  );
  const height = Math.max(
    1,
    Math.ceil(rect.height),
    root.scrollHeight,
    document.documentElement.clientHeight,
  );
  const backgroundColor = getComputedStyle(root).backgroundColor || '#0d0d0f';

  const blob = await toBlob(root, {
    cacheBust: true,
    pixelRatio: Math.max(1, Math.min(2, window.devicePixelRatio || 1)),
    canvasWidth: width,
    canvasHeight: height,
    backgroundColor,
    skipFonts: false,
    style: {
      width: `${width}px`,
      height: `${height}px`,
    },
    filter: (node) => {
      if (!(node instanceof HTMLElement)) return true;
      return node.tagName !== 'SCRIPT';
    },
  });

  if (!blob) {
    throw new Error('Failed to create screenshot image');
  }

  downloadBlob(blob, fileName);
}