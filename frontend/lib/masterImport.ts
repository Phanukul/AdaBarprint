export interface MasterImportRow {
  sourceRow: number;
  pdtCode: string;
  pdtName: string;
  barCode: string;
  unitCode: string;
  unitName: string;
  unitFactor?: number;
  basePrice?: number;
  effectiveStart: string;
  effectiveStop: string;
  promoPrice?: number;
  promoName: string;
  promoStart: string;
  promoStop: string;
}

function colIndexFromRef(cellRef: string): number {
  const letters = cellRef.replace(/[^A-Za-z]/g, '').toUpperCase();
  let idx = 0;
  for (let i = 0; i < letters.length; i++) {
    idx = idx * 26 + (letters.charCodeAt(i) - 64);
  }
  return Math.max(0, idx - 1);
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
      UnitFactor: 1,
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
      UnitFactor: 1,
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

type WorksheetRow = { number: number; cells: string[] };

export async function parseXlsxRows(buffer: ArrayBuffer): Promise<string[][]> {
  return (await readMasterSheets(buffer)).products.map(row => row.cells);
}

async function readMasterSheets(buffer: ArrayBuffer) {
  const files = await readZipEntries(buffer);
  const parser = new DOMParser();
  const xml = (path: string) => parser.parseFromString(files.get(path) || '', 'text/xml');
  const sharedStrings = Array.from(xml('xl/sharedStrings.xml').querySelectorAll('si'), si =>
    Array.from(si.querySelectorAll('t'), t => t.textContent || '').join(''));
  const relationships = Array.from(xml('xl/_rels/workbook.xml.rels').querySelectorAll('Relationship'));
  const sheets = Array.from(xml('xl/workbook.xml').querySelectorAll('sheets > sheet'));
  function read(sheet: Element) {
    const id = sheet.getAttributeNS('http://schemas.openxmlformats.org/officeDocument/2006/relationships', 'id');
    const target = relationships.find(rel => rel.getAttribute('Id') === id)?.getAttribute('Target');
    const path = target && new URL(target, 'https://workbook.invalid/xl/workbook.xml').pathname.slice(1);
    if (!path || !files.has(path)) throw new Error('อ่านแท็บ ' + sheet.getAttribute('name') + ' ไม่สำเร็จ');
    return worksheetRows(xml(path), sharedStrings);
  }
  const named = (name: string) => sheets.find(sheet => sheet.getAttribute('name') === name);
  const product = named('สินค้าและราคา') || (sheets.length === 1 ? sheets[0] : undefined);
  if (!product) throw new Error('ไม่พบแท็บ สินค้าและราคา กรุณาใช้แม่แบบ Master ของ AdaBarprint');
  const unit = named('หน่วยขาย');
  return { products: read(product), units: unit ? read(unit) : [] };
}

function worksheetRows(sheetDoc: Document, sharedStrings: string[]): WorksheetRow[] {
const rows: WorksheetRow[] = [];
  sheetDoc.querySelectorAll('sheetData > row').forEach((rowEl, index) => {
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
      rows.push({ number: Number(rowEl.getAttribute('r')) || index + 1, cells: rowCols });
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

function mapObjectsToMaster(records: Array<Record<string, unknown>>, excel = false, rowNumbers?: number[]): MasterImportRow[] {
  const today = localDay();
  const out: MasterImportRow[] = [];
  records.forEach((rec, i) => {
    const sourceRow = rowNumbers?.[i] ?? i + 1;
    const pdtCode = pickField(rec, ['ftpdtcode', 'pdtcode', 'รหัสสินค้า', 'sku', 'productcode']);
    const barCode = pickField(rec, ['ftbarcode', 'barcode', 'บาร์โค้ด']);
    const pdtName = pickField(rec, ['ftpdtname', 'pdtname', 'ชื่อสินค้า', 'name']);
    const baseStr = pickField(rec, ['baseprice', 'fcpgdpriceret', 'fcpdtpriceret', 'saleprice', 'price', 'ราคาปกติ', 'ราคาขาย', 'ราคา']);
    const basePrice = Number(baseStr.replace(/,/g, ''));
    if (!barCode) {
      throw new Error(`แถวที่ ${sourceRow}: ไม่พบบาร์โค้ด`);
    }
    if (baseStr && (!Number.isFinite(basePrice) || basePrice < 0)) {
      throw new Error(`แถวที่ ${sourceRow} (${barCode}): ราคาปกติไม่ถูกต้อง`);
    }
    const promoStr = pickField(rec, ['priceoff', 'promoprice', 'ราคาโปรโมชัน', 'ราคาโปร']);
    const promoParsed = promoStr ? Number(promoStr.replace(/,/g, '')) : NaN;
    if (promoStr && (!Number.isFinite(promoParsed) || promoParsed < 0)) throw new Error(`แถวที่ ${sourceRow}: ราคาโปรไม่ถูกต้อง`);
    if (!pdtCode) throw new Error(`แถวที่ ${sourceRow}: กรุณาระบุรหัสสินค้า`);
    const factor = pickField(rec, ['ตัวคูณหน่วย', 'unitfactor', 'fcpdtunitfact']);
    if (factor && (!Number.isFinite(Number(factor)) || Number(factor) <= 0)) throw new Error(`แถวที่ ${sourceRow}: ตัวคูณหน่วยต้องมากกว่า 0`);
    out.push({
      sourceRow,
      pdtCode,
      pdtName,
      barCode,
      unitCode: pickField(rec, ['ftpuncode', 'unitcode', 'รหัสหน่วย']),
      unitName: pickField(rec, ['ftpunname', 'unitname', 'ชื่อหน่วย', 'หน่วย']),
      unitFactor: factor ? Number(factor) : undefined,
      basePrice: baseStr ? basePrice : undefined,
      effectiveStart: importDate(pickField(rec, ['fdpghdstart', 'effectivestart', 'วันเริ่มมีผล']), sourceRow, excel) || today,
      effectiveStop: importDate(pickField(rec, ['fdpghdstop', 'effectivestop', 'วันสิ้นสุด']), sourceRow, excel) || '2099-12-31',
      promoPrice: Number.isFinite(promoParsed) && promoParsed >= 0 ? promoParsed : undefined,
      promoName: pickField(rec, ['promoname', 'ftpmhname', 'ชื่อโปรโมชัน']),
      promoStart: importDate(pickField(rec, ['fdpmhdstart', 'promostart', 'วันเริ่มโปร']), sourceRow, excel),
      promoStop: importDate(pickField(rec, ['fdpmhdstop', 'promostop', 'วันหมดโปร']), sourceRow, excel),
    });
  });
  return out;
}

function parseMasterTable(rows: WorksheetRow[], excel: boolean): MasterImportRow[] {
  const [headers, ...data] = rows;
  if (!headers || !data.length) throw new Error('ไม่พบรายการสินค้า กรุณากรอกข้อมูลในแม่แบบ Master ก่อนนำเข้า');
  return mapObjectsToMaster(data.map(row => rowToMasterObject(headers.cells, row.cells)), excel, data.map(row => row.number));
}

export async function parseMasterFile(file: File): Promise<MasterImportRow[]> {
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
    return mapObjectsToMaster(arr);
  }
  if (lower.endsWith('.xlsx')) {
    const { products, units } = await readMasterSheets(await file.arrayBuffer());
    const result = parseMasterTable(products, true);
    const [header, ...entries] = units;
    const names = new Map<string, string>();
    for (const row of entries) {
      const fields = rowToMasterObject(header.cells, row.cells);
      const code = pickField(fields, ['รหัสหน่วย']);
      const name = pickField(fields, ['ชื่อหน่วย']);
      if (!code || !name) throw new Error(`แท็บหน่วยขาย แถวที่ ${row.number}: กรุณาระบุรหัสและชื่อหน่วย`);
      if (names.has(code) && names.get(code) !== name) {
        throw new Error(`แท็บหน่วยขาย แถวที่ ${row.number}: รหัส ${code} มีชื่อหน่วยไม่ตรงกัน`);
      }
      names.set(code, name);
    }
    for (const row of result) {
      row.unitName ||= names.get(row.unitCode) || '';
    }
    return result;
  }
  if (!lower.endsWith('.csv')) throw new Error('กรุณาเลือกไฟล์ Excel (.xlsx), JSON หรือ CSV');
  const text = (await file.text()).replace(/^\uFEFF/, '');
  const firstLine = text.split(/\r?\n/, 1)[0] || '';
  const delimiter = ['\t', ';', ','].sort((a, b) => firstLine.split(b).length - firstLine.split(a).length)[0];
  const rawRows: WorksheetRow[] = [];
  let recordNumber = 1;
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
        if (row.some(Boolean)) rawRows.push({ number: recordNumber, cells: row });
        recordNumber++;
        row = [];
        if (ch === '\r' && text[i + 1] === '\n') i++;
      }
    } else {
      cell += ch;
    }
  }
  row.push(cell.trim());
  if (row.some(Boolean)) rawRows.push({ number: recordNumber, cells: row });
  if (!rawRows.length) throw new Error('ไม่พบข้อมูลในไฟล์');

  return parseMasterTable(rawRows, false);
}
