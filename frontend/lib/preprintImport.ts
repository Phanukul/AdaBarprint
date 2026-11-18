/// <reference types="vite/client" />
import * as pdfjsLib from 'pdfjs-dist';
import pdfWorkerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url';

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
  detectedPaperSize?: 'A4' | 'B5_JIS' | 'B5_ISO' | 'LETTER' | 'CUSTOM';
}

export interface ParsedPreprintResult {
  fileName: string;
  isPdf: boolean;
  numPages: number;
  pages: ParsedPreprintPage[];
}

function detectPdfPaperPreset(
  mmWidth: number,
  mmHeight: number
): 'A4' | 'B5_JIS' | 'B5_ISO' | 'LETTER' | 'CUSTOM' {
  const presets: Array<{
    code: 'A4' | 'B5_JIS' | 'B5_ISO' | 'LETTER';
    w: number;
    h: number;
  }> = [
    { code: 'A4', w: 210, h: 297 },
    { code: 'B5_JIS', w: 182, h: 257 },
    { code: 'B5_ISO', w: 176, h: 250 },
    { code: 'LETTER', w: 215.9, h: 279.4 },
  ];
  for (const p of presets) {
    const matchDirect = Math.abs(mmWidth - p.w) <= 3 && Math.abs(mmHeight - p.h) <= 3;
    const matchRotated = Math.abs(mmWidth - p.h) <= 3 && Math.abs(mmHeight - p.w) <= 3;
    if (matchDirect || matchRotated) return p.code;
  }
  return 'CUSTOM';
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

  const buf = await file.arrayBuffer();
  const loadingTask = pdfjsLib.getDocument({ data: new Uint8Array(buf) });
  const pdf = await loadingTask.promise;
  const pageCount = Math.min(pdf.numPages, maxPdfPages);
  const pages: ParsedPreprintPage[] = [];

  for (let p = 1; p <= pageCount; p++) {
    const page = await pdf.getPage(p);
    const baseVp = page.getViewport({ scale: 1 });
    const mmWidth = Number(((baseVp.width * 25.4) / 72).toFixed(1));
    const mmHeight = Number(((baseVp.height * 25.4) / 72).toFixed(1));
    const detectedPaperSize = detectPdfPaperPreset(mmWidth, mmHeight);
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
