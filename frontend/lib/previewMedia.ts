import { PAPER_SIZE_PRESETS, type PaperSizeCode } from '@/types/design';

export type PreviewMediaMode =
  | 'AUTO'
  | 'A3'
  | 'A4'
  | 'A4_L'
  | 'A5'
  | 'A5_L'
  | 'A6'
  | 'B5_JIS'
  | 'B5_ISO'
  | 'LETTER'
  | 'LEGAL'
  | 'ROLL'
  | 'SINGLE'
  | 'CUSTOM';

export type PreviewRefFrame = 'NONE' | 'A4' | 'B5_JIS';

export interface PreviewMediaConfig {
  mode: PreviewMediaMode;
  customWidthMm: number;
  customHeightMm: number;
  rollCols: number;
  rollGapV: number;
  refFrame: PreviewRefFrame;
}

export interface PreviewBaseSpecs {
  lbsGroup?: string;
  labelWidthMm: number;
  labelHeightMm: number;
  savedPaperSize: PaperSizeCode;
  savedPaperWidthMm: number;
  savedPaperHeightMm: number;
  savedCols: number;
  savedRows: number;
  magTopMm: number;
  magLeftMm: number;
  magBottomMm: number;
  magRightMm: number;
  gapHMm: number;
  gapVMm: number;
}

export interface ComputedPreviewMediaLayout {
  effectiveMode: Exclude<PreviewMediaMode, 'AUTO'>;
  isRoll: boolean;
  isSingle: boolean;
  paperSize: PaperSizeCode;
  paperLabel: string;
  paperWidthMm: number;
  paperHeightMm: number;
  cols: number;
  rows: number;
  perPage: number;
  magTopMm: number;
  magLeftMm: number;
  magBottomMm: number;
  magRightMm: number;
  gapHMm: number;
  gapVMm: number;
  areaPct: number;
}

export function getPaperSizeDisplayName(
  paperSize?: string | null,
  widthMm?: number,
  heightMm?: number
): string {
  switch (paperSize) {
    case 'A3':
      return 'A3';
    case 'A4':
      return 'A4';
    case 'A4_L':
      return 'A4 แนวนอน';
    case 'A5':
      return 'A5';
    case 'A5_L':
      return 'A5 แนวนอน';
    case 'A6':
      return 'A6';
    case 'B5_JIS':
      return 'B5 JIS';
    case 'B5_ISO':
      return 'B5 ISO';
    case 'LETTER':
      return 'Letter';
    case 'LEGAL':
      return 'Legal / F4';
    case 'ROLL':
      return 'ม้วน Thermal (Roll)';
    case 'SINGLE':
      return 'ป้ายเดี่ยว 1:1';
    case 'CUSTOM':
      return widthMm && heightMm ? `กำหนดเอง (${widthMm}×${heightMm} mm)` : 'กำหนดเอง';
    default:
      if (paperSize && paperSize in PAPER_SIZE_PRESETS) {
        return PAPER_SIZE_PRESETS[paperSize].label;
      }
      return widthMm && heightMm ? `${paperSize || 'A4'} (${widthMm}×${heightMm} mm)` : paperSize || 'A4';
  }
}

export function getDefaultPreviewMediaConfig(params: {
  savedPaperSize?: string | null;
  savedPaperWidthMm?: number;
  savedPaperHeightMm?: number;
  lbsGroup?: string;
  gapVMm?: number;
  savedCols?: number | null;
  cellWidthMm?: number;
  gapHMm?: number | null;
}): PreviewMediaConfig {
  const saved = params.savedPaperSize || 'A4';
  const refFrame: PreviewRefFrame =
    saved === 'B5_JIS' || saved === 'B5_ISO' || saved === 'A5' || saved === 'A5_L' || saved === 'A6'
      ? 'A4'
      : 'NONE';
  const gap = Math.max(0, params.gapHMm ?? 2);
  const fitCols = params.savedPaperWidthMm && params.cellWidthMm
    ? Math.max(1, Math.floor((params.savedPaperWidthMm - 4 + gap + 0.01) / (params.cellWidthMm + gap))) : 1;
  return {
    mode: params.lbsGroup === '2' && saved === 'ROLL' ? 'ROLL' : 'AUTO',
    customWidthMm: params.savedPaperWidthMm || 210,
    customHeightMm: params.savedPaperHeightMm || 297,
    rollCols: saved === 'ROLL' ? Math.max(1, Math.min(8, fitCols, params.savedCols || fitCols)) : 1,
    rollGapV: Math.max(0, params.gapVMm ?? 2),
    refFrame,
  };
}

export function getReferenceFrameDims(refFrame: PreviewRefFrame): {
  label: string;
  widthMm: number;
  heightMm: number;
} | null {
  if (refFrame === 'A4') {
    return { label: 'A4 (210 × 297 mm)', widthMm: 210, heightMm: 297 };
  }
  if (refFrame === 'B5_JIS') {
    return { label: 'B5 JIS (182 × 257 mm)', widthMm: 182, heightMm: 257 };
  }
  return null;
}

export function computeEffectivePreviewLayout(
  base: PreviewBaseSpecs,
  cfg: PreviewMediaConfig
): ComputedPreviewMediaLayout {
  const w = Math.max(5, base.labelWidthMm);
  const h = Math.max(5, base.labelHeightMm);

  const effectiveMode: Exclude<PreviewMediaMode, 'AUTO'> =
    cfg.mode !== 'AUTO'
      ? cfg.mode
      : base.savedPaperSize === 'ROLL'
        ? 'ROLL'
        : base.savedPaperSize in PAPER_SIZE_PRESETS
          ? (base.savedPaperSize as Exclude<PreviewMediaMode, 'AUTO'>)
          : 'CUSTOM';

  if (effectiveMode === 'SINGLE') {
    return {
      effectiveMode,
      isRoll: false,
      isSingle: true,
      paperSize: 'CUSTOM',
      paperLabel: `ป้ายเดี่ยว 1:1 (${w}×${h} mm)`,
      paperWidthMm: w,
      paperHeightMm: h,
      cols: 1,
      rows: 1,
      perPage: 1,
      magTopMm: 0,
      magLeftMm: 0,
      magBottomMm: 0,
      magRightMm: 0,
      gapHMm: 0,
      gapVMm: 0,
      areaPct: 100,
    };
  }

  if (effectiveMode === 'ROLL') {
    const cols = Math.max(1, Math.min(8, Math.round(cfg.rollCols || 1)));
    const linerPad = 2;
    const gapH = cols > 1 ? Math.max(0, base.gapHMm ?? 2) : 0;
    const gapV = Math.max(0, cfg.rollGapV ?? 2);
    const rollW = Number((linerPad * 2 + cols * w + (cols - 1) * gapH).toFixed(1));
    const singleRowPrintH = Number((h + gapV).toFixed(1));
    return {
      effectiveMode,
      isRoll: true,
      isSingle: false,
      paperSize: 'ROLL',
      paperLabel: `ม้วน Thermal (${rollW} mm · ${cols} ดวง/แถว)`,
      paperWidthMm: rollW,
      paperHeightMm: singleRowPrintH,
      cols,
      rows: 1,
      perPage: cols,
      magTopMm: 0,
      magLeftMm: linerPad,
      magBottomMm: gapV,
      magRightMm: linerPad,
      gapHMm: gapH,
      gapVMm: gapV,
      areaPct: Math.min(100, ((cols * w * h) / (rollW * singleRowPrintH)) * 100),
    };
  }

  const preset = PAPER_SIZE_PRESETS[effectiveMode];
  const isSameAsSaved = cfg.mode === 'AUTO' || effectiveMode === base.savedPaperSize;
  const paperWidthMm =
    effectiveMode === 'CUSTOM'
      ? cfg.mode === 'AUTO'
        ? Math.max(10, base.savedPaperWidthMm)
        : Math.max(10, cfg.customWidthMm || 210)
      : preset?.widthMm || base.savedPaperWidthMm || 210;
  const paperHeightMm =
    effectiveMode === 'CUSTOM'
      ? cfg.mode === 'AUTO'
        ? Math.max(10, base.savedPaperHeightMm)
        : Math.max(10, cfg.customHeightMm || 297)
      : preset?.heightMm || base.savedPaperHeightMm || 297;

  const magTopMm = isSameAsSaved
    ? base.magTopMm
    : Math.min(3, Math.max(0, (paperHeightMm - h) / 2));
  const magLeftMm = isSameAsSaved
    ? base.magLeftMm
    : Math.min(3, Math.max(0, (paperWidthMm - w) / 2));
  const magBottomMm = isSameAsSaved ? base.magBottomMm : magTopMm;
  const magRightMm = isSameAsSaved ? base.magRightMm : magLeftMm;
  const gapHMm = base.gapHMm;
  const gapVMm = base.gapVMm;

  const usableW = Math.max(w, paperWidthMm - magLeftMm - magRightMm);
  const usableH = Math.max(h, paperHeightMm - magTopMm - magBottomMm);
  const calcCols = Math.max(1, Math.floor((usableW + gapHMm + 0.05) / (w + gapHMm)));
  const calcRows = Math.max(1, Math.floor((usableH + gapVMm + 0.05) / (h + gapVMm)));
  const cols = cfg.mode === 'AUTO' && base.savedCols > 0 ? base.savedCols : calcCols;
  const rows = cfg.mode === 'AUTO' && base.savedRows > 0 ? base.savedRows : calcRows;
  const perPage = Math.max(1, cols * rows);
  const areaPct = Math.min(100, ((w * h * perPage) / (paperWidthMm * paperHeightMm)) * 100);

  return {
    effectiveMode,
    isRoll: false,
    isSingle: false,
    paperSize: effectiveMode,
    paperLabel: getPaperSizeDisplayName(effectiveMode, paperWidthMm, paperHeightMm),
    paperWidthMm,
    paperHeightMm,
    cols,
    rows,
    perPage,
    magTopMm,
    magLeftMm,
    magBottomMm,
    magRightMm,
    gapHMm,
    gapVMm,
    areaPct,
  };
}
