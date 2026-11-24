import JsBarcode from 'jsbarcode';

// Build the bars on the thermal head's dot grid, including quiet zones.
export function prepareThermalBarcodes(sheet: HTMLElement, clone: HTMLElement, dpi: number) {
  const selector = 'svg[data-barcode-value]';
  const clonedCodes = clone.querySelectorAll<SVGSVGElement>(selector);
  const symbols: { x: number; y: number; width: number; height: number; dots: number; bits: string }[] = [];
  const origin = sheet.getBoundingClientRect();
  const sheetTranslation = new DOMMatrixReadOnly(getComputedStyle(sheet).transform === 'none' ? undefined : getComputedStyle(sheet).transform);
  const scale = dpi / 96;
  sheet.querySelectorAll<SVGSVGElement>(selector).forEach((source, index) => {
    const value = source.dataset.barcodeValue!;
    const bounds = source.getBoundingClientRect();
    let { left, right, top, bottom } = bounds;
    for (let parent = source.parentElement; parent && parent !== sheet; parent = parent.parentElement) {
      const style = getComputedStyle(parent);
      // A rotated barcode already has its transform in the captured SVG.
      if (style.transform !== 'none') {
        const matrix = new DOMMatrixReadOnly(style.transform);
        if (matrix.a !== 1 || matrix.d !== 1 || matrix.b !== 0 || matrix.c !== 0) return;
      }
      const box = parent.getBoundingClientRect();
      if (style.overflowX === 'hidden') { left = Math.max(left, box.left); right = Math.min(right, box.right); }
      if (style.overflowY === 'hidden') { top = Math.max(top, box.top); bottom = Math.min(bottom, box.bottom); }
    }
    const originX = origin.left - sheetTranslation.e;
    const originY = origin.top - sheetTranslation.f;
    const x = Math.round((left - originX) * scale);
    const y = Math.round((top - originY) * scale);
    const width = Math.round((right - originX) * scale) - x;
    const height = Math.round((bottom - originY) * scale) - y;
    const format = source.dataset.barcodeFormat!;
    const encode = (format: string) => {
      const output: { encodings?: { data: string }[] } = {};
      JsBarcode(output, value, { format, flat: true, displayValue: false });
      return output.encodings!.map(part => part.data).join('');
    };
    let bits: string;
    try {
      bits = encode(format);
    } catch {
      bits = encode('CODE128');
    }
    const quiet = 11;
    const moduleSize = (data: string) => Math.floor(width / (data.length + 2 * quiet));
    if (format === 'CODE39' && moduleSize(bits) < 2) bits = encode('CODE128');
    const dots = moduleSize(bits);
    if (dots < 1 || height < 1) throw new Error(`พื้นที่บาร์โค้ดแคบเกินไปสำหรับ ${value} กรุณาขยายช่องบาร์โค้ด`);
    // Keep the SVG's layout box, but only paint the dot-aligned bars once.
    clonedCodes[index].style.visibility = 'hidden';
    symbols.push({ x, y, width, height, dots, bits });
  });
  return (ctx: CanvasRenderingContext2D) => {
    for (const { x, y, width, height, dots, bits } of symbols) {
      ctx.fillStyle = 'white';
      ctx.fillRect(x, y, width, height);
      ctx.fillStyle = 'black';
      const start = x + Math.floor((width - bits.length * dots) / 2);
      for (const run of bits.matchAll(/1+/g)) {
        ctx.fillRect(start + run.index! * dots, y, run[0].length * dots, height);
      }
    }
  };
}
