export interface MasterImportRow {
  pdtCode: string;
  pdtName: string;
  barCode: string;
  unitCode: string;
  unitName: string;
  basePrice: number;
  effectiveStart: string;
  effectiveStop: string;
  promoPrice?: number;
  promoName: string;
  promoStart: string;
  promoStop: string;
  quantity: number;
}

interface PrintFileFields {
  code: string;
  name: string;
  unit: string;
  normal?: number;
  promo?: number;
}

export interface ParsedImportResult {
  kind: 'master' | 'barcode_only';
  masterRows: MasterImportRow[];
  barcodeRows: Array<{ barcode: string; quantity: number; fileFields?: PrintFileFields; rowNumber?: number }>;
}

function xmlEscape(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&apos;');
}

function colLetter(indexZero: number): string {
  let n = indexZero + 1;
  let out = '';
  while (n > 0) {
    const rem = (n - 1) % 26;
    out = String.fromCharCode(65 + rem) + out;
    n = Math.floor((n - 1) / 26);
  }
  return out;
}

function colIndexFromRef(cellRef: string): number {
  const letters = cellRef.replace(/[^A-Za-z]/g, '').toUpperCase();
  let idx = 0;
  for (let i = 0; i < letters.length; i++) {
    idx = idx * 26 + (letters.charCodeAt(i) - 64);
  }
  return Math.max(0, idx - 1);
}

const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c >>> 0;
  }
  return table;
})();

function crc32(bytes: Uint8Array): number {
  let crc = 0xffffffff;
  for (let i = 0; i < bytes.length; i++) {
    crc = CRC_TABLE[(crc ^ bytes[i]) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function buildZipStore(entries: Array<{ name: string; content: string }>): Uint8Array {
  const enc = new TextEncoder();
  const localParts: Uint8Array[] = [];
  const centralParts: Uint8Array[] = [];
  let offset = 0;

  for (const entry of entries) {
    const nameBytes = enc.encode(entry.name);
    const dataBytes = enc.encode(entry.content);
    const crc = crc32(dataBytes);

    const local = new Uint8Array(30 + nameBytes.length + dataBytes.length);
    const lv = new DataView(local.buffer);
    lv.setUint32(0, 0x04034b50, true);
    lv.setUint16(4, 20, true);
    lv.setUint16(6, 0x0800, true); // UTF-8
    lv.setUint16(8, 0, true); // Store
    lv.setUint16(10, 0, true);
    lv.setUint16(12, 0x5421, true);
    lv.setUint32(14, crc, true);
    lv.setUint32(18, dataBytes.length, true);
    lv.setUint32(22, dataBytes.length, true);
    lv.setUint16(26, nameBytes.length, true);
    lv.setUint16(28, 0, true);
    local.set(nameBytes, 30);
    local.set(dataBytes, 30 + nameBytes.length);
    localParts.push(local);

    const central = new Uint8Array(46 + nameBytes.length);
    const cv = new DataView(central.buffer);
    cv.setUint32(0, 0x02014b50, true);
    cv.setUint16(4, 20, true);
    cv.setUint16(6, 20, true);
    cv.setUint16(8, 0x0800, true);
    cv.setUint16(10, 0, true);
    cv.setUint16(12, 0, true);
    cv.setUint16(14, 0x5421, true);
    cv.setUint32(16, crc, true);
    cv.setUint32(20, dataBytes.length, true);
    cv.setUint32(24, dataBytes.length, true);
    cv.setUint16(28, nameBytes.length, true);
    cv.setUint16(30, 0, true);
    cv.setUint16(32, 0, true);
    cv.setUint16(34, 0, true);
    cv.setUint16(36, 0, true);
    cv.setUint32(38, 0, true);
    cv.setUint32(42, offset, true);
    central.set(nameBytes, 46);
    centralParts.push(central);

    offset += local.length;
  }

  const centralSize = centralParts.reduce((s, p) => s + p.length, 0);
  const eocd = new Uint8Array(22);
  const ev = new DataView(eocd.buffer);
  ev.setUint32(0, 0x06054b50, true);
  ev.setUint16(4, 0, true);
  ev.setUint16(6, 0, true);
  ev.setUint16(8, entries.length, true);
  ev.setUint16(10, entries.length, true);
  ev.setUint32(12, centralSize, true);
  ev.setUint32(16, offset, true);
  ev.setUint16(20, 0, true);

  const total = offset + centralSize + eocd.length;
  const archive = new Uint8Array(total);
  let pos = 0;
  for (const part of [...localParts, ...centralParts, eocd]) {
    archive.set(part, pos);
    pos += part.length;
  }
  return archive;
}

const TEMPLATE_HEADERS = [
  'รหัสสินค้า',
  'ชื่อสินค้า',
  'บาร์โค้ด',
  'รหัสหน่วย',
  'ชื่อหน่วย',
  'ราคาปกติ',
  'วันเริ่มมีผล (YYYY-MM-DD)',
  'วันสิ้นสุด (YYYY-MM-DD)',
  'ราคาโปรโมชัน',
  'ชื่อโปรโมชัน',
  'วันเริ่มโปรฯ (YYYY-MM-DD)',
  'วันหมดโปรฯ (YYYY-MM-DD)',
  'จำนวนพิมพ์',
];

export function buildMasterXlsxTemplate(): Blob {
  return buildXlsxTemplate(TEMPLATE_HEADERS);
}

export function buildPrintXlsxTemplate(): Blob {
  return buildXlsxTemplate(['รหัสสินค้า', 'ชื่อสินค้า', 'บาร์โค้ด', 'หน่วย', 'ราคาปกติ', 'ราคาโปรโมชัน', 'จำนวนดวง']);
}

function buildXlsxTemplate(headers: string[]): Blob {
  const headerCells = headers
    .map((header, index) => `<c r="${colLetter(index)}1" t="inlineStr"><is><t>${xmlEscape(header)}</t></is></c>`)
    .join('');

  const sheetXml =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
    '<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">' +
    `<cols><col min="${headers.indexOf('บาร์โค้ด') + 1}" max="${headers.indexOf('บาร์โค้ด') + 1}" width="22" customWidth="1" style="1"/></cols>` +
    `<sheetData><row r="1">${headerCells}</row></sheetData>` +
    '</worksheet>';

  const contentTypes =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
    '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">' +
    '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>' +
    '<Default Extension="xml" ContentType="application/xml"/>' +
    '<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>' +
    '<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>' +
    '<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>' +
    '</Types>';

  const rootRels =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
    '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
    '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>' +
    '</Relationships>';

  const workbookXml =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
    '<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">' +
    '<sheets><sheet name="MasterTemplate" sheetId="1" r:id="rId1"/></sheets>' +
    '</workbook>';

  const wbRels =
    '<?xml version="1.0" encoding="UTF-8" standalone="yes"?>' +
    '<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">' +
    '<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>' +
    '<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>' +
    '</Relationships>';

  const zipBytes = buildZipStore([
    { name: '[Content_Types].xml', content: contentTypes },
    { name: '_rels/.rels', content: rootRels },
    { name: 'xl/workbook.xml', content: workbookXml },
    { name: 'xl/_rels/workbook.xml.rels', content: wbRels },
    { name: 'xl/styles.xml', content: '<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><fonts count="1"><font/></fonts><fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills><borders count="1"><border/></borders><cellStyleXfs count="1"><xf/></cellStyleXfs><cellXfs count="2"><xf xfId="0"/><xf xfId="0" numFmtId="49" applyNumberFormat="1"/></cellXfs><cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles></styleSheet>' },
    { name: 'xl/worksheets/sheet1.xml', content: sheetXml },
  ]);

  return new Blob([zipBytes.buffer as ArrayBuffer], {
    type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
  });
}

export function buildMasterJsonTemplate(): Blob {
  const today = localDay();
  const sample = [
    {
      FTPdtCode: 'PDT0001',
      FTPdtName: 'กาแฟคั่วบด 250 กรัม',
      FTBarCode: '8850123456789',
      FTPunCode: 'PCS',
      FTPunName: 'ชิ้น',
      BasePrice: 159.0,
      FDPghDStart: today,
      FDPghDStop: '2099-12-31',
      PriceOff: null,
      PromoName: '',
      FDPmhDStart: '',
      FDPmhDStop: '',
      Qty: 1,
    },
    {
      FTPdtCode: 'PDT0002',
      FTPdtName: 'ชาเขียวพร้อมดื่ม 500 มล.',
      FTBarCode: '8850987654321',
      FTPunCode: 'BTL',
      FTPunName: 'ขวด',
      BasePrice: 25.0,
      FDPghDStart: today,
      FDPghDStop: '2099-12-31',
      PriceOff: 19.0,
      PromoName: 'โปรโมชันประจำเดือน',
      FDPmhDStart: today,
      FDPmhDStop: today,
      Qty: 2,
    },
  ];
  return new Blob([JSON.stringify(sample, null, 2)], { type: 'application/json;charset=utf-8' });
}

export function triggerFileDownload(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  a.remove();
  window.setTimeout(() => URL.revokeObjectURL(url), 1000);
}

async function inflateRaw(compressed: Uint8Array): Promise<Uint8Array> {
  if (typeof DecompressionStream === 'undefined') {
    throw new Error('เบราว์เซอร์/Runtime นี้ไม่รองรับการแตกไฟล์ .xlsx อัตโนมัติ');
  }
  const ds = new DecompressionStream('deflate-raw');
  const stream = new Blob([compressed.slice().buffer]).stream().pipeThrough(ds);
  const buf = await new Response(stream).arrayBuffer();
  return new Uint8Array(buf);
}

async function readZipEntries(buffer: ArrayBuffer): Promise<Map<string, string>> {
  const bytes = new Uint8Array(buffer);
  const view = new DataView(buffer);
  const dec = new TextDecoder('utf-8');
  const out = new Map<string, string>();

  // Locate End of Central Directory record for accurate compressed sizes
  let eocdOffset = -1;
  for (let i = bytes.length - 22; i >= Math.max(0, bytes.length - 65558); i--) {
    if (view.getUint32(i, true) === 0x06054b50) {
      eocdOffset = i;
      break;
    }
  }
  if (eocdOffset < 0) {
    throw new Error('ไฟล์ Excel (.xlsx) ไม่ถูกต้องหรือเสียหาย');
  }

  const totalEntries = view.getUint16(eocdOffset + 10, true);
  let cdOffset = view.getUint32(eocdOffset + 16, true);

  for (let idx = 0; idx < totalEntries; idx++) {
    if (cdOffset + 46 > bytes.length || view.getUint32(cdOffset, true) !== 0x02014b50) {
      break;
    }
    const method = view.getUint16(cdOffset + 10, true);
    const compSize = view.getUint32(cdOffset + 20, true);
    const nameLen = view.getUint16(cdOffset + 28, true);
    const extraLen = view.getUint16(cdOffset + 30, true);
    const commentLen = view.getUint16(cdOffset + 32, true);
    const localOffset = view.getUint32(cdOffset + 42, true);
    const name = dec.decode(bytes.subarray(cdOffset + 46, cdOffset + 46 + nameLen));
    cdOffset += 46 + nameLen + extraLen + commentLen;

    if (localOffset + 30 > bytes.length || view.getUint32(localOffset, true) !== 0x04034b50) {
      continue;
    }
    const lNameLen = view.getUint16(localOffset + 26, true);
    const lExtraLen = view.getUint16(localOffset + 28, true);
    const dataStart = localOffset + 30 + lNameLen + lExtraLen;
    const rawSlice = bytes.subarray(dataStart, dataStart + compSize);

    if (name.endsWith('.xml') || name.endsWith('.rels')) {
      const decoded = method === 0 ? rawSlice : method === 8 ? await inflateRaw(rawSlice) : null;
      if (decoded) {
        out.set(name, dec.decode(decoded));
      }
    }
  }

  return out;
}

export async function parseXlsxRows(buffer: ArrayBuffer): Promise<string[][]> {
  const files = await readZipEntries(buffer);
  const sharedStrings: string[] = [];
  const ssXml = files.get('xl/sharedStrings.xml');
  const parser = new DOMParser();

  if (ssXml) {
    const doc = parser.parseFromString(ssXml, 'text/xml');
    doc.querySelectorAll('si').forEach((si) => {
      const parts = Array.from(si.querySelectorAll('t')).map((t) => t.textContent || '');
      sharedStrings.push(parts.join(''));
    });
  }

  const sheetPath =
    Array.from(files.keys()).find((k) => /^xl\/worksheets\/sheet1\.xml$/i.test(k)) ||
    Array.from(files.keys()).find((k) => /^xl\/worksheets\/sheet.*\.xml$/i.test(k));
  if (!sheetPath) {
    throw new Error('ไม่พบข้อมูล Worksheet ในไฟล์ Excel (.xlsx)');
  }

  const sheetDoc = parser.parseFromString(files.get(sheetPath) || '', 'text/xml');
  const rows: string[][] = [];
  sheetDoc.querySelectorAll('sheetData > row').forEach((rowEl) => {
    const rowCols: string[] = [];
    rowEl.querySelectorAll('c').forEach((cellEl) => {
      const ref = cellEl.getAttribute('r') || '';
      const colIdx = ref ? colIndexFromRef(ref) : rowCols.length;
      const type = cellEl.getAttribute('t') || '';
      let val = '';
      if (type === 's') {
        const idx = Number(cellEl.querySelector('v')?.textContent || '-1');
        val = sharedStrings[idx] ?? '';
      } else if (type === 'inlineStr') {
        val = Array.from(cellEl.querySelectorAll('is t'))
          .map((t) => t.textContent || '')
          .join('');
      } else {
        val = cellEl.querySelector('v')?.textContent ?? '';
      }
      while (rowCols.length <= colIdx) rowCols.push('');
      rowCols[colIdx] = val.trim();
    });
    if (rowCols.some(Boolean)) {
      rows.push(rowCols);
    }
  });
  return rows;
}

function normalizeKey(raw: string): string {
  return raw.replace(/\s*\([^)]*\)\s*$/u, '').replace(/ฯ/g, '').trim().toLowerCase().replace(/[^a-z0-9ก-๙]/g, '');
}

function rowToMasterObject(headers: string[], cols: string[]): Record<string, unknown> {
  const obj: Record<string, unknown> = {};
  headers.forEach((h, i) => {
    obj[h] = cols[i] ?? '';
  });
  return obj;
}

function pickField(obj: Record<string, unknown>, candidates: string[]): string {
  const normalizedEntries = Object.entries(obj).map(([k, v]) => [normalizeKey(k), v] as const);
  for (const cand of candidates) {
    const target = normalizeKey(cand);
    const hit = normalizedEntries.find(([k]) => k === target);
    if (hit && hit[1] !== undefined && hit[1] !== null && String(hit[1]).trim() !== '') {
      return String(hit[1]).trim();
    }
  }
  return '';
}

function localDay(): string {
  const d = new Date();
  return [d.getFullYear(), String(d.getMonth()+1).padStart(2,'0'), String(d.getDate()).padStart(2,'0')].join('-');
}

function importDate(raw: string, row: number, excel: boolean): string {
  if (!raw) return '';
  if (excel && /^\d{1,5}$/.test(raw) && Number(raw)>60) {
    return new Date(Date.UTC(1899,11,30)+Number(raw)*86400000).toISOString().slice(0,10);
  }
  const d = new Date(raw+'T00:00:00Z');
  if (!/^\d{4}-\d{2}-\d{2}$/.test(raw) || !Number.isFinite(d.valueOf()) || d.toISOString().slice(0,10)!==raw)
    throw new Error(`แถวที่ ${row}: วันที่ ${raw} ไม่ถูกต้อง`);
  return raw;
}

function mapObjectsToMaster(records: Array<Record<string, unknown>>, excel = false): MasterImportRow[] {
  const today = localDay();
  const out: MasterImportRow[] = [];
  records.forEach((rec, i) => {
    const pdtCode = pickField(rec, ['ftpdtcode', 'pdtcode', 'รหัสสินค้า', 'sku', 'productcode']);
    const barCode = pickField(rec, ['ftbarcode', 'barcode', 'บาร์โค้ด']);
    const pdtName = pickField(rec, ['ftpdtname', 'pdtname', 'ชื่อสินค้า', 'name']) || pdtCode || barCode;
    const baseStr = pickField(rec, ['baseprice', 'fcpgdpriceret', 'fcpdtpriceret', 'saleprice', 'price', 'ราคาปกติ', 'ราคาขาย', 'ราคา']);
    const basePrice = Number(baseStr.replace(/,/g, ''));
    if (!barCode) {
      throw new Error(`แถวที่ ${i + 1}: ไม่พบบาร์โค้ด`);
    }
    if (!baseStr || !Number.isFinite(basePrice) || basePrice < 0) {
      throw new Error(`แถวที่ ${i + 1} (${barCode}): ราคาปกติไม่ถูกต้อง`);
    }
    const promoStr = pickField(rec, ['priceoff', 'promoprice', 'ราคาโปรโมชัน', 'ราคาโปร']);
    const promoParsed = promoStr ? Number(promoStr.replace(/,/g, '')) : NaN;
    if (promoStr && (!Number.isFinite(promoParsed) || promoParsed < 0)) throw new Error(`แถวที่ ${i+1}: ราคาโปรไม่ถูกต้อง`);
    const qtyRaw = pickField(rec, ['qty', 'quantity', 'fnplbqty', 'จำนวนพิมพ์', 'จำนวน']);
    const quantity = qtyRaw ? Math.max(1, Math.min(10000, Math.trunc(Number(qtyRaw) || 1))) : 1;

    out.push({
      pdtCode: pdtCode || barCode,
      pdtName,
      barCode,
      unitCode: pickField(rec, ['ftpuncode', 'unitcode', 'รหัสหน่วย']) || 'PCS',
      unitName: pickField(rec, ['ftpunname', 'unitname', 'ชื่อหน่วย', 'หน่วย']) || 'ชิ้น',
      basePrice,
      effectiveStart: importDate(pickField(rec, ['fdpghdstart', 'effectivestart', 'วันเริ่มมีผล']), i+1, excel) || today,
      effectiveStop: importDate(pickField(rec, ['fdpghdstop', 'effectivestop', 'วันสิ้นสุด']), i+1, excel) || '2099-12-31',
      promoPrice: Number.isFinite(promoParsed) && promoParsed >= 0 ? promoParsed : undefined,
      promoName: pickField(rec, ['promoname', 'ftpmhname', 'ชื่อโปรโมชัน']),
      promoStart: importDate(pickField(rec, ['fdpmhdstart', 'promostart', 'วันเริ่มโปร']), i+1, excel),
      promoStop: importDate(pickField(rec, ['fdpmhdstop', 'promostop', 'วันหมดโปร']), i+1, excel),
      quantity,
    });
  });
  return out;
}

function printWorksheet(headers: string[], rows: string[][]): ParsedImportResult {
  const barcodeRows = rows.map((cols, i) => {
    const rowNumber = i + 2;
    const record = rowToMasterObject(headers, cols);
    const field = (...names: string[]) => pickField(record, names);
    const barcode = field('บาร์โค้ด', 'barcode', 'ftbarcode');
    const quantity = Number(field('จำนวนดวง', 'จำนวนพิมพ์', 'จำนวน', 'qty', 'quantity', 'fnplbqty') || 1);
    if (!barcode || !Number.isInteger(quantity) || quantity < 1 || quantity > 10000)
      throw new Error(`แถวที่ ${rowNumber}: ต้องระบุบาร์โค้ดและจำนวนเต็ม 1–10,000`);
    const price = (label: string, ...aliases: string[]) => {
      const raw = field(label, ...aliases);
      if (!raw) return undefined;
      const value = Number(raw.replace(/,/g, ''));
      if (!Number.isFinite(value) || value < 0) throw new Error(`แถวที่ ${rowNumber}: ${label}ไม่ถูกต้อง`);
      return value;
    };
    const fields = {
      code: field('รหัสสินค้า', 'ftpdtcode', 'sku'), name: field('ชื่อสินค้า', 'ftpdtname', 'name'),
      unit: field('หน่วย', 'ชื่อหน่วย', 'ftpunname'), normal: price('ราคาปกติ', 'baseprice', 'price'),
      promo: price('ราคาโปรโมชัน', 'ราคาโปร', 'priceoff', 'promoprice'),
    };
    return {barcode, quantity, rowNumber, fileFields: Object.values(fields).some(v => v !== '' && v !== undefined) ? fields : undefined};
  });
  if (!barcodeRows.length) throw new Error('กรุณากรอกข้อมูลสินค้าในไฟล์ก่อนนำเข้า');
  return {kind: 'barcode_only', masterRows: [], barcodeRows};
}

export async function parseMasterOrPrintFile(file: File, target?: 'print' | 'master'): Promise<ParsedImportResult> {
  const lower = file.name.toLowerCase();
  if (lower.endsWith('.json')) {
    const parsed = JSON.parse(await file.text());
    const arr = Array.isArray(parsed)
      ? parsed
      : Array.isArray(parsed?.items)
        ? parsed.items
        : Array.isArray(parsed?.raPdt)
          ? parsed.raPdt
          : null;
    if (!arr || !arr.length) {
      throw new Error('ไฟล์ JSON ต้องเป็น Array ของรายการสินค้า หรือมีฟิลด์ items');
    }
    const hasPrice = arr.some((item: Record<string, unknown>) =>
      Object.keys(item).some((k) => {
        const nk = normalizeKey(k);
        return nk.includes('price') || nk.includes('ราคา') || nk.includes('fcpgd');
      })
    );
    if (hasPrice) {
      const masterRows = mapObjectsToMaster(arr);
      return { kind: 'master', masterRows, barcodeRows: masterRows.map((r) => ({ barcode: r.barCode, quantity: r.quantity })) };
    }
    const barcodeRows = arr.map((item: Record<string, unknown>, idx: number) => {
      const barcode = pickField(item, ['ftbarcode', 'barcode', 'บาร์โค้ด']);
      const quantity = Math.max(1, Math.min(10000, Math.trunc(Number(pickField(item, ['qty', 'quantity', 'จำนวน']) || '1') || 1)));
      if (!barcode) throw new Error(`JSON รายการที่ ${idx + 1}: ไม่มีบาร์โค้ด`);
      return { barcode, quantity };
    });
    return { kind: 'barcode_only', masterRows: [], barcodeRows };
  }

  if (lower.endsWith('.xlsx')) {
    const rows = await parseXlsxRows(await file.arrayBuffer());
    if (!rows.length) throw new Error('ไม่พบข้อมูลในไฟล์ Excel');
    const headers = rows[0];
    if (target === 'print' && headers.some(h => ['บาร์โค้ด', 'barcode', 'ftbarcode'].includes(normalizeKey(h)))) return printWorksheet(headers, rows.slice(1));
    const hasMasterHeader = headers.some((h) => {
      const nh = normalizeKey(h);
      return nh.includes('baseprice') || nh.includes('ราคา') || nh.includes('ftpdtcode') || nh.includes('pdtcode');
    });
    if (hasMasterHeader && rows.length > 1) {
      const objs = rows.slice(1).map((cols) => rowToMasterObject(headers, cols));
      const masterRows = mapObjectsToMaster(objs, true);
      return { kind: 'master', masterRows, barcodeRows: masterRows.map((r) => ({ barcode: r.barCode, quantity: r.quantity })) };
    }
    const hasBarcodeHeader = headers.some((h) => ['barcode', 'ftbarcode', 'บาร์โค้ด'].includes(normalizeKey(h)));
    const bIdx = hasBarcodeHeader ? headers.findIndex((h) => ['barcode', 'ftbarcode', 'บาร์โค้ด'].includes(normalizeKey(h))) : 0;
    const qIdx = hasBarcodeHeader ? headers.findIndex((h) => ['qty', 'quantity', 'fnplbqty', 'จำนวน', 'จำนวนพิมพ์', 'จำนวนดวง'].includes(normalizeKey(h))) : 1;
    const data = hasBarcodeHeader ? rows.slice(1) : rows;
    const barcodeRows = data.map((cols, idx) => {
      const barcode = (cols[bIdx] || '').trim();
      const quantity = qIdx >= 0 && cols[qIdx] ? Number(cols[qIdx]) : 1;
      if (!barcode || !Number.isInteger(quantity) || quantity < 1 || quantity > 10000) {
        throw new Error(`Excel แถวที่ ${idx + (hasBarcodeHeader ? 2 : 1)}: ต้องระบุบาร์โค้ดและจำนวนเต็ม 1–10,000`);
      }
      return { barcode, quantity };
    });
    return { kind: 'barcode_only', masterRows: [], barcodeRows };
  }

  // Fallback: CSV / TXT
  const text = (await file.text()).replace(/^\uFEFF/, '');
  const firstLine = text.split(/\r?\n/, 1)[0] || '';
  const delimiter = ['\t', ';', ','].sort((a, b) => firstLine.split(b).length - firstLine.split(a).length)[0];
  const rawRows: string[][] = [];
  let row: string[] = [];
  let cell = '';
  let quoted = false;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (ch === '"') {
      if (quoted && text[i + 1] === '"') {
        cell += '"';
        i++;
      } else {
        quoted = !quoted;
      }
    } else if (!quoted && (ch === delimiter || ch === '\n' || ch === '\r')) {
      row.push(cell.trim());
      cell = '';
      if (ch !== delimiter) {
        if (row.some(Boolean)) rawRows.push(row);
        row = [];
        if (ch === '\r' && text[i + 1] === '\n') i++;
      }
    } else {
      cell += ch;
    }
  }
  row.push(cell.trim());
  if (row.some(Boolean)) rawRows.push(row);
  if (!rawRows.length) throw new Error('ไม่พบข้อมูลในไฟล์');

  const headers = rawRows[0];
  if (target === 'print' && headers.some(h => ['บาร์โค้ด', 'barcode', 'ftbarcode'].includes(normalizeKey(h)))) return printWorksheet(headers, rawRows.slice(1));
  const hasMasterHeader = headers.some((h) => {
    const nh = normalizeKey(h);
    return nh.includes('baseprice') || nh.includes('ราคา') || nh.includes('ftpdtcode');
  });
  if (hasMasterHeader && rawRows.length > 1) {
    const objs = rawRows.slice(1).map((cols) => rowToMasterObject(headers, cols));
    const masterRows = mapObjectsToMaster(objs);
    return { kind: 'master', masterRows, barcodeRows: masterRows.map((r) => ({ barcode: r.barCode, quantity: r.quantity })) };
  }

  const barcodeIdx = headers.findIndex((h) => ['barcode', 'ftbarcode', 'บาร์โค้ด'].includes(normalizeKey(h)));
  const qtyIdx = headers.findIndex((h) => ['qty', 'quantity', 'fnplbqty', 'จำนวน', 'จำนวนดวง'].includes(normalizeKey(h)));
  const data = barcodeIdx >= 0 ? rawRows.slice(1) : rawRows;
  const barcodeRows = data.map((cols, idx) => {
    const barcode = (cols[barcodeIdx >= 0 ? barcodeIdx : 0] || '').trim();
    const rawQty = cols[barcodeIdx >= 0 ? qtyIdx : 1];
    const quantity = rawQty === undefined || rawQty === '' ? 1 : Number(rawQty);
    if (!barcode || !Number.isInteger(quantity) || quantity < 1 || quantity > 10000) {
      throw new Error(`ข้อมูลแถวที่ ${idx + (barcodeIdx >= 0 ? 2 : 1)} ไม่ถูกต้อง`);
    }
    return { barcode, quantity };
  });
  return { kind: 'barcode_only', masterRows: [], barcodeRows };
}
