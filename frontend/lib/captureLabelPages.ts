import { BarcodeResolutionError, prepareThermalBarcodes } from '@/lib/thermalBarcode';

async function toDataUrl(src: string): Promise<string> {
  if (!src || src.startsWith('data:')) return src;
  let res: Response;
  try {
    res = await fetch(src);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
  } catch {
    // External URLs (e.g. AdaFileServer) may block CORS in browser fetch; fall back to local backend proxy.
    if (/^https?:\/\//i.test(src)) {
      res = await fetch(`/api/image-proxy?url=${encodeURIComponent(src)}`);
      if (!res.ok) throw new Error(`Proxy HTTP ${res.status}`);
    } else {
      throw new Error(`Failed to load ${src}`);
    }
  }
  const blob = await res.blob();
  return await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => resolve(String(reader.result || ''));
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}

export async function capturePagesToPngBase64(
  containerId: string,
  paperWidthMm = 210,
  paperHeightMm = 297,
  godexRoll = false
): Promise<string[]> {
  const container = document.getElementById(containerId);
  if (!container) return [];
  const sheetEls = Array.from(container.querySelectorAll<HTMLElement>('[data-capture-sheet="1"]'));
  const wPx = Math.max(40, Math.round((paperWidthMm / 25.4) * 96));
  const hPx = Math.max(40, Math.round((paperHeightMm / 25.4) * 96));
  const results: string[] = [];
  for (const el of sheetEls) {
    const invalidCode = el.querySelector<HTMLElement>('[data-print-code-error]');
    if (invalidCode) throw new Error(invalidCode.dataset.printCodeError);
    const clone = el.cloneNode(true) as HTMLElement;
    clone.setAttribute('xmlns', 'http://www.w3.org/1999/xhtml');
    const imgs = Array.from(clone.querySelectorAll('img'));
    await Promise.all(
      imgs.map(async (img) => {
        const rawSrc = img.getAttribute('src') || '';
        if (rawSrc && !rawSrc.startsWith('data:')) {
          try {
            img.setAttribute('src', await toDataUrl(rawSrc));
          } catch {
            // ignore broken img
          }
        }
      })
    );
    let dpi = godexRoll ? 203 : 192;
    let paintBarcodes: ReturnType<typeof prepareThermalBarcodes>;
    try {
      paintBarcodes = prepareThermalBarcodes(el, clone, dpi);
    } catch (error) {
      if (godexRoll || !(error instanceof BarcodeResolutionError)) throw error;
      dpi = 384;
      paintBarcodes = prepareThermalBarcodes(el, clone, dpi);
    }
    const viewportWidth = godexRoll ? paperWidthMm * 96 / 25.4 : wPx;
    const viewportHeight = godexRoll ? paperHeightMm * 96 / 25.4 : hPx;
    const outputWidth = Math.round(viewportWidth * dpi / 96);
    const outputHeight = Math.round(viewportHeight * dpi / 96);
    const serialized = new XMLSerializer().serializeToString(clone);
    const svgXml = `<svg xmlns="http://www.w3.org/2000/svg" width="${outputWidth}" height="${outputHeight}" viewBox="0 0 ${viewportWidth} ${viewportHeight}" preserveAspectRatio="none"><foreignObject x="0" y="0" width="100%" height="100%">${serialized}</foreignObject></svg>`;
    const svgUrl = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svgXml);
    const pngDataUrl = await new Promise<string>((resolve, reject) => {
      const image = new Image();
      image.onload = () => {
        const canvas = document.createElement('canvas');
        canvas.width = outputWidth;
        canvas.height = outputHeight;
        const ctx = canvas.getContext('2d');
        if (!ctx) {
          reject(new Error('Canvas context unavailable'));
          return;
        }
        ctx.fillStyle = '#FFFFFF';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        try {
          ctx.drawImage(image, 0, 0);
          if (godexRoll) {
            const pixels = ctx.getImageData(0, 0, outputWidth, outputHeight);
            const data = pixels.data;
            for (let i = 0; i < data.length; i += 4) {
              const luminance = 0.299 * data[i] + 0.587 * data[i + 1] + 0.114 * data[i + 2];
              const tone = luminance < 160 ? 0 : 255;
              data[i] = data[i + 1] = data[i + 2] = tone;
            }
            ctx.putImageData(pixels, 0, 0);
          }
          paintBarcodes(ctx);
          resolve(canvas.toDataURL('image/png'));
        } catch (error) { reject(error); }
      };
      image.onerror = reject;
      image.src = svgUrl;
    });
    results.push(pngDataUrl);
  }
  return results;
}
