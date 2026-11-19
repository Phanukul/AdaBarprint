export type DesignElementType =
  | 'price'
  | 'text'
  | 'static'
  | 'barcode'
  | 'image'
  | 'rect'
  | 'circle'
  | 'badge'
  | 'line'
  | 'icon'
  | 'date';

export type DesignIconName =
  | 'tag'
  | 'percent'
  | 'star'
  | 'flame'
  | 'sparkles'
  | 'gift'
  | 'badge-check'
  | 'clock'
  | 'shopping-bag'
  | 'heart'
  | 'award'
  | 'zap'
  | 'megaphone';

export interface DesignElement {
  id: string;
  type: DesignElementType;
  label: string;
  x: number;
  y: number;
  width: number;
  height: number;
  rotation?: number;
  opacity?: number;
  visible?: boolean;
  locked?: boolean;
  field?: string;
  text?: string;
  prefix?: string;
  suffix?: string;
  fontSize?: number; // pt
  fontWeight?: 'normal' | 'medium' | 'semibold' | 'bold' | 'extrabold' | 'black';
  color?: string;
  fill?: string;
  borderColor?: string;
  borderWidth?: number; // mm
  borderRadius?: number; // mm
  align?: 'left' | 'center' | 'right';
  multiLine?: boolean;
  strike?: boolean;
  showText?: boolean;
  iconName?: DesignIconName;
  src?: string;
  objectFit?: 'contain' | 'cover' | 'fill';
}

export type PaperSizeCode = string;

export interface PaperSizeRecord {
  FTPpsCode: string;
  FTPpsName: string;
  FCPpsWidth: number;
  FCPpsHeight: number;
  FTPpsWinKind?: string | null;
  FTPpsStaCustom: string;
  FNPpsSeq?: number | null;
  FTPpsStaUse: string;
}

export interface PreprintRecord {
  FTPprCode: string;
  FTPprName: string;
  FTPprFileName?: string | null;
  FTPpsCode: string;
  FCPprWidth: number;
  FCPprHeight: number;
  FNPprCol: number;
  FNPprRow: number;
  FCPprLabelW?: number | null;
  FCPprLabelH?: number | null;
  FCPprMagTop?: number | null;
  FCPprMagLeft?: number | null;
  FCPprMagBottom?: number | null;
  FCPprMagRight?: number | null;
  FCPprGapH?: number | null;
  FCPprGapV?: number | null;
  FNPprPageNo?: number | null;
  FTPprScope: string;
  FCPprOpacity?: number | null;
  FTPprSheetImg?: string | null;
  FTPprCellImg?: string | null;
  FTPprCropJson?: string | null;
  FTPprStaUse: string;
}

export interface LabelFieldRecord {
  FTLbfCode: string;
  FTLbfName: string;
  FTLbfType: string;
  FTLbfSample?: string | null;
  FCLbfDefW?: number | null;
  FCLbfDefH?: number | null;
  FNLbfDefFont?: number | null;
  FTLbfDefWeight?: string | null;
  FTLbfDefColor?: string | null;
  FNLbfSeq?: number | null;
  FTLbfStaAdv: string;
  FTLbfStaUse: string;
}

export interface CompanyInfoRecord {
  cmpCode: string;
  cmpName: string;
  shopName: string;
  defaultBchCode: string;
  defaultBchName: string;
}

export const PAPER_SIZE_PRESETS: Record<
  string,
  { label: string; widthMm: number; heightMm: number; isCustom?: boolean }
> = {
  A4: { label: 'A4 (210 × 297 mm)', widthMm: 210, heightMm: 297 },
  A4_L: { label: 'A4 แนวนอน (297 × 210 mm)', widthMm: 297, heightMm: 210 },
  A5: { label: 'A5 (148 × 210 mm)', widthMm: 148, heightMm: 210 },
  A5_L: { label: 'A5 แนวนอน (210 × 148 mm)', widthMm: 210, heightMm: 148 },
  A6: { label: 'A6 (105 × 148 mm)', widthMm: 105, heightMm: 148 },
  A3: { label: 'A3 (297 × 420 mm)', widthMm: 297, heightMm: 420 },
  B5_JIS: { label: 'B5 JIS (182 × 257 mm)', widthMm: 182, heightMm: 257 },
  B5_ISO: { label: 'B5 ISO (176 × 250 mm)', widthMm: 176, heightMm: 250 },
  LETTER: { label: 'Letter (215.9 × 279.4 mm)', widthMm: 215.9, heightMm: 279.4 },
  LEGAL: { label: 'Legal / F4 (215.9 × 355.6 mm)', widthMm: 215.9, heightMm: 355.6 },
  ROLL: { label: 'ม้วน Thermal (Roll)', widthMm: 50, heightMm: 38, isCustom: true },
  CUSTOM: { label: 'กำหนดเอง (mm)', widthMm: 210, heightMm: 297, isCustom: true },
};

/** รวมขนาดกระดาษจากฐานข้อมูลกับ Preset มาตรฐานให้แสดงครบทุกขนาด (ไม่ซ้ำ) */
export function mergePaperSizeOptions(dbPaperSizes: PaperSizeRecord[]): PaperSizeRecord[] {
  const byCode = new Map<string, PaperSizeRecord>();
  for (const p of dbPaperSizes) {
    byCode.set(p.FTPpsCode, p);
  }
  const merged: PaperSizeRecord[] = [...dbPaperSizes];
  Object.entries(PAPER_SIZE_PRESETS).forEach(([code, preset], idx) => {
    if (!byCode.has(code)) {
      merged.push({
        FTPpsCode: code,
        FTPpsName: preset.label,
        FCPpsWidth: preset.widthMm,
        FCPpsHeight: preset.heightMm,
        FTPpsStaCustom: preset.isCustom ? '1' : '2',
        FNPpsSeq: 50 + idx,
        FTPpsStaUse: '1',
      });
    }
  });
  return merged;
}

export interface PreprintGuideConfig {
  preprintCode?: string;
  sourceName?: string;
  scope: 'sheet' | 'label';
  sheetBgDataUrl?: string;
  cellBgDataUrl?: string;
  pageIndex?: number;
  opacity: number; // 0.15 - 1
  contrast?: number; // 50 - 200 (%)
  paperSize: PaperSizeCode;
  paperWidthMm: number;
  paperHeightMm: number;
  cropBox?: { xPct: number; yPct: number; wPct: number; hPct: number };
}

export interface LabelDesignLayout {
  customDesign: boolean;
  designMode?: 'full' | 'preprint';
  preprint?: PreprintGuideConfig;
  lbsCode: string;
  priType: '1' | '2';
  name: string;
  width: number; // mm
  height: number; // mm
  backgroundColor: string;
  borderColor: string;
  borderWidth: number; // mm
  borderRadius: number; // mm
  elements: DesignElement[];
}

export const DESIGN_COLOR_SWATCHES = [
  '#000000',
  '#1E293B',
  '#FFFFFF',
  '#FFEA00',
  '#FFF3C4',
  '#DC2626',
  '#EF4444',
  '#EA580C',
  '#007A87',
  '#0284C7',
  '#16A34A',
  '#7C3AED',
  '#E2E8F0',
  'transparent',
];

export const DESIGN_ICON_LIST: Array<{
  name: DesignIconName;
  label: string;
  defaultColor: string;
  paths: string[];
}> = [
  {
    name: 'tag',
    label: 'ป้ายราคา',
    defaultColor: '#dc2626',
    paths: [
      'M12.586 2.586A2 2 0 0 0 11.172 2H4a2 2 0 0 0-2 2v7.172a2 2 0 0 0 .586 1.414l8.704 8.704a2.426 2.426 0 0 0 3.42 0l6.58-6.58a2.426 2.426 0 0 0 0-3.42z',
      'M7.5 7.5m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0',
    ],
  },
  {
    name: 'percent',
    label: 'ลด %',
    defaultColor: '#dc2626',
    paths: [
      'M19 5 5 19',
      'M6.5 6.5m-2.5 0a2.5 2.5 0 1 0 5 0a2.5 2.5 0 1 0 -5 0',
      'M17.5 17.5m-2.5 0a2.5 2.5 0 1 0 5 0a2.5 2.5 0 1 0 -5 0',
    ],
  },
  {
    name: 'star',
    label: 'แนะนำ / ขายดี',
    defaultColor: '#eab308',
    paths: [
      'M11.525 2.295a.53.53 0 0 1 .95 0l2.31 4.679a2.123 2.123 0 0 0 1.595 1.16l5.166.756a.53.53 0 0 1 .294.904l-3.736 3.638a2.123 2.123 0 0 0-.611 1.878l.882 5.14a.53.53 0 0 1-.771.56l-4.618-2.428a2.122 2.122 0 0 0-1.973 0L6.396 21.01a.53.53 0 0 1-.77-.56l.881-5.139a2.122 2.122 0 0 0-.611-1.879L2.16 9.795a.53.53 0 0 1 .294-.906l5.165-.755a2.122 2.122 0 0 0 1.597-1.16z',
    ],
  },
  {
    name: 'flame',
    label: 'สินค้าฮอต',
    defaultColor: '#ea580c',
    paths: [
      'M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z',
    ],
  },
  {
    name: 'sparkles',
    label: 'ใหม่ / พิเศษ',
    defaultColor: '#007a87',
    paths: [
      'M9.937 15.5A2 2 0 0 0 8.5 14.063l-6.135-1.582a.5.5 0 0 1 0-.962L8.5 9.936A2 2 0 0 0 9.937 8.5l1.582-6.135a.5.5 0 0 1 .963 0L14.063 8.5A2 2 0 0 0 15.5 9.937l6.135 1.581a.5.5 0 0 1 0 .964L15.5 14.063a2 2 0 0 0-1.437 1.437l-1.582 6.135a.5.5 0 0 1-.963 0z',
      'M20 3v4',
      'M22 5h-4',
    ],
  },
  {
    name: 'gift',
    label: 'ของแถม / โปรฯ',
    defaultColor: '#db2777',
    paths: [
      'M20 12v10H4V12',
      'M2 7h20v5H2z',
      'M12 22V7',
      'M12 7H7.5a2.5 2.5 0 0 1 0-5C11 2 12 7 12 7z',
      'M12 7h4.5a2.5 2.5 0 0 0 0-5C13 2 12 7 12 7z',
    ],
  },
  {
    name: 'badge-check',
    label: 'รับรองราคา',
    defaultColor: '#16a34a',
    paths: [
      'M3.85 8.62a4 4 0 0 1 4.78-4.77 4 4 0 0 1 6.74 0 4 4 0 0 1 4.78 4.78 4 4 0 0 1 0 6.74 4 4 0 0 1-4.77 4.78 4 4 0 0 1-6.75 0 4 4 0 0 1-4.78-4.77 4 4 0 0 1 0-6.76Z',
      'm9 12 2 2 4-4',
    ],
  },
  {
    name: 'clock',
    label: 'Time Sale',
    defaultColor: '#dc2626',
    paths: ['M12 12m-10 0a10 10 0 1 0 20 0a10 10 0 1 0 -20 0', 'M12 6v6l4 2'],
  },
  {
    name: 'shopping-bag',
    label: 'ช้อปเลย',
    defaultColor: '#0284c7',
    paths: [
      'M6 2 3 6v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V6l-3-4Z',
      'M3 6h18',
      'M16 10a4 4 0 0 1-8 0',
    ],
  },
  {
    name: 'heart',
    label: 'ถูกใจ / ยอดนิยม',
    defaultColor: '#e11d48',
    paths: [
      'M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z',
    ],
  },
  {
    name: 'award',
    label: 'สินค้าคุณภาพ',
    defaultColor: '#d97706',
    paths: [
      'M12 8m-6 0a6 6 0 1 0 12 0a6 6 0 1 0 -12 0',
      'M15.477 12.89 17 22l-5-3-5 3 1.523-9.11',
    ],
  },
  {
    name: 'zap',
    label: 'Flash Sale',
    defaultColor: '#eab308',
    paths: [
      'M4 14a1 1 0 0 1-.78-1.63l9.9-10.2a.5.5 0 0 1 .86.46l-1.92 6.02A1 1 0 0 0 13 10h7a1 1 0 0 1 .78 1.63l-9.9 10.2a.5.5 0 0 1-.86-.46l1.92-6.02A1 1 0 0 0 11 14z',
    ],
  },
  {
    name: 'megaphone',
    label: 'ประกาศโปรฯ',
    defaultColor: '#dc2626',
    paths: [
      'm3 11 18-5v12L3 14v-3z',
      'M11.6 16.8a3 3 0 1 1-5.8-1.6',
    ],
  },
];

export const DESIGN_DATA_FIELDS: Array<{
  type: DesignElementType;
  field: string;
  label: string;
  defaultW: number;
  defaultH: number;
  fontSize: number;
  fontWeight?: DesignElement['fontWeight'];
  prefix?: string;
  suffix?: string;
  strike?: boolean;
  color?: string;
  align?: DesignElement['align'];
  multiLine?: boolean;
}> = [
  { type: 'price', field: 'price.value', label: 'ราคาขาย (Price)', defaultW: 36, defaultH: 14, fontSize: 28, fontWeight: 'black', suffix: ' บาท', align: 'center', color: '#000000' },
  { type: 'price', field: 'price.old_value', label: 'ราคาเดิมขีดฆ่า (Old Price)', defaultW: 24, defaultH: 5, fontSize: 9, fontWeight: 'bold', prefix: 'ปกติ ', strike: true, align: 'left', color: '#475569' },
  { type: 'text', field: 'product.name', label: 'ชื่อสินค้า (ไทย)', defaultW: 52, defaultH: 10, fontSize: 10, fontWeight: 'bold', align: 'left', color: '#111111', multiLine: true },
  { type: 'text', field: 'product.name_oth', label: 'ชื่อสินค้า (รอง/Eng)', defaultW: 52, defaultH: 5, fontSize: 8, fontWeight: 'normal', align: 'left', color: '#334155' },
  { type: 'barcode', field: 'product.barcode', label: 'บาร์โค้ด (Barcode)', defaultW: 38, defaultH: 11, fontSize: 8 },
  { type: 'text', field: 'product.code', label: 'รหัสสินค้า (Code)', defaultW: 26, defaultH: 4, fontSize: 7, align: 'left', color: '#334155' },
  { type: 'text', field: 'tag.selling_unit', label: 'หน่วยขาย (Unit)', defaultW: 22, defaultH: 4, fontSize: 7, align: 'right', color: '#111111' },
  { type: 'text', field: 'promotion.text', label: 'ข้อความโปรโมชัน', defaultW: 40, defaultH: 6, fontSize: 10, fontWeight: 'bold', align: 'center', color: '#DC2626' },
  { type: 'text', field: 'promotion.interval', label: 'ระยะเวลาโปรฯ', defaultW: 44, defaultH: 4, fontSize: 7, align: 'left', color: '#DC2626' },
  { type: 'date', field: 'tag.print_date', label: 'วันที่พิมพ์ป้าย', defaultW: 22, defaultH: 4, fontSize: 7, align: 'right', color: '#334155' },
  { type: 'text', field: 'store.name', label: 'ชื่อสาขา/ร้าน', defaultW: 36, defaultH: 5, fontSize: 8, fontWeight: 'semibold', align: 'left', color: '#007A87' },
];

export const DESIGN_ADVANCED_FIELDS: typeof DESIGN_DATA_FIELDS = [
  { type: 'text', field: 'product.dim', label: 'ขนาด/ปริมาตร (ถ้ามี)', defaultW: 26, defaultH: 4, fontSize: 7, align: 'left', color: '#334155' },
  { type: 'text', field: 'tag.pri_per_unit', label: 'ราคาต่อหน่วย (ถ้ามี)', defaultW: 32, defaultH: 4, fontSize: 7, align: 'left', color: '#475569' },
  { type: 'text', field: 'tag.pri_status', label: 'สถานะราคา (AC)', defaultW: 12, defaultH: 4, fontSize: 7, fontWeight: 'semibold', align: 'right', color: '#111111' },
  { type: 'text', field: 'tag.sub_dept', label: 'รหัสหมวดย่อย (SubDept)', defaultW: 20, defaultH: 4, fontSize: 7, align: 'left', color: '#475569' },
];

export const DESIGN_BADGE_PRESETS: Array<{
  label: string;
  text: string;
  fill: string;
  color: string;
  borderRadius: number;
  width: number;
  height: number;
  fontSize: number;
}> = [
  { label: 'ป้ายลดราคา', text: 'ลดราคา!', fill: '#DC2626', color: '#FFFFFF', borderRadius: 1.5, width: 24, height: 6.5, fontSize: 9 },
  { label: 'ป้ายราคาพิเศษ', text: 'ราคาพิเศษ', fill: '#007A87', color: '#FFFFFF', borderRadius: 1.5, width: 26, height: 6.5, fontSize: 9 },
  { label: 'ป้าย HOT!', text: 'HOT SALE', fill: '#EA580C', color: '#FFFFFF', borderRadius: 3, width: 24, height: 6.5, fontSize: 8.5 },
  { label: 'ป้ายแนะนำ', text: 'สินค้าแนะนำ', fill: '#EAB308', color: '#111111', borderRadius: 1.5, width: 26, height: 6.5, fontSize: 8.5 },
  { label: 'ป้าย 1 แถม 1', text: 'ซื้อ 1 แถม 1', fill: '#DB2777', color: '#FFFFFF', borderRadius: 1.5, width: 28, height: 6.5, fontSize: 8.5 },
  { label: 'ป้าย Time Sale', text: 'TIME SALE', fill: '#1E293B', color: '#FFEA00', borderRadius: 1.5, width: 26, height: 6.5, fontSize: 8.5 },
];

export function uidDesign(prefix = 'el'): string {
  return `${prefix}_${Math.random().toString(36).slice(2, 8)}`;
}

export function roundMm(n: number, step = 0.5): number {
  if (!Number.isFinite(n)) return 0;
  return Math.round(n / step) * step;
}

export function buildStarterLayout(
  lbsCode: string,
  widthMm: number,
  heightMm: number,
  priType: '1' | '2',
  name?: string
): LabelDesignLayout {
  const w = Math.max(20, widthMm || 100);
  const h = Math.max(15, heightMm || 38);
  const isPromo = priType === '2';
  const bg = isPromo ? '#FFEA00' : '#FFFFFF';
  const elements: DesignElement[] = [];

  // Large / Landscape cards (e.g. 200x142, 200x95)
  if (w >= 140 && h >= 80) {
    const pad = 6;
    if (isPromo) {
      elements.push({
        id: uidDesign(),
        type: 'rect',
        label: 'แถบหัวโปรโมชัน',
        x: 0,
        y: 0,
        width: w,
        height: 22,
        fill: '#DC2626',
        borderWidth: 0,
        borderRadius: 0,
      });
      elements.push({
        id: uidDesign(),
        type: 'icon',
        label: 'ไอคอนโปรโมชัน',
        iconName: 'flame',
        x: 7,
        y: 4,
        width: 14,
        height: 14,
        color: '#FFEA00',
      });
      elements.push({
        id: uidDesign(),
        type: 'static',
        label: 'หัวข้อโปรฯ',
        text: lbsCode.includes('SP') ? 'สินค้าพิเศษ!' : 'ลดกระหน่ำ!',
        x: 24,
        y: 3,
        width: w - 32,
        height: 16,
        fontSize: 34,
        fontWeight: 'black',
        color: '#FFFFFF',
        align: 'left',
      });
    } else {
      elements.push({
        id: uidDesign(),
        type: 'rect',
        label: 'แถบหัวป้าย',
        x: 0,
        y: 0,
        width: w,
        height: 14,
        fill: '#007A87',
        borderWidth: 0,
        borderRadius: 0,
      });
      elements.push({
        id: uidDesign(),
        type: 'text',
        label: 'ชื่อสาขา/ร้าน',
        field: 'store.name',
        x: pad,
        y: 2.5,
        width: w - pad * 2,
        height: 9,
        fontSize: 16,
        fontWeight: 'bold',
        color: '#FFFFFF',
        align: 'left',
      });
    }

    const topOffset = isPromo ? 26 : 18;
    elements.push({
      id: uidDesign(),
      type: 'text',
      label: 'ชื่อสินค้า',
      field: 'product.name',
      x: pad,
      y: topOffset,
      width: w - pad * 2,
      height: 12,
      fontSize: 22,
      fontWeight: 'bold',
      color: '#111111',
      align: 'left',
    });
    elements.push({
      id: uidDesign(),
      type: 'text',
      label: 'ชื่อรอง / ขนาด',
      field: 'product.name_oth',
      x: pad,
      y: topOffset + 12,
      width: w - pad * 2,
      height: 7,
      fontSize: 12,
      fontWeight: 'normal',
      color: '#334155',
      align: 'left',
    });
    if (isPromo) {
      elements.push({
        id: uidDesign(),
        type: 'price',
        label: 'ราคาปกติ (ขีดฆ่า)',
        field: 'price.old_value',
        prefix: 'ปกติ ',
        strike: true,
        x: pad,
        y: topOffset + 22,
        width: 55,
        height: 12,
        fontSize: 18,
        fontWeight: 'bold',
        color: '#475569',
        align: 'left',
      });
    }
    elements.push({
      id: uidDesign(),
      type: 'price',
      label: 'ราคาขาย',
      field: 'price.value',
      suffix: ' บาท',
      x: pad,
      y: roundMm(h - 54, 0.5),
      width: roundMm(w * 0.62, 0.5),
      height: 36,
      fontSize: h >= 120 ? 48 : 38,
      fontWeight: 'black',
      color: '#000000',
      align: 'center',
    });
    elements.push({
      id: uidDesign(),
      type: 'barcode',
      label: 'บาร์โค้ด',
      field: 'product.barcode',
      showText: true,
      x: roundMm(w - 66, 0.5),
      y: roundMm(h - 32, 0.5),
      width: 60,
      height: 22,
    });
    elements.push({
      id: uidDesign(),
      type: 'text',
      label: 'หน่วยขาย',
      field: 'tag.selling_unit',
      x: roundMm(w - 66, 0.5),
      y: roundMm(h - 42, 0.5),
      width: 60,
      height: 8,
      fontSize: 12,
      fontWeight: 'semibold',
      color: '#111111',
      align: 'right',
    });
    elements.push({
      id: uidDesign(),
      type: 'date',
      label: 'วันที่พิมพ์',
      field: 'tag.print_date',
      x: pad,
      y: roundMm(h - 10, 0.5),
      width: 45,
      height: 6,
      fontSize: 10,
      color: '#334155',
      align: 'left',
    });
  } else if (w >= 85 && h <= 50) {
    // Wide shelf strip (e.g. 100x38, 100x35)
    if (isPromo) {
      elements.push({
        id: uidDesign(),
        type: 'badge',
        label: 'แถบลดราคา',
        text: 'ลดราคา',
        fill: '#DC2626',
        color: '#FFFFFF',
        borderRadius: 1,
        x: 2,
        y: 2,
        width: 18,
        height: 5.5,
        fontSize: 8.5,
        fontWeight: 'bold',
        align: 'center',
      });
      elements.push({
        id: uidDesign(),
        type: 'price',
        label: 'ราคาเดิม (ขีดฆ่า)',
        field: 'price.old_value',
        prefix: 'ปกติ ',
        strike: true,
        x: 21,
        y: 2.2,
        width: 21,
        height: 5,
        fontSize: 8,
        fontWeight: 'bold',
        color: '#475569',
        align: 'left',
      });
    }
    elements.push({
      id: uidDesign(),
      type: 'price',
      label: 'ราคาขาย',
      field: 'price.value',
      suffix: ' บาท',
      x: 2,
      y: isPromo ? 9 : 6,
      width: 39,
      height: 22,
      fontSize: 30,
      fontWeight: 'black',
      color: '#000000',
      align: 'center',
    });
    elements.push({
      id: uidDesign(),
      type: 'line',
      label: 'เส้นแบ่งกลาง',
      x: 42.5,
      y: 3,
      width: 0.4,
      height: h - 6,
      fill: '#CBD5E1',
      color: '#CBD5E1',
    });
    elements.push({
      id: uidDesign(),
      type: 'text',
      label: 'ชื่อสินค้า',
      field: 'product.name',
      x: 44.5,
      y: 2.5,
      width: w - 47,
      height: 6.5,
      fontSize: 10.5,
      fontWeight: 'bold',
      color: '#111111',
      align: 'left',
    });
    elements.push({
      id: uidDesign(),
      type: 'text',
      label: 'ชื่อรองสินค้า',
      field: 'product.name_oth',
      x: 44.5,
      y: 9,
      width: w - 47,
      height: 5,
      fontSize: 8,
      color: '#334155',
      align: 'left',
    });
    elements.push({
      id: uidDesign(),
      type: 'barcode',
      label: 'บาร์โค้ด',
      field: 'product.barcode',
      showText: true,
      x: 44.5,
      y: 16,
      width: 36,
      height: 13.5,
    });
    elements.push({
      id: uidDesign(),
      type: 'text',
      label: 'หน่วยขาย',
      field: 'tag.selling_unit',
      x: w - 20,
      y: 19,
      width: 18,
      height: 5,
      fontSize: 7.5,
      fontWeight: 'semibold',
      color: '#111111',
      align: 'right',
    });
    elements.push({
      id: uidDesign(),
      type: 'date',
      label: 'วันที่พิมพ์',
      field: 'tag.print_date',
      x: w - 20,
      y: h - 6,
      width: 18,
      height: 4.5,
      fontSize: 6.5,
      color: '#334155',
      align: 'right',
    });
  } else if (h > w) {
    // Portrait card (e.g. ADA-100x142, UFM-NOR-16 49x70, SH-48x72)
    const pad = roundMm(Math.max(2, w * 0.05), 0.5);
    const headerH = roundMm(Math.max(7, h * 0.11), 0.5);
    elements.push({
      id: uidDesign(),
      type: 'rect',
      label: 'แถบหัวป้าย',
      x: 0,
      y: 0,
      width: w,
      height: headerH,
      fill: isPromo ? '#DC2626' : '#007A87',
    });
    elements.push({
      id: uidDesign(),
      type: 'static',
      label: 'ข้อความหัวป้าย',
      text: isPromo ? 'โปรโมชันพิเศษ' : 'ราคาสินค้า',
      x: pad,
      y: roundMm(headerH * 0.18, 0.5),
      width: roundMm(w - pad * 2, 0.5),
      height: roundMm(headerH * 0.65, 0.5),
      fontSize: w >= 80 ? 18 : 10,
      fontWeight: 'bold',
      color: '#FFFFFF',
      align: 'center',
    });
    elements.push({
      id: uidDesign(),
      type: 'text',
      label: 'ชื่อสินค้า',
      field: 'product.name',
      x: pad,
      y: roundMm(headerH + 3, 0.5),
      width: roundMm(w - pad * 2, 0.5),
      height: roundMm(Math.max(6, h * 0.14), 0.5),
      fontSize: w >= 80 ? 15 : 8.5,
      fontWeight: 'bold',
      color: '#111111',
      align: 'left',
    });
    elements.push({
      id: uidDesign(),
      type: 'price',
      label: 'ราคาขาย',
      field: 'price.value',
      suffix: ' บาท',
      x: pad,
      y: roundMm(h * 0.42, 0.5),
      width: roundMm(w - pad * 2, 0.5),
      height: roundMm(Math.max(12, h * 0.22), 0.5),
      fontSize: w >= 80 ? 34 : 18,
      fontWeight: 'black',
      color: '#000000',
      align: 'center',
    });
    elements.push({
      id: uidDesign(),
      type: 'barcode',
      label: 'บาร์โค้ด',
      field: 'product.barcode',
      showText: true,
      x: pad,
      y: roundMm(h - Math.max(12, h * 0.21) - 3, 0.5),
      width: roundMm(w - pad * 2, 0.5),
      height: roundMm(Math.max(10, h * 0.17), 0.5),
    });
  } else {
    // Compact landscape tag (e.g. 50x38, 32x24)
    const pad = 2;
    if (isPromo) {
      elements.push({
        id: uidDesign(),
        type: 'badge',
        label: 'ป้ายลดราคา',
        text: 'ลดราคา',
        fill: '#DC2626',
        color: '#FFFFFF',
        borderRadius: 0.8,
        x: w - 15,
        y: 1.5,
        width: 13.5,
        height: 4.5,
        fontSize: 7,
        fontWeight: 'bold',
        align: 'center',
      });
    }
    elements.push({
      id: uidDesign(),
      type: 'price',
      label: 'ราคาขาย',
      field: 'price.value',
      suffix: ' บาท',
      x: pad,
      y: isPromo ? 5.5 : 3,
      width: w - pad * 2,
      height: Math.max(10, h * 0.36),
      fontSize: w >= 45 ? 20 : 14,
      fontWeight: 'black',
      color: '#000000',
      align: 'center',
    });
    elements.push({
      id: uidDesign(),
      type: 'text',
      label: 'ชื่อสินค้า',
      field: 'product.name',
      x: pad,
      y: h * 0.48,
      width: w - pad * 2,
      height: 5,
      fontSize: 7.5,
      fontWeight: 'bold',
      color: '#111111',
      align: 'left',
    });
    elements.push({
      id: uidDesign(),
      type: 'barcode',
      label: 'บาร์โค้ด',
      field: 'product.barcode',
      showText: true,
      x: pad,
      y: h - 11.5,
      width: w - pad * 2,
      height: 10,
    });
  }

  return {
    customDesign: true,
    lbsCode,
    priType,
    name: name || `แบบกำหนดเอง ${lbsCode} (${isPromo ? 'โปรโมชัน' : 'ปกติ'})`,
    width: w,
    height: h,
    backgroundColor: bg,
    borderColor: '#d1d5db',
    borderWidth: 0.3,
    borderRadius: 0,
    elements,
  };
}
