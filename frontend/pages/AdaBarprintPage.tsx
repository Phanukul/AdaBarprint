import React, { useEffect, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';
import {
  Barcode,
  Calendar,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  ChevronUp,
  Download,
  FileUp,
  Loader2,
  Plus,
  Printer,
  RotateCcw,
  ScanLine,
  Search,
  Trash2,
  ArrowLeft,
  CheckCircle2,
  AlertTriangle,
  Settings2,
  Palette,
  PanelLeftClose,
  PanelLeftOpen,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import { Badge } from '@/components/ui/badge';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { SearchableSelect } from '@/components/ui/searchable-select';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { LabelPreview } from '@/components/LabelPreview';
import { LabelDesigner, type SaveAsPayload, type SheetConfigMeta } from '@/components/LabelDesigner';
import {
  SettingsPanel,
  type ConfigLabelSizeRow,
  type PrinterServerRow,
  type SysOptionRow,
} from '@/components/SettingsPanel';
import {
  PAPER_SIZE_PRESETS,
  type CompanyInfoRecord,
  type LabelDesignLayout,
  type LabelFieldRecord,
  type PaperSizeCode,
  type PaperSizeRecord,
  type PreprintRecord,
} from '@/types/design';
import type { PrinterConfig } from '@/types/printer';

interface BranchItem {
  FTBchCode: string;
  FTBchName: string | null;
}

interface LabelSizeItem {
  FTLbsCode: string;
  FTLbsGroup?: string;
  FTLbsName: string | null;
  FCLbsWidth: number;
  FCLbsHeight: number;
  FNLbsCol?: number | null;
  FNLbsRow?: number | null;
  FCLbsMagTop?: number | null;
  FCLbsMagLeft?: number | null;
  FCLbsMagBottom?: number | null;
  FCLbsMagRight?: number | null;
  FCLbsGapH?: number | null;
  FCLbsGapV?: number | null;
  FNLbsQtyPageNor?: number | null;
  FNLbsQtyPagePmt?: number | null;
  FTLbsHhtCode?: string | null;
  FTLbsStaUse: string;
  FTLbsPaperSize?: string | null;
  FCLbsPaperWidth?: number | null;
  FCLbsPaperHeight?: number | null;
  FCLbsOffsetX?: number | null;
  FCLbsOffsetY?: number | null;
  templates?: Array<{
    FTLtmCode: string;
    FTLtmName: string;
    FTLtmPriType: string;
    FTLtmLayout?: string | null;
  }>;
}

interface ProductItem {
  FTPdtCode: string;
  FTPdtName: string;
  FTPdtNameOth?: string | null;
  FTPdtBrand?: string | null;
  FTPdtCgyName?: string | null;
  FTPdtUnit?: string | null;
  FCPdtPrice?: number | null;
  FTPdtImg?: string | null;
  bars?: Array<{ FTPdbBarCode: string }>;
}

interface ScanTmpRow {
  seqNo: number;
  pdtCode: string;
  pdtName: string;
  barCode: string;
  qty: number;
  selected: boolean;
}

interface PrintQueueRow {
  pdtCode: string;
  pdtName: string;
  barCode: string;
  price: number;
  priType: '1' | '2'; // 1=Normal, 2=Promotion
  qty: number;
  selected: boolean;
  lbsCode: string;
  tmplCode: string;
  ctx: Record<string, any>;
  promoName?: string;
  statusText: string;
}

interface WindowsPrinterInfo {
  name: string;
  isDefault?: boolean;
  trays?: string[];
}

interface LabelPrinterMapping {
  lbsCode: string;
  lbsName: string;
  lbsGroup: string;
  normalPrinter: string;
  normalTray: string;
  normalProtocol: string;
  promoPrinter: string;
  promoTray: string;
  promoProtocol: string;
}

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

async function capturePagesToPngBase64(
  containerId: string,
  paperWidthMm = 210,
  paperHeightMm = 297
): Promise<string[]> {
  const container = document.getElementById(containerId);
  if (!container) return [];
  const sheetEls = Array.from(container.querySelectorAll<HTMLElement>('[data-capture-sheet="1"]'));
  const wPx = Math.max(200, Math.round((paperWidthMm / 25.4) * 96));
  const hPx = Math.max(200, Math.round((paperHeightMm / 25.4) * 96));
  const results: string[] = [];
  for (const el of sheetEls) {
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
    const scale = 2; // 192dpi crisp print
    const serialized = new XMLSerializer().serializeToString(clone);
    const svgXml = `<svg xmlns="http://www.w3.org/2000/svg" width="${wPx}" height="${hPx}"><foreignObject x="0" y="0" width="100%" height="100%">${serialized}</foreignObject></svg>`;
    const svgUrl = 'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(svgXml);
    const pngDataUrl = await new Promise<string>((resolve, reject) => {
      const image = new Image();
      image.onload = () => {
        const canvas = document.createElement('canvas');
        canvas.width = wPx * scale;
        canvas.height = hPx * scale;
        const ctx = canvas.getContext('2d');
        if (!ctx) {
          reject(new Error('Canvas context unavailable'));
          return;
        }
        ctx.fillStyle = '#FFFFFF';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.scale(scale, scale);
        ctx.drawImage(image, 0, 0, wPx, hPx);
        resolve(canvas.toDataURL('image/png'));
      };
      image.onerror = reject;
      image.src = svgUrl;
    });
    results.push(pngDataUrl);
  }
  return results;
}

function formatTrayLabel(tray: string): string {
  const t = tray.trim();
  if (!t) return '';
  const lower = t.toLowerCase();
  if (lower === 'automatically select' || lower === 'auto select') {
    return `${t} (โหมดให้เครื่องเลือกถาดอัตโนมัติ)`;
  }
  if (lower === 'mp tray' || lower === 'manual feed') {
    return `${t} (ช่องป้อนมือด้านหน้า)`;
  }
  if (lower === 'tray 1' || lower === 'upper') {
    return `${t} (ลิ้นชักกระดาษหลัก)`;
  }
  return t;
}

function isoToDisplayDate(iso: string): string {
  if (!iso || !/^\d{4}-\d{2}-\d{2}$/.test(iso)) return '';
  const [y, m, d] = iso.split('-');
  return `${d}/${m}/${y}`;
}

function DateField({
  value,
  onChange,
  disabled = false,
  placeholder = 'dd/mm/yyyy',
}: {
  value: string;
  onChange: (val: string) => void;
  disabled?: boolean;
  placeholder?: string;
}) {
  const pickerRef = useRef<HTMLInputElement>(null);
  const [text, setText] = useState(() => isoToDisplayDate(value));

  useEffect(() => {
    setText(isoToDisplayDate(value));
  }, [value]);

  const handleTextChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const next = e.target.value;
    setText(next);
    if (!next.trim()) {
      onChange('');
      return;
    }
    const m = next.trim().match(/^(\d{2})\/(\d{2})\/(\d{4})$/);
    if (m) {
      const [, dd, mm, yyyy] = m;
      const iso = `${yyyy}-${mm}-${dd}`;
      const dt = new Date(`${iso}T00:00:00`);
      if (!Number.isNaN(dt.getTime())) {
        onChange(iso);
      }
    }
  };

  const openPicker = () => {
    if (disabled || !pickerRef.current) return;
    if (typeof pickerRef.current.showPicker === 'function') {
      pickerRef.current.showPicker();
    } else {
      pickerRef.current.click();
    }
  };

  return (
    <div className="relative flex items-center w-full">
      <Input
        type="text"
        value={text}
        onChange={handleTextChange}
        onBlur={() => setText(isoToDisplayDate(value))}
        placeholder={placeholder}
        disabled={disabled}
        maxLength={10}
        className="h-8 text-xs pr-8"
      />
      <button
        type="button"
        onClick={openPicker}
        disabled={disabled}
        tabIndex={-1}
        aria-label="เลือกวันที่"
        className="absolute right-2 text-muted-foreground hover:text-foreground disabled:opacity-40 disabled:pointer-events-none"
      >
        <Calendar className="h-3.5 w-3.5" />
      </button>
      <input
        ref={pickerRef}
        type="date"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled}
        tabIndex={-1}
        className="sr-only"
      />
    </div>
  );
}

interface MasterOption {
  code: string;
  name: string;
}

function FilterSelect({
  value,
  onChange,
  options,
  placeholder = 'ทั้งหมด',
}: {
  value: string;
  onChange: (val: string) => void;
  options: MasterOption[];
  placeholder?: string;
}) {
  const searchOptions = React.useMemo(
    () => [
      { value: '__ALL__', label: placeholder },
      ...options.map((opt) => ({
        value: opt.code,
        label: opt.name,
        keywords: opt.code,
      })),
    ],
    [options, placeholder]
  );

  return (
    <SearchableSelect
      value={value || '__ALL__'}
      onValueChange={(v) => onChange(v === '__ALL__' ? '' : v)}
      options={searchOptions}
      placeholder={placeholder}
      searchPlaceholder="พิมพ์ค้นหา..."
      className="h-8 text-xs"
    />
  );
}

export default function AdaBarprintPage() {
  const [loading, setLoading] = useState(true);
  const [activeMenu, setActiveMenu] = useState<'print' | 'design' | 'settings'>('print');
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [customLayouts, setCustomLayouts] = useState<Record<string, LabelDesignLayout>>({});
  const [allConfigSizes, setAllConfigSizes] = useState<ConfigLabelSizeRow[]>([]);
  const [sysOptions, setSysOptions] = useState<SysOptionRow[]>([]);
  const [printerServers, setPrinterServers] = useState<PrinterServerRow[]>([]);
  const [settingsEditLbsCode, setSettingsEditLbsCode] = useState<string | null>(null);
  const [filterOpen, setFilterOpen] = useState(true);

  // Master data
  const [branches, setBranches] = useState<BranchItem[]>([]);
  const [labelSizes, setLabelSizes] = useState<LabelSizeItem[]>([]);
  const [paperSizes, setPaperSizes] = useState<PaperSizeRecord[]>([]);
  const [preprints, setPreprints] = useState<PreprintRecord[]>([]);
  const [labelFields, setLabelFields] = useState<LabelFieldRecord[]>([]);
  const [companyInfo, setCompanyInfo] = useState<CompanyInfoRecord | undefined>(undefined);
  const [printers, setPrinters] = useState<PrinterConfig[]>([]);
  const [catalogProducts, setCatalogProducts] = useState<ProductItem[]>([]);

  const sysOptValue = (code: string) =>
    sysOptions.find((o) => o.FTSopCode === code)?.FTSopValue?.trim() || '';

  const resolveDefaultBranch = () =>
    bchFrom || sysOptValue('DefaultBranch') || companyInfo?.defaultBchCode || branches[0]?.FTBchCode || '';

  const resolveDefaultLabelCode = (sizesList: LabelSizeItem[] = labelSizes) => {
    const optDef = sysOptValue('DefaultTemplate');
    if (optDef && sizesList.some((s) => s.FTLbsCode === optDef && s.FTLbsStaUse === '1')) {
      return optDef;
    }
    return sizesList.find((s) => s.FTLbsStaUse === '1')?.FTLbsCode || '';
  };

  const mergePrintersFromDbAndOs = (
    dbLanList: PrinterConfig[],
    winList: WindowsPrinterInfo[]
  ): PrinterConfig[] => {
    const merged: PrinterConfig[] = [...dbLanList];
    for (const wp of winList) {
      if (!merged.some((p) => p.name.toLowerCase() === wp.name.toLowerCase())) {
        merged.push({
          id: `win-${wp.name}`,
          name: wp.name,
          brand: 'SAMSUNG',
          model: wp.name,
          ip: 'Windows',
          port: 9100,
          dpi: 600,
          labelWidth: 210,
          labelHeight: 297,
          speed: 3,
          darkness: 10,
          enabled: true,
          availableTrays: wp.trays || [],
        });
      }
    }
    return merged;
  };

  // Left Filter Panel state (Version 5 style)
  const [sheetPriceType, setSheetPriceType] = useState<'Normal' | 'Promotion' | 'All'>('All');
  const [useEffectiveDate, setUseEffectiveDate] = useState(false);
  const [effectiveDate, setEffectiveDate] = useState(() => new Date().toISOString().slice(0, 10));
  const [docDateFrom, setDocDateFrom] = useState('');
  const [docDateTo, setDocDateTo] = useState('');
  const [docNoFrom, setDocNoFrom] = useState('');
  const [docNoTo, setDocNoTo] = useState('');
  const [bchFrom, setBchFrom] = useState('');
  const [bchTo, setBchTo] = useState('');
  const [pdtFrom, setPdtFrom] = useState('');
  const [pdtTo, setPdtTo] = useState('');
  const [pdtGrpFilter, setPdtGrpFilter] = useState('');
  const [pdtGrpTo, setPdtGrpTo] = useState('');
  const [pdtTypeFrom, setPdtTypeFrom] = useState('');
  const [pdtTypeTo, setPdtTypeTo] = useState('');
  const [pdtBrandFilter, setPdtBrandFilter] = useState('');
  const [pdtBrandTo, setPdtBrandTo] = useState('');
  const [pdtModelFrom, setPdtModelFrom] = useState('');
  const [pdtModelTo, setPdtModelTo] = useState('');
  const [agencyCode, setAgencyCode] = useState('');
  const [priceGroupCode, setPriceGroupCode] = useState('');
  const [defaultQty, setDefaultQty] = useState(1);

  // Master options for V5 filters (from AdaAccD.db)
  const [masterGroups, setMasterGroups] = useState<MasterOption[]>([]);
  const [masterTypes, setMasterTypes] = useState<MasterOption[]>([]);
  const [masterBrands, setMasterBrands] = useState<MasterOption[]>([]);
  const [masterModels, setMasterModels] = useState<MasterOption[]>([]);
  const [masterPriceGroups, setMasterPriceGroups] = useState<MasterOption[]>([]);
  const [masterAgencies, setMasterAgencies] = useState<MasterOption[]>([]);

  // Range Browse Modal (ปุ่มแว่นขยายแบบ V.5 — 5 คอลัมน์ + ค้นหา + แบ่งหน้า)
  const [rangeBrowseTarget, setRangeBrowseTarget] = useState<{
    title: string;
    onSelect: (code: string) => void;
  } | null>(null);
  const [rangeBrowseSearch, setRangeBrowseSearch] = useState('');
  const [rangeBrowseField, setRangeBrowseField] = useState<'all' | 'name' | 'code' | 'barcode'>('all');
  const [rangeBrowsePage, setRangeBrowsePage] = useState(1);
  const RANGE_BROWSE_PAGE_SIZE = 10;

  // Right Print Terms (default ว่าง ให้ผู้ใช้เลือกเองตามแบบ V.5)
  const [selectedLabelCode, setSelectedLabelCode] = useState('');
  const [designLabelCode, setDesignLabelCode] = useState('');
  const [selectedPrinterId, setSelectedPrinterId] = useState('');

  // Right Main Print Table
  const [mainSearch, setMainSearch] = useState('');
  const [mainRows, setMainRows] = useState<PrintQueueRow[]>([]);
  const [movingData, setMovingData] = useState(false);

  // Modal: Scan Product (dcmPrintBarEventGetPdtInTmp)
  const [scanModalOpen, setScanModalOpen] = useState(false);
  const [scanSearch, setScanSearch] = useState('');
  const [scanInputBarcode, setScanInputBarcode] = useState('');
  const [scanTmpRows, setScanTmpRows] = useState<ScanTmpRow[]>([]);
  const [scanPage, setScanPage] = useState(1);
  const SCAN_PAGE_SIZE = 10;

  // Modal: Browse Product (+)
  const [browseOpen, setBrowseOpen] = useState(false);
  const [browseSearch, setBrowseSearch] = useState('');
  const [browseList, setBrowseList] = useState<ProductItem[]>([]);
  const [browseLoading, setBrowseLoading] = useState(false);

  // Modal: Preview Scan Import (odvPrnPreviewModal)
  const [importPreviewOpen, setImportPreviewOpen] = useState(false);
  const [previewPass, setPreviewPass] = useState<Array<PrintQueueRow>>([]);
  const [previewFail, setPreviewFail] = useState<Array<{ pdtCode: string; barCode: string; pdtName: string; reason: string }>>([]);

  // Modal: Send Print Preview (odvPRNModalSendPrint — Normal / Promotion tabs)
  const [sendPrintOpen, setSendPrintOpen] = useState(false);
  const [printing, setPrinting] = useState(false);
  const [previewPreprintView, setPreviewPreprintView] = useState<'guide' | 'ink'>('guide');
  const [printOffsetX, setPrintOffsetX] = useState<number>(0);
  const [printOffsetY, setPrintOffsetY] = useState<number>(0);
  const [savingOffset, setSavingOffset] = useState(false);
  const [advancedFiltersOpen, setAdvancedFiltersOpen] = useState(false);
  const [sheetPreviewScale, setSheetPreviewScale] = useState<number>(0.45);

  // Modal: Import Excel / CSV (Version 5)
  const [importModalOpen, setImportModalOpen] = useState(false);
  const [importingFile, setImportingFile] = useState(false);

  // Driver Mapping (ตั้งค่าไดร์เวอร์เครื่องพิมพ์ — แยกปกติ / โปรโมชัน ตามรหัสฉลาก)
  const [windowsPrinters, setWindowsPrinters] = useState<WindowsPrinterInfo[]>([]);
  const [driverMappings, setDriverMappings] = useState<LabelPrinterMapping[]>([]);
  const [driverServerCode, setDriverServerCode] = useState('LOCAL');
  const [driverModalOpen, setDriverModalOpen] = useState(false);
  const [driverEditRow, setDriverEditRow] = useState<LabelPrinterMapping | null>(null);
  const [savingDriver, setSavingDriver] = useState(false);
  const [testingDriver, setTestingDriver] = useState<'normal' | 'promo' | null>(null);

  useEffect(() => {
    async function init() {
      try {
        const [bRes, lRes, pRes, fmRes, wpRes, lpRes, cfgRes] = await Promise.all([
          fetch('/api/branches'),
          fetch('/api/label-sizes'),
          fetch('/api/products?limit=200'),
          fetch('/api/filters/master').catch(() => null),
          fetch('/api/printers/windows').catch(() => null),
          fetch('/api/label-printers').catch(() => null),
          fetch('/api/config/all').catch(() => null),
        ]);
        const [bData, lData, pData] = await Promise.all([bRes.json(), lRes.json(), pRes.json()]);
        const fmData = fmRes ? await fmRes.json().catch(() => null) : null;
        const wpData = wpRes ? await wpRes.json().catch(() => ({ printers: [] })) : { printers: [] };
        const lpData = lpRes ? await lpRes.json().catch(() => null) : null;
        const cfgData = cfgRes ? await cfgRes.json().catch(() => null) : null;
        if (cfgData) {
          if (Array.isArray(cfgData.labelSizes)) setAllConfigSizes(cfgData.labelSizes);
          if (Array.isArray(cfgData.options)) setSysOptions(cfgData.options);
          if (Array.isArray(cfgData.servers)) setPrinterServers(cfgData.servers);
          if (Array.isArray(cfgData.paperSizes)) setPaperSizes(cfgData.paperSizes);
          if (Array.isArray(cfgData.preprints)) setPreprints(cfgData.preprints);
          if (Array.isArray(cfgData.labelFields)) setLabelFields(cfgData.labelFields);
          if (cfgData.company) setCompanyInfo(cfgData.company);
        }
        if (fmData) {
          if (Array.isArray(fmData.groups)) setMasterGroups(fmData.groups);
          if (Array.isArray(fmData.types)) setMasterTypes(fmData.types);
          if (Array.isArray(fmData.brands)) setMasterBrands(fmData.brands);
          if (Array.isArray(fmData.models)) setMasterModels(fmData.models);
          if (Array.isArray(fmData.priceGroups)) setMasterPriceGroups(fmData.priceGroups);
          if (Array.isArray(fmData.agencies)) setMasterAgencies(fmData.agencies);
        }
        const winList: WindowsPrinterInfo[] = Array.isArray(wpData?.printers) ? wpData.printers : [];
        setWindowsPrinters(winList);
        if (lpData) {
          if (lpData.serverCode) setDriverServerCode(lpData.serverCode);
          if (Array.isArray(lpData.mappings)) setDriverMappings(lpData.mappings);
        }
        if (Array.isArray(bData)) {
          setBranches(bData);
        }
        if (Array.isArray(lData)) {
          setLabelSizes(lData);
          const parsedLayouts: Record<string, LabelDesignLayout> = {};
          for (const sz of lData as LabelSizeItem[]) {
            if (!Array.isArray(sz.templates)) continue;
            for (const tmpl of sz.templates) {
              if (!tmpl.FTLtmLayout) continue;
              try {
                const obj = JSON.parse(tmpl.FTLtmLayout);
                if (obj && obj.customDesign && Array.isArray(obj.elements)) {
                  const key = `${sz.FTLbsCode}:${tmpl.FTLtmPriType === '2' ? '2' : '1'}`;
                  parsedLayouts[key] = obj as LabelDesignLayout;
                }
              } catch {
                // ignore invalid JSON layout
              }
            }
          }
          setCustomLayouts(parsedLayouts);
        }
        if (Array.isArray(pData?.products)) {
          setCatalogProducts(pData.products);
          setBrowseList(pData.products);
        }

        try {
          let dbLan: PrinterConfig[] = Array.isArray(cfgData?.lanPrinters) ? cfgData.lanPrinters : [];
          const legacySaved = localStorage.getItem('printbar_printers');
          if (dbLan.length === 0 && legacySaved) {
            const legacyList: PrinterConfig[] = JSON.parse(legacySaved);
            const lanOnly = legacyList.filter((p) => p.ip && p.ip !== 'Windows');
            for (const lp of lanOnly) {
              await fetch('/api/lan-printers/save', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(lp),
              }).catch(() => null);
            }
            if (lanOnly.length > 0) {
              dbLan = lanOnly;
            }
          }
          if (legacySaved) {
            localStorage.removeItem('printbar_printers');
          }

          const list = mergePrintersFromDbAndOs(dbLan, winList);
          setPrinters(list);

          // Apply system defaults from TCNSSysOption (DB)
          const optsList: SysOptionRow[] = Array.isArray(cfgData?.options) ? cfgData.options : [];
          const findOpt = (code: string) =>
            optsList.find((o) => o.FTSopCode === code)?.FTSopValue?.trim() || '';
          const defBch = findOpt('DefaultBranch') || cfgData?.company?.defaultBchCode || '';
          if (defBch && Array.isArray(bData) && bData.some((b: BranchItem) => b.FTBchCode === defBch)) {
            setBchFrom(defBch);
          }
          const defTmpl = findOpt('DefaultTemplate');
          if (
            defTmpl &&
            Array.isArray(lData) &&
            lData.some((s: LabelSizeItem) => s.FTLbsCode === defTmpl && s.FTLbsStaUse === '1')
          ) {
            setSelectedLabelCode(defTmpl);
          }
          const defPrinter = findOpt('DefaultPrinter');
          if (defPrinter) {
            const mp = list.find((p) => p.name.toLowerCase() === defPrinter.toLowerCase());
            if (mp) setSelectedPrinterId(mp.id);
          }
        } catch {
          // ignore
        }
      } catch {
        toast.error('โหลดข้อมูลเริ่มต้นไม่สำเร็จ');
      } finally {
        setLoading(false);
      }
    }
    init();
  }, []);

  // ===== dcmPrintBarEventGetPdtInTmp (Filtered + Paginated Temp Table) =====
  const filteredScanTmp = useMemo(() => {
    const q = scanSearch.trim().toLowerCase();
    if (!q) return scanTmpRows;
    return scanTmpRows.filter(
      r =>
        r.pdtCode.toLowerCase().includes(q) ||
        r.pdtName.toLowerCase().includes(q) ||
        r.barCode.toLowerCase().includes(q)
    );
  }, [scanTmpRows, scanSearch]);

  const scanTotalPages = Math.max(1, Math.ceil(filteredScanTmp.length / SCAN_PAGE_SIZE));
  const pagedScanTmp = useMemo(() => {
    const safePage = Math.min(scanPage, scanTotalPages);
    const start = (safePage - 1) * SCAN_PAGE_SIZE;
    return filteredScanTmp.slice(start, start + SCAN_PAGE_SIZE);
  }, [filteredScanTmp, scanPage, scanTotalPages]);

  const allScanSelected = pagedScanTmp.length > 0 && pagedScanTmp.every(r => r.selected);

  // dcmPrintBarEventAddPdtToTmp (Scan barcode or Browse add)
  const addPdtToScanTmp = (pdtCode: string, pdtName: string, barCode: string) => {
    setScanTmpRows(prev => {
      const idx = prev.findIndex(r => r.pdtCode === pdtCode && r.barCode === barCode);
      if (idx >= 0) {
        const next = [...prev];
        next[idx] = { ...next[idx], qty: next[idx].qty + 1 };
        return next;
      }
      return [
        ...prev,
        {
          seqNo: prev.length + 1,
          pdtCode,
          pdtName,
          barCode,
          qty: Math.max(1, defaultQty),
          selected: true,
        },
      ];
    });
  };

  // Scan barcode Enter handler
  const handleScanBarcodeEnter = async () => {
    const code = scanInputBarcode.trim();
    if (!code) return;
    try {
      const res = await fetch(`/api/products?keyword=${encodeURIComponent(code)}&limit=10`);
      const data = await res.json();
      const found: ProductItem | undefined = (data.products || []).find(
        (p: ProductItem) =>
          p.FTPdtCode.toLowerCase() === code.toLowerCase() ||
          p.bars?.some(b => b.FTPdbBarCode.toLowerCase() === code.toLowerCase())
      ) || data.products?.[0];

      if (!found) {
        toast.error(`ไม่พบสินค้าหรือบาร์โค้ด: ${code}`);
        return;
      }
      const bar = found.bars?.[0]?.FTPdbBarCode || found.FTPdtCode;
      addPdtToScanTmp(found.FTPdtCode, found.FTPdtName, bar);
      setScanInputBarcode('');
      toast.success(`เพิ่ม ${found.FTPdtName} เข้าตารางสแกนแล้ว`);
    } catch {
      toast.error('ค้นหาบาร์โค้ดไม่สำเร็จ');
    }
  };

  // Search Browse modal
  const handleBrowseSearch = async (kw: string) => {
    setBrowseSearch(kw);
    setBrowseLoading(true);
    try {
      const res = await fetch(`/api/products?keyword=${encodeURIComponent(kw)}&limit=50`);
      const data = await res.json();
      setBrowseList(data.products || []);
    } catch {
      toast.error('ค้นหาสินค้าไม่สำเร็จ');
    } finally {
      setBrowseLoading(false);
    }
  };

  // dcmPrintBarEventPreviewScanImport
  const handlePreviewScanImport = async () => {
    const selected = scanTmpRows.filter(r => r.selected);
    if (selected.length === 0) {
      toast.error('กรุณาเลือกรายการในตารางสแกนอย่างน้อย 1 รายการ');
      return;
    }
    const bchCode = resolveDefaultBranch();
    const pass: PrintQueueRow[] = [];
    const fail: Array<{ pdtCode: string; barCode: string; pdtName: string; reason: string }> = [];

    for (const item of selected) {
      try {
        const res = await fetch('/api/print/build-line', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            barcode: item.barCode,
            qty: item.qty,
            bchCode,
            priType: '1',
            verify: sheetPriceType !== 'Normal',
            overrideLbs: selectedLabelCode,
            effectiveDate: useEffectiveDate && effectiveDate ? effectiveDate : undefined,
            priceGroup: priceGroupCode || undefined,
          }),
        });
        const data = await res.json();
        if (!res.ok) {
          fail.push({ pdtCode: item.pdtCode, barCode: item.barCode, pdtName: item.pdtName, reason: data.error || 'ไม่พบข้อมูล' });
          continue;
        }
        const isPromo = Boolean(data.isPromo);
        if (sheetPriceType === 'Promotion' && !isPromo) {
          fail.push({ pdtCode: item.pdtCode, barCode: item.barCode, pdtName: item.pdtName, reason: 'ไม่มีโปรโมชันในช่วงวันที่เลือก' });
          continue;
        }
        pass.push({
          pdtCode: data.pdtCode,
          pdtName: data.pdtName,
          barCode: data.barcode,
          price: data.setPrice ?? 0,
          priType: isPromo ? '2' : '1',
          qty: item.qty,
          selected: true,
          lbsCode: data.lbsCode,
          tmplCode: data.tmplCode,
          ctx: data.ctx,
          promoName: data.promo?.name,
          statusText: 'ผ่านเงื่อนไข',
        });
      } catch {
        fail.push({ pdtCode: item.pdtCode, barCode: item.barCode, pdtName: item.pdtName, reason: 'เชื่อมต่อ API ไม่สำเร็จ' });
      }
    }

    setPreviewPass(pass);
    setPreviewFail(fail);
    setImportPreviewOpen(true);
  };

  // Handle CSV / TXT import -> Preview modal
  const handleExcelFileUpload = async (file: File) => {
    setImportingFile(true);
    try {
      const text = await file.text();
      const lines = text.split(/\r?\n/).map(l => l.trim()).filter(Boolean);
      if (lines.length === 0) {
        toast.error('ไม่พบข้อมูลในไฟล์');
        return;
      }

      const bchCode = resolveDefaultBranch();
      const pass: PrintQueueRow[] = [];
      const fail: Array<{ pdtCode: string; barCode: string; pdtName: string; reason: string }> = [];

      for (let i = 0; i < lines.length; i++) {
        const cols = lines[i].split(/[,\t;]/).map(c => c.replace(/^"|"$/g, '').trim());
        const first = cols[0] || '';
        if (i === 0 && /barcode|ftbarcode|ftpdtcode/i.test(first)) continue; // skip header row
        const bar = first;
        const qty = Math.max(1, parseInt(cols[1] || '1') || 1);
        if (!bar) continue;

        const res = await fetch('/api/print/build-line', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            barcode: bar,
            qty,
            bchCode,
            priType: '1',
            verify: sheetPriceType !== 'Normal',
            overrideLbs: selectedLabelCode,
            effectiveDate: useEffectiveDate && effectiveDate ? effectiveDate : undefined,
            priceGroup: priceGroupCode || undefined,
          }),
        });
        const data = await res.json();
        if (!res.ok) {
          fail.push({ pdtCode: '-', barCode: bar, pdtName: '-', reason: data.error || 'ไม่พบบาร์โค้ดในฐานข้อมูล' });
          continue;
        }
        const isPromo = Boolean(data.isPromo);
        if (sheetPriceType === 'Promotion' && !isPromo) {
          fail.push({ pdtCode: data.pdtCode, barCode: bar, pdtName: data.pdtName, reason: 'ไม่มีโปรโมชันในช่วงวันที่เลือก' });
          continue;
        }
        pass.push({
          pdtCode: data.pdtCode,
          pdtName: data.pdtName,
          barCode: data.barcode,
          price: data.setPrice ?? 0,
          priType: isPromo ? '2' : '1',
          qty,
          selected: true,
          lbsCode: data.lbsCode,
          tmplCode: data.tmplCode,
          ctx: data.ctx,
          promoName: data.promo?.name,
          statusText: 'ผ่านเงื่อนไข',
        });
      }

      setImportModalOpen(false);
      setPreviewPass(pass);
      setPreviewFail(fail);
      setImportPreviewOpen(true);
    } catch {
      toast.error('อ่านไฟล์ CSV/TXT ไม่สำเร็จ');
    } finally {
      setImportingFile(false);
    }
  };

  // dcmPrintBarEventImpPdtToPrnLabel
  const handleConfirmImportToMain = () => {
    if (previewPass.length === 0) {
      toast.error('ไม่มีรายการที่ผ่านเงื่อนไขสำหรับนำเข้า');
      return;
    }
    setMainRows(prev => {
      const merged = [...prev];
      for (const item of previewPass) {
        const idx = merged.findIndex(m => m.barCode === item.barCode && m.priType === item.priType);
        if (idx >= 0) {
          merged[idx] = { ...merged[idx], qty: merged[idx].qty + item.qty };
        } else {
          merged.push(item);
        }
      }
      return merged;
    });
    setImportPreviewOpen(false);
    setScanModalOpen(false);
    toast.success(`นำเข้า ${previewPass.length} รายการลงตารางพิมพ์สำเร็จ`);
  };

  // MoveDataIntoTable (ปุ่ม แสดงข้อมูล จากแผงเงื่อนไขซ้าย)
  const handleMoveDataIntoTable = async () => {
    setMovingData(true);
    try {
      const params = new URLSearchParams({ limit: '100' });
      if (pdtFrom.trim()) params.set('pdtFrom', pdtFrom.trim());
      if (pdtTo.trim()) params.set('pdtTo', pdtTo.trim());
      if (pdtGrpFilter) params.set('grpFrom', pdtGrpFilter);
      if (pdtGrpTo) params.set('grpTo', pdtGrpTo);
      if (pdtTypeFrom) params.set('typeFrom', pdtTypeFrom);
      if (pdtTypeTo) params.set('typeTo', pdtTypeTo);
      if (pdtBrandFilter) params.set('brandFrom', pdtBrandFilter);
      if (pdtBrandTo) params.set('brandTo', pdtBrandTo);
      if (pdtModelFrom) params.set('modelFrom', pdtModelFrom);
      if (pdtModelTo) params.set('modelTo', pdtModelTo);
      if (bchFrom) params.set('bchFrom', bchFrom);
      if (bchTo) params.set('bchTo', bchTo);
      if (docNoFrom.trim()) params.set('docNoFrom', docNoFrom.trim());
      if (docNoTo.trim()) params.set('docNoTo', docNoTo.trim());
      if (docDateFrom) params.set('docDateFrom', docDateFrom);
      if (docDateTo) params.set('docDateTo', docDateTo);
      if (priceGroupCode) params.set('priceGroup', priceGroupCode);
      if (agencyCode) params.set('agency', agencyCode);
      if (useEffectiveDate && effectiveDate) params.set('effectiveDate', effectiveDate);
      if (sheetPriceType !== 'All') params.set('priceType', sheetPriceType);

      const res = await fetch(`/api/products?${params.toString()}`);
      const data = await res.json();
      const list: ProductItem[] = data.products || [];
      const bchCode = resolveDefaultBranch();

      const built: PrintQueueRow[] = [];
      for (const p of list.slice(0, 100)) {
        const bar = p.bars?.[0]?.FTPdbBarCode || p.FTPdtCode;
        const bRes = await fetch('/api/print/build-line', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            barcode: bar,
            qty: Math.max(1, defaultQty),
            bchCode,
            priType: '1',
            verify: sheetPriceType !== 'Normal',
            overrideLbs: selectedLabelCode,
            effectiveDate: useEffectiveDate && effectiveDate ? effectiveDate : undefined,
            priceGroup: priceGroupCode || undefined,
          }),
        });
        if (!bRes.ok) continue;
        const line = await bRes.json();
        const isPromo = Boolean(line.isPromo);
        if (sheetPriceType === 'Promotion' && !isPromo) continue;
        built.push({
          pdtCode: line.pdtCode,
          pdtName: line.pdtName,
          barCode: line.barcode,
          price: line.setPrice ?? 0,
          priType: isPromo ? '2' : '1',
          qty: Math.max(1, defaultQty),
          selected: true,
          lbsCode: line.lbsCode,
          tmplCode: line.tmplCode,
          ctx: line.ctx,
          promoName: line.promo?.name,
          statusText: 'พร้อมพิมพ์',
        });
      }
      setMainRows(built);
      toast.success(
        list.length > 100
          ? `แสดงข้อมูลสินค้า ${built.length} รายการ (จำกัดสูงสุด 100 รายการแรก)`
          : `แสดงข้อมูลสินค้า ${built.length} รายการ`
      );
    } catch {
      toast.error('ดึงข้อมูลสินค้าตามเงื่อนไขไม่สำเร็จ');
    } finally {
      setMovingData(false);
    }
  };

  const handleClearFilters = () => {
    setSheetPriceType('All');
    setUseEffectiveDate(false);
    setDocDateFrom('');
    setDocDateTo('');
    setDocNoFrom('');
    setDocNoTo('');
    setBchFrom('');
    setBchTo('');
    setPdtFrom('');
    setPdtTo('');
    setPdtGrpFilter('');
    setPdtGrpTo('');
    setPdtTypeFrom('');
    setPdtTypeTo('');
    setPdtBrandFilter('');
    setPdtBrandTo('');
    setPdtModelFrom('');
    setPdtModelTo('');
    setAgencyCode('');
    setPriceGroupCode('');
    setDefaultQty(1);
    setMainRows([]);
    toast.info('ล้างเงื่อนไขและตารางพิมพ์แล้ว');
  };

  const filteredMainRows = useMemo(() => {
    const q = mainSearch.trim().toLowerCase();
    if (!q) return mainRows;
    return mainRows.filter(
      r =>
        r.pdtCode.toLowerCase().includes(q) ||
        r.pdtName.toLowerCase().includes(q) ||
        r.barCode.toLowerCase().includes(q)
    );
  }, [mainRows, mainSearch]);

  const normalRows = useMemo(() => mainRows.filter(r => r.selected && r.priType === '1'), [mainRows]);
  const promoRows = useMemo(() => mainRows.filter(r => r.selected && r.priType === '2'), [mainRows]);
  const activeDriverMapping = useMemo(
    () => driverMappings.find(m => m.lbsCode === selectedLabelCode) || null,
    [driverMappings, selectedLabelCode]
  );

  // Auto-select Printer dropdown when Label Format has driver mapping configured
  useEffect(() => {
    if (!activeDriverMapping || printers.length === 0) return;
    const preferredName =
      sheetPriceType === 'Promotion'
        ? activeDriverMapping.promoPrinter || activeDriverMapping.normalPrinter
        : activeDriverMapping.normalPrinter || activeDriverMapping.promoPrinter;
    if (!preferredName) return;
    const matched = printers.find(
      p => p.name.toLowerCase() === preferredName.toLowerCase()
    );
    if (matched) {
      setSelectedPrinterId(matched.id);
    }
  }, [activeDriverMapping, sheetPriceType, printers]);

  const handleSaveDriverMapping = async () => {
    if (!driverEditRow) return;
    setSavingDriver(true);
    try {
      const res = await fetch('/api/label-printers', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          lbsCode: driverEditRow.lbsCode,
          serverCode: driverServerCode,
          normalPrinter: driverEditRow.normalPrinter,
          normalTray: driverEditRow.normalTray,
          normalProtocol: 'WINDOWS',
          promoPrinter: driverEditRow.promoPrinter,
          promoTray: driverEditRow.promoTray,
          promoProtocol: 'WINDOWS',
        }),
      });
      if (!res.ok) throw new Error('save failed');
      setDriverMappings(prev =>
        prev.map(m => (m.lbsCode === driverEditRow.lbsCode ? driverEditRow : m))
      );
      toast.success(`บันทึกการตั้งค่าไดร์เวอร์ ${driverEditRow.lbsCode} สำเร็จ`);
      setDriverEditRow(null);
    } catch {
      toast.error('บันทึกการตั้งค่าไดร์เวอร์ไม่สำเร็จ');
    } finally {
      setSavingDriver(false);
    }
  };

  const handleTestDriverPrint = async (kind: 'normal' | 'promo') => {
    if (!driverEditRow) return;
    const printerName = kind === 'normal' ? driverEditRow.normalPrinter : driverEditRow.promoPrinter;
    const trayName = kind === 'normal' ? driverEditRow.normalTray : driverEditRow.promoTray;
    if (!printerName) {
      toast.error(`กรุณาเลือกเครื่องพิมพ์${kind === 'normal' ? 'ราคาปกติ' : 'ราคาโปรโมชัน'}ก่อนทดสอบ`);
      return;
    }
    setTestingDriver(kind);
    try {
      const res = await fetch('/api/print/windows-direct', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          printerName,
          trayName,
          lbsCode: driverEditRow.lbsCode,
          lbsName: driverEditRow.lbsName,
          priTypeLabel: kind === 'normal' ? 'ราคาปกติ' : 'ราคาโปรโมชัน',
          testMode: true,
        }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data.error || 'test print failed');
      toast.success(
        data.outputFile
          ? `ทดสอบพิมพ์สำเร็จ (บันทึกไฟล์ที่ ${data.outputFile})`
          : data.message || 'ส่งพิมพ์ทดสอบสำเร็จ'
      );
    } catch (err: any) {
      toast.error(err?.message || 'ส่งพิมพ์ทดสอบไม่สำเร็จ');
    } finally {
      setTestingDriver(null);
    }
  };

  const handleConfirmSendPrint = async () => {
    const selected = mainRows.filter(r => r.selected);
    if (selected.length === 0) {
      toast.error('ไม่มีรายการที่เลือกพิมพ์');
      return;
    }
    const printer = printers.find(p => p.id === selectedPrinterId);
    setPrinting(true);
    try {
      const bchCode = resolveDefaultBranch();
      const fallbackPrinterName = printer?.name || '';
      const norPrtDesc = activeDriverMapping?.normalPrinter
        ? `${activeDriverMapping.normalPrinter}${activeDriverMapping.normalTray ? ` (${activeDriverMapping.normalTray})` : ''}`
        : fallbackPrinterName;
      const pmtPrtDesc = activeDriverMapping?.promoPrinter
        ? `${activeDriverMapping.promoPrinter}${activeDriverMapping.promoTray ? ` (${activeDriverMapping.promoTray})` : ''}`
        : fallbackPrinterName;
      await fetch('/api/print/create-job', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          lines: selected.map(r => ({
            barcode: r.barCode,
            qty: r.qty,
            ctx: r.ctx,
            priType: r.priType,
            lbsCode: selectedLabelCode || r.lbsCode,
            salePrice: r.price,
            setPrice: r.price,
            promo: null,
            tmplCode: r.tmplCode,
            pdtCode: r.pdtCode,
          })),
          bchCode,
          source: '1',
          verify: true,
          user: sysOptValue('DefaultUser') || undefined,
          serverCode: driverServerCode,
          normalPrinter: norPrtDesc,
          promoPrinter: pmtPrtDesc,
          defaultPrinter: fallbackPrinterName,
        }),
      });

      const hasSplitMapping = Boolean(
        (normalRows.length > 0 && activeDriverMapping?.normalPrinter) ||
        (promoRows.length > 0 && activeDriverMapping?.promoPrinter)
      );

      if (hasSplitMapping && activeDriverMapping) {
        const fallbackWin = printers.find(p => p.id === selectedPrinterId)?.name || windowsPrinters[0]?.name || '';
        const dispatches: Array<{ label: string; rowsCount: number; printer: string; tray: string; containerId: string }> = [];

        if (normalRows.length > 0) {
          const pName = activeDriverMapping.normalPrinter || fallbackWin;
          if (!pName) throw new Error('ยังไม่ได้ตั้งค่าเครื่องพิมพ์ราคาปกติ');
          dispatches.push({
            label: 'ราคาปกติ',
            rowsCount: normalRows.length,
            printer: pName,
            tray: activeDriverMapping.normalTray || '',
            containerId: 'v5-capture-normal',
          });
        }
        if (promoRows.length > 0) {
          const pName = activeDriverMapping.promoPrinter || fallbackWin;
          if (!pName) throw new Error('ยังไม่ได้ตั้งค่าเครื่องพิมพ์ราคาโปรโมชัน');
          dispatches.push({
            label: 'ราคาโปรโมชัน',
            rowsCount: promoRows.length,
            printer: pName,
            tray: activeDriverMapping.promoTray || '',
            containerId: 'v5-capture-promo',
          });
        }

        const messages: string[] = [];
        for (let idx = 0; idx < dispatches.length; idx++) {
          const d = dispatches[idx];
          if (idx > 0) {
            await new Promise(r => setTimeout(r, 900));
          }
          const pageImages = await capturePagesToPngBase64(
            d.containerId,
            a4Layout.paperWidthMm,
            a4Layout.paperHeightMm
          );
          const wRes = await fetch('/api/print/windows-direct', {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              printerName: d.printer,
              trayName: d.tray,
              lbsCode: selectedLabelCode,
              lbsName: activeDriverMapping.lbsName,
              priTypeLabel: d.label,
              testMode: false,
              pageImages,
              paperSize: a4Layout.paperSize,
              paperWidthMm: a4Layout.paperWidthMm,
              paperHeightMm: a4Layout.paperHeightMm,
            }),
          });
          const wData = await wRes.json();
          if (!wRes.ok) throw new Error(wData.error || 'ส่งพิมพ์ล้มเหลว');
          messages.push(
            wData.outputFile
              ? `${d.label} (${d.rowsCount}): บันทึก PDF ที่ ${wData.outputFile}`
              : `${d.label} (${d.rowsCount}): ${d.printer}${d.tray ? ` [ถาด ${d.tray}]` : ''}`
          );
        }
        toast.success(`ส่งพิมพ์แยกปลายทางสำเร็จ — ${messages.join(' | ')}`);
        setSendPrintOpen(false);
        return;
      }

      if (printer && printer.brand !== 'SAMSUNG') {
        const res = await fetch('/api/print/send', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            printerIp: printer.ip,
            printerPort: printer.port,
            brand: printer.brand,
            lines: selected.map(r => ({ ...r, barcode: r.barCode, selected: true })),
            options: { dpi: printer.dpi, speed: printer.speed, darkness: printer.darkness },
          }),
        });
        const data = await res.json();
        toast.success(data.message || `ส่งพิมพ์ ${selected.length} รายการสำเร็จ`);
      } else {
        toast.success(`บันทึกงานพิมพ์ ${selected.length} รายการแล้ว (เปิดหน้าต่างพิมพ์ A4)`);
        setSendPrintOpen(false);
        setTimeout(() => window.print(), 350);
        return;
      }
      setSendPrintOpen(false);
    } catch (err: any) {
      toast.error(err?.message || 'ส่งคำสั่งพิมพ์ไม่สำเร็จ');
    } finally {
      setPrinting(false);
    }
  };

  const handleExportCsv = () => {
    const selected = mainRows.filter(r => r.selected);
    if (selected.length === 0) {
      toast.error('ไม่มีรายการสำหรับส่งออก');
      return;
    }
    const header = 'FTPdtCode,FTPdtName,FTBarCode,FCPdtPrice,FTPlbPriType,FNPlbQty,FTLbsCode\n';
    const body = selected
      .map(r => `"${r.pdtCode}","${r.pdtName}","${r.barCode}",${r.price},"${r.priType}",${r.qty},"${r.lbsCode}"`)
      .join('\n');
    const blob = new Blob([header + body], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `PrintBar_V5_${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
    toast.success('ส่งออกไฟล์ CSV เรียบร้อย');
  };

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key !== 'Enter') return;
      if (activeMenu !== 'print') return;
      if (scanModalOpen || browseOpen || importPreviewOpen || sendPrintOpen || importModalOpen || rangeBrowseTarget || driverModalOpen || driverEditRow) {
        return;
      }
      const el = e.target as HTMLElement | null;
      if (!el) return;
      // If typing inside an input/textarea outside the filter form (e.g. table search or qty), do not hijack Enter
      if ((el.tagName === 'INPUT' || el.tagName === 'TEXTAREA') && !el.closest('form')) {
        return;
      }
      // Avoid interfering when Radix Select listbox is open
      if (el.getAttribute('role') === 'combobox' || el.closest('[role="listbox"]')) {
        return;
      }
      if (!movingData) {
        e.preventDefault();
        handleMoveDataIntoTable();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  const a4Layout = useMemo(() => {
    const ls = labelSizes.find(s => s.FTLbsCode === selectedLabelCode);
    const widthMm = ls?.FCLbsWidth || 100;
    const heightMm = ls?.FCLbsHeight || 38;
    const magTopMm = ls?.FCLbsMagTop ?? 3;
    const magLeftMm = ls?.FCLbsMagLeft ?? 3;
    const magBottomMm = ls?.FCLbsMagBottom ?? magTopMm;
    const magRightMm = ls?.FCLbsMagRight ?? magLeftMm;
    const gapHMm = ls?.FCLbsGapH ?? 2;
    const gapVMm = ls?.FCLbsGapV ?? 2;
    const rawPs = (ls?.FTLbsPaperSize || paperSizes[0]?.FTPpsCode || 'A4') as PaperSizeCode;
    const dbPaper = paperSizes.find((p) => p.FTPpsCode === rawPs);
    const paperPreset = PAPER_SIZE_PRESETS[rawPs] || PAPER_SIZE_PRESETS.A4;
    const paperSize: PaperSizeCode = dbPaper?.FTPpsCode || (rawPs in PAPER_SIZE_PRESETS ? rawPs : 'A4');
    const paperWidthMm = ls?.FCLbsPaperWidth || dbPaper?.FCPpsWidth || paperPreset.widthMm;
    const paperHeightMm = ls?.FCLbsPaperHeight || dbPaper?.FCPpsHeight || paperPreset.heightMm;
    const offsetX = ls?.FCLbsOffsetX ?? 0;
    const offsetY = ls?.FCLbsOffsetY ?? 0;
    const calcCols = Math.max(1, Math.floor((paperWidthMm - magLeftMm - magRightMm + gapHMm) / (widthMm + gapHMm)));
    const calcRows = Math.max(1, Math.floor((paperHeightMm - magTopMm - magBottomMm + gapVMm) / (heightMm + gapVMm)));
    const cols = ls?.FNLbsCol || calcCols;
    const rows = ls?.FNLbsRow || calcRows;
    const perPage = Math.max(1, ls?.FNLbsQtyPageNor || (cols * rows));
    const paginate = (items: PrintQueueRow[]) => {
      const exp = items.flatMap(r =>
        Array.from({ length: r.qty || 1 }, () => ({
          barcode: r.barCode,
          priType: r.priType,
          ctx: r.ctx,
        }))
      );
      const pg: Array<Array<{ barcode: string; priType: string; ctx: Record<string, any> }>> = [];
      for (let i = 0; i < exp.length; i += perPage) {
        pg.push(exp.slice(i, i + perPage));
      }
      return { exp, pg };
    };
    const all = paginate(mainRows.filter(r => r.selected));
    const nor = paginate(mainRows.filter(r => r.selected && r.priType === '1'));
    const pmt = paginate(mainRows.filter(r => r.selected && r.priType === '2'));
    return {
      widthMm,
      heightMm,
      magTopMm,
      magLeftMm,
      magBottomMm,
      magRightMm,
      gapHMm,
      gapVMm,
      paperSize,
      paperWidthMm,
      paperHeightMm,
      offsetX,
      offsetY,
      cols,
      rows,
      perPage,
      pages: all.pg,
      totalLabels: all.exp.length,
      normalPages: nor.pg,
      normalLabelsCount: nor.exp.length,
      promoPages: pmt.pg,
      promoLabelsCount: pmt.exp.length,
    };
  }, [labelSizes, selectedLabelCode, mainRows]);

  useEffect(() => {
    setPrintOffsetX(a4Layout.offsetX);
    setPrintOffsetY(a4Layout.offsetY);
  }, [selectedLabelCode, a4Layout.offsetX, a4Layout.offsetY]);

  const hasSplitPreview = Boolean(
    (normalRows.length > 0 && activeDriverMapping?.normalPrinter) ||
    (promoRows.length > 0 && activeDriverMapping?.promoPrinter)
  );
  const previewSheetCount = hasSplitPreview
    ? a4Layout.normalPages.length + a4Layout.promoPages.length
    : a4Layout.pages.length;

  const refreshLabelSizesAndMappings = async () => {
    const [lRes, lpRes, cfgRes] = await Promise.all([
      fetch('/api/label-sizes'),
      fetch('/api/label-printers').catch(() => null),
      fetch('/api/config/all').catch(() => null),
    ]);
    const lData = await lRes.json().catch(() => null);
    const lpData = lpRes ? await lpRes.json().catch(() => null) : null;
    const cfgData = cfgRes ? await cfgRes.json().catch(() => null) : null;
    if (cfgData) {
      if (Array.isArray(cfgData.labelSizes)) setAllConfigSizes(cfgData.labelSizes);
      if (Array.isArray(cfgData.options)) setSysOptions(cfgData.options);
      if (Array.isArray(cfgData.servers)) setPrinterServers(cfgData.servers);
      if (Array.isArray(cfgData.paperSizes)) setPaperSizes(cfgData.paperSizes);
      if (Array.isArray(cfgData.preprints)) setPreprints(cfgData.preprints);
      if (Array.isArray(cfgData.labelFields)) setLabelFields(cfgData.labelFields);
      if (cfgData.company) setCompanyInfo(cfgData.company);
      if (Array.isArray(cfgData.lanPrinters)) {
        setPrinters(mergePrintersFromDbAndOs(cfgData.lanPrinters, windowsPrinters));
      }
    }
    if (Array.isArray(lData)) {
      setLabelSizes(lData);
      const parsedLayouts: Record<string, LabelDesignLayout> = {};
      for (const sz of lData as LabelSizeItem[]) {
        if (!Array.isArray(sz.templates)) continue;
        for (const tmpl of sz.templates) {
          if (!tmpl.FTLtmLayout) continue;
          try {
            const obj = JSON.parse(tmpl.FTLtmLayout);
            if (obj && obj.customDesign && Array.isArray(obj.elements)) {
              const key = `${sz.FTLbsCode}:${tmpl.FTLtmPriType === '2' ? '2' : '1'}`;
              parsedLayouts[key] = obj as LabelDesignLayout;
            }
          } catch {
            // ignore invalid JSON
          }
        }
      }
      setCustomLayouts(parsedLayouts);
    }
    if (lpData?.mappings && Array.isArray(lpData.mappings)) {
      setDriverMappings(lpData.mappings);
    }
  };

  const parseApiJson = async (res: Response) => {
    const ct = res.headers.get('content-type') || '';
    const raw = await res.text();
    if (!ct.includes('application/json') || raw.trimStart().startsWith('<')) {
      throw new Error(
        'เซิร์ฟเวอร์ API ยังเป็นเวอร์ชันเก่า (ไม่พบเส้นทางนี้) กรุณากด Ctrl+C ปิด npm run dev:web แล้วรันใหม่อีกครั้ง'
      );
    }
    try {
      return JSON.parse(raw);
    } catch {
      throw new Error(raw || `HTTP ${res.status}`);
    }
  };

  const handleSaveAsCustomLayout = async (payload: SaveAsPayload): Promise<boolean> => {
    try {
      const res = await fetch('/api/templates/save-as', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'สร้างแบบป้ายใหม่ไม่สำเร็จ');
        return false;
      }
      await refreshLabelSizesAndMappings();
      if (data.lbsCode) {
        setSelectedLabelCode(data.lbsCode);
        setDesignLabelCode(data.lbsCode);
      }
      toast.success(`สร้างแบบใหม่ "${data.lbsName}" (${data.lbsCode}) เรียบร้อยแล้ว`);
      return true;
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
      return false;
    }
  };

  const handleDeleteCustomFormat = async (lbsCode: string): Promise<boolean> => {
    try {
      const res = await fetch('/api/templates/delete-custom', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ lbsCode }),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'ลบแบบป้ายไม่สำเร็จ');
        return false;
      }
      await refreshLabelSizesAndMappings();
      setSelectedLabelCode(resolveDefaultLabelCode());
      setDesignLabelCode((prev) => (prev === lbsCode ? '' : prev));
      toast.info(`ลบแบบป้าย ${lbsCode} เรียบร้อยแล้ว`);
      return true;
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
      return false;
    }
  };

  const handleSaveCustomLayout = async (
    layout: LabelDesignLayout,
    meta?: SheetConfigMeta
  ): Promise<boolean> => {
    try {
      const res = await fetch('/api/templates/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          lbsCode: layout.lbsCode,
          priType: layout.priType,
          name: layout.name,
          layout,
          ...(meta || {}),
        }),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'บันทึกแบบป้ายไม่สำเร็จ');
        return false;
      }
      const key = `${layout.lbsCode}:${layout.priType}`;
      setCustomLayouts((prev) => ({ ...prev, [key]: layout }));
      await refreshLabelSizesAndMappings();
      toast.success(`บันทึกแบบป้าย ${layout.lbsCode} (${layout.priType === '2' ? 'โปรโมชัน' : 'ปกติ'}) เรียบร้อย`);
      return true;
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
      return false;
    }
  };

  const handleResetCustomLayout = async (lbsCode: string, priType: '1' | '2'): Promise<boolean> => {
    try {
      const res = await fetch('/api/templates/reset', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ lbsCode, priType }),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'รีเซ็ตแบบป้ายไม่สำเร็จ');
        return false;
      }
      const key = `${lbsCode}:${priType}`;
      setCustomLayouts((prev) => {
        const next = { ...prev };
        delete next[key];
        return next;
      });
      toast.info(`คืนค่าแบบป้าย ${lbsCode} (${priType === '2' ? 'โปรโมชัน' : 'ปกติ'}) เป็นค่าเริ่มต้นแล้ว`);
      return true;
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
      return false;
    }
  };

  const resolveCustomLayout = (lbsCode: string, priType: string) => {
    const t = priType === '2' ? '2' : '1';
    return (
      customLayouts[`${lbsCode}:${t}`] ||
      customLayouts[`${lbsCode}:1`] ||
      customLayouts[`${lbsCode}:2`]
    );
  };

  const handleSaveQuickPrintOffset = async () => {
    const ls = labelSizes.find((s) => s.FTLbsCode === selectedLabelCode);
    if (!ls) return;
    setSavingOffset(true);
    try {
      const res = await fetch('/api/label-sizes/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          lbsCode: ls.FTLbsCode,
          lbsName: ls.FTLbsName || ls.FTLbsCode,
          lbsGroup: ls.FTLbsGroup === '2' ? '2' : '1',
          widthMm: ls.FCLbsWidth,
          heightMm: ls.FCLbsHeight,
          cols: ls.FNLbsCol || a4Layout.cols,
          rows: ls.FNLbsRow || a4Layout.rows,
          magTop: ls.FCLbsMagTop ?? 3,
          magLeft: ls.FCLbsMagLeft ?? 3,
          magBottom: ls.FCLbsMagBottom ?? ls.FCLbsMagTop ?? 3,
          magRight: ls.FCLbsMagRight ?? ls.FCLbsMagLeft ?? 3,
          gapH: ls.FCLbsGapH ?? 2,
          gapV: ls.FCLbsGapV ?? 2,
          qtyPageNor: ls.FNLbsQtyPageNor || a4Layout.cols * a4Layout.rows,
          qtyPagePmt: ls.FNLbsQtyPagePmt || a4Layout.cols * a4Layout.rows,
          staUse: ls.FTLbsStaUse || '1',
          isNew: false,
          paperSize: a4Layout.paperSize,
          paperWidthMm: a4Layout.paperWidthMm,
          paperHeightMm: a4Layout.paperHeightMm,
          offsetX: printOffsetX,
          offsetY: printOffsetY,
        }),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'บันทึกค่าชดเชยไม่สำเร็จ');
        return;
      }
      await refreshLabelSizesAndMappings();
      toast.success(`บันทึกค่าชดเชย X=${printOffsetX}mm, Y=${printOffsetY}mm เรียบร้อย`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setSavingOffset(false);
    }
  };

  const renderPreviewSheets = (
    sheetPages: Array<Array<{ barcode: string; priType: string; ctx: Record<string, any> }>>,
    keyPrefix: string
  ) =>
    sheetPages.map((pageItems, pageIdx) => {
      const sheetFactor = sheetPreviewScale;
      const firstPreprintLayout = pageItems
        .map((it) => resolveCustomLayout(selectedLabelCode, it.priType))
        .find((ly) => ly?.designMode === 'preprint');
      const sheetBgUrl = firstPreprintLayout?.preprint?.sheetBgDataUrl;
      const showSheetGuide =
        Boolean(firstPreprintLayout) && previewPreprintView === 'guide' && Boolean(sheetBgUrl);
      const isSmallerThanA4 =
        a4Layout.paperWidthMm <= 209 &&
        a4Layout.paperHeightMm <= 297 &&
        (a4Layout.paperWidthMm < 209 || a4Layout.paperHeightMm < 296);

      const sheetBox = (
        <div
          className={`border-2 bg-white shadow-sm relative overflow-hidden ${isSmallerThanA4 ? 'border-[#007a87]' : 'border-gray-300 mx-auto'}`}
          style={{
            width: `${a4Layout.paperWidthMm * sheetFactor}mm`,
            height: `${a4Layout.paperHeightMm * sheetFactor}mm`,
            boxSizing: 'border-box',
          }}
        >
          {showSheetGuide && sheetBgUrl && (
            <img
              src={sheetBgUrl}
              alt="Pre-print Sheet Guide"
              style={{
                position: 'absolute',
                inset: 0,
                width: '100%',
                height: '100%',
                objectFit: 'fill',
                opacity: Math.max(0.2, Math.min(1, firstPreprintLayout?.preprint?.opacity ?? 0.8)),
                filter:
                  firstPreprintLayout?.preprint?.contrast && firstPreprintLayout.preprint.contrast !== 100
                    ? `contrast(${firstPreprintLayout.preprint.contrast}%)`
                    : undefined,
                pointerEvents: 'none',
              }}
            />
          )}
          <div
            style={{
              position: 'relative',
              width: '100%',
              height: '100%',
              paddingTop: `${a4Layout.magTopMm * sheetFactor}mm`,
              paddingBottom: `${a4Layout.magBottomMm * sheetFactor}mm`,
              paddingLeft: `${a4Layout.magLeftMm * sheetFactor}mm`,
              paddingRight: `${a4Layout.magRightMm * sheetFactor}mm`,
              display: 'grid',
              gridTemplateColumns: `repeat(${a4Layout.cols}, ${a4Layout.widthMm * sheetFactor}mm)`,
              gridAutoRows: `${a4Layout.heightMm * sheetFactor}mm`,
              columnGap: `${a4Layout.gapHMm * sheetFactor}mm`,
              rowGap: `${a4Layout.gapVMm * sheetFactor}mm`,
              alignContent: 'start',
              justifyContent: 'start',
              boxSizing: 'border-box',
              transform:
                printOffsetX !== 0 || printOffsetY !== 0
                  ? `translate(${printOffsetX * sheetFactor}mm, ${printOffsetY * sheetFactor}mm)`
                  : undefined,
            }}
          >
            {pageItems.map((line, idx) => {
              const customLy = resolveCustomLayout(selectedLabelCode, line.priType);
              const cellView =
                customLy?.designMode === 'preprint'
                  ? previewPreprintView === 'ink'
                    ? 'ink'
                    : sheetBgUrl
                      ? 'sheet-overlay'
                      : 'cell-guide'
                  : 'ink';
              return (
                <LabelPreview
                  key={`${keyPrefix}-${line.barcode}-${pageIdx}-${idx}`}
                  line={line}
                  scale={sheetFactor}
                  labelW={a4Layout.widthMm}
                  labelH={a4Layout.heightMm}
                  lbsCode={selectedLabelCode}
                  customLayout={customLy}
                  preprintView={cellView}
                />
              );
            })}
          </div>
          {isSmallerThanA4 && (
            <div className="absolute bottom-0.5 right-1 bg-[#007a87]/90 text-white text-[9px] font-mono px-1 rounded pointer-events-none">
              {a4Layout.paperSize}: {a4Layout.paperWidthMm}×{a4Layout.paperHeightMm} mm
            </div>
          )}
        </div>
      );

      return (
        <div key={`${keyPrefix}-${pageIdx}`} className="space-y-1.5">
          <div className="text-xs font-medium text-muted-foreground flex flex-wrap items-center justify-between gap-2">
            <span>
              แผ่นที่ {pageIdx + 1} / {sheetPages.length} ({pageItems.length} ดวง · ขนาดจริง {a4Layout.widthMm}×{a4Layout.heightMm} mm · กระดาษ {a4Layout.paperWidthMm}×{a4Layout.paperHeightMm} mm)
            </span>
            {isSmallerThanA4 && (
              <span className="text-[11px] text-[#00606b]">กรอบประ = พื้นที่ A4 (210×297 mm)</span>
            )}
          </div>
          {isSmallerThanA4 ? (
            <div
              className="mx-auto relative border-2 border-dashed border-slate-400 bg-slate-100/80 shadow-inner"
              style={{
                width: `${210 * sheetFactor}mm`,
                height: `${297 * sheetFactor}mm`,
                boxSizing: 'border-box',
              }}
            >
              {sheetBox}
              <div className="absolute bottom-1 right-1.5 text-[10px] font-mono text-slate-500 bg-white/80 px-1 rounded border border-slate-200 pointer-events-none">
                A4 (210×297 mm)
              </div>
            </div>
          ) : (
            sheetBox
          )}
        </div>
      );
    });

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-slate-100">
        <Loader2 className="h-6 w-6 animate-spin text-teal-600 mr-2" />
        <span className="text-sm text-slate-600">กำลังโหลดหน้าจอ...</span>
        
    </div>
    );
  }

  return (
    <>
    <div className="min-h-screen flex bg-[#f0f2f5] text-slate-800 print:hidden">
      {/* Left Application Sidebar */}
      <aside
        className={`bg-[#2c3742] text-slate-100 flex flex-col shrink-0 transition-all duration-200 border-r border-slate-700/60 sticky top-0 h-screen z-50 ${
          sidebarOpen ? 'w-56' : 'w-16'
        }`}
      >
        <div className="h-[54px] px-3 flex items-center justify-between border-b border-slate-700/60">
          {sidebarOpen && (
            <div className="flex items-center gap-2 min-w-0">
              <div className="w-7 h-7 rounded bg-[#007a87] flex items-center justify-center text-white shrink-0">
                <Barcode className="h-4 w-4" />
              </div>
              <div className="min-w-0">
                <div className="text-sm font-bold text-white truncate leading-tight">AdaBarprint</div>
                <div className="text-[10px] text-slate-400 truncate">PrintBar Studio</div>
              </div>
            </div>
          )}
          <Button
            type="button"
            size="icon"
            variant="ghost"
            onClick={() => setSidebarOpen((s) => !s)}
            className="h-8 w-8 text-slate-300 hover:text-white hover:bg-slate-700/60 mx-auto"
            title={sidebarOpen ? 'พับแถบเมนูซ้าย' : 'ขยายแถบเมนูซ้าย'}
          >
            {sidebarOpen ? <PanelLeftClose className="h-4 w-4" /> : <PanelLeftOpen className="h-4 w-4" />}
          </Button>
        </div>

        <nav className="flex-1 p-2 space-y-1">
          <button
            type="button"
            onClick={() => setActiveMenu('print')}
            className={`w-full flex items-center gap-2.5 px-2.5 py-2.5 rounded-md text-xs font-medium transition-colors ${
              activeMenu === 'print'
                ? 'bg-[#007a87] text-white shadow-xs'
                : 'text-slate-300 hover:bg-slate-700/50 hover:text-white'
            }`}
            title="พิมพ์ป้ายราคา / บาร์โค้ด"
          >
            <Printer className="h-4 w-4 shrink-0" />
            {sidebarOpen && <span className="truncate">พิมพ์ป้ายราคา</span>}
          </button>

          <button
            type="button"
            onClick={() => {
              setDesignLabelCode('');
              setActiveMenu('design');
            }}
            className={`w-full flex items-center gap-2.5 px-2.5 py-2.5 rounded-md text-xs font-medium transition-colors ${
              activeMenu === 'design'
                ? 'bg-[#007a87] text-white shadow-xs'
                : 'text-slate-300 hover:bg-slate-700/50 hover:text-white'
            }`}
            title="Design Studio — ออกแบบป้ายสไตล์ Figma / Canva"
          >
            <Palette className="h-4 w-4 shrink-0" />
            {sidebarOpen && (
              <span className="truncate flex items-center justify-between flex-1">
                <span>Design (ออกแบบป้าย)</span>
              </span>
            )}
          </button>

          <div className="my-2 border-t border-slate-700/60" />

          <button
            type="button"
            onClick={() => {
              setSettingsEditLbsCode(null);
              setActiveMenu('settings');
            }}
            className={`w-full flex items-center gap-2.5 px-2.5 py-2.5 rounded-md text-xs font-medium transition-colors ${
              activeMenu === 'settings'
                ? 'bg-[#007a87] text-white shadow-xs'
                : 'text-slate-300 hover:bg-slate-700/50 hover:text-white'
            }`}
            title="ตั้งค่าขนาดป้าย, เครื่องพิมพ์, ถาดกระดาษ และระบบ"
          >
            <Settings2 className="h-4 w-4 shrink-0" />
            {sidebarOpen && <span className="truncate">ตั้งค่าระบบ</span>}
          </button>
        </nav>

        {sidebarOpen && (
          <div className="p-3 border-t border-slate-700/60 text-[10px] text-slate-400 space-y-0.5">
            <div className="flex items-center justify-between">
              <span>สถานะระบบ</span>
              <span className="text-emerald-400 font-medium">พร้อมใช้งาน</span>
            </div>
            <div className="flex items-center justify-between">
              <span>แบบที่ออกแบบเอง</span>
              <span className="text-slate-200 font-mono">{Object.keys(customLayouts).length} แบบ</span>
            </div>
          </div>
        )}
      </aside>

      {/* Right Main Area */}
      <div className="flex-1 min-w-0 flex flex-col">
      {/* Header Menu Bar */}
      <header className="bg-white border-b border-slate-200 sticky top-0 z-40 shadow-xs">
        <div className="max-w-[1600px] mx-auto px-4 py-2 flex items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-base font-bold text-[#007a87]">
                  {activeMenu === 'design'
                    ? 'ออกแบบป้ายราคา (Design Studio)'
                    : activeMenu === 'settings'
                      ? 'ตั้งค่าระบบและขนาดป้าย'
                      : 'พิมพ์ป้ายราคา / บาร์โค้ด'}
                </h1>
                <Badge variant="outline" className="text-[11px] border-[#007a87] text-[#007a87]">AdaBarprint</Badge>
              </div>
              <p className="text-[11px] text-slate-500">
                {activeMenu === 'design'
                  ? 'ลากจัดวางองค์ประกอบ ปรับสี ไอคอน และ Preview เทียบกระดาษ A4'
                  : activeMenu === 'settings'
                    ? 'จัดการขนาดรูปแบบป้าย ระยะขอบ เครื่องพิมพ์ ถาดกระดาษ และค่าเริ่มต้นระบบ'
                    : 'เลือกสินค้า สแกนบาร์โค้ด ตรวจสอบราคา และส่งพิมพ์ป้าย'}
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            {activeMenu === 'print' && (
              <>
                <Button
                  variant="outline"
                  size="sm"
                  className="border-[#007a87] text-[#007a87] hover:bg-teal-50/70"
                  onClick={() => {
                    setDriverEditRow(null);
                    setDriverModalOpen(true);
                  }}
                >
                  <Settings2 className="h-3.5 w-3.5 mr-1" />ตั้งค่าไดร์เวอร์เครื่องพิมพ์
                </Button>
                <Button
                  size="sm"
                  className="bg-[#007a87] hover:bg-[#00606b] text-white"
                  onClick={() => {
                    if (!selectedLabelCode) {
                      toast.error('กรุณาเลือกรูปแบบป้าย (Label Format)');
                      return;
                    }
                    const hasMapped = Boolean(activeDriverMapping?.normalPrinter || activeDriverMapping?.promoPrinter);
                    if (!selectedPrinterId && !hasMapped) {
                      toast.error('กรุณาเลือกเครื่องพิมพ์ (Printer) หรือตั้งค่าไดร์เวอร์ตามรหัสฉลาก');
                      return;
                    }
                    if (mainRows.filter(r => r.selected).length === 0) {
                      toast.error('กรุณาเลือกรายการที่ต้องการพิมพ์');
                      return;
                    }
                    setSendPrintOpen(true);
                  }}
                >
                  <Printer className="h-3.5 w-3.5 mr-1" />พิมพ์
                </Button>
                <Button size="sm" className="bg-[#007a87] hover:bg-[#00606b] text-white" onClick={handleExportCsv}>
                  <Download className="h-3.5 w-3.5 mr-1" />ส่งออก CSV
                </Button>
              </>
            )}
          </div>
        </div>
      </header>

      {activeMenu === 'design' ? (
        <LabelDesigner
          labelSizes={labelSizes}
          products={catalogProducts}
          paperSizes={paperSizes}
          preprints={preprints}
          labelFields={labelFields}
          companyInfo={companyInfo}
          selectedLabelCode={designLabelCode}
          onChangeLabelCode={setDesignLabelCode}
          customLayouts={customLayouts}
          onSaveLayout={handleSaveCustomLayout}
          onSaveAsLayout={handleSaveAsCustomLayout}
          onDeleteCustomFormat={handleDeleteCustomFormat}
          onResetLayout={handleResetCustomLayout}
        />
      ) : activeMenu === 'settings' ? (
        <SettingsPanel
          allLabelSizes={
            allConfigSizes.length > 0
              ? allConfigSizes
              : labelSizes.map((l) => ({ ...l, FTLbsGroup: l.FTLbsGroup || '1' }))
          }
          paperSizes={paperSizes}
          preprints={preprints}
          sysOptions={sysOptions}
          printerServers={printerServers}
          branches={branches}
          windowsPrinters={windowsPrinters}
          driverMappings={driverMappings}
          driverServerCode={driverServerCode}
          printers={printers}
          onRefreshAll={refreshLabelSizesAndMappings}
          parseApiJson={parseApiJson}
          formatTrayLabel={formatTrayLabel}
          initialEditLbsCode={settingsEditLbsCode}
          onClearInitialEdit={() => setSettingsEditLbsCode(null)}
        />
      ) : (
      <main className="max-w-[1600px] w-full mx-auto p-4">
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-4 items-start">
          {/* Left Panel (4 cols): กำหนดเงื่อนไข */}
          <div className="lg:col-span-4">
            <Card className="border-slate-200 shadow-xs overflow-hidden py-0 gap-0">
              <CardHeader
                className="bg-[#3d4a57] text-white py-2.5 px-4 flex flex-row items-center justify-between cursor-pointer"
                onClick={() => setFilterOpen(o => !o)}
              >
                <CardTitle className="text-sm font-semibold">กำหนดเงื่อนไข</CardTitle>
                {filterOpen ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
              </CardHeader>
              {filterOpen && (
                <CardContent className="p-4 bg-white">
                  <form
                    className="space-y-3"
                    onSubmit={(e) => {
                      e.preventDefault();
                      handleMoveDataIntoTable();
                    }}
                  >
                  {/* Row 1: Price Type + Total Print */}
                  <div className="grid grid-cols-2 gap-3">
                    <div className="space-y-1">
                      <Label className="text-xs font-medium">ประเภทราคา</Label>
                      <Select value={sheetPriceType} onValueChange={(v: any) => setSheetPriceType(v)}>
                        <SelectTrigger className="w-full h-8 text-xs"><SelectValue /></SelectTrigger>
                        <SelectContent>
                          <SelectItem value="Normal">ราคาปกติ</SelectItem>
                          <SelectItem value="Promotion">ราคาโปรโมชัน</SelectItem>
                          <SelectItem value="All">ทั้งหมด</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-1">
                      <Label className="text-xs font-medium">จำนวนพิมพ์ (ดวง)</Label>
                      <Input
                        type="number"
                        min={1}
                        value={defaultQty}
                        onChange={e => setDefaultQty(Math.max(1, parseInt(e.target.value) || 1))}
                        className="h-8 text-xs text-right"
                      />
                    </div>
                  </div>

                  {/* Row 2: Product From - To (Upfront — used frequently) */}
                  <div className="grid grid-cols-2 gap-3">
                    <div className="space-y-1">
                      <Label className="text-xs font-medium">จากสินค้า</Label>
                      <div
                        className="flex cursor-pointer group"
                        title="คลิกเพื่อเลือกสินค้า"
                        onClick={() => {
                          setRangeBrowseSearch('');
                          setRangeBrowsePage(1);
                          setRangeBrowseTarget({ title: 'แสดงข้อมูล สินค้า (จากสินค้า)', onSelect: setPdtFrom });
                        }}
                      >
                        <Input
                          readOnly
                          placeholder="ทั้งหมด"
                          value={pdtFrom}
                          className="h-8 text-xs rounded-r-none cursor-pointer bg-white group-hover:bg-slate-50"
                        />
                        <Button
                          type="button"
                          variant="outline"
                          size="icon"
                          tabIndex={-1}
                          className="h-8 w-8 shrink-0 rounded-l-none border-l-0 bg-slate-50 group-hover:bg-slate-100"
                        >
                          <Search className="h-3.5 w-3.5" />
                        </Button>
                      </div>
                    </div>
                    <div className="space-y-1">
                      <Label className="text-xs font-medium">ถึงสินค้า</Label>
                      <div
                        className="flex cursor-pointer group"
                        title="คลิกเพื่อเลือกสินค้า"
                        onClick={() => {
                          setRangeBrowseSearch('');
                          setRangeBrowsePage(1);
                          setRangeBrowseTarget({ title: 'แสดงข้อมูล สินค้า (ถึงสินค้า)', onSelect: setPdtTo });
                        }}
                      >
                        <Input
                          readOnly
                          placeholder="ทั้งหมด"
                          value={pdtTo}
                          className="h-8 text-xs rounded-r-none cursor-pointer bg-white group-hover:bg-slate-50"
                        />
                        <Button
                          type="button"
                          variant="outline"
                          size="icon"
                          tabIndex={-1}
                          className="h-8 w-8 shrink-0 rounded-l-none border-l-0 bg-slate-50 group-hover:bg-slate-100"
                        >
                          <Search className="h-3.5 w-3.5" />
                        </Button>
                      </div>
                    </div>
                  </div>

                  {/* Row 3: Effective Date */}
                  <div className="space-y-1">
                    <label className="inline-flex items-center gap-1.5 text-xs font-medium cursor-pointer select-none">
                      <Checkbox checked={useEffectiveDate} onCheckedChange={v => setUseEffectiveDate(!!v)} />
                      <span>{useEffectiveDate ? 'กรองวันที่เริ่มมีผล' : 'กรองตามวันที่มีผล (ปิดอยู่ — แสดงทุกวัน)'}</span>
                    </label>
                    <DateField value={effectiveDate} onChange={setEffectiveDate} disabled={!useEffectiveDate} />
                  </div>

                  {/* Collapsible Secondary Range Filters */}
                  {(() => {
                    const activeExtraCount = [
                      bchFrom, bchTo, docDateFrom, docDateTo, docNoFrom, docNoTo,
                      pdtGrpFilter, pdtGrpTo, pdtTypeFrom, pdtTypeTo,
                      pdtBrandFilter, pdtBrandTo, pdtModelFrom, pdtModelTo,
                      agencyCode, priceGroupCode,
                    ].filter(Boolean).length;
                    return (
                      <div className="border border-slate-200 rounded-md overflow-hidden">
                        <button
                          type="button"
                          onClick={() => setAdvancedFiltersOpen(o => !o)}
                          className="w-full flex items-center justify-between px-3 py-2 bg-slate-50 hover:bg-slate-100 text-xs font-medium text-slate-700 transition-colors"
                        >
                          <span className="flex items-center gap-1.5">
                            <span>ตัวกรองเพิ่มเติม (สาขา / เอกสาร / หมวด / ยี่ห้อ)</span>
                            {activeExtraCount > 0 && (
                              <Badge variant="secondary" className="h-4 px-1.5 text-[10px] bg-teal-100 text-teal-800">
                                {activeExtraCount}
                              </Badge>
                            )}
                          </span>
                          {advancedFiltersOpen ? <ChevronUp className="h-3.5 w-3.5 text-slate-500" /> : <ChevronDown className="h-3.5 w-3.5 text-slate-500" />}
                        </button>
                        {advancedFiltersOpen && (
                          <div className="p-3 space-y-3 bg-white border-t border-slate-200">
                            {/* Branch From - To */}
                            <div className="grid grid-cols-2 gap-3">
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">จากสาขา</Label>
                                <FilterSelect
                                  value={bchFrom}
                                  onChange={setBchFrom}
                                  options={branches.map(b => ({ code: b.FTBchCode, name: b.FTBchName || b.FTBchCode }))}
                                />
                              </div>
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">ถึงสาขา</Label>
                                <FilterSelect
                                  value={bchTo}
                                  onChange={setBchTo}
                                  options={branches.map(b => ({ code: b.FTBchCode, name: b.FTBchName || b.FTBchCode }))}
                                />
                              </div>
                            </div>

                            {/* Doc Date From - To */}
                            <div className="grid grid-cols-2 gap-3">
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">วันที่เอกสารจาก</Label>
                                <DateField value={docDateFrom} onChange={setDocDateFrom} />
                              </div>
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">ถึงวันที่เอกสาร</Label>
                                <DateField value={docDateTo} onChange={setDocDateTo} />
                              </div>
                            </div>

                            {/* Doc No From - To */}
                            <div className="grid grid-cols-2 gap-3">
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">เลขที่เอกสารจาก</Label>
                                <Input placeholder="ทั้งหมด" value={docNoFrom} onChange={e => setDocNoFrom(e.target.value)} className="h-8 text-xs" />
                              </div>
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">ถึงเลขที่เอกสาร</Label>
                                <Input placeholder="ทั้งหมด" value={docNoTo} onChange={e => setDocNoTo(e.target.value)} className="h-8 text-xs" />
                              </div>
                            </div>

                            <hr className="border-slate-200" />

                            {/* Product Group From - To */}
                            <div className="grid grid-cols-2 gap-3">
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">จากกลุ่มสินค้า</Label>
                                <FilterSelect value={pdtGrpFilter} onChange={setPdtGrpFilter} options={masterGroups} />
                              </div>
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">ถึงกลุ่มสินค้า</Label>
                                <FilterSelect value={pdtGrpTo} onChange={setPdtGrpTo} options={masterGroups} />
                              </div>
                            </div>

                            {/* Product Type From - To */}
                            <div className="grid grid-cols-2 gap-3">
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">จากประเภท</Label>
                                <FilterSelect value={pdtTypeFrom} onChange={setPdtTypeFrom} options={masterTypes} />
                              </div>
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">ถึงประเภท</Label>
                                <FilterSelect value={pdtTypeTo} onChange={setPdtTypeTo} options={masterTypes} />
                              </div>
                            </div>

                            {/* Brand From - To */}
                            <div className="grid grid-cols-2 gap-3">
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">จากยี่ห้อ</Label>
                                <FilterSelect value={pdtBrandFilter} onChange={setPdtBrandFilter} options={masterBrands} />
                              </div>
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">ถึงยี่ห้อ</Label>
                                <FilterSelect value={pdtBrandTo} onChange={setPdtBrandTo} options={masterBrands} />
                              </div>
                            </div>

                            {/* Model From - To */}
                            <div className="grid grid-cols-2 gap-3">
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">จากรุ่น</Label>
                                <FilterSelect value={pdtModelFrom} onChange={setPdtModelFrom} options={masterModels} />
                              </div>
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">ถึงรุ่น</Label>
                                <FilterSelect value={pdtModelTo} onChange={setPdtModelTo} options={masterModels} />
                              </div>
                            </div>

                            {/* Agency & Price Group */}
                            <div className="grid grid-cols-2 gap-3">
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">ตัวแทนขาย</Label>
                                <FilterSelect value={agencyCode} onChange={setAgencyCode} options={masterAgencies} />
                              </div>
                              <div className="space-y-1">
                                <Label className="text-xs font-medium">กลุ่มราคา</Label>
                                <FilterSelect value={priceGroupCode} onChange={setPriceGroupCode} options={masterPriceGroups} />
                              </div>
                            </div>
                          </div>
                        )}
                      </div>
                    );
                  })()}

                  {/* Bottom Action Buttons (Version 5 exact action bar) */}
                  <div className="pt-2 flex flex-wrap items-center justify-between gap-2">
                    <div className="flex gap-1.5">
                      <Button type="button" size="sm" variant="outline" className="h-8 text-xs border-[#007a87] text-[#007a87]" onClick={() => setImportModalOpen(true)}>
                        <FileUp className="h-3.5 w-3.5 mr-1" />นำเข้า
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        className="h-8 text-xs bg-amber-600 hover:bg-amber-700 text-white"
                        onClick={() => { setScanPage(1); setScanModalOpen(true); }}
                      >
                        <ScanLine className="h-3.5 w-3.5 mr-1" />สแกนสินค้า
                      </Button>
                    </div>
                    <div className="flex gap-1.5">
                      <Button type="button" size="sm" variant="outline" className="h-8 text-xs" onClick={handleClearFilters}>
                        <RotateCcw className="h-3.5 w-3.5 mr-1" />ล้างข้อมูล
                      </Button>
                      <Button
                        type="submit"
                        size="sm"
                        className="h-8 text-xs bg-[#007a87] hover:bg-[#00606b] text-white"
                        disabled={movingData}
                      >
                        {movingData && <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />}
                        แสดงข้อมูล
                      </Button>
                    </div>
                  </div>
                  </form>
                </CardContent>
              )}
            </Card>
            
    </div>

          {/* Right Panel (8 cols): เงื่อนไขการพิมพ์ + ตารางข้อมูล */}
          <div className="lg:col-span-8 space-y-4">
            {/* Top Card: เงื่อนไขการพิมพ์ */}
            <Card className="border-slate-200 shadow-xs overflow-hidden py-0 gap-0">
              <CardHeader className="bg-[#3d4a57] text-white py-2.5 px-4">
                <CardTitle className="text-sm font-semibold">เงื่อนไขการพิมพ์</CardTitle>
              </CardHeader>
              <CardContent className="p-4 bg-white">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-1">
                    <div className="flex items-center justify-between gap-2">
                      <Label className="text-xs font-medium"><span className="text-red-500">*</span> รูปแบบป้าย (Label Format)</Label>
                      {selectedLabelCode && (
                        <button
                          type="button"
                          onClick={() => {
                            setDesignLabelCode(selectedLabelCode);
                            setActiveMenu('design');
                          }}
                          className="text-[11px] font-medium text-[#007a87] hover:underline flex items-center gap-1"
                          title="เปิดหน้าออกแบบเพื่อแก้ไขรูปแบบป้ายนี้"
                        >
                          <Palette className="h-3 w-3" />แก้ไขแบบป้ายนี้
                        </button>
                      )}
                    </div>
                    <SearchableSelect
                      value={selectedLabelCode}
                      onValueChange={setSelectedLabelCode}
                      placeholder="เลือกรูปแบบป้าย..."
                      searchPlaceholder="ค้นหารหัสหรือชื่อรูปแบบป้าย..."
                      className="h-9 text-xs"
                      options={labelSizes
                        .filter((l) => l.FTLbsStaUse === '1')
                        .map((l) => {
                          const saved = customLayouts[`${l.FTLbsCode}:1`] || customLayouts[`${l.FTLbsCode}:2`];
                          const badge =
                            saved?.designMode === 'preprint'
                              ? 'Pre-Print'
                              : saved || l.FTLbsGroup === '2'
                                ? 'ออกแบบเอง'
                                : 'STD';
                          const badgeClassName =
                            badge === 'Pre-Print'
                              ? 'bg-amber-100 text-amber-800'
                              : badge === 'ออกแบบเอง'
                                ? 'bg-sky-100 text-sky-800'
                                : 'bg-slate-100 text-slate-600';
                          return {
                            value: l.FTLbsCode,
                            label: `${l.FTLbsCode} — ${l.FTLbsName} (${l.FCLbsWidth}×${l.FCLbsHeight}mm)`,
                            keywords: `${l.FTLbsCode} ${l.FTLbsName || ''} ${l.FCLbsWidth}x${l.FCLbsHeight}`,
                            badge,
                            badgeClassName,
                          };
                        })}
                    />
                  </div>
                  <div className="space-y-1">
                    <Label className="text-xs font-medium"><span className="text-red-500">*</span> เครื่องพิมพ์ (สำรอง / LAN Printer)</Label>
                    <SearchableSelect
                      value={selectedPrinterId}
                      onValueChange={setSelectedPrinterId}
                      placeholder="เลือกเครื่องพิมพ์..."
                      searchPlaceholder="ค้นหาชื่อเครื่องพิมพ์ หรือ IP..."
                      className="h-9 text-xs"
                      options={printers.map((p) => ({
                        value: p.id,
                        label: `${p.name} (${p.ip}:${p.port})`,
                        keywords: `${p.name} ${p.ip} ${p.brand}`,
                        badge: p.ip === 'Windows' ? 'WIN' : 'LAN',
                        badgeClassName:
                          p.ip === 'Windows'
                            ? 'bg-slate-100 text-slate-700'
                            : 'bg-emerald-100 text-emerald-800',
                      }))}
                    />
                  </div>
                  
                  </div>
                {selectedLabelCode && (
                  <div className="mt-3 pt-2.5 border-t border-slate-100 flex flex-wrap items-center justify-between gap-2 text-xs">
                    <div className="flex flex-wrap items-center gap-3 text-slate-600">
                      <span>
                        <strong className="text-slate-800">ราคาปกติ:</strong>{' '}
                        {activeDriverMapping?.normalPrinter
                          ? `${activeDriverMapping.normalPrinter}${activeDriverMapping.normalTray ? ` (${activeDriverMapping.normalTray})` : ''}`
                          : <span className="text-slate-400">ใช้เครื่องพิมพ์ที่เลือกด้านบน</span>}
                      </span>
                      <span className="text-slate-300">|</span>
                      <span>
                        <strong className="text-red-700">ราคาโปรโมชัน:</strong>{' '}
                        {activeDriverMapping?.promoPrinter
                          ? `${activeDriverMapping.promoPrinter}${activeDriverMapping.promoTray ? ` (${activeDriverMapping.promoTray})` : ''}`
                          : <span className="text-slate-400">ใช้เครื่องพิมพ์ที่เลือกด้านบน</span>}
                      </span>
                    </div>
                    <Button
                      type="button"
                      variant="link"
                      size="sm"
                      className="h-auto p-0 text-xs text-[#007a87] hover:text-[#00606b]"
                      onClick={() => {
                        const row = driverMappings.find(m => m.lbsCode === selectedLabelCode);
                        if (row) setDriverEditRow({ ...row });
                        setDriverModalOpen(true);
                      }}
                    >
                      ตั้งค่าเครื่อง/ถาดของป้ายนี้
                    </Button>
                  </div>
                )}
              </CardContent>
            </Card>

            {/* Bottom Card: ตารางรายการพิมพ์ */}
            <Card className="border-slate-200 shadow-xs bg-white py-0 gap-0">
              <CardContent className="p-4 space-y-3">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <Label className="text-xs font-semibold text-slate-700">ค้นหาข้อมูลสินค้า ({filteredMainRows.length} รายการ)</Label>
                  <div className="relative w-full sm:w-72">
                    <Search className="h-3.5 w-3.5 absolute left-2.5 top-2.5 text-slate-400" />
                    <Input
                      placeholder="กรอกคำค้นหา..."
                      value={mainSearch}
                      onChange={e => setMainSearch(e.target.value)}
                      className="pl-8 h-8 text-xs"
                    />
                    
    </div>
                  
    </div>

                <div className="border rounded overflow-x-auto">
                  <Table>
                    <TableHeader className="bg-[#3d4a57]">
                      <TableRow>
                        <TableHead className="w-10 text-center">
                          <Checkbox
                            checked={filteredMainRows.length > 0 && filteredMainRows.every(r => r.selected)}
                            onCheckedChange={v => {
                              const on = !!v;
                              setMainRows(prev => prev.map(r => ({ ...r, selected: on })));
                            }}
                          />
                        </TableHead>
                        <TableHead className="text-white text-xs">รหัสสินค้า</TableHead>
                        <TableHead className="text-white text-xs">ชื่อสินค้า</TableHead>
                        <TableHead className="text-white text-xs">บาร์โค้ด</TableHead>
                        <TableHead className="text-white text-xs text-right">ราคา</TableHead>
                        <TableHead className="text-white text-xs text-center">ประเภทราคา</TableHead>
                        <TableHead className="text-white text-xs text-center w-24">จำนวน</TableHead>
                        <TableHead className="text-white text-xs text-center w-12">ลบ</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {filteredMainRows.length === 0 ? (
                        <TableRow>
                          <TableCell colSpan={8} className="text-center py-10 text-xs text-slate-400">
                            ไม่พบข้อมูล — กดปุ่ม <strong>สแกนสินค้า</strong> หรือ <strong>แสดงข้อมูล</strong> ด้านซ้ายเพื่อเพิ่มรายการ
                          </TableCell>
                        </TableRow>
                      ) : (
                        filteredMainRows.map(row => (
                          <TableRow key={`${row.barCode}-${row.priType}`}>
                            <TableCell className="text-center">
                              <Checkbox
                                checked={row.selected}
                                onCheckedChange={v =>
                                  setMainRows(prev =>
                                    prev.map(r =>
                                      r.barCode === row.barCode && r.priType === row.priType
                                        ? { ...r, selected: !!v }
                                        : r
                                    )
                                  )
                                }
                              />
                            </TableCell>
                            <TableCell className="text-xs font-mono">{row.pdtCode}</TableCell>
                            <TableCell className="text-xs">{row.pdtName}</TableCell>
                            <TableCell className="text-xs font-mono">{row.barCode}</TableCell>
                            <TableCell className="text-xs text-right font-semibold">{row.price.toFixed(2)}</TableCell>
                            <TableCell className="text-center">
                              <Badge variant={row.priType === '2' ? 'destructive' : 'secondary'} className="text-[11px]">
                                {row.priType === '2' ? 'ราคาโปรโมชัน' : 'ราคาปกติ'}
                              </Badge>
                            </TableCell>
                            <TableCell>
                              <Input
                                type="number"
                                min={1}
                                value={row.qty}
                                onChange={e => {
                                  const qty = Math.max(1, parseInt(e.target.value) || 1);
                                  setMainRows(prev =>
                                    prev.map(r =>
                                      r.barCode === row.barCode && r.priType === row.priType ? { ...r, qty } : r
                                    )
                                  );
                                }}
                                className="h-7 text-xs text-right"
                              />
                            </TableCell>
                            <TableCell className="text-center">
                              <Button
                                size="icon"
                                variant="ghost"
                                className="h-7 w-7 text-red-500"
                                onClick={() =>
                                  setMainRows(prev =>
                                    prev.filter(r => !(r.barCode === row.barCode && r.priType === row.priType))
                                  )
                                }
                              >
                                <Trash2 className="h-3.5 w-3.5" />
                              </Button>
                            </TableCell>
                          </TableRow>
                        ))
                      )}
                    </TableBody>
                  </Table>
                  
                  </div>
              </CardContent>
            </Card>
            
    </div>
          
    </div>
      </main>
      )}
      </div>

      {/* ===== Modal: สแกนสินค้า (dcmPrintBarEventGetPdtInTmp) ===== */}
      <Dialog open={scanModalOpen} onOpenChange={setScanModalOpen}>
        <DialogContent className="w-[95vw] sm:max-w-4xl max-h-[90vh]">
          <DialogHeader className="bg-[#3d4a57] text-white -mx-6 -mt-6 px-6 py-3 rounded-t-lg">
            <DialogTitle className="text-base font-bold">สแกนสินค้า</DialogTitle>
            <DialogDescription className="text-slate-300 text-xs">
              สแกนบาร์โค้ด หรือกด + เพื่อเลือกสินค้าเข้าคิวพิมพ์
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3 pt-2">
            {/* Top Toolbar inside Scan Modal */}
            <div className="flex flex-wrap items-end justify-between gap-3">
              <div className="w-full sm:w-64 space-y-1">
                <Label className="text-xs font-medium">ค้นหาในรายการสแกน</Label>
                <div className="relative">
                  <Search className="h-3.5 w-3.5 absolute left-2.5 top-2.5 text-slate-400" />
                  <Input
                    placeholder="รหัส / ชื่อ / บาร์โค้ด"
                    value={scanSearch}
                    onChange={e => { setScanSearch(e.target.value); setScanPage(1); }}
                    className="pl-8 h-8 text-xs"
                  />
                  
    </div>
                
    </div>

              <div className="flex flex-wrap items-center gap-2">
                <Button
                  size="sm"
                  variant="outline"
                  className="h-8 text-xs text-red-600 border-red-200"
                  disabled={!scanTmpRows.some(r => r.selected)}
                  onClick={() => {
                    setScanTmpRows(prev => prev.filter(r => !r.selected));
                    toast.info('ลบรายการที่เลือกออกจากตารางสแกนแล้ว');
                  }}
                >
                  <Trash2 className="h-3.5 w-3.5 mr-1" />ลบที่เลือก
                </Button>
                <Input
                  placeholder="สแกนบาร์โค้ดแล้วกด Enter"
                  value={scanInputBarcode}
                  onChange={e => setScanInputBarcode(e.target.value)}
                  onKeyDown={e => { if (e.key === 'Enter') { e.preventDefault(); handleScanBarcodeEnter(); } }}
                  className="w-56 h-8 text-xs"
                />
                <Button
                  size="sm"
                  className="h-8 bg-[#007a87] hover:bg-[#00606b] text-white"
                  onClick={() => setBrowseOpen(true)}
                  title="Browse สินค้า"
                >
                  <Plus className="h-4 w-4" />
                </Button>
                
    </div>
              
    </div>

            {/* Temp DataTable (10 rows per page like FSaPriBarGetPdtTmpEvent) */}
            <div className="border rounded overflow-x-auto">
              <Table>
                <TableHeader className="bg-[#3d4a57]">
                  <TableRow>
                    <TableHead className="w-10 text-center">
                      <Checkbox
                        checked={allScanSelected}
                        onCheckedChange={v => {
                          const on = !!v;
                          const pageIds = new Set(pagedScanTmp.map(r => `${r.pdtCode}|${r.barCode}`));
                          setScanTmpRows(prev =>
                            prev.map(r => (pageIds.has(`${r.pdtCode}|${r.barCode}`) ? { ...r, selected: on } : r))
                          );
                        }}
                      />
                    </TableHead>
                    <TableHead className="text-white text-xs text-center">รหัสสินค้า</TableHead>
                    <TableHead className="text-white text-xs text-center">ชื่อสินค้า</TableHead>
                    <TableHead className="text-white text-xs text-center">บาร์โค้ด</TableHead>
                    <TableHead className="text-white text-xs text-center w-24">จำนวนพิมพ์</TableHead>
                    <TableHead className="text-white text-xs text-center w-14">ลบ</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {pagedScanTmp.length === 0 ? (
                    <TableRow>
                      <TableCell colSpan={6} className="text-center py-8 text-xs text-slate-400">
                        ไม่พบข้อมูล — สแกนบาร์โค้ดด้านบน หรือกดปุ่ม <strong>+</strong> เพื่อเลือกสินค้า
                      </TableCell>
                    </TableRow>
                  ) : (
                    pagedScanTmp.map(r => (
                      <TableRow key={`${r.pdtCode}-${r.barCode}`}>
                        <TableCell className="text-center">
                          <Checkbox
                            checked={r.selected}
                            onCheckedChange={v =>
                              setScanTmpRows(prev =>
                                prev.map(item =>
                                  item.pdtCode === r.pdtCode && item.barCode === r.barCode
                                    ? { ...item, selected: !!v }
                                    : item
                                )
                              )
                            }
                          />
                        </TableCell>
                        <TableCell className="text-xs font-mono">{r.pdtCode}</TableCell>
                        <TableCell className="text-xs">{r.pdtName}</TableCell>
                        <TableCell className="text-xs font-mono">{r.barCode}</TableCell>
                        <TableCell>
                          <Input
                            type="number"
                            min={1}
                            value={r.qty}
                            onChange={e => {
                              const qty = Math.max(1, parseInt(e.target.value) || 1);
                              setScanTmpRows(prev =>
                                prev.map(item =>
                                  item.pdtCode === r.pdtCode && item.barCode === r.barCode ? { ...item, qty } : item
                                )
                              );
                            }}
                            className="h-7 text-xs text-right"
                          />
                        </TableCell>
                        <TableCell className="text-center">
                          <Button
                            size="icon"
                            variant="ghost"
                            className="h-7 w-7 text-red-500"
                            onClick={() =>
                              setScanTmpRows(prev =>
                                prev.filter(item => !(item.pdtCode === r.pdtCode && item.barCode === r.barCode))
                              )
                            }
                          >
                            <Trash2 className="h-3.5 w-3.5" />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                </TableBody>
              </Table>
              
    </div>

            {/* Pagination Footer */}
            <div className="flex items-center justify-between text-xs text-slate-600">
              <span>
                พบข้อมูลทั้งหมด {filteredScanTmp.length} รายการ · หน้า {Math.min(scanPage, scanTotalPages)} / {scanTotalPages}
              </span>
              <div className="flex items-center gap-1">
                <Button size="icon" variant="outline" className="h-7 w-7" disabled={scanPage <= 1} onClick={() => setScanPage(p => Math.max(1, p - 1))}>
                  <ChevronLeft className="h-3.5 w-3.5" />
                </Button>
                <span className="px-2 font-medium">{scanPage}</span>
                <Button size="icon" variant="outline" className="h-7 w-7" disabled={scanPage >= scanTotalPages} onClick={() => setScanPage(p => Math.min(scanTotalPages, p + 1))}>
                  <ChevronRight className="h-3.5 w-3.5" />
                </Button>
                
    </div>
              
    </div>
            
    </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setScanModalOpen(false)}>ยกเลิก</Button>
            <Button className="bg-[#007a87] hover:bg-[#00606b] text-white" onClick={handlePreviewScanImport}>
              ยืนยันการนำเข้า
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ===== Modal: Browse สินค้า (+) ===== */}
      <Dialog open={browseOpen} onOpenChange={setBrowseOpen}>
        <DialogContent className="w-[95vw] sm:max-w-2xl max-h-[85vh] flex flex-col overflow-hidden">
          <DialogHeader>
            <DialogTitle className="text-base">เลือกสินค้า</DialogTitle>
            <DialogDescription className="text-xs">คลิกเลือกสินค้าเพื่อเพิ่มเข้าตารางสแกน</DialogDescription>
          </DialogHeader>
          <div className="flex gap-2">
            <Input
              placeholder="ค้นหารหัส / ชื่อสินค้า / บาร์โค้ด..."
              value={browseSearch}
              onChange={e => handleBrowseSearch(e.target.value)}
              className="h-8 text-xs"
            />
          </div>
          <div className="max-h-72 overflow-y-auto border rounded-md">
            {browseLoading ? (
              <div className="p-6 text-center text-xs text-slate-500">กำลังค้นหา...</div>
            ) : (
              <Table>
                <TableHeader className="sticky top-0 z-10 bg-[#34495e]">
                  <TableRow className="bg-[#34495e] hover:bg-[#34495e]">
                    <TableHead className="text-white text-xs h-8">รหัส</TableHead>
                    <TableHead className="text-white text-xs h-8">ชื่อสินค้า</TableHead>
                    <TableHead className="text-white text-xs h-8">บาร์โค้ด</TableHead>
                    <TableHead className="text-white text-xs h-8 w-20 text-center">เลือก</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {browseList.map(p => {
                    const bar = p.bars?.[0]?.FTPdbBarCode || p.FTPdtCode;
                    return (
                      <TableRow key={`${p.FTPdtCode}-${bar}`} className="hover:bg-slate-50">
                        <TableCell className="text-xs font-mono py-1.5">{p.FTPdtCode}</TableCell>
                        <TableCell className="text-xs py-1.5">{p.FTPdtName}</TableCell>
                        <TableCell className="text-xs font-mono py-1.5">{bar}</TableCell>
                        <TableCell className="text-center py-1.5">
                          <Button
                            size="sm"
                            variant="outline"
                            className="h-6 text-xs"
                            onClick={() => {
                              addPdtToScanTmp(p.FTPdtCode, p.FTPdtName, bar);
                              toast.success(`เพิ่ม ${p.FTPdtName}`);
                            }}
                          >
                            + เพิ่ม
                          </Button>
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            )}
          </div>
          <DialogFooter>
            <Button onClick={() => setBrowseOpen(false)}>เสร็จสิ้น</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ===== Modal: Preview ผลการตรวจสอบเงื่อนไขก่อนนำเข้า (odvPrnPreviewModal) ===== */}
      <Dialog open={importPreviewOpen} onOpenChange={setImportPreviewOpen}>
        <DialogContent className="w-[95vw] sm:max-w-3xl max-h-[85vh] flex flex-col overflow-hidden">
          <DialogHeader>
            <DialogTitle className="text-base">ตรวจสอบเงื่อนไขก่อนนำเข้า (Preview)</DialogTitle>
            <DialogDescription className="text-xs">
              ผ่านเงื่อนไข {previewPass.length} รายการ · ไม่ผ่านเงื่อนไข {previewFail.length} รายการ
            </DialogDescription>
          </DialogHeader>
          <div className="max-h-[55vh] overflow-y-auto space-y-4 pr-1">
            {previewPass.length > 0 && (
              <div className="space-y-1.5 mb-4">
                <div className="flex items-center gap-1.5 text-xs font-semibold text-emerald-700">
                  <CheckCircle2 className="h-4 w-4" />รายการที่ผ่านเงื่อนไข ({previewPass.length})
                  
    </div>
                <div className="border rounded">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="text-xs">รหัส</TableHead>
                        <TableHead className="text-xs">ชื่อสินค้า</TableHead>
                        <TableHead className="text-xs">บาร์โค้ด</TableHead>
                        <TableHead className="text-xs text-right">ราคา</TableHead>
                        <TableHead className="text-xs text-center">จำนวน</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {previewPass.map((r, i) => (
                        <TableRow key={i}>
                          <TableCell className="text-xs font-mono">{r.pdtCode}</TableCell>
                          <TableCell className="text-xs">{r.pdtName}</TableCell>
                          <TableCell className="text-xs font-mono">{r.barCode}</TableCell>
                          <TableCell className="text-xs text-right">{r.price.toFixed(2)}</TableCell>
                          <TableCell className="text-xs text-center">{r.qty}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                  
    </div>
                
    </div>
            )}

            {previewFail.length > 0 && (
              <div className="space-y-1.5">
                <div className="flex items-center gap-1.5 text-xs font-semibold text-amber-700">
                  <AlertTriangle className="h-4 w-4" />รายการที่ไม่ผ่านเงื่อนไข ({previewFail.length})
                  
    </div>
                <div className="border rounded">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="text-xs">รหัส</TableHead>
                        <TableHead className="text-xs">บาร์โค้ด</TableHead>
                        <TableHead className="text-xs">สาเหตุ</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {previewFail.map((f, i) => (
                        <TableRow key={i}>
                          <TableCell className="text-xs font-mono">{f.pdtCode}</TableCell>
                          <TableCell className="text-xs font-mono">{f.barCode}</TableCell>
                          <TableCell className="text-xs text-red-600">{f.reason}</TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                  
    </div>
                
    </div>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setImportPreviewOpen(false)}>ยกเลิก</Button>
            <Button className="bg-[#007a87] hover:bg-[#00606b] text-white" onClick={handleConfirmImportToMain} disabled={previewPass.length === 0}>
              ยืนยันการนำเข้า ({previewPass.length})
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ===== Modal: ส่งพิมพ์แยกแท็บ Preview แผ่นกระดาษ / ราคาปกติ / ราคาโปรโมชัน (odvPRNModalSendPrint) ===== */}
      <Dialog open={sendPrintOpen} onOpenChange={setSendPrintOpen}>
        <DialogContent className="w-[95vw] sm:max-w-4xl max-h-[90vh] flex flex-col overflow-hidden">
          <DialogHeader>
            <DialogTitle className="text-base">
              Preview แผ่นกระดาษ {a4Layout.paperSize === 'B5_JIS' ? 'B5 JIS' : a4Layout.paperSize === 'B5_ISO' ? 'B5 ISO' : a4Layout.paperSize === 'LETTER' ? 'Letter' : a4Layout.paperSize === 'CUSTOM' ? 'กำหนดเอง' : 'A4'} ({previewSheetCount} แผ่น · {a4Layout.totalLabels} ดวง)
            </DialogTitle>
            <DialogDescription className="text-xs">
              รูปแบบป้าย: {selectedLabelCode} ({a4Layout.widthMm}×{a4Layout.heightMm} mm บนกระดาษ {a4Layout.paperWidthMm}×{a4Layout.paperHeightMm} mm · จัดเรียง {a4Layout.cols}×{a4Layout.rows} = {a4Layout.perPage} ดวง/แผ่น){hasSplitPreview ? ' · แยกชุดส่งพิมพ์ตามปลายทาง' : ` · เครื่องพิมพ์: ${printers.find(p => p.id === selectedPrinterId)?.name || 'Windows Print'}`}
            </DialogDescription>
          </DialogHeader>
          <Tabs defaultValue="preview" className="flex-1 min-h-0 flex flex-col">
            <TabsList className="shrink-0">
              <TabsTrigger value="preview">
                Preview แผ่น ({previewSheetCount})
              </TabsTrigger>
              <TabsTrigger value="normal">ราคาปกติ ({normalRows.length})</TabsTrigger>
              <TabsTrigger value="promo">ราคาโปรโมชัน ({promoRows.length})</TabsTrigger>
            </TabsList>
            <TabsContent value="preview" className="border rounded p-2 flex-1 min-h-0 overflow-hidden flex flex-col gap-2">
              {(() => {
                const anyPreprint =
                  resolveCustomLayout(selectedLabelCode, '1')?.designMode === 'preprint' ||
                  resolveCustomLayout(selectedLabelCode, '2')?.designMode === 'preprint';
                const showOffsetControls = anyPreprint || printOffsetX !== 0 || printOffsetY !== 0;
                return (
                  <div className="flex flex-wrap items-center justify-between gap-2 bg-amber-50/70 border border-amber-200 rounded px-2.5 py-1.5 shrink-0">
                    <div className="flex flex-wrap items-center gap-2">
                      {anyPreprint ? (
                        <div className="inline-flex rounded border border-amber-300 bg-white p-0.5">
                          <button
                            type="button"
                            onClick={() => setPreviewPreprintView('guide')}
                            className={`px-2 py-0.5 text-[11px] font-medium rounded-xs ${
                              previewPreprintView === 'guide'
                                ? 'bg-amber-600 text-white'
                                : 'text-slate-600 hover:text-slate-900'
                            }`}
                          >
                            จำลองทาบลง Pre-Print
                          </button>
                          <button
                            type="button"
                            onClick={() => setPreviewPreprintView('ink')}
                            className={`px-2 py-0.5 text-[11px] font-medium rounded-xs ${
                              previewPreprintView === 'ink'
                                ? 'bg-slate-800 text-white'
                                : 'text-slate-600 hover:text-slate-900'
                            }`}
                          >
                            เฉพาะหมึกที่จะพิมพ์จริง
                          </button>
                        </div>
                      ) : (
                        <span className="text-[11px] text-slate-700 font-medium">ตัวอย่างแผ่นกระดาษจริง</span>
                      )}

                      <div className="inline-flex items-center rounded border border-slate-200 bg-white p-0.5 text-[10px]">
                        <span className="px-1.5 text-slate-500">ซูม:</span>
                        {[
                          { label: '45%', value: 0.45 },
                          { label: '65%', value: 0.65 },
                          { label: '85%', value: 0.85 },
                          { label: '100%', value: 1 },
                        ].map((z) => (
                          <button
                            key={z.label}
                            type="button"
                            onClick={() => setSheetPreviewScale(z.value)}
                            className={`px-1.5 py-0.5 rounded-xs font-mono ${
                              Math.abs(sheetPreviewScale - z.value) < 0.01
                                ? 'bg-[#007a87] text-white'
                                : 'text-slate-600 hover:bg-slate-100'
                            }`}
                          >
                            {z.label}
                          </button>
                        ))}
                      </div>
                    </div>

                    {showOffsetControls && (
                      <div className="flex flex-wrap items-center gap-2 text-[11px]">
                        <label className="inline-flex items-center gap-1" title="ค่าบวกขยับขวา ค่าลบขยับซ้าย">
                          <span>Offset X:</span>
                          <Input
                            type="number"
                            step={0.5}
                            min={-30}
                            max={30}
                            value={printOffsetX}
                            onChange={(e) => setPrintOffsetX(parseFloat(e.target.value) || 0)}
                            className="h-6 w-14 text-[11px] font-mono bg-white px-1.5"
                          />
                        </label>
                        <label className="inline-flex items-center gap-1" title="ค่าบวกขยับลง ค่าลบขยับขึ้น">
                          <span>Offset Y:</span>
                          <Input
                            type="number"
                            step={0.5}
                            min={-30}
                            max={30}
                            value={printOffsetY}
                            onChange={(e) => setPrintOffsetY(parseFloat(e.target.value) || 0)}
                            className="h-6 w-14 text-[11px] font-mono bg-white px-1.5"
                          />
                        </label>
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          className="h-6 text-[11px] px-2 bg-white border-amber-300 text-amber-900"
                          disabled={savingOffset}
                          onClick={handleSaveQuickPrintOffset}
                        >
                          บันทึก Offset
                        </Button>
                      </div>
                    )}
                  </div>
                );
              })()}
              <div className="max-h-[48vh] overflow-y-auto pr-1 flex-1">
                {hasSplitPreview ? (
                  <div className="space-y-6 py-2 px-1">
                    {a4Layout.normalPages.length > 0 && (
                      <div className="space-y-3 rounded-lg border border-teal-200/80 bg-teal-50/30 p-3">
                        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-teal-100 pb-2">
                          <span className="text-xs font-semibold text-slate-800 flex items-center gap-1.5">
                            <span className="inline-block w-2 h-2 rounded-full bg-[#007a87]" />
                            ชุดราคาปกติ ({a4Layout.normalLabelsCount} ดวง · {a4Layout.normalPages.length} แผ่น)
                          </span>
                          <span className="text-[11px] font-medium text-[#007a87] bg-white border border-teal-200 rounded px-2 py-0.5">
                            {activeDriverMapping?.normalPrinter || printers.find(p => p.id === selectedPrinterId)?.name || 'Default'}
                            {activeDriverMapping?.normalTray ? ` · ถาด: ${activeDriverMapping.normalTray}` : ''}
                          </span>
                        </div>
                        <div className="space-y-5">
                          {renderPreviewSheets(a4Layout.normalPages, 'nor')}
                        </div>
                      </div>
                    )}
                    {a4Layout.promoPages.length > 0 && (
                      <div className="space-y-3 rounded-lg border border-red-200/80 bg-red-50/25 p-3">
                        <div className="flex flex-wrap items-center justify-between gap-2 border-b border-red-100 pb-2">
                          <span className="text-xs font-semibold text-red-800 flex items-center gap-1.5">
                            <span className="inline-block w-2 h-2 rounded-full bg-red-600" />
                            ชุดราคาโปรโมชัน ({a4Layout.promoLabelsCount} ดวง · {a4Layout.promoPages.length} แผ่น)
                          </span>
                          <span className="text-[11px] font-medium text-red-700 bg-white border border-red-200 rounded px-2 py-0.5">
                            {activeDriverMapping?.promoPrinter || printers.find(p => p.id === selectedPrinterId)?.name || 'Default'}
                            {activeDriverMapping?.promoTray ? ` · ถาด: ${activeDriverMapping.promoTray}` : ''}
                          </span>
                        </div>
                        <div className="space-y-5">
                          {renderPreviewSheets(a4Layout.promoPages, 'pmt')}
                        </div>
                      </div>
                    )}
                  </div>
                ) : (
                  <div className="space-y-6 py-2 px-1">
                    {renderPreviewSheets(a4Layout.pages, 'all')}
                  </div>
                )}
              </div>
            </TabsContent>
            <TabsContent value="normal" className="border rounded p-2 max-h-64 overflow-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="text-xs">รหัส</TableHead>
                    <TableHead className="text-xs">ชื่อสินค้า</TableHead>
                    <TableHead className="text-xs">บาร์โค้ด</TableHead>
                    <TableHead className="text-xs text-right">ราคา</TableHead>
                    <TableHead className="text-xs text-right">จำนวน</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {normalRows.length === 0 ? (
                    <TableRow><TableCell colSpan={5} className="text-center text-xs text-slate-400 py-4">ไม่มีรายการราคาปกติ</TableCell></TableRow>
                  ) : normalRows.map((r, i) => (
                    <TableRow key={i}>
                      <TableCell className="text-xs font-mono">{r.pdtCode}</TableCell>
                      <TableCell className="text-xs">{r.pdtName}</TableCell>
                      <TableCell className="text-xs font-mono">{r.barCode}</TableCell>
                      <TableCell className="text-xs text-right">{r.price.toFixed(2)}</TableCell>
                      <TableCell className="text-xs text-right">{r.qty}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TabsContent>
            <TabsContent value="promo" className="border rounded p-2 max-h-64 overflow-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="text-xs">รหัส</TableHead>
                    <TableHead className="text-xs">ชื่อสินค้า</TableHead>
                    <TableHead className="text-xs">บาร์โค้ด</TableHead>
                    <TableHead className="text-xs text-right">ราคาโปรฯ</TableHead>
                    <TableHead className="text-xs text-right">จำนวน</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {promoRows.length === 0 ? (
                    <TableRow><TableCell colSpan={5} className="text-center text-xs text-slate-400 py-4">ไม่มีรายการราคาโปรโมชัน</TableCell></TableRow>
                  ) : promoRows.map((r, i) => (
                    <TableRow key={i}>
                      <TableCell className="text-xs font-mono">{r.pdtCode}</TableCell>
                      <TableCell className="text-xs">{r.pdtName}</TableCell>
                      <TableCell className="text-xs font-mono">{r.barCode}</TableCell>
                      <TableCell className="text-xs text-right text-red-600 font-semibold">{r.price.toFixed(2)}</TableCell>
                      <TableCell className="text-xs text-right">{r.qty}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TabsContent>
          </Tabs>
          <DialogFooter className="flex-wrap gap-2">
            {hasSplitPreview && (
              <div className="w-full text-[11px] bg-teal-50/80 border border-teal-200 rounded px-2.5 py-1.5 text-slate-800 flex flex-wrap gap-x-4 gap-y-1">
                {normalRows.length > 0 && (
                  <span>
                    <strong>ราคาปกติ ({normalRows.length} รายการ · {a4Layout.normalLabelsCount} ดวง):</strong>{' '}
                    {activeDriverMapping?.normalPrinter || 'ใช้เครื่องด้านบน'}
                    {activeDriverMapping?.normalTray ? ` · ถาด: ${activeDriverMapping.normalTray}` : ''}
                  </span>
                )}
                {promoRows.length > 0 && (
                  <span>
                    <strong>ราคาโปรโมชัน ({promoRows.length} รายการ · {a4Layout.promoLabelsCount} ดวง):</strong>{' '}
                    {activeDriverMapping?.promoPrinter || 'ใช้เครื่องด้านบน'}
                    {activeDriverMapping?.promoTray ? ` · ถาด: ${activeDriverMapping.promoTray}` : ''}
                  </span>
                )}
                {labelSizes.find(s => s.FTLbsCode === selectedLabelCode)?.FTLbsGroup === '2' && (
                  <span className="w-full text-amber-700 font-medium">
                    ⚠ รูปแบบ {selectedLabelCode} เป็นสติ๊กเกอร์ม้วน (Thermal) — โหมด Windows Driver จะจัดหน้าเป็นแผ่น A4 (หากใช้เครื่องพิมพ์สติ๊กเกอร์ม้วนแนะนำให้ใช้ LAN EZPL/TSPL)
                  </span>
                )}
              </div>
            )}
            <Button variant="outline" onClick={() => setSendPrintOpen(false)}>ยกเลิก</Button>
            <Button className="bg-[#007a87] hover:bg-[#00606b] text-white" onClick={() => handleConfirmSendPrint()} disabled={printing}>
              {printing && <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />}
              {hasSplitPreview
                ? 'ยืนยันสั่งพิมพ์แยกเครื่อง/ถาด'
                : 'ยืนยันสั่งพิมพ์'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      
{/* ===== Modal: นำเข้าไฟล์ Excel / CSV (Version 5) ===== */}
      <Dialog open={importModalOpen} onOpenChange={setImportModalOpen}>
        <DialogContent className="w-[95vw] sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="text-base">นำเข้าข้อมูลสินค้า (CSV / TXT)</DialogTitle>
            <DialogDescription className="text-xs">
              รองรับไฟล์ .csv หรือ .txt ที่มีคอลัมน์ บาร์โค้ด, จำนวน (เช่น 7622201755638, 2)
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-3 py-2">
            <Input
              type="file"
              accept=".csv,.txt"
              disabled={importingFile}
              onChange={e => {
                const f = e.target.files?.[0];
                if (f) handleExcelFileUpload(f);
              }}
              className="text-xs"
            />
            {importingFile && (
              <div className="flex items-center gap-2 text-xs text-slate-500">
                <Loader2 className="h-4 w-4 animate-spin" />กำลังตรวจสอบข้อมูลในไฟล์...
              </div>
            )}
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setImportModalOpen(false)}>ปิด</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Browse Product Modal (แบบ AdaPos 5.0: รหัสสินค้า | ชื่อสินค้า | หน่วยสินค้า | บาร์โค้ด | ขายปลีก) */}
      <Dialog open={Boolean(rangeBrowseTarget)} onOpenChange={(o) => !o && setRangeBrowseTarget(null)}>
        <DialogContent className="w-[95vw] sm:max-w-3xl max-h-[85vh] flex flex-col overflow-hidden">
          <DialogHeader className="bg-[#3d4a57] text-white -mx-6 -mt-6 px-6 py-3 rounded-t-lg">
            <DialogTitle className="text-sm font-bold">{rangeBrowseTarget?.title || 'แสดงข้อมูล สินค้า'}</DialogTitle>
            <DialogDescription className="text-slate-300 text-xs">
              คลิกเลือกรายการสินค้า (รหัสสินค้า / หน่วย / บาร์โค้ด / ราคาขายปลีก)
            </DialogDescription>
          </DialogHeader>
          {(() => {
            const q = rangeBrowseSearch.trim().toLowerCase();
            const filtered = catalogProducts.filter((p) => {
              if (!q) return true;
              const bar = (p.bars?.[0]?.FTPdbBarCode || p.FTPdtCode).toLowerCase();
              if (rangeBrowseField === 'code') return p.FTPdtCode.toLowerCase().includes(q);
              if (rangeBrowseField === 'name') return p.FTPdtName.toLowerCase().includes(q);
              if (rangeBrowseField === 'barcode') return bar.includes(q);
              return (
                p.FTPdtCode.toLowerCase().includes(q) ||
                p.FTPdtName.toLowerCase().includes(q) ||
                bar.includes(q)
              );
            });
            const totalPages = Math.max(1, Math.ceil(filtered.length / RANGE_BROWSE_PAGE_SIZE));
            const curPage = Math.min(rangeBrowsePage, totalPages);
            const paged = filtered.slice((curPage - 1) * RANGE_BROWSE_PAGE_SIZE, curPage * RANGE_BROWSE_PAGE_SIZE);

            return (
              <div className="space-y-3 flex-1 min-h-0 flex flex-col pt-2">
                <div className="flex flex-wrap items-center gap-2">
                  <Select
                    value={rangeBrowseField}
                    onValueChange={(v: any) => {
                      setRangeBrowseField(v);
                      setRangeBrowsePage(1);
                    }}
                  >
                    <SelectTrigger className="w-36 h-8 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">ทั้งหมด</SelectItem>
                      <SelectItem value="name">ชื่อสินค้า</SelectItem>
                      <SelectItem value="code">รหัสสินค้า</SelectItem>
                      <SelectItem value="barcode">บาร์โค้ด</SelectItem>
                    </SelectContent>
                  </Select>
                  <div className="relative flex-1 min-w-[200px]">
                    <Search className="h-3.5 w-3.5 text-slate-400 absolute left-2.5 top-2.5" />
                    <Input
                      placeholder="กรอกคำค้นหา (รหัส / ชื่อสินค้า / บาร์โค้ด)..."
                      value={rangeBrowseSearch}
                      onChange={(e) => {
                        setRangeBrowseSearch(e.target.value);
                        setRangeBrowsePage(1);
                      }}
                      className="h-8 text-xs pl-8"
                    />
                  </div>
                </div>
                <div className="max-h-72 overflow-y-auto border rounded-md">
                  <Table>
                    <TableHeader className="sticky top-0 z-10 bg-[#34495e]">
                      <TableRow className="bg-[#34495e] hover:bg-[#34495e]">
                        <TableHead className="text-white text-xs h-8 w-24">รหัสสินค้า</TableHead>
                        <TableHead className="text-white text-xs h-8">ชื่อสินค้า</TableHead>
                        <TableHead className="text-white text-xs h-8 w-24">หน่วยสินค้า</TableHead>
                        <TableHead className="text-white text-xs h-8 w-36">บาร์โค้ด</TableHead>
                        <TableHead className="text-white text-xs h-8 w-24 text-right">ขายปลีก</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {paged.length === 0 ? (
                        <TableRow>
                          <TableCell colSpan={5} className="text-center text-xs text-slate-400 py-6">
                            ไม่พบข้อมูลสินค้า
                          </TableCell>
                        </TableRow>
                      ) : (
                        paged.map((p) => {
                          const bar = p.bars?.[0]?.FTPdbBarCode || p.FTPdtCode;
                          const retail = Number(p.FCPdtPrice ?? 0).toFixed(2);
                          return (
                            <TableRow
                              key={`${p.FTPdtCode}-${bar}`}
                              className="cursor-pointer hover:bg-teal-50/60"
                              onClick={() => {
                                rangeBrowseTarget?.onSelect(p.FTPdtCode);
                                setRangeBrowseTarget(null);
                              }}
                            >
                              <TableCell className="text-xs font-mono py-1.5">{p.FTPdtCode}</TableCell>
                              <TableCell className="text-xs py-1.5">{p.FTPdtName}</TableCell>
                              <TableCell className="text-xs py-1.5 text-slate-600">{p.FTPdtUnit || '-'}</TableCell>
                              <TableCell className="text-xs font-mono py-1.5">{bar}</TableCell>
                              <TableCell className="text-xs font-mono py-1.5 text-right">{retail}</TableCell>
                            </TableRow>
                          );
                        })
                      )}
                    </TableBody>
                  </Table>
                </div>
                <div className="flex items-center justify-between text-xs text-slate-500">
                  <span>พบข้อมูล {filtered.length} รายการ · แสดงหน้า {curPage} / {totalPages}</span>
                  <div className="flex items-center gap-1">
                    <Button
                      type="button"
                      variant="outline"
                      size="icon"
                      className="h-7 w-7"
                      disabled={curPage <= 1}
                      onClick={() => setRangeBrowsePage(p => Math.max(1, p - 1))}
                    >
                      <ChevronLeft className="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      size="icon"
                      className="h-7 w-7"
                      disabled={curPage >= totalPages}
                      onClick={() => setRangeBrowsePage(p => Math.min(totalPages, p + 1))}
                    >
                      <ChevronRight className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </div>
              </div>
            );
          })()}
          <DialogFooter className="gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={() => {
                rangeBrowseTarget?.onSelect('');
                setRangeBrowseTarget(null);
              }}
            >
              ล้างค่า (ทั้งหมด)
            </Button>
            <Button variant="outline" size="sm" onClick={() => setRangeBrowseTarget(null)}>ปิด</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ===== Modal เดียวรวม List + Edit: ตั้งค่าไดร์เวอร์เครื่องพิมพ์ (ธีม /AdaBarprint) ===== */}
      <Dialog
        open={driverModalOpen}
        onOpenChange={(o) => {
          if (!o) {
            setDriverModalOpen(false);
            setDriverEditRow(null);
          }
        }}
      >
        <DialogContent
          className={`w-[96vw] max-h-[88vh] flex flex-col overflow-hidden ${
            driverEditRow ? 'sm:max-w-3xl' : 'sm:max-w-5xl'
          }`}
        >
          <DialogHeader className="bg-[#3d4a57] text-white -mx-6 -mt-6 px-6 py-3.5 rounded-t-lg">
            <div className="flex items-center gap-2 pr-6">
              {driverEditRow && (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="h-7 px-2 text-xs text-slate-200 hover:text-white hover:bg-white/10 -ml-2"
                  onClick={() => setDriverEditRow(null)}
                >
                  <ArrowLeft className="h-3.5 w-3.5 mr-1" />
                  กลับหน้ารายการ
                </Button>
              )}
              <DialogTitle className="text-base font-semibold">
                {driverEditRow ? 'กำหนดเครื่องพิมพ์และถาดกระดาษ' : 'ตั้งค่าไดร์เวอร์เครื่องพิมพ์'}
              </DialogTitle>
            </div>
            <DialogDescription className="text-slate-300 text-xs">
              {driverEditRow
                ? `รหัสฉลาก ${driverEditRow.lbsCode} · ${driverEditRow.lbsName}`
                : `กำหนดเครื่องพิมพ์และถาดกระดาษแยกตามรหัสฉลาก (ราคาปกติ vs ราคาโปรโมชัน) — Server: ${driverServerCode}`}
            </DialogDescription>
          </DialogHeader>

          {!driverEditRow ? (
            <>
              <div className="flex-1 min-h-0 flex flex-col pt-2 space-y-3">
                <div className="max-h-[58vh] overflow-y-auto border border-slate-200 rounded-md">
                  <Table>
                    <TableHeader className="sticky top-0 z-10 bg-[#34495e]">
                      <TableRow className="bg-[#34495e] hover:bg-[#34495e]">
                        <TableHead className="text-white text-xs h-9">รหัสฉลาก</TableHead>
                        <TableHead className="text-white text-xs h-9">ชื่อฉลาก</TableHead>
                        <TableHead className="text-white text-xs h-9">เครื่องพิมพ์ราคาปกติ</TableHead>
                        <TableHead className="text-white text-xs h-9">ถาดกระดาษราคาปกติ</TableHead>
                        <TableHead className="text-white text-xs h-9">เครื่องพิมพ์ราคาโปรโมชัน</TableHead>
                        <TableHead className="text-white text-xs h-9">ถาดกระดาษราคาโปรโมชัน</TableHead>
                        <TableHead className="text-white text-xs h-9 w-24 text-center">จัดการ</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {driverMappings.map((m) => (
                        <TableRow key={m.lbsCode} className="hover:bg-slate-50">
                          <TableCell className="text-xs font-mono font-semibold py-2">{m.lbsCode}</TableCell>
                          <TableCell className="text-xs py-2">{m.lbsName}</TableCell>
                          <TableCell className="text-xs py-2">{m.normalPrinter || <span className="text-slate-400">-</span>}</TableCell>
                          <TableCell className="text-xs py-2">{m.normalTray || <span className="text-slate-400">-</span>}</TableCell>
                          <TableCell className="text-xs py-2">{m.promoPrinter || <span className="text-slate-400">-</span>}</TableCell>
                          <TableCell className="text-xs py-2">{m.promoTray || <span className="text-slate-400">-</span>}</TableCell>
                          <TableCell className="text-center py-2">
                            <Button
                              size="sm"
                              variant="outline"
                              className="h-7 text-xs border-[#007a87] text-[#007a87] hover:bg-teal-50/70"
                              onClick={() => setDriverEditRow({ ...m })}
                            >
                              ตั้งค่า
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              </div>
              <DialogFooter>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    setDriverModalOpen(false);
                    setDriverEditRow(null);
                  }}
                >
                  ปิด
                </Button>
              </DialogFooter>
            </>
          ) : (
            <>
              <div className="flex-1 min-h-0 overflow-y-auto space-y-4 py-2 pr-1">
                <div className="grid grid-cols-1 sm:grid-cols-3 gap-3 bg-slate-50 border border-slate-200 rounded-lg p-3">
                  <div className="space-y-1">
                    <Label className="text-[11px] text-slate-500">รหัสฉลาก</Label>
                    <Input value={driverEditRow.lbsCode} readOnly className="h-8 text-xs bg-white font-mono font-semibold" />
                  </div>
                  <div className="space-y-1 sm:col-span-2">
                    <Label className="text-[11px] text-slate-500">ชื่อฉลาก</Label>
                    <Input value={driverEditRow.lbsName} readOnly className="h-8 text-xs bg-white" />
                  </div>
                </div>
                {driverEditRow.lbsGroup === '2' && (
                  <div className="text-[11px] bg-amber-50 border border-amber-200 rounded-md px-3 py-2 text-amber-800">
                    ⚠ รหัสฉลากนี้เป็นกลุ่ม <strong>สติ๊กเกอร์บาร์โค้ด / ม้วน (Thermal)</strong> — การส่งพิมพ์ผ่าน Windows Driver จะจัดวางเป็นแผ่น A4 (หากใช้เครื่องพิมพ์สติ๊กเกอร์ม้วนแนะนำให้ใช้ LAN EZPL/TSPL แทน)
                  </div>
                )}

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {(['normal', 'promo'] as const).map((kind) => {
                    const isPromo = kind === 'promo';
                    const printerVal = isPromo ? driverEditRow.promoPrinter : driverEditRow.normalPrinter;
                    const trayVal = isPromo ? driverEditRow.promoTray : driverEditRow.normalTray;
                    const availableTrays = windowsPrinters.find(w => w.name === printerVal)?.trays || [];
                    const invalidSaved = Boolean(printerVal && trayVal && !availableTrays.includes(trayVal));
                    return (
                      <div
                        key={kind}
                        className={`rounded-lg border p-3.5 space-y-3 shadow-2xs ${
                          isPromo ? 'border-red-200/80 bg-red-50/20' : 'border-slate-200 bg-white'
                        }`}
                      >
                        <div
                          className={`flex items-center justify-between border-b pb-2 ${
                            isPromo ? 'border-red-100' : 'border-slate-100'
                          }`}
                        >
                          <span
                            className={`text-xs font-semibold flex items-center gap-1.5 ${
                              isPromo ? 'text-red-800' : 'text-slate-800'
                            }`}
                          >
                            <span
                              className={`inline-block w-2 h-2 rounded-full ${
                                isPromo ? 'bg-red-600' : 'bg-[#007a87]'
                              }`}
                            />
                            {isPromo ? 'ราคาโปรโมชัน (Promotion Price)' : 'ราคาปกติ (Normal Price)'}
                          </span>
                        </div>
                        <div className="space-y-1">
                          <Label className="text-xs font-medium text-slate-700">
                            {isPromo ? 'เครื่องพิมพ์ราคาโปรโมชัน' : 'เครื่องพิมพ์ราคาปกติ'}
                          </Label>
                          <Select
                            value={printerVal || '__NONE__'}
                            onValueChange={(v) => {
                              const pName = v === '__NONE__' ? '' : v;
                              const wp = windowsPrinters.find(w => w.name === pName);
                              const nextTray = wp?.trays?.[0] || '';
                              setDriverEditRow(
                                isPromo
                                  ? { ...driverEditRow, promoPrinter: pName, promoTray: nextTray }
                                  : { ...driverEditRow, normalPrinter: pName, normalTray: nextTray }
                              );
                            }}
                          >
                            <SelectTrigger className={`w-full h-8 text-xs ${isPromo ? 'bg-white' : ''}`}>
                              <SelectValue placeholder="เลือกเครื่องพิมพ์..." />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="__NONE__">-- ไม่กำหนด --</SelectItem>
                              {windowsPrinters.map(w => (
                                <SelectItem key={`${kind}-${w.name}`} value={w.name}>{w.name}</SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                        </div>
                        <div className="space-y-1">
                          <Label className="text-xs font-medium text-slate-700">
                            {isPromo ? 'ถาดกระดาษราคาโปรโมชัน' : 'ถาดกระดาษราคาปกติ'}
                          </Label>
                          <Select
                            value={trayVal || '__AUTO__'}
                            onValueChange={(v) => {
                              const nextTray = v === '__AUTO__' ? '' : v;
                              setDriverEditRow(
                                isPromo
                                  ? { ...driverEditRow, promoTray: nextTray }
                                  : { ...driverEditRow, normalTray: nextTray }
                              );
                            }}
                            disabled={!printerVal}
                          >
                            <SelectTrigger className={`w-full h-8 text-xs ${isPromo ? 'bg-white' : ''}`}>
                              <SelectValue placeholder="อัตโนมัติ" />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="__AUTO__">-- อัตโนมัติ (Default) --</SelectItem>
                              {invalidSaved && (
                                <SelectItem value={trayVal} className="text-amber-700">
                                  {trayVal} (ไม่พบถาดนี้ในเครื่อง)
                                </SelectItem>
                              )}
                              {availableTrays.map(t => (
                                <SelectItem key={`${kind}-t-${t}`} value={t}>{formatTrayLabel(t)}</SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                          {invalidSaved && (
                            <p className="text-[11px] text-amber-700">
                              ถาด &ldquo;{trayVal}&rdquo; ไม่ได้ติดตั้งบนเครื่องนี้ กรุณาเลือกถาดที่มีอยู่จริง
                            </p>
                          )}
                        </div>
                        <Button
                          type="button"
                          variant="outline"
                          size="sm"
                          className={`w-full h-8 text-xs ${
                            isPromo
                              ? 'border-red-200 text-red-700 bg-white hover:bg-red-50'
                              : 'border-slate-300 text-slate-700 hover:bg-slate-50'
                          }`}
                          disabled={testingDriver !== null || !printerVal}
                          onClick={() => handleTestDriverPrint(kind)}
                        >
                          {testingDriver === kind ? (
                            <Loader2 className="h-3.5 w-3.5 mr-1.5 animate-spin" />
                          ) : (
                            <Printer className="h-3.5 w-3.5 mr-1.5" />
                          )}
                          {isPromo ? 'ทดสอบพิมพ์ราคาโปรโมชัน' : 'ทดสอบพิมพ์ราคาปกติ'}
                        </Button>
                      </div>
                    );
                  })}
                </div>
              </div>
              <DialogFooter className="gap-2 pt-2 border-t border-slate-100">
                <Button type="button" variant="outline" size="sm" onClick={() => setDriverEditRow(null)}>
                  <ArrowLeft className="h-3.5 w-3.5 mr-1" />
                  กลับรายการ
                </Button>
                <Button
                  type="button"
                  size="sm"
                  className="bg-[#007a87] hover:bg-[#00606b] text-white"
                  disabled={savingDriver}
                  onClick={handleSaveDriverMapping}
                >
                  {savingDriver && <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />}
                  บันทึก
                </Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>

    {/* Hidden High-Res Offscreen Capture Sheets for Split Server Direct Printing (Normal vs Promo) */}
    <div aria-hidden="true" style={{ position: 'fixed', left: '-99999px', top: 0, pointerEvents: 'none' }}>
      <div id="v5-capture-normal">
        {a4Layout.normalPages.map((pageItems, pageIdx) => (
          <div
            key={`cap-nor-${pageIdx}`}
            data-capture-sheet="1"
            style={{
              width: `${a4Layout.paperWidthMm}mm`,
              height: `${a4Layout.paperHeightMm}mm`,
              paddingTop: `${a4Layout.magTopMm}mm`,
              paddingBottom: `${a4Layout.magBottomMm}mm`,
              paddingLeft: `${a4Layout.magLeftMm}mm`,
              paddingRight: `${a4Layout.magRightMm}mm`,
              background: '#FFFFFF',
              display: 'grid',
              gridTemplateColumns: `repeat(${a4Layout.cols}, ${a4Layout.widthMm}mm)`,
              gridAutoRows: `${a4Layout.heightMm}mm`,
              columnGap: `${a4Layout.gapHMm}mm`,
              rowGap: `${a4Layout.gapVMm}mm`,
              alignContent: 'start',
              justifyContent: 'start',
              boxSizing: 'border-box',
              overflow: 'hidden',
              transform:
                printOffsetX !== 0 || printOffsetY !== 0
                  ? `translate(${printOffsetX}mm, ${printOffsetY}mm)`
                  : undefined,
            }}
          >
            {pageItems.map((line, idx) => (
              <LabelPreview
                key={`cn-${line.barcode}-${pageIdx}-${idx}`}
                line={line}
                scale={1}
                labelW={a4Layout.widthMm}
                labelH={a4Layout.heightMm}
                lbsCode={selectedLabelCode}
                customLayout={resolveCustomLayout(selectedLabelCode, line.priType)}
                preprintView="ink"
              />
            ))}
          </div>
        ))}
      </div>
      <div id="v5-capture-promo">
        {a4Layout.promoPages.map((pageItems, pageIdx) => (
          <div
            key={`cap-pmt-${pageIdx}`}
            data-capture-sheet="1"
            style={{
              width: `${a4Layout.paperWidthMm}mm`,
              height: `${a4Layout.paperHeightMm}mm`,
              paddingTop: `${a4Layout.magTopMm}mm`,
              paddingBottom: `${a4Layout.magBottomMm}mm`,
              paddingLeft: `${a4Layout.magLeftMm}mm`,
              paddingRight: `${a4Layout.magRightMm}mm`,
              background: '#FFFFFF',
              display: 'grid',
              gridTemplateColumns: `repeat(${a4Layout.cols}, ${a4Layout.widthMm}mm)`,
              gridAutoRows: `${a4Layout.heightMm}mm`,
              columnGap: `${a4Layout.gapHMm}mm`,
              rowGap: `${a4Layout.gapVMm}mm`,
              alignContent: 'start',
              justifyContent: 'start',
              boxSizing: 'border-box',
              overflow: 'hidden',
              transform:
                printOffsetX !== 0 || printOffsetY !== 0
                  ? `translate(${printOffsetX}mm, ${printOffsetY}mm)`
                  : undefined,
            }}
          >
            {pageItems.map((line, idx) => (
              <LabelPreview
                key={`cp-${line.barcode}-${pageIdx}-${idx}`}
                line={line}
                scale={1}
                labelW={a4Layout.widthMm}
                labelH={a4Layout.heightMm}
                lbsCode={selectedLabelCode}
                customLayout={resolveCustomLayout(selectedLabelCode, line.priType)}
                preprintView="ink"
              />
            ))}
          </div>
        ))}
      </div>
    </div>
    {/* Sheet Print Area for Windows / Browser Fallback (1:1 mm) */}
    <style>{`
      @media print {
        @page { size: ${a4Layout.paperWidthMm}mm ${a4Layout.paperHeightMm}mm; margin: 0; }
        * {
          -webkit-print-color-adjust: exact !important;
          print-color-adjust: exact !important;
        }
        body > *:not(#root), [data-sonner-toaster], [data-radix-portal] {
          display: none !important;
        }
      }
    `}</style>
    <div className="hidden print:block p-0 bg-white text-black">
      {a4Layout.pages.map((pageItems, pIdx) => (
        <div
          key={`print-page-${pIdx}`}
          className="break-after-page"
          style={{
            width: `${a4Layout.paperWidthMm}mm`,
            height: `${a4Layout.paperHeightMm}mm`,
            paddingTop: `${a4Layout.magTopMm}mm`,
            paddingBottom: `${a4Layout.magBottomMm}mm`,
            paddingLeft: `${a4Layout.magLeftMm}mm`,
            paddingRight: `${a4Layout.magRightMm}mm`,
            display: 'grid',
            gridTemplateColumns: `repeat(${a4Layout.cols}, ${a4Layout.widthMm}mm)`,
            gridAutoRows: `${a4Layout.heightMm}mm`,
            columnGap: `${a4Layout.gapHMm}mm`,
            rowGap: `${a4Layout.gapVMm}mm`,
            alignContent: 'start',
            justifyContent: 'start',
            boxSizing: 'border-box',
            overflow: 'hidden',
            transform:
              printOffsetX !== 0 || printOffsetY !== 0
                ? `translate(${printOffsetX}mm, ${printOffsetY}mm)`
                : undefined,
          }}
        >
          {pageItems.map((line, idx) => (
            <div key={`print-${line.barcode}-${pIdx}-${idx}`} className="break-inside-avoid">
              <LabelPreview
                line={line}
                scale={1}
                labelW={a4Layout.widthMm}
                labelH={a4Layout.heightMm}
                lbsCode={selectedLabelCode}
                customLayout={resolveCustomLayout(selectedLabelCode, line.priType)}
                preprintView="ink"
              />
            </div>
          ))}
        </div>
      ))}
    </div>
    </>
  );
}
