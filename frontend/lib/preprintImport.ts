/// <reference types="vite/client" />
import * as pdfjsLib from 'pdfjs-dist';
import pdfWorkerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url';
import { PAPER_SIZE_PRESETS, type PaperSizeRecord } from '@/types/design';

if (typeof window !== 'undefined' && pdfjsLib.GlobalWorkerOptions) {
  pdfjsLib.GlobalWorkerOptions.workerSrc = pdfWorkerUrl;
}

export interface ParsedPreprintPage {
  pageNumber: number;
  dataUrl: string;
  pixelWidth: number;
  pixelHeight: number;
  aspectRatio: number;
  mmWidth?: number;
  mmHeight?: number;
  detectedPaperSize?: string;
}

export interface ParsedPreprintResult {
  fileName: string;
  isPdf: boolean;
  numPages: number;
  pages: ParsedPreprintPage[];
}

export interface InferredPreprintGrid {
  labelW: number;
  labelH: number;
  cols: number;
  rows: number;
  magTop: number;
  magBottom: number;
  magLeft: number;
  magRight: number;
  gapH: number;
  gapV: number;
}

export function detectPaperPresetFromDimensions(
  mmWidth: number,
  mmHeight: number,
  dbPaperSizes: PaperSizeRecord[] = []
): string {
  const candidates: Array<{ code: string; w: number; h: number }> = [];
  for (const p of dbPaperSizes) {
    if (p.FTPpsStaCustom !== '1' && p.FTPpsCode !== 'ROLL' && p.FTPpsCode !== 'CUSTOM') {
      candidates.push({ code: p.FTPpsCode, w: p.FCPpsWidth, h: p.FCPpsHeight });
    }
  }
  for (const [code, preset] of Object.entries(PAPER_SIZE_PRESETS)) {
    if (!preset.isCustom && !candidates.some((c) => c.code === code)) {
      candidates.push({ code, w: preset.widthMm, h: preset.heightMm });
    }
  }

  for (const c of candidates) {
    if (Math.abs(mmWidth - c.w) <= 3 && Math.abs(mmHeight - c.h) <= 3) {
      return c.code;
    }
  }
  return 'CUSTOM';
}

export function detectPaperCodeFromFilename(fileName: string): string | null {
  const name = fileName || '';
  const sizeTag = name.match(/size[-_\s]?(a3|a4|a5|a6|b5|letter|legal|f4)/i)?.[1]?.toLowerCase();
  const token =
    sizeTag ||
    name.match(/\b(a3|a4|a5|a6|b5|letter|legal|f4)\b/i)?.[1]?.toLowerCase();
  switch (token) {
    case 'a3':
      return 'A3';
    case 'a4':
      return 'A4';
    case 'a5':
      return 'A5';
    case 'a6':
      return 'A6';
    case 'b5':
      return 'B5_JIS';
    case 'letter':
      return 'LETTER';
    case 'legal':
    case 'f4':
      return 'LEGAL';
    default:
      return null;
  }
}

export function inferPreprintGridFromFile(params: {
  fileName: string;
  paperWidthMm: number;
  paperHeightMm: number;
}): InferredPreprintGrid | null {
  const { fileName, paperWidthMm, paperHeightMm } = params;
  if (!fileName || paperWidthMm < 20 || paperHeightMm < 20) return null;

  const dimMatches = Array.from(fileName.matchAll(/(\d+(?:\.\d+)?)\s*[x×]\s*(\d+(?:\.\d+)?)/gi));
  let labelW = 0;
  let labelH = 0;
  for (const m of dimMatches) {
    const w = parseFloat(m[1]);
    const h = parseFloat(m[2]);
    if (!(w >= 10 && h >= 10)) continue;
    // ข้ามถ้าตรงกับขนาดแผ่นกระดาษ
    if (Math.abs(w - paperWidthMm) <= 2 && Math.abs(h - paperHeightMm) <= 2) continue;
    if (w <= paperWidthMm + 1 && h <= paperHeightMm + 1) {
      labelW = w;
      labelH = h;
      break;
    }
  }
  if (!labelW || !labelH) return null;

  const upMatch = fileName.match(/(?:^|[-_\s])(\d+)\s*up\b/i);
  const upHint = upMatch ? parseInt(upMatch[1], 10) : 0;

  const maxCols = Math.max(1, Math.floor((paperWidthMm + 0.2) / labelW));
  const maxRows = Math.max(1, Math.floor((paperHeightMm + 0.2) / labelH));

  let cols = maxCols;
  let rows = maxRows;
  if (upHint > 0) {
    for (let c = 1; c <= maxCols; c++) {
      if (upHint % c === 0) {
        const r = upHint / c;
        if (r >= 1 && r <= maxRows) {
          cols = c;
          rows = r;
          break;
        }
      }
    }
  }

  const magLeft = Math.max(0, Number(((paperWidthMm - cols * labelW) / 2).toFixed(1)));
  const magRight = magLeft;
  const magTop = Math.max(0, Number(((paperHeightMm - rows * labelH) / 2).toFixed(1)));
  const magBottom = magTop;

  return {
    labelW,
    labelH,
    cols,
    rows,
    magTop,
    magBottom,
    magLeft,
    magRight,
    gapH: 0,
    gapV: 0,
  };
}

export interface CropBoxPct {
  xPct: number;
  yPct: number;
  wPct: number;
  hPct: number;
}

function clampPct(v: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, v));
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = () => reject(new Error('ไม่สามารถโหลดภาพได้'));
    img.src = src;
  });
}

async function compressCanvasToJpeg(
  canvas: HTMLCanvasElement,
  maxWidth = 1480,
  quality = 0.84
): Promise<{ dataUrl: string; width: number; height: number }> {
  const scale = canvas.width > maxWidth ? maxWidth / canvas.width : 1;
  if (scale >= 1) {
    return {
      dataUrl: canvas.toDataURL('image/jpeg', quality),
      width: canvas.width,
      height: canvas.height,
    };
  }
  const dst = document.createElement('canvas');
  dst.width = Math.max(1, Math.round(canvas.width * scale));
  dst.height = Math.max(1, Math.round(canvas.height * scale));
  const ctx = dst.getContext('2d');
  if (!ctx) {
    return {
      dataUrl: canvas.toDataURL('image/jpeg', quality),
      width: canvas.width,
      height: canvas.height,
    };
  }
  ctx.fillStyle = '#FFFFFF';
  ctx.fillRect(0, 0, dst.width, dst.height);
  ctx.drawImage(canvas, 0, 0, dst.width, dst.height);
  return {
    dataUrl: dst.toDataURL('image/jpeg', quality),
    width: dst.width,
    height: dst.height,
  };
}

/**
 * อ่านไฟล์ Pre-Print (.pdf หรือไฟล์รูปภาพ) แล้วแปลงหน้าเอกสารเป็น DataURL สำหรับใช้ทาบใน Design Studio และ Preview
 */
export async function parsePreprintFile(file: File, maxPdfPages = 6): Promise<ParsedPreprintResult> {
  const lower = file.name.toLowerCase();
  const isPdf = file.type === 'application/pdf' || lower.endsWith('.pdf');

  if (!isPdf) {
    const rawUrl = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result || ''));
      reader.onerror = () => reject(new Error('ไม่สามารถอ่านไฟล์รูปภาพได้'));
      reader.readAsDataURL(file);
    });
    const img = await loadImage(rawUrl);
    const canvas = document.createElement('canvas');
    canvas.width = img.naturalWidth || img.width || 1200;
    canvas.height = img.naturalHeight || img.height || 1600;
    const ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.fillStyle = '#FFFFFF';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      ctx.drawImage(img, 0, 0, canvas.width, canvas.height);
    }
    const compressed = await compressCanvasToJpeg(canvas, 1480, 0.86);
    return {
      fileName: file.name,
      isPdf: false,
      numPages: 1,
      pages: [
        {
          pageNumber: 1,
          dataUrl: compressed.dataUrl,
          pixelWidth: compressed.width,
          pixelHeight: compressed.height,
          aspectRatio: compressed.width / Math.max(1, compressed.height),
        },
      ],
    };
  }

  const baseOrigin = typeof window !== 'undefined' ? window.location.origin : '';
  const pdfAssetUrl = (sub: string) =>
    baseOrigin ? `${baseOrigin}/pdfjs/${sub}/` : `/pdfjs/${sub}/`;

  const buf = await file.arrayBuffer();
  const loadingTask = pdfjsLib.getDocument({
    data: new Uint8Array(buf),
    wasmUrl: pdfAssetUrl('wasm'),
    iccUrl: pdfAssetUrl('iccs'),
    cMapUrl: pdfAssetUrl('cmaps'),
    cMapPacked: true,
    standardFontDataUrl: pdfAssetUrl('standard_fonts'),
  });
  const pdf = await loadingTask.promise;
  const pageCount = Math.min(pdf.numPages, maxPdfPages);
  const pages: ParsedPreprintPage[] = [];

  for (let p = 1; p <= pageCount; p++) {
    const page = await pdf.getPage(p);
    const baseVp = page.getViewport({ scale: 1 });
    const mmWidth = Number(((baseVp.width * 25.4) / 72).toFixed(1));
    const mmHeight = Number(((baseVp.height * 25.4) / 72).toFixed(1));
    const detectedPaperSize = detectPaperPresetFromDimensions(mmWidth, mmHeight);
    const desiredWidth = 1480;
    const renderScale = Math.max(0.4, Math.min(3.5, desiredWidth / Math.max(1, baseVp.width)));
    const vp = page.getViewport({ scale: renderScale });

    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round(vp.width));
    canvas.height = Math.max(1, Math.round(vp.height));
    const ctx = canvas.getContext('2d');
    if (!ctx) throw new Error('Canvas 2D context ไม่พร้อมใช้งาน');

    ctx.fillStyle = '#FFFFFF';
    ctx.fillRect(0, 0, canvas.width, canvas.height);
    await page.render({ canvasContext: ctx, viewport: vp, canvas }).promise;

    const compressed = await compressCanvasToJpeg(canvas, 1480, 0.85);
    pages.push({
      pageNumber: p,
      dataUrl: compressed.dataUrl,
      pixelWidth: compressed.width,
      pixelHeight: compressed.height,
      aspectRatio: compressed.width / Math.max(1, compressed.height),
      mmWidth,
      mmHeight,
      detectedPaperSize,
    });
  }

  return {
    fileName: file.name,
    isPdf: true,
    numPages: pdf.numPages,
    pages,
  };
}

/**
 * คำนวณกรอบตัด (Crop Box %) ของป้ายดวงแรกบนแผ่น Pre-Print จากระยะขอบและขนาดป้าย (mm)
 */
export function computeFirstCellCropBox(params: {
  paperWidthMm: number;
  paperHeightMm: number;
  magTopMm: number;
  magLeftMm: number;
  labelWidthMm: number;
  labelHeightMm: number;
}): CropBoxPct {
  const pw = Math.max(30, params.paperWidthMm);
  const ph = Math.max(30, params.paperHeightMm);
  const xPct = clampPct((params.magLeftMm / pw) * 100, 0, 95);
  const yPct = clampPct((params.magTopMm / ph) * 100, 0, 95);
  const wPct = clampPct((params.labelWidthMm / pw) * 100, 2, 100 - xPct);
  const hPct = clampPct((params.labelHeightMm / ph) * 100, 2, 100 - yPct);
  return {
    xPct: Number(xPct.toFixed(2)),
    yPct: Number(yPct.toFixed(2)),
    wPct: Number(wPct.toFixed(2)),
    hPct: Number(hPct.toFixed(2)),
  };
}

/**
 * ตัดภาพ 1 ดวงออกจากภาพฟอร์มเต็มแผ่น ตาม Crop Box (%) เพื่อใช้ทาบใน Canvas หน้า Design
 */
export async function cropSheetCellToDataUrl(
  sheetDataUrl: string,
  crop: CropBoxPct,
  maxWidth = 900
): Promise<string> {
  const img = await loadImage(sheetDataUrl);
  const sw = img.naturalWidth || img.width;
  const sh = img.naturalHeight || img.height;

  const sx = Math.max(0, Math.round((crop.xPct / 100) * sw));
  const sy = Math.max(0, Math.round((crop.yPct / 100) * sh));
  const cw = Math.max(16, Math.min(sw - sx, Math.round((crop.wPct / 100) * sw)));
  const ch = Math.max(16, Math.min(sh - sy, Math.round((crop.hPct / 100) * sh)));

  const scale = cw > maxWidth ? maxWidth / cw : 1;
  const dst = document.createElement('canvas');
  dst.width = Math.max(1, Math.round(cw * scale));
  dst.height = Math.max(1, Math.round(ch * scale));
  const ctx = dst.getContext('2d');
  if (!ctx) return sheetDataUrl;

  ctx.fillStyle = '#FFFFFF';
  ctx.fillRect(0, 0, dst.width, dst.height);
  ctx.drawImage(img, sx, sy, cw, ch, 0, 0, dst.width, dst.height);
  return dst.toDataURL('image/jpeg', 0.88);
}
