import type { LabelFormatOption } from '@/lib/labelFormatOptions';
import { LabelPrinterRoutesEditor } from './LabelPrinterRoutesEditor';
import { MasterDatabaseSettings } from './MasterDatabaseSettings';
import type { AuthSessionUser } from '@/lib/auth';
import React, { useMemo, useState } from 'react';
import { toast } from 'sonner';
import {
  ArrowLeft,
  Calculator,
  ChevronDown,
  ChevronUp,
  Edit3,
  FileText,
  Loader2,
  Network,
  Plus,
  Printer,
  Save,
  Search,
  Trash2,
  Wifi,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { TablePagination } from './TablePagination';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { AutoScheduleSettings } from '@/components/AutoScheduleSettings';
import { AutoScheduleHistory } from '@/components/AutoScheduleHistory';
import type { ManualLoadLine } from '@/components/MorningOpsBanner';
import type { PrinterBrand, PrinterConfig } from '@/types/printer';

/** Toggle visibility of Auto-Print setup tab & banner while keeping read-only History tab visible. */
export const ENABLE_AUTO_PRINT_UI = true;
export const ENABLE_AUTO_PRINT_HISTORY_TAB = false;
import {
  mergePaperSizeOptions,
  PAPER_SIZE_PRESETS,
  type PaperSizeCode,
  type PaperSizeRecord,
  type PreprintRecord,
} from '@/types/design';
import { getPaperSizeDisplayName } from '@/lib/previewMedia';

export interface ConfigLabelSizeRow {
  FTLbsCode: string;
  FTLbsGroup: string;
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
}

export interface SysOptionRow {
  FTSopCode: string;
  FTSopApp?: string | null;
  FTSopName?: string | null;
  FTSopValue?: string | null;
  FTSopDesc?: string | null;
  FNSopSeq?: number | null;
}

export interface PrinterServerRow {
  FTPsvCode: string;
  FTPsvName?: string | null;
  FTPsvRmk?: string | null;
  FTPsvStaUse: string;
}

export interface SettingsDriverMapping {
  templateRoutingSupported?: boolean;
  templateRoutes?: Record<string, {printer:string;tray:string;protocol:string}>;
  eventRoutingSupported?: boolean;
  eventRoutes?: Record<string, {printer:string;tray:string;protocol:string}>;
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

export interface SettingsWindowsPrinter {
  name: string;
  isDefault?: boolean;
  trays?: string[];
}

interface SettingsPanelProps {
  workspace?: 'paper' | 'settings' | 'jobs' | 'profiles';

  authUser: AuthSessionUser;
  allLabelSizes: ConfigLabelSizeRow[];
  paperSizes: PaperSizeRecord[];
  preprints: PreprintRecord[];
  sysOptions: SysOptionRow[];
  printerServers: PrinterServerRow[];
  branches: Array<{ FTBchCode: string; FTBchName: string | null }>;
  windowsPrinters: SettingsWindowsPrinter[];
  driverMappings: SettingsDriverMapping[];
  driverServerCode: string;
  printers: PrinterConfig[];
  onRefreshAll: () => Promise<void>;
  parseApiJson: (res: Response) => Promise<any>;
  formatTrayLabel: (tray: string) => string;
  formatOptions: LabelFormatOption[];
  initialEditFormat?: string;
  initialEditLbsCode?: string | null;
  onClearInitialEdit?: () => void;
  initialSubTab?: 'sizes' | 'preprints' | 'printers' | 'devices' | 'autoprint' | 'autoprint_history' | 'connection' | 'jobs' | null;
  onClearInitialSubTab?: () => void;
  onOpenDesign?: (size: string, template: string, promo: boolean, route?: import('@/lib/printProfile').PrinterRoute, onReturn?: (size?: string, template?: string) => void) => void;
  onLoadManualRows?: (lines: ManualLoadLine[]) => void;
}

interface SizeFormState {
  isNew: boolean;
  lbsCode: string;
  lbsName: string;
  lbsGroup: '1' | '2';
  widthMm: number;
  heightMm: number;
  cols: number;
  rows: number;
  magTop: number;
  magLeft: number;
  magBottom: number;
  magRight: number;
  gapH: number;
  gapV: number;
  qtyPageNor: number;
  qtyPagePmt: number;
  staUse: '1' | '2';
  paperSize: PaperSizeCode;
  paperWidthMm: number;
  paperHeightMm: number;
  offsetX: number;
  offsetY: number;
}

export function SettingsPanel({
  workspace = 'settings',
  authUser,
  allLabelSizes,
  paperSizes,
  preprints,
  sysOptions,
  branches,
  windowsPrinters,
  driverMappings,
  driverServerCode,
  printers,
  onRefreshAll,
  parseApiJson,
  formatTrayLabel,
  formatOptions, initialEditFormat, initialEditLbsCode,
  onClearInitialEdit,
  initialSubTab,
  onClearInitialSubTab,
  onLoadManualRows,
  onOpenDesign,
}: SettingsPanelProps) {
  const [subTab, setSubTab] = useState<'sizes' | 'preprints' | 'printers' | 'devices' | 'autoprint' | 'autoprint_history' | 'connection' | 'jobs'>(() => {
    if (initialEditLbsCode) return 'printers';
    if (!ENABLE_AUTO_PRINT_UI && initialSubTab === 'autoprint') {
      return 'devices';
    }
    if (!ENABLE_AUTO_PRINT_HISTORY_TAB && initialSubTab === 'autoprint_history') {
      return 'jobs';
    }
    if (workspace === 'jobs') return 'jobs';
    if (workspace === 'profiles') return 'autoprint';
    return initialSubTab || (workspace === 'paper' ? 'sizes' : 'devices');
  });

  // ── Tab 1: Label Sizes State ───────────────────────────────────────────────
  const [sizeSearch, setSizeSearch] = useState('');
  const [sizeStatusFilter, setSizeStatusFilter] = useState<'all' | '1' | '2'>('all');
  const [sizeGroupFilter, setSizeGroupFilter] = useState<'all' | '1' | '2'>('all');
  const [sizePage, setSizePage] = useState(1);
  const [sizePageSize, setSizePageSize] = useState(10);
  const [sizeForm, setSizeForm] = useState<SizeFormState | null>(null);
  const [sizeFormAdvancedOpen, setSizeFormAdvancedOpen] = useState(false);
  const [savingSize, setSavingSize] = useState(false);
  const [deleteSizeTarget, setDeleteSizeTarget] = useState<ConfigLabelSizeRow | null>(null);
  const [deletingSize, setDeletingSize] = useState(false);

  // ── Pre-Print Vault State ──────────────────────────────────────────────────
  const [preprintSearch, setPreprintSearch] = useState('');
  const [preprintPage, setPreprintPage] = useState(1);
  const [preprintPageSize, setPreprintPageSize] = useState(10);

  // ── Tab 2: Driver Mapping + LAN Printers State ─────────────────────────────
  const [removeFormat, setRemoveFormat] = useState<LabelFormatOption | null>(null);
  const [removingFormat, setRemovingFormat] = useState(false);
  async function deleteFormat() {
    if (!removeFormat || removingFormat) return;
    setRemovingFormat(true);
    try {
      const res = await fetch('/api/templates/delete', {
        method: 'POST', headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code: removeFormat.value }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data.error || 'ลบแบบป้ายไม่สำเร็จ');
      setRemoveFormat(null);
      await onRefreshAll();
      toast.success('ลบแบบป้ายแล้ว');
    } catch (error) {
      toast.error(error instanceof Error ? error.message : 'ลบแบบป้ายไม่สำเร็จ');
    } finally {
      setRemovingFormat(false);
    }
  }
  const [driverSearch, setDriverSearch] = useState('');
  const [editFormat, setEditFormat] = useState(initialEditFormat);
  const [driverPage, setDriverPage] = useState(1);
  const [driverPageSize, setDriverPageSize] = useState(10);
  const [printerPage, setPrinterPage] = useState(1);
  const [printerPageSize, setPrinterPageSize] = useState(10);
  const [bulkPrinterOpen, setBulkPrinterOpen] = useState(false);
  const [bulkNormalPrinter, setBulkNormalPrinter] = useState('');
  const [bulkNormalTray, setBulkNormalTray] = useState('');
  const [bulkPromoPrinter, setBulkPromoPrinter] = useState('');
  const [bulkPromoTray, setBulkPromoTray] = useState('');
  const [savingBulkDriver, setSavingBulkDriver] = useState(false);
  const [driverEditRow, setDriverEditRow] = useState<SettingsDriverMapping | null>(() => {
    if (!initialEditLbsCode) return null;
    const found = driverMappings.find((m) => m.lbsCode === initialEditLbsCode);
    return found ? { ...found } : null;
  });
  const [savingDriver, setSavingDriver] = useState(false);
  const [driverEditorReady,setDriverEditorReady]=useState(false);
  const [testingDriver, setTestingDriver] = useState<'normal' | 'promo' | null>(null);

  const [lanFormOpen, setLanFormOpen] = useState(false);
  const [lanId, setLanId] = useState('');
  const [lanName, setLanName] = useState('');
  const [lanBrand, setLanBrand] = useState<PrinterBrand>('GODEX');
  const [lanIp, setLanIp] = useState('');
  const [lanPort, setLanPort] = useState(9100);
  const [lanDpi, setLanDpi] = useState(203);
  const [lanSpeed, setLanSpeed] = useState(4);
  const [lanDarkness, setLanDarkness] = useState(10);
  const [testingLanId, setTestingLanId] = useState<string | null>(null);

  React.useEffect(() => {
    if (initialEditLbsCode) {
      setSubTab('printers');
      const found = driverMappings.find((m) => m.lbsCode === initialEditLbsCode);
      if (found) { setEditFormat(initialEditFormat); setDriverEditRow({ ...found }); }
      onClearInitialEdit?.();
    }
  }, [initialEditLbsCode, initialEditFormat, driverMappings, onClearInitialEdit]);

  React.useEffect(() => {
    if (initialSubTab) {
      if (
        (!ENABLE_AUTO_PRINT_UI && initialSubTab === 'autoprint') ||
        (!ENABLE_AUTO_PRINT_HISTORY_TAB && initialSubTab === 'autoprint_history')
      ) {
        setSubTab(initialSubTab === 'autoprint_history' ? 'jobs' : 'devices');
      } else {
        setSubTab(initialSubTab);
      }
      onClearInitialSubTab?.();
    }
  }, [initialSubTab, onClearInitialSubTab]);

  // ── Filtered Lists ─────────────────────────────────────────────────────────
  const filteredSizes = useMemo(() => {
    const q = sizeSearch.trim().toLowerCase();
    return allLabelSizes.filter((s) => {
      if (sizeStatusFilter !== 'all' && s.FTLbsStaUse !== sizeStatusFilter) return false;
      if (sizeGroupFilter !== 'all' && s.FTLbsGroup !== sizeGroupFilter) return false;
      if (!q) return true;
      return (
        s.FTLbsCode.toLowerCase().includes(q) ||
        (s.FTLbsName || '').toLowerCase().includes(q)
      );
    });
  }, [allLabelSizes, sizeSearch, sizeStatusFilter, sizeGroupFilter]);

  const filteredMappings = useMemo(() => {
    const q = driverSearch.trim().toLowerCase();
    const mappings = new Map(driverMappings.map(mapping => [mapping.lbsCode, mapping]));
    return formatOptions.flatMap(format => {
      const mapping = mappings.get(format.lbsCode);
      if (!mapping) return [];
      const terms = [format.label, format.keywords, mapping.normalPrinter, mapping.promoPrinter, mapping.templateRoutes?.[format.value]?.printer];
      if (q && !terms.some(term => term?.toLowerCase().includes(q))) return [];
      return [{ format, mapping }];
    });
  }, [driverMappings, driverSearch, formatOptions]);

  const filteredPreprints = useMemo(() => {
    const q = preprintSearch.trim().toLowerCase();
    if (!q) return preprints;
    return preprints.filter(
      (pp) =>
        pp.FTPprCode.toLowerCase().includes(q) ||
        pp.FTPprName.toLowerCase().includes(q) ||
        (pp.FTPprFileName || '').toLowerCase().includes(q)
    );
  }, [preprints, preprintSearch]);

  React.useEffect(() => {
    setSizePage(1);
  }, [sizeSearch, sizeStatusFilter, sizeGroupFilter, sizePageSize]);

  React.useEffect(() => {
    setPreprintPage(1);
  }, [preprintSearch, preprintPageSize]);

  React.useEffect(() => {
    setDriverPage(1);
  }, [driverSearch, driverPageSize]);

  const pagedSizes = useMemo(() => {
    const maxP = Math.max(1, Math.ceil(filteredSizes.length / sizePageSize));
    const cur = Math.min(sizePage, maxP);
    return filteredSizes.slice((cur - 1) * sizePageSize, cur * sizePageSize);
  }, [filteredSizes, sizePage, sizePageSize]);

  const pagedPreprints = useMemo(() => {
    const maxP = Math.max(1, Math.ceil(filteredPreprints.length / preprintPageSize));
    const cur = Math.min(preprintPage, maxP);
    return filteredPreprints.slice((cur - 1) * preprintPageSize, cur * preprintPageSize);
  }, [filteredPreprints, preprintPage, preprintPageSize]);

  const pagedMappings = useMemo(() => {
    const maxP = Math.max(1, Math.ceil(filteredMappings.length / driverPageSize));
    const cur = Math.min(driverPage, maxP);
    return filteredMappings.slice((cur - 1) * driverPageSize, cur * driverPageSize);
  }, [filteredMappings, driverPage, driverPageSize]);

  const pagedPrinters = useMemo(() => {
    const maxP = Math.max(1, Math.ceil(printers.length / printerPageSize));
    const cur = Math.min(printerPage, maxP);
    return printers.slice((cur - 1) * printerPageSize, cur * printerPageSize);
  }, [printers, printerPage, printerPageSize]);

  // ── Handlers: Tab 1 (Label Sizes) ──────────────────────────────────────────
  const openNewSizeForm = () => {
    setSizeForm({
      isNew: true,
      lbsCode: '',
      lbsName: '',
      lbsGroup: '1',
      widthMm: 100,
      heightMm: 38,
      cols: 2,
      rows: 7,
      magTop: 3,
      magLeft: 3,
      magBottom: 3,
      magRight: 3,
      gapH: 2,
      gapV: 2,
      qtyPageNor: 14,
      qtyPagePmt: 14,
      staUse: '1',
      paperSize: 'A4',
      paperWidthMm: 210,
      paperHeightMm: 297,
      offsetX: 0,
      offsetY: 0,
    });
  };

  const openEditSizeForm = (s: ConfigLabelSizeRow) => {
    const cols = Math.max(1, s.FNLbsCol || 1);
    const rows = Math.max(1, s.FNLbsRow || 1);
    const ps = s.FTLbsPaperSize || paperSizes[0]?.FTPpsCode || 'A4';
    const dbPs = paperSizes.find((p) => p.FTPpsCode === ps);
    const fallbackPs = PAPER_SIZE_PRESETS[ps] || PAPER_SIZE_PRESETS.A4;
    setSizeForm({
      isNew: false,
      lbsCode: s.FTLbsCode,
      lbsName: s.FTLbsName || s.FTLbsCode,
      lbsGroup: s.FTLbsGroup === '2' ? '2' : '1',
      widthMm: s.FCLbsWidth,
      heightMm: s.FCLbsHeight,
      cols,
      rows,
      magTop: s.FCLbsMagTop ?? 3,
      magLeft: s.FCLbsMagLeft ?? 3,
      magBottom: s.FCLbsMagBottom ?? s.FCLbsMagTop ?? 3,
      magRight: s.FCLbsMagRight ?? s.FCLbsMagLeft ?? 3,
      gapH: s.FCLbsGapH ?? 2,
      gapV: s.FCLbsGapV ?? 2,
      qtyPageNor: s.FNLbsQtyPageNor || cols * rows,
      qtyPagePmt: s.FNLbsQtyPagePmt || cols * rows,
      staUse: s.FTLbsStaUse === '2' ? '2' : '1',
      paperSize: ps,
      paperWidthMm: s.FCLbsPaperWidth ?? dbPs?.FCPpsWidth ?? fallbackPs.widthMm,
      paperHeightMm: s.FCLbsPaperHeight ?? dbPs?.FCPpsHeight ?? fallbackPs.heightMm,
      offsetX: s.FCLbsOffsetX ?? 0,
      offsetY: s.FCLbsOffsetY ?? 0,
    });
  };

  const autoCalculateGrid = () => {
    if (!sizeForm) return;
    const usableW = Math.max(10, (sizeForm.paperWidthMm || 210) - sizeForm.magLeft - sizeForm.magRight);
    const usableH = Math.max(10, (sizeForm.paperHeightMm || 297) - sizeForm.magTop - sizeForm.magBottom);
    const cols = Math.max(
      1,
      Math.floor((usableW + sizeForm.gapH) / (Math.max(5, sizeForm.widthMm) + sizeForm.gapH))
    );
    const rows = Math.max(
      1,
      Math.floor((usableH + sizeForm.gapV) / (Math.max(5, sizeForm.heightMm) + sizeForm.gapV))
    );
    const total = cols * rows;
    setSizeForm({
      ...sizeForm,
      cols,
      rows,
      qtyPageNor: total,
      qtyPagePmt: total,
    });
  };

  const handleSaveSize = async () => {
    if (!sizeForm) return;
    if (!sizeForm.lbsCode.trim() || !sizeForm.lbsName.trim()) {
      toast.error('กรุณาระบุรหัสป้ายและชื่อขนาดป้าย');
      return;
    }
    setSavingSize(true);
    try {
      const res = await fetch('/api/label-sizes/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          lbsCode: sizeForm.lbsCode.trim(),
          lbsName: sizeForm.lbsName.trim(),
          lbsGroup: sizeForm.lbsGroup,
          widthMm: Number(sizeForm.widthMm) || 100,
          heightMm: Number(sizeForm.heightMm) || 38,
          cols: Math.max(1, Number(sizeForm.cols) || 1),
          rows: Math.max(1, Number(sizeForm.rows) || 1),
          magTop: Math.max(0, Number(sizeForm.magTop) || 0),
          magLeft: Math.max(0, Number(sizeForm.magLeft) || 0),
          magBottom: Math.max(0, Number(sizeForm.magBottom) || 0),
          magRight: Math.max(0, Number(sizeForm.magRight) || 0),
          gapH: Math.max(0, Number(sizeForm.gapH) || 0),
          gapV: Math.max(0, Number(sizeForm.gapV) || 0),
          qtyPageNor: Math.max(1, (Number(sizeForm.cols) || 1) * (Number(sizeForm.rows) || 1)),
          qtyPagePmt: Math.max(1, (Number(sizeForm.cols) || 1) * (Number(sizeForm.rows) || 1)),
          staUse: sizeForm.staUse,
          isNew: sizeForm.isNew,
          paperSize: sizeForm.paperSize,
          paperWidthMm: Math.max(10, Number(sizeForm.paperWidthMm) || 210),
          paperHeightMm: Math.max(10, Number(sizeForm.paperHeightMm) || 297),
          offsetX: Number(sizeForm.offsetX) || 0,
          offsetY: Number(sizeForm.offsetY) || 0,
        }),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'บันทึกขนาดป้ายไม่สำเร็จ');
        return;
      }
      await onRefreshAll();
      toast.success(`บันทึกขนาดป้าย ${sizeForm.lbsCode} เรียบร้อย`);
      setSizeForm(null);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setSavingSize(false);
    }
  };

  const handleToggleSizeStatus = async (s: ConfigLabelSizeRow) => {
    const nextSta = s.FTLbsStaUse === '1' ? '2' : '1';
    try {
      const res = await fetch('/api/label-sizes/toggle', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ lbsCode: s.FTLbsCode, staUse: nextSta }),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'เปลี่ยนสถานะไม่สำเร็จ');
        return;
      }
      await onRefreshAll();
      toast.success(
        `${s.FTLbsCode}: ${nextSta === '1' ? 'เปิดใช้งานแล้ว' : 'ปิดใช้งานชั่วคราวแล้ว'}`
      );
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const handleConfirmDeleteSize = async () => {
    if (!deleteSizeTarget) return;
    setDeletingSize(true);
    try {
      const res = await fetch('/api/label-sizes/delete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ lbsCode: deleteSizeTarget.FTLbsCode, allowBuiltIn: true }),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'ลบขนาดป้ายไม่สำเร็จ');
        return;
      }
      await onRefreshAll();
      toast.info(`ลบขนาดป้าย ${deleteSizeTarget.FTLbsCode} ออกจากระบบเรียบร้อย`);
      setDeleteSizeTarget(null);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setDeletingSize(false);
    }
  };

  // ── Handlers: Tab 2 (Driver Mapping & LAN Printers) ────────────────────────
  const handleSaveDriverRow = async () => {
    if (!driverEditRow || !driverEditorReady) return;
    if (driverEditRow.templateRoutes && !driverEditRow.templateRoutingSupported) {
      toast.error('บริการรุ่นเดิมยังบันทึกเครื่องพิมพ์รายแบบไม่ได้ กรุณาอัปเดตบริการก่อน');return;
    }
    setSavingDriver(true);
    try {
      const res = await fetch('/api/label-printers', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          lbsCode: driverEditRow.lbsCode,
          templateRoutes: driverEditRow.templateRoutes,
          serverCode: driverServerCode,
          normalPrinter: driverEditRow.normalPrinter,
          normalTray: driverEditRow.normalTray,
          normalProtocol: driverEditRow.normalProtocol || 'WINDOWS',
          promoPrinter: driverEditRow.promoPrinter,
          promoTray: driverEditRow.promoTray,
          promoProtocol: driverEditRow.promoProtocol || 'WINDOWS',
        }),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'บันทึกเครื่องพิมพ์ไม่สำเร็จ');
        return;
      }
      await onRefreshAll();
      toast.success(`บันทึกการตั้งค่าเครื่องพิมพ์ ${driverEditRow.lbsCode} สำเร็จ`);
      setDriverEditRow(null);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setSavingDriver(false);
    }
  };

  const handleBulkApplyPrinters = async () => {
    const activeRows = driverMappings.filter((m) =>
      allLabelSizes.some((s) => s.FTLbsCode === m.lbsCode && s.FTLbsStaUse === '1')
    );
    if (activeRows.length === 0) {
      toast.error('ไม่พบรูปแบบป้ายที่เปิดใช้งาน');
      return;
    }
    if (!bulkNormalPrinter && !bulkPromoPrinter) {
      toast.error('กรุณาเลือกอย่างน้อย 1 เครื่องพิมพ์ (ปกติ หรือ โปรโมชัน)');
      return;
    }
    setSavingBulkDriver(true);
    try {
      for (const m of activeRows) {
        const res = await fetch('/api/label-printers', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            lbsCode: m.lbsCode,
            serverCode: driverServerCode,
            normalPrinter: bulkNormalPrinter || m.normalPrinter || '',
            normalTray: bulkNormalPrinter ? bulkNormalTray : m.normalTray || '',
            normalProtocol: m.normalProtocol || 'WINDOWS',
            promoPrinter: bulkPromoPrinter || m.promoPrinter || '',
            promoTray: bulkPromoPrinter ? bulkPromoTray : m.promoTray || '',
            promoProtocol: m.promoProtocol || 'WINDOWS',
          }),
        });
        const data = await parseApiJson(res);
        if (!res.ok || !data?.ok) {
          throw new Error(data?.error || `บันทึก ${m.lbsCode} ไม่สำเร็จ`);
        }
      }
      await onRefreshAll();
      toast.success(`ตั้งค่าเครื่องพิมพ์ให้กับป้ายที่เปิดใช้งานทั้งหมด (${activeRows.length} แบบ) เรียบร้อย`);
      setBulkPrinterOpen(false);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setSavingBulkDriver(false);
    }
  };

  const handleTestDriver = async (kind: 'normal' | 'promo') => {
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
      const data = await parseApiJson(res);
      if (!res.ok) throw new Error(data.error || 'test print failed');
      toast.success(
        data.outputFile
          ? `ทดสอบพิมพ์สำเร็จ (บันทึกไฟล์ที่ ${data.outputFile})`
          : data.message || 'ส่งพิมพ์ทดสอบสำเร็จ'
      );
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setTestingDriver(null);
    }
  };

  const openAddLanPrinter = () => {
    setLanId('');
    setLanName('');
    setLanBrand('GODEX');
    setLanIp('');
    setLanPort(9100);
    setLanDpi(203);
    setLanSpeed(4);
    setLanDarkness(10);
    setLanFormOpen(true);
  };

  const openEditLanPrinter = (p: PrinterConfig) => {
    setLanId(p.id);
    setLanName(p.name);
    setLanBrand(p.brand);
    setLanIp(p.ip);
    setLanPort(p.port);
    setLanDpi(p.dpi);
    setLanSpeed(p.speed);
    setLanDarkness(p.darkness);
    setLanFormOpen(true);
  };

  const handleSaveLanPrinter = async () => {
    if (!lanName.trim() || !lanIp.trim()) {
      toast.error('กรุณาระบุชื่อเครื่องพิมพ์และ IP');
      return;
    }
    try {
      const res = await fetch('/api/lan-printers/save', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          id: lanId || undefined,
          name: lanName.trim(),
          brand: lanBrand,
          model: lanBrand,
          ip: lanIp.trim(),
          port: Number(lanPort) || 9100,
          dpi: Number(lanDpi) || 203,
          labelWidth: 100,
          labelHeight: 38,
          speed: Number(lanSpeed) || 4,
          darkness: Number(lanDarkness) || 10,
          enabled: true,
        }),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'บันทึกเครื่องพิมพ์ LAN ลงฐานข้อมูลไม่สำเร็จ');
        return;
      }
      await onRefreshAll();
      toast.success(`บันทึกเครื่องพิมพ์ ${lanName.trim()} ลงฐานข้อมูลเรียบร้อย`);
      setLanFormOpen(false);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const handleDeleteLanPrinter = async (id: string) => {
    try {
      const res = await fetch('/api/lan-printers/delete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code: id }),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'ลบเครื่องพิมพ์ไม่สำเร็จ');
        return;
      }
      await onRefreshAll();
      toast.info('ลบรายการเครื่องพิมพ์ LAN ออกจากฐานข้อมูลแล้ว');
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const handleDeletePreprintForm = async (pprCode: string, pprName: string) => {
    try {
      const res = await fetch('/api/preprints/delete', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code: pprCode }),
      });
      const data = await parseApiJson(res);
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'ลบฟอร์ม Pre-Print ไม่สำเร็จ');
        return;
      }
      await onRefreshAll();
      toast.info(`ลบฟอร์ม Pre-Print ${pprName} (${pprCode}) แล้ว`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    }
  };

  const handleTestLanConnection = async (p: PrinterConfig) => {
    setTestingLanId(p.id);
    try {
      const res = await fetch('/api/print/check', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ printerIp: p.ip, printerPort: p.port }),
      });
      const data = await parseApiJson(res);
      if (data?.online) {
        toast.success(`เชื่อมต่อ ${p.name} (${p.ip}:${p.port}) สำเร็จ (${data.latencyMs}ms)`);
      } else {
        toast.error(data?.error || `ไม่สามารถเชื่อมต่อ ${p.ip}:${p.port}`);
      }
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setTestingLanId(null);
    }
  };

  const jobsPanel = <AutoScheduleSettings mode="jobs" authUser={authUser} allLabelSizes={allLabelSizes} sysOptions={sysOptions} branches={branches} windowsPrinters={windowsPrinters} printers={printers} driverMappings={driverMappings} driverServerCode={driverServerCode} formatTrayLabel={formatTrayLabel} onRefreshAll={onRefreshAll}/>;
  const settingsTabs = workspace === 'paper'
    ? [['sizes', 'ขนาดและกระดาษ'], ['printers', 'เครื่องพิมพ์ตามแบบป้าย'], ['preprints', 'กระดาษพิมพ์พื้นหลัง']]
    : [
        ['devices', 'เครื่องพิมพ์'],
        ...(ENABLE_AUTO_PRINT_UI ? [['autoprint', 'จัดการพิมพ์อัตโนมัติ']] : []),
        ['connection', 'แหล่งข้อมูล'],
      ];

  if (workspace === 'jobs' && !ENABLE_AUTO_PRINT_HISTORY_TAB) {
    return <main className="max-w-[1600px] min-w-0 w-full mx-auto p-2 sm:p-4">{jobsPanel}</main>;
  }

  return (
    <main className="max-w-[1600px] min-w-0 w-full mx-auto p-2 sm:p-4 space-y-4">
      <Tabs value={subTab} onValueChange={(v) => setSubTab(v as typeof subTab)} className="space-y-4">
        <div className="space-y-3">
          {workspace==='jobs'?<TabsList className="bg-white border p-1 h-auto w-full justify-start overflow-x-auto"><TabsTrigger value="jobs" className="min-h-10">งานพิมพ์</TabsTrigger>{ENABLE_AUTO_PRINT_HISTORY_TAB&&<TabsTrigger value="autoprint_history" className="min-h-10">ประวัติก่อนย้ายระบบ</TabsTrigger>}</TabsList>:workspace==='profiles'?null:<>
            <label className="block sm:hidden text-sm space-y-2"><span className="font-medium">หมวดการตั้งค่า</span><select className="w-full rounded-lg border bg-white p-3" value={subTab} onChange={e=>setSubTab(e.target.value as typeof subTab)}>{settingsTabs.map(([value,label])=><option key={value} value={value}>{label}</option>)}</select></label>
            <div className="hidden sm:block"><TabsList className="bg-white border border-slate-200 p-1 h-auto min-h-11 w-full max-w-full flex-nowrap overflow-x-auto justify-start gap-1">
              {settingsTabs.map(([value,label])=><TabsTrigger key={value} value={value} className="flex-none min-h-10 text-sm px-3">{label}</TabsTrigger>)}
            </TabsList></div>
          </>}
        </div>
        <TabsContent value="connection">
          <MasterDatabaseSettings
            onSaved={() => window.location.reload()}
          />
        </TabsContent>
        <TabsContent value="jobs">{jobsPanel}</TabsContent>


        {/* ══════════════════ TAB 1: LABEL SIZES & MARGINS ══════════════════ */}
        <TabsContent value="sizes" className="space-y-4 mt-0">
          <Card className="border-slate-200 shadow-xs bg-white py-0 gap-0">
            <CardHeader className="bg-[#3d4a57] text-white py-2.5 px-4 flex flex-row flex-wrap items-center justify-between gap-2">
              <CardTitle className="text-sm font-semibold">
                ขนาดป้ายและการจัดวางบนกระดาษ
              </CardTitle>
              <div className="flex flex-wrap items-center justify-end gap-2">
              <Button
                size="sm"
                className="h-8 text-xs bg-[#007a87] hover:bg-[#00606b] text-white"
                onClick={openNewSizeForm}
              >
                <Plus className="h-3.5 w-3.5 mr-1" />
                เพิ่มขนาดป้ายใหม่
              </Button>
              </div>
            </CardHeader>
            <CardContent className="p-4 space-y-3">
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="flex flex-wrap items-center gap-2">
                  <div className="relative w-64">
                    <Search className="h-3.5 w-3.5 absolute left-2.5 top-2.5 text-slate-400" />
                    <Input
                      placeholder="ค้นหารหัสหรือชื่อขนาดป้าย..."
                      value={sizeSearch}
                      onChange={(e) => setSizeSearch(e.target.value)}
                      className="pl-8 h-8 text-xs"
                    />
                  </div>
                  <Select
                    value={sizeStatusFilter}
                    onValueChange={(v) => setSizeStatusFilter(v as typeof sizeStatusFilter)}
                  >
                    <SelectTrigger className="h-8 w-36 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">สถานะ: ทั้งหมด</SelectItem>
                      <SelectItem value="1">เปิดใช้งาน ({allLabelSizes.filter((s) => s.FTLbsStaUse === '1').length})</SelectItem>
                      <SelectItem value="2">ปิดใช้งาน ({allLabelSizes.filter((s) => s.FTLbsStaUse === '2').length})</SelectItem>
                    </SelectContent>
                  </Select>
                  <Select
                    value={sizeGroupFilter}
                    onValueChange={(v) => setSizeGroupFilter(v as typeof sizeGroupFilter)}
                  >
                    <SelectTrigger className="h-8 w-40 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">กลุ่ม: ทั้งหมด</SelectItem>
                      <SelectItem value="1">กลุ่ม 1: แผ่น A4</SelectItem>
                      <SelectItem value="2">กลุ่ม 2: สติ๊กเกอร์ม้วน</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="text-xs text-slate-500">
                  แสดง {filteredSizes.length} จากทั้งหมด {allLabelSizes.length} ขนาด
                </div>
              </div>

              <div className="border border-slate-200 rounded-md overflow-x-auto">
                <Table>
                  <TableHeader className="bg-[#34495e]">
                    <TableRow className="bg-[#34495e] hover:bg-[#34495e]">
                      <TableHead className="text-white text-xs text-center w-14">ลำดับ</TableHead>
                      <TableHead className="text-white text-xs">รหัสป้าย</TableHead>
                      <TableHead className="text-white text-xs">ชื่อขนาดป้าย</TableHead>
                      <TableHead className="text-white text-xs text-center">กลุ่ม</TableHead>
                      <TableHead className="text-white text-xs text-center">กว้าง×สูง (mm)</TableHead>
                      <TableHead className="text-white text-xs text-center">คอลัมน์×แถว</TableHead>
                      <TableHead className="text-white text-xs text-center">ขอบ บน/ล่าง · ซ้าย/ขวา</TableHead>
                      <TableHead className="text-white text-xs text-center">ห่าง H/V (mm)</TableHead>
                      <TableHead className="text-white text-xs text-center">สถานะ</TableHead>
                      <TableHead className="text-white text-xs text-center w-36">จัดการ</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {pagedSizes.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={10} className="text-center text-xs text-slate-400 py-8">
                          ไม่พบรูปแบบขนาดป้ายตามเงื่อนไขที่ค้นหา
                        </TableCell>
                      </TableRow>
                    ) : (
                    pagedSizes.map((s, idx) => {
                      const isCustom = s.FTLbsHhtCode === 'CUSTOM';
                      const active = s.FTLbsStaUse === '1';
                      const curSizePage = Math.min(sizePage, Math.max(1, Math.ceil(filteredSizes.length / sizePageSize)));
                      const rowNum = (curSizePage - 1) * sizePageSize + idx + 1;
                      const magT = s.FCLbsMagTop ?? 3;
                      const magB = s.FCLbsMagBottom ?? magT;
                      const magL = s.FCLbsMagLeft ?? 3;
                      const magR = s.FCLbsMagRight ?? magL;
                      return (
                        <TableRow key={s.FTLbsCode} className={!active ? 'bg-slate-50/70 opacity-75' : ''}>
                          <TableCell className="text-xs text-center font-mono text-slate-500">{rowNum}</TableCell>
                          <TableCell className="text-xs font-mono font-semibold">
                            <div className="flex items-center gap-1.5">
                              <span>{s.FTLbsCode}</span>
                              {isCustom && (
                                <Badge className="bg-sky-100 text-sky-800 hover:bg-sky-100 text-[10px]">
                                  สร้างเอง
                                </Badge>
                              )}
                            </div>
                          </TableCell>
                          <TableCell className="text-xs">{s.FTLbsName}</TableCell>
                          <TableCell className="text-xs text-center">
                            <Badge variant="outline" className="text-[10px]">
                              {s.FTLbsGroup === '2' && (!s.FTLbsPaperSize || s.FTLbsPaperSize === 'ROLL')
                                ? 'ม้วน Thermal'
                                : s.FTLbsGroup === '2'
                                  ? `ม้วน (${getPaperSizeDisplayName(s.FTLbsPaperSize, s.FCLbsPaperWidth ?? undefined, s.FCLbsPaperHeight ?? undefined)})`
                                  : `แผ่น ${getPaperSizeDisplayName(s.FTLbsPaperSize, s.FCLbsPaperWidth ?? undefined, s.FCLbsPaperHeight ?? undefined)}`}
                            </Badge>
                          </TableCell>
                          <TableCell className="text-xs text-center font-mono">
                            {s.FCLbsWidth} × {s.FCLbsHeight}
                          </TableCell>
                          <TableCell className="text-xs text-center font-mono">
                            {s.FNLbsCol || 1} × {s.FNLbsRow || 1} ({s.FNLbsQtyPageNor || (s.FNLbsCol || 1) * (s.FNLbsRow || 1)})
                          </TableCell>
                          <TableCell className="text-xs text-center font-mono">
                            {magT}/{magB} · {magL}/{magR}
                          </TableCell>
                          <TableCell className="text-xs text-center font-mono">
                            {s.FCLbsGapH ?? 2} / {s.FCLbsGapV ?? 2}
                          </TableCell>
                          <TableCell className="text-center">
                            <button
                              type="button"
                              onClick={() => handleToggleSizeStatus(s)}
                              className={`px-2 py-0.5 rounded-full text-[11px] font-medium border transition-colors ${
                                active
                                  ? 'bg-emerald-50 text-emerald-700 border-emerald-300 hover:bg-emerald-100'
                                  : 'bg-slate-100 text-slate-500 border-slate-300 hover:bg-slate-200'
                              }`}
                              title="คลิกเพื่อสลับเปิด/ปิดใช้งาน"
                            >
                              {active ? 'เปิดใช้งาน' : 'ปิดไว้'}
                            </button>
                          </TableCell>
                          <TableCell className="text-center">
                            <div className="flex items-center justify-center gap-1">
                              <Button
                                size="sm"
                                variant="outline"
                                className="h-7 text-xs px-2 border-[#007a87] text-[#007a87]"
                                onClick={() => openEditSizeForm(s)}
                              >
                                <Edit3 className="h-3 w-3 mr-1" />
                                แก้ไข
                              </Button>
                              <Button
                                size="icon"
                                variant="ghost"
                                className="h-7 w-7 text-red-500 hover:bg-red-50"
                                onClick={() => setDeleteSizeTarget(s)}
                                title="ลบ หรือ ปิดใช้งานขนาดป้ายนี้"
                              >
                                <Trash2 className="h-3.5 w-3.5" />
                              </Button>
                            </div>
                          </TableCell>
                        </TableRow>
                      );
                    })
                    )}
                  </TableBody>
                </Table>
              </div>
              <TablePagination
                total={filteredSizes.length}
                page={sizePage}
                pageSize={sizePageSize}
                onPageChange={setSizePage}
                onPageSizeChange={(s) => {
                  setSizePageSize(s);
                  setSizePage(1);
                }}
              />
            </CardContent>
          </Card>
        </TabsContent>

        {/* ══════════════════ TAB 2: PRE-PRINT VAULT ════════════════════════ */}
        <TabsContent value="preprints" className="space-y-4 mt-0">
          <Card className="border-slate-200 shadow-xs bg-white py-0 gap-0">
            <CardHeader className="bg-[#3d4a57] text-white py-2.5 px-4 flex flex-row flex-wrap items-center justify-between gap-2">
              <CardTitle className="text-sm font-semibold flex items-center gap-1.5">
                <FileText className="h-4 w-4 text-amber-300" />
                คลังฟอร์มกระดาษ Pre-Print ({preprints.length})
              </CardTitle>
              <div className="relative w-64">
                <Search className="h-3.5 w-3.5 absolute left-2.5 top-2.5 text-slate-400" />
                <Input
                  placeholder="ค้นหารหัส / ชื่อฟอร์ม / ชื่อไฟล์..."
                  value={preprintSearch}
                  onChange={(e) => setPreprintSearch(e.target.value)}
                  className="pl-8 h-8 text-xs bg-white text-slate-800"
                />
              </div>
            </CardHeader>
            <CardContent className="p-4 space-y-3">
              <div className="border border-slate-200 rounded-md overflow-x-auto">
                <Table>
                  <TableHeader className="bg-[#34495e]">
                    <TableRow className="bg-[#34495e] hover:bg-[#34495e]">
                      <TableHead className="text-white text-xs text-center w-14">ลำดับ</TableHead>
                      <TableHead className="text-white text-xs">รหัสฟอร์ม</TableHead>
                      <TableHead className="text-white text-xs">ชื่อฟอร์ม Pre-Print</TableHead>
                      <TableHead className="text-white text-xs">ไฟล์ต้นฉบับ</TableHead>
                      <TableHead className="text-white text-xs text-center">กระดาษ (mm)</TableHead>
                      <TableHead className="text-white text-xs text-center">ตาราง (ดวง/แผ่น)</TableHead>
                      <TableHead className="text-white text-xs text-center w-20">ลบ</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {pagedPreprints.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={7} className="text-center text-xs text-slate-400 py-8">
                          ยังไม่มีฟอร์ม Pre-Print ที่บันทึกไว้ — สามารถนำเข้าจากหน้า Design Studio &gt; โหมด Pre-Print แล้วบันทึกแบบ
                        </TableCell>
                      </TableRow>
                    ) : (
                      pagedPreprints.map((pp, idx) => {
                        const curPpPage = Math.min(preprintPage, Math.max(1, Math.ceil(filteredPreprints.length / preprintPageSize)));
                        const rowNum = (curPpPage - 1) * preprintPageSize + idx + 1;
                        return (
                        <TableRow key={pp.FTPprCode}>
                          <TableCell className="text-xs text-center font-mono text-slate-500">{rowNum}</TableCell>
                          <TableCell className="text-xs font-mono font-semibold">{pp.FTPprCode}</TableCell>
                          <TableCell className="text-xs">{pp.FTPprName}</TableCell>
                          <TableCell className="text-xs text-slate-600">{pp.FTPprFileName || '-'}</TableCell>
                          <TableCell className="text-xs text-center font-mono">
                            {pp.FTPpsCode} ({pp.FCPprWidth}×{pp.FCPprHeight})
                          </TableCell>
                          <TableCell className="text-xs text-center font-mono">
                            {pp.FNPprCol}×{pp.FNPprRow} ({pp.FNPprCol * pp.FNPprRow})
                          </TableCell>
                          <TableCell className="text-center">
                            <Button
                              size="icon"
                              variant="ghost"
                              className="h-7 w-7 text-red-500 hover:bg-red-50"
                              onClick={() => void handleDeletePreprintForm(pp.FTPprCode, pp.FTPprName)}
                              title="ลบฟอร์ม Pre-Print"
                            >
                              <Trash2 className="h-3.5 w-3.5" />
                            </Button>
                          </TableCell>
                        </TableRow>
                        );
                      })
                    )}
                  </TableBody>
                </Table>
              </div>
              <TablePagination
                total={filteredPreprints.length}
                page={preprintPage}
                pageSize={preprintPageSize}
                onPageChange={setPreprintPage}
                onPageSizeChange={(s) => {
                  setPreprintPageSize(s);
                  setPreprintPage(1);
                }}
              />
            </CardContent>
          </Card>
        </TabsContent>

        {/* ══════════════════ TAB 2: PRINTERS & TRAYS ═══════════════════════ */}
        <TabsContent value="printers" className="space-y-4 mt-0">
          <section className="rounded-xl border bg-white p-4 space-y-4">
            <div><h2 className="font-semibold">เครื่องพิมพ์ตามแบบป้าย — พิมพ์เอง</h2><p className="text-sm text-slate-500 mt-1">เลือกขนาดและแบบป้ายเพื่อกำหนดเครื่องพิมพ์และถาด การตั้งค่านี้ไม่เปลี่ยนงานพิมพ์อัตโนมัติ</p></div>
            <Input aria-label="ค้นหาแบบป้าย" placeholder="ค้นหารหัสหรือชื่อป้าย…" value={driverSearch} onChange={e=>setDriverSearch(e.target.value)} className="max-w-md"/>
            <div className="overflow-x-auto rounded-lg border border-slate-200">
              <table className="w-full min-w-[720px] table-fixed text-left text-sm">
                <thead className="bg-[#344b60] text-white"><tr>
                  <th scope="col" className="w-14 px-3 py-3 text-center font-medium">ลำดับ</th>
                  <th scope="col" className="px-3 py-3 font-medium">แบบป้าย</th>
                  <th scope="col" className="w-32 px-3 py-3 font-medium">ขนาด (มม.)</th>
                  <th scope="col" className="w-24 px-3 py-3 font-medium">กระดาษ</th>
                  <th scope="col" className="w-32 px-3 py-3 text-center font-medium">ตั้งค่า</th>
                  <th scope="col" className="w-24 border-l border-white/20 px-3 py-3 text-center font-medium">ลบแบบป้าย</th>
                </tr></thead>
                <tbody className="divide-y divide-slate-200">{pagedMappings.map(({format, mapping:m},index)=>{
                  const size=allLabelSizes.find(s=>s.FTLbsCode===m.lbsCode);
                  const page=Math.min(driverPage,Math.max(1,Math.ceil(filteredMappings.length/driverPageSize)));
                  return <tr key={format.value} className="even:bg-slate-50/60 hover:bg-teal-50/50">
                    <td className="px-3 py-3 text-center text-slate-500">{(page-1)*driverPageSize+index+1}</td>
                    <td className="px-3 py-3"><p className="font-medium break-words">{format.label}</p><p className="mt-1 text-xs text-slate-500 break-all">รหัสขนาด {m.lbsCode}</p></td>
                    <td className="px-3 py-3">{size?`${size.FCLbsWidth} × ${size.FCLbsHeight}`:'—'}</td>
                    <td className="px-3 py-3 break-words">{size?.FTLbsPaperSize||'—'}</td>
                    <td className="px-3 py-3 text-center"><Button size="sm" variant="outline" aria-label={`ตั้งค่า ${format.label}`} onClick={()=>{setEditFormat(format.value);setDriverEditRow({...m});}}>ตั้งค่า</Button></td>
                    <td className="border-l border-slate-200 px-3 py-3 text-center">
                      {!format.value.startsWith('legacy:') && <Button size="sm" variant="ghost" className="text-red-600" aria-label={`ลบแบบป้าย ${format.label}`} title="ลบแบบป้าย" onClick={()=>setRemoveFormat(format)}><Trash2 className="size-4"/></Button>}
                    </td>
                  </tr>;
                })}{!pagedMappings.length&&<tr><td colSpan={6} className="px-3 py-8 text-center text-slate-500">ไม่พบแบบป้ายที่ค้นหา</td></tr>}</tbody>
              </table>
            </div>
            <TablePagination total={filteredMappings.length} page={driverPage} pageSize={driverPageSize} onPageChange={setDriverPage} onPageSizeChange={s=>{setDriverPageSize(s);setDriverPage(1);}}/>

          </section>
        </TabsContent>

        {/* ══════════════════ TAB 4: PRINTER DEVICES & LAN ══════════════════ */}
        <TabsContent value="devices" className="space-y-4 mt-0">
          {/* Card: LAN / Network Printers (Thermal EZPL / TSPL / ESC-POS) */}
          <Card className="border-slate-200 shadow-xs bg-white py-0 gap-0">
            <CardHeader className="bg-[#3d4a57] text-white py-2.5 px-4 flex flex-row items-center justify-between">
              <div>
                <CardTitle className="text-sm font-semibold flex items-center gap-1.5">
                  <Network className="h-4 w-4" />
                  รายชื่อเครื่องพิมพ์ในเครื่องและเครื่องพิมพ์ LAN (TCP/IP)
                </CardTitle>
              </div>
              <div className="flex flex-wrap items-center justify-end gap-2">
              <Button
                size="sm"
                className="h-8 text-xs bg-[#007a87] hover:bg-[#00606b] text-white"
                onClick={openAddLanPrinter}
              >
                <Plus className="h-3.5 w-3.5 mr-1" />
                เพิ่มเครื่องพิมพ์ LAN
              </Button>
              </div>
            </CardHeader>
            <CardContent className="p-4 space-y-3">
              <div className="border border-slate-200 rounded-md overflow-x-auto">
                <Table>
                  <TableHeader className="bg-[#34495e]">
                    <TableRow className="bg-[#34495e] hover:bg-[#34495e]">
                      <TableHead className="text-white text-xs text-center w-14">ลำดับ</TableHead>
                      <TableHead className="text-white text-xs">ชื่อเครื่องพิมพ์</TableHead>
                      <TableHead className="text-white text-xs">ชนิด / โปรโตคอล</TableHead>
                      <TableHead className="text-white text-xs">IP / ช่องทาง</TableHead>
                      <TableHead className="text-white text-xs text-center">DPI / Speed</TableHead>
                      <TableHead className="text-white text-xs text-center w-44">จัดการ</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {pagedPrinters.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={6} className="text-center text-xs text-slate-400 py-8">
                          ยังไม่มีรายชื่อเครื่องพิมพ์ในระบบ
                        </TableCell>
                      </TableRow>
                    ) : (
                    pagedPrinters.map((p, idx) => {
                      const isWin = p.ip === 'Windows';
                      const curPrnPage = Math.min(printerPage, Math.max(1, Math.ceil(printers.length / printerPageSize)));
                      const rowNum = (curPrnPage - 1) * printerPageSize + idx + 1;
                      return (
                        <TableRow key={p.id}>
                          <TableCell className="text-xs text-center font-mono text-slate-500">{rowNum}</TableCell>
                          <TableCell className="text-xs font-medium">{p.name}</TableCell>
                          <TableCell className="text-xs">
                            <div className="flex items-center gap-1">
                              <Badge
                                className={
                                  isWin
                                    ? 'text-[10px] bg-slate-100 text-slate-700 hover:bg-slate-100 border border-slate-200'
                                    : 'text-[10px] bg-emerald-100 text-emerald-800 hover:bg-emerald-100 border border-emerald-200'
                                }
                              >
                                {isWin ? 'Windows Driver' : 'LAN TCP/IP'}
                              </Badge>
                              {!isWin && (
                                <Badge variant="outline" className="text-[10px]">
                                  {p.brand}
                                </Badge>
                              )}
                            </div>
                          </TableCell>
                          <TableCell className="text-xs font-mono">
                            {isWin ? 'Windows Spooler' : `${p.ip}:${p.port}`}
                          </TableCell>
                          <TableCell className="text-xs text-center font-mono">
                            {isWin ? <span className="text-slate-400">ตามไดร์เวอร์ Windows</span> : `${p.dpi} DPI · Speed ${p.speed}`}
                          </TableCell>
                          <TableCell className="text-center">
                            {isWin ? (
                              <span className="text-[11px] text-slate-400">ดึงจาก Windows อัตโนมัติ</span>
                            ) : (
                              <div className="flex items-center justify-center gap-1">
                                <Button
                                  size="sm"
                                  variant="outline"
                                  className="h-7 text-xs px-2"
                                  disabled={testingLanId === p.id}
                                  onClick={() => handleTestLanConnection(p)}
                                >
                                  {testingLanId === p.id ? (
                                    <Loader2 className="h-3 w-3 animate-spin" />
                                  ) : (
                                    <Wifi className="h-3 w-3 mr-1" />
                                  )}
                                  ทดสอบ
                                </Button>
                                <Button
                                  size="sm"
                                  variant="outline"
                                  className="h-7 text-xs px-2"
                                  onClick={() => openEditLanPrinter(p)}
                                >
                                  แก้ไข
                                </Button>
                                <Button
                                  size="icon"
                                  variant="ghost"
                                  className="h-7 w-7 text-red-500"
                                  onClick={() => handleDeleteLanPrinter(p.id)}
                                >
                                  <Trash2 className="h-3.5 w-3.5" />
                                </Button>
                              </div>
                            )}
                          </TableCell>
                        </TableRow>
                      );
                    })
                    )}
                  </TableBody>
                </Table>
              </div>
              <TablePagination
                total={printers.length}
                page={printerPage}
                pageSize={printerPageSize}
                onPageChange={setPrinterPage}
                onPageSizeChange={(s) => {
                  setPrinterPageSize(s);
                  setPrinterPage(1);
                }}
              />
            </CardContent>
          </Card>

        </TabsContent>

        {/* ══════════════════ TAB 3: SYSTEM OPTIONS ═════════════════════════ */}


        {ENABLE_AUTO_PRINT_UI && (
          <TabsContent value="autoprint" className="space-y-4 mt-0">
            <AutoScheduleSettings
              onOpenDesign={onOpenDesign}
              onOpenConnection={()=>setSubTab('connection')}
              authUser={authUser}
              allLabelSizes={allLabelSizes}
              sysOptions={sysOptions}
              branches={branches}
              windowsPrinters={windowsPrinters}
              printers={printers}
              driverMappings={driverMappings}
              driverServerCode={driverServerCode}
              formatTrayLabel={formatTrayLabel}
              onRefreshAll={onRefreshAll}
            />
          </TabsContent>
        )}

        {ENABLE_AUTO_PRINT_HISTORY_TAB && (
          <TabsContent value="autoprint_history" className="space-y-4 mt-0">
            <AutoScheduleHistory
              onLoadManualRows={onLoadManualRows}
              onOpenAutoPrintTab={ENABLE_AUTO_PRINT_UI ? () => setSubTab('autoprint') : undefined}
              readOnly={true}
            />
          </TabsContent>
        )}
      </Tabs>

      {/* ════════════ MODAL: ADD / EDIT LABEL SIZE ════════════════════════════ */}
      <Dialog open={Boolean(removeFormat)} onOpenChange={open=>{if (!open && !removingFormat) setRemoveFormat(null);}}>
        <DialogContent>
          <DialogHeader><DialogTitle>ลบแบบป้ายนี้?</DialogTitle>
            <DialogDescription>{removeFormat?.label} จะถูกนำออกจากรายการแบบป้าย ขนาดกระดาษและฟอร์ม Pre-Print ยังใช้กับแบบอื่นได้</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" disabled={removingFormat} onClick={()=>setRemoveFormat(null)}>ยกเลิก</Button>
            <Button variant="destructive" disabled={removingFormat} onClick={deleteFormat}>ยืนยันลบแบบป้าย</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Dialog open={Boolean(sizeForm)} onOpenChange={(o) => { if (!o) setSizeForm(null); }}>
        <DialogContent className="sm:max-w-xl">
          <DialogHeader>
            <DialogTitle className="text-base">
              {sizeForm?.isNew ? 'เพิ่มขนาดรูปแบบป้ายใหม่' : `แก้ไขขนาดป้าย: ${sizeForm?.lbsCode}`}
            </DialogTitle>
            <DialogDescription className="text-xs">
              กำหนดขนาด กว้าง×สูง, คอลัมน์×แถว, ระยะขอบกระดาษ (Margin) และช่องว่างระหว่างดวง (Gap) ในหน่วยมิลลิเมตร
            </DialogDescription>
          </DialogHeader>

          {sizeForm && (
            <div className="space-y-3 py-1">
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
                <div className="space-y-1">
                  <Label className="text-xs font-medium">
                    <span className="text-red-500">*</span> รหัสป้าย
                  </Label>
                  <Input
                    value={sizeForm.lbsCode}
                    readOnly={!sizeForm.isNew}
                    placeholder="เช่น ADA-70x40"
                    onChange={(e) =>
                      setSizeForm({ ...sizeForm, lbsCode: e.target.value.toUpperCase() })
                    }
                    className={`h-8 text-xs font-mono ${!sizeForm.isNew ? 'bg-slate-50' : ''}`}
                  />
                </div>
                <div className="space-y-1 sm:col-span-2">
                  <Label className="text-xs font-medium">
                    <span className="text-red-500">*</span> ชื่อขนาดป้าย
                  </Label>
                  <Input
                    value={sizeForm.lbsName}
                    placeholder="เช่น ป้ายราคา 7x4 ซม."
                    onChange={(e) => setSizeForm({ ...sizeForm, lbsName: e.target.value })}
                    className="h-8 text-xs"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                <div className="space-y-1">
                  <Label className="text-xs font-medium">กลุ่มป้าย</Label>
                  <Select
                    value={sizeForm.lbsGroup}
                    onValueChange={(v) => setSizeForm({ ...sizeForm, lbsGroup: v as '1' | '2' })}
                  >
                    <SelectTrigger className="h-8 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1">1 = แผ่น A4</SelectItem>
                      <SelectItem value="2">2 = ม้วน Thermal</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1">
                  <Label className="text-xs font-medium">กว้าง (mm)</Label>
                  <Input
                    type="number"
                    step={0.5}
                    min={5}
                    value={sizeForm.widthMm}
                    onChange={(e) =>
                      setSizeForm({ ...sizeForm, widthMm: parseFloat(e.target.value) || 0 })
                    }
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-xs font-medium">สูง (mm)</Label>
                  <Input
                    type="number"
                    step={0.5}
                    min={5}
                    value={sizeForm.heightMm}
                    onChange={(e) =>
                      setSizeForm({ ...sizeForm, heightMm: parseFloat(e.target.value) || 0 })
                    }
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-xs font-medium">สถานะ</Label>
                  <Select
                    value={sizeForm.staUse}
                    onValueChange={(v) => setSizeForm({ ...sizeForm, staUse: v as '1' | '2' })}
                  >
                    <SelectTrigger className="h-8 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1">เปิดใช้งาน</SelectItem>
                      <SelectItem value="2">ปิดใช้งาน</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="border border-slate-200 rounded-md overflow-hidden">
                <button
                  type="button"
                  onClick={() => setSizeFormAdvancedOpen((o) => !o)}
                  className="w-full flex items-center justify-between px-3 py-2 bg-slate-50 hover:bg-slate-100 text-xs font-medium text-slate-700 transition-colors"
                >
                  <span>ตั้งค่าขนาดแผ่นกระดาษ ระยะขอบ (Margin) ช่องว่าง & Offset</span>
                  {sizeFormAdvancedOpen ? (
                    <ChevronUp className="h-3.5 w-3.5 text-slate-500" />
                  ) : (
                    <ChevronDown className="h-3.5 w-3.5 text-slate-500" />
                  )}
                </button>

                {sizeFormAdvancedOpen && (
                  <div className="p-3 space-y-3 border-t border-slate-200 bg-slate-50/40">
                    <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 bg-white p-3 rounded-md border border-slate-200">
                      <div className="space-y-1 sm:col-span-2">
                        <Label className="text-[11px] font-medium">ขนาดแผ่นกระดาษ (Paper Size)</Label>
                        <Select
                          value={sizeForm.paperSize}
                          onValueChange={(v) => {
                            const code = v as PaperSizeCode;
                            const dbP = paperSizes.find((x) => x.FTPpsCode === code);
                            const fallbackP = PAPER_SIZE_PRESETS[code] || PAPER_SIZE_PRESETS.A4;
                            const isCust = dbP ? dbP.FTPpsStaCustom === '1' : code === 'CUSTOM';
                            setSizeForm({
                              ...sizeForm,
                              paperSize: code,
                              paperWidthMm: isCust ? sizeForm.paperWidthMm : (dbP?.FCPpsWidth ?? fallbackP.widthMm),
                              paperHeightMm: isCust ? sizeForm.paperHeightMm : (dbP?.FCPpsHeight ?? fallbackP.heightMm),
                            });
                          }}
                        >
                          <SelectTrigger className="h-8 text-xs bg-white">
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            {mergePaperSizeOptions(paperSizes).map((p) => (
                              <SelectItem key={p.FTPpsCode} value={p.FTPpsCode}>
                                {p.FTPpsName}
                              </SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                      <div className="space-y-1">
                        <Label className="text-[11px] font-medium">กว้างแผ่น (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={10}
                          disabled={sizeForm.paperSize !== 'CUSTOM' && sizeForm.paperSize !== 'ROLL'}
                          value={sizeForm.paperWidthMm}
                          onChange={(e) =>
                            setSizeForm({ ...sizeForm, paperWidthMm: parseFloat(e.target.value) || 210 })
                          }
                          className="h-8 text-xs font-mono bg-white"
                        />
                      </div>
                      <div className="space-y-1">
                        <Label className="text-[11px] font-medium">สูงแผ่น / แถว (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={10}
                          disabled={sizeForm.paperSize !== 'CUSTOM' && sizeForm.paperSize !== 'ROLL'}
                          value={sizeForm.paperHeightMm}
                          onChange={(e) =>
                            setSizeForm({ ...sizeForm, paperHeightMm: parseFloat(e.target.value) || 297 })
                          }
                          className="h-8 text-xs font-mono bg-white"
                        />
                      </div>
                      <div className="space-y-1">
                        <Label className="text-[11px] font-medium">ชดเชยพิมพ์ X (±mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={-30}
                          max={30}
                          value={sizeForm.offsetX}
                          onChange={(e) =>
                            setSizeForm({ ...sizeForm, offsetX: parseFloat(e.target.value) || 0 })
                          }
                          className="h-8 text-xs font-mono bg-white"
                          title="เลื่อนตำแหน่งพิมพ์ทั้งแผ่น ซ้าย (-) / ขวา (+)"
                        />
                      </div>
                      <div className="space-y-1">
                        <Label className="text-[11px] font-medium">ชดเชยพิมพ์ Y (±mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={-30}
                          max={30}
                          value={sizeForm.offsetY}
                          onChange={(e) =>
                            setSizeForm({ ...sizeForm, offsetY: parseFloat(e.target.value) || 0 })
                          }
                          className="h-8 text-xs font-mono bg-white"
                          title="เลื่อนตำแหน่งพิมพ์ทั้งแผ่น ขึ้น (-) / ลง (+)"
                        />
                      </div>
                      <div className="sm:col-span-2 flex items-end text-[11px] text-slate-500 pb-1">
                        <span>
                          <strong className="text-slate-700">X+</strong> เลื่อนขวา · <strong className="text-slate-700">Y+</strong> เลื่อนลง (จูนรอยปรุ / Pre-Print)
                        </span>
                      </div>
                    </div>

                    <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 bg-white p-3 rounded-md border border-slate-200">
                      <div className="space-y-1">
                        <Label className="text-[11px] font-medium">ขอบบน (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={0}
                          value={sizeForm.magTop}
                          onChange={(e) =>
                            setSizeForm({ ...sizeForm, magTop: parseFloat(e.target.value) || 0 })
                          }
                          className="h-8 text-xs font-mono bg-white"
                        />
                      </div>
                      <div className="space-y-1">
                        <Label className="text-[11px] font-medium">ขอบล่าง (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={0}
                          value={sizeForm.magBottom}
                          onChange={(e) =>
                            setSizeForm({ ...sizeForm, magBottom: parseFloat(e.target.value) || 0 })
                          }
                          className="h-8 text-xs font-mono bg-white"
                        />
                      </div>
                      <div className="space-y-1">
                        <Label className="text-[11px] font-medium">ขอบซ้าย (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={0}
                          value={sizeForm.magLeft}
                          onChange={(e) =>
                            setSizeForm({ ...sizeForm, magLeft: parseFloat(e.target.value) || 0 })
                          }
                          className="h-8 text-xs font-mono bg-white"
                        />
                      </div>
                      <div className="space-y-1">
                        <Label className="text-[11px] font-medium">ขอบขวา (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={0}
                          value={sizeForm.magRight}
                          onChange={(e) =>
                            setSizeForm({ ...sizeForm, magRight: parseFloat(e.target.value) || 0 })
                          }
                          className="h-8 text-xs font-mono bg-white"
                        />
                      </div>
                      <div className="space-y-1 sm:col-span-2">
                        <Label className="text-[11px] font-medium">ช่องว่างแนวนอน H (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={0}
                          value={sizeForm.gapH}
                          onChange={(e) =>
                            setSizeForm({ ...sizeForm, gapH: parseFloat(e.target.value) || 0 })
                          }
                          className="h-8 text-xs font-mono bg-white"
                        />
                      </div>
                      <div className="space-y-1 sm:col-span-2">
                        <Label className="text-[11px] font-medium">ช่องว่างแนวตั้ง V (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={0}
                          value={sizeForm.gapV}
                          onChange={(e) =>
                            setSizeForm({ ...sizeForm, gapV: parseFloat(e.target.value) || 0 })
                          }
                          className="h-8 text-xs font-mono bg-white"
                        />
                      </div>
                    </div>
                  </div>
                )}
              </div>

              <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 items-end">
                <div className="space-y-1">
                  <Label className="text-[11px] font-medium">คอลัมน์/แผ่น</Label>
                  <Input
                    type="number"
                    min={1}
                    value={sizeForm.cols}
                    onChange={(e) => {
                      const c = Math.max(1, parseInt(e.target.value) || 1);
                      setSizeForm({
                        ...sizeForm,
                        cols: c,
                        qtyPageNor: c * sizeForm.rows,
                        qtyPagePmt: c * sizeForm.rows,
                      });
                    }}
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[11px] font-medium">แถว/แผ่น</Label>
                  <Input
                    type="number"
                    min={1}
                    value={sizeForm.rows}
                    onChange={(e) => {
                      const r = Math.max(1, parseInt(e.target.value) || 1);
                      setSizeForm({
                        ...sizeForm,
                        rows: r,
                        qtyPageNor: sizeForm.cols * r,
                        qtyPagePmt: sizeForm.cols * r,
                      });
                    }}
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[11px] font-medium text-slate-500">จำนวนรวม/แผ่น</Label>
                  <div className="h-8 px-2.5 flex items-center rounded-md border border-slate-200 bg-slate-50 text-xs font-mono font-semibold text-[#007a87]">
                    {sizeForm.cols * sizeForm.rows} ดวง
                  </div>
                </div>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  className="h-8 text-xs border-[#007a87] text-[#007a87]"
                  onClick={autoCalculateGrid}
                  title={`คำนวณจำนวนคอลัมน์×แถวที่พอดีบน ${sizeForm.paperWidthMm}×${sizeForm.paperHeightMm}mm`}
                >
                  <Calculator className="h-3.5 w-3.5 mr-1" />
                  คำนวณกริด
                </Button>
              </div>
              {(() => {
                const pw = sizeForm.paperWidthMm || 210;
                const ph = sizeForm.paperHeightMm || 297;
                const usedW =
                  sizeForm.magLeft +
                  sizeForm.magRight +
                  sizeForm.cols * sizeForm.widthMm +
                  Math.max(0, sizeForm.cols - 1) * sizeForm.gapH;
                const usedH =
                  sizeForm.magTop +
                  sizeForm.magBottom +
                  sizeForm.rows * sizeForm.heightMm +
                  Math.max(0, sizeForm.rows - 1) * sizeForm.gapV;
                const overflow = sizeForm.lbsGroup === '1' && (usedW > pw + 0.5 || usedH > ph + 0.5);
                return (
                  <div
                    className={`text-[11px] rounded px-2.5 py-1.5 border ${
                      overflow
                        ? 'bg-amber-50 border-amber-300 text-amber-800'
                        : 'bg-slate-50 border-slate-200 text-slate-600'
                    }`}
                  >
                    พื้นที่รวมบน {getPaperSizeDisplayName(sizeForm.paperSize, sizeForm.paperWidthMm, sizeForm.paperHeightMm)}:{' '}
                    <strong>{usedW.toFixed(1)} / {pw} mm</strong> (กว้าง) ×{' '}
                    <strong>{usedH.toFixed(1)} / {ph} mm</strong> (สูง)
                    {overflow && ' — เกินขอบกระดาษ แนะนำปรับ Margin/Gap หรือกด "คำนวณกริด"'}
                  </div>
                );
              })()}
            </div>
          )}

          <DialogFooter>
            <Button variant="outline" onClick={() => setSizeForm(null)}>
              ยกเลิก
            </Button>
            <Button
              className="bg-[#007a87] hover:bg-[#00606b] text-white"
              disabled={savingSize}
              onClick={handleSaveSize}
            >
              {savingSize && <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />}
              บันทึกขนาดป้าย
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ════════════ MODAL: CONFIRM DELETE OR HIDE LABEL SIZE ════════════════ */}
      <Dialog open={Boolean(deleteSizeTarget)} onOpenChange={(o) => { if (!o) setDeleteSizeTarget(null); }}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="text-base text-red-700 flex items-center gap-2">
              <Trash2 className="h-4 w-4" />
              จัดการลบ / ปิดใช้งานขนาดป้าย
            </DialogTitle>
            <DialogDescription className="text-xs leading-relaxed pt-1">
              ต้องการดำเนินการกับรูปแบบป้าย{' '}
              <strong className="text-slate-900">
                {deleteSizeTarget?.FTLbsCode} — {deleteSizeTarget?.FTLbsName}
              </strong>{' '}
              อย่างไร?
            </DialogDescription>
          </DialogHeader>
          <div className="rounded-md bg-amber-50 border border-amber-200 p-3 text-[11px] text-amber-900 space-y-1">
            <div>• <strong>ปิดใช้งานชั่วคราว:</strong> ซ่อนออกจากหน้าพิมพ์และตัวกรองหลัก (เปิดกลับมาใช้ใหม่ได้ตลอด)</div>
            <div>• <strong>ลบถาวรออกจากฐานข้อมูล:</strong> ลบขนาดป้าย แบบที่ออกแบบไว้ และการผูกเครื่องพิมพ์ของรหัสนี้ออกทั้งหมด (ย้อนกลับไม่ได้)</div>
          </div>
          <DialogFooter className="flex flex-wrap gap-2 sm:justify-between">
            <Button variant="outline" onClick={() => setDeleteSizeTarget(null)} disabled={deletingSize}>
              ยกเลิก
            </Button>
            <div className="flex flex-wrap gap-2">
              {deleteSizeTarget?.FTLbsStaUse === '1' && (
                <Button
                  type="button"
                  variant="outline"
                  className="border-amber-300 text-amber-800 hover:bg-amber-50"
                  disabled={deletingSize}
                  onClick={async () => {
                    const t = deleteSizeTarget;
                    setDeleteSizeTarget(null);
                    if (t) await handleToggleSizeStatus(t);
                  }}
                >
                  ปิดใช้งานชั่วคราว
                </Button>
              )}
              <Button
                type="button"
                variant="destructive"
                disabled={deletingSize}
                onClick={handleConfirmDeleteSize}
              >
                {deletingSize && <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />}
                ลบถาวรออกจากระบบ
              </Button>
            </div>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ════════════ MODAL: EDIT DRIVER & TRAY MAPPING ══════════════════════ */}
      <Dialog open={Boolean(driverEditRow)} onOpenChange={(o) => { if (!o) setDriverEditRow(null); }}>
        <DialogContent className="sm:max-w-3xl max-h-[90dvh] flex flex-col overflow-hidden p-0 gap-0">
          <DialogHeader className="bg-[#3d4a57] text-white px-5 py-4 shrink-0">
            <div className="flex items-center gap-2 pr-6">
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="h-7 px-2 text-xs text-slate-200 hover:text-white hover:bg-white/10 -ml-2"
                onClick={() => setDriverEditRow(null)}
              >
                <ArrowLeft className="h-3.5 w-3.5 mr-1" />
                ปิด
              </Button>
              <DialogTitle className="text-base font-semibold">
                เครื่องพิมพ์และกระดาษของแบบป้าย
              </DialogTitle>
            </div>
            <DialogDescription className="text-slate-300 text-xs">
              รหัสฉลาก {driverEditRow?.lbsCode} · {driverEditRow?.lbsName}
            </DialogDescription>
          </DialogHeader>

          <div className="min-h-0 overflow-y-auto px-5 py-4">{driverEditRow && <LabelPrinterRoutesEditor initialFormat={editFormat} key={driverEditRow.lbsCode} onReadyChange={setDriverEditorReady} value={driverEditRow} onChange={setDriverEditRow} windowsPrinters={windowsPrinters} printers={printers} formatTrayLabel={formatTrayLabel} paper={allLabelSizes.find(s=>s.FTLbsCode===driverEditRow.lbsCode)} disabled={savingDriver}/>}</div>

          <DialogFooter className="shrink-0 border-t px-5 py-3 bg-white">
            <Button variant="outline" onClick={() => setDriverEditRow(null)}>
              ยกเลิก
            </Button>
            <Button
              className="bg-[#007a87] hover:bg-[#00606b] text-white"
              disabled={savingDriver||!driverEditorReady}
              onClick={handleSaveDriverRow}
            >
              {savingDriver && <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />}
              <Save className="h-3.5 w-3.5 mr-1" />
              บันทึก
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* ════════════ MODAL: ADD / EDIT LAN PRINTER ═══════════════════════════ */}
      <Dialog open={lanFormOpen} onOpenChange={setLanFormOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="text-base">
              {lanId ? 'แก้ไขข้อมูลเครื่องพิมพ์' : 'เพิ่มเครื่องพิมพ์ LAN / TCP-IP'}
            </DialogTitle>
          </DialogHeader>
          <div className="space-y-3 py-1">
            <div className="space-y-1">
              <Label className="text-xs font-medium">ชื่อเครื่องพิมพ์</Label>
              <Input
                value={lanName}
                onChange={(e) => setLanName(e.target.value)}
                placeholder="เช่น Godex EZ1100 จุดที่ 1"
                className="h-8 text-xs"
              />
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label className="text-xs font-medium">โปรโตคอล / ยี่ห้อ</Label>
                <Select value={lanBrand} onValueChange={(v) => setLanBrand(v as PrinterBrand)}>
                  <SelectTrigger className="h-8 text-xs">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="GODEX">GODEX (EZPL)</SelectItem>
                    <SelectItem value="TSC">TSC (TSPL)</SelectItem>
                    <SelectItem value="ZEBRA">ZEBRA (ZPL)</SelectItem>
                    <SelectItem value="HONEYWELL">HONEYWELL (Fingerprint)</SelectItem>
                    <SelectItem value="ARGOX">ARGOX (PPLZ)</SelectItem>
                    <SelectItem value="BIXOLON">BIXOLON (BXL)</SelectItem>
                    <SelectItem value="WINDOWS">WINDOWS (A4 / Spooler)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-1">
                <Label className="text-xs font-medium">DPI</Label>
                <Input
                  type="number"
                  value={lanDpi}
                  onChange={(e) => setLanDpi(parseInt(e.target.value) || 203)}
                  className="h-8 text-xs font-mono"
                />
              </div>
            </div>
            <div className="grid grid-cols-3 gap-2">
              <div className="col-span-2 space-y-1">
                <Label className="text-xs font-medium">IP Address</Label>
                <Input
                  value={lanIp}
                  onChange={(e) => setLanIp(e.target.value)}
                  placeholder="เช่น 192.168.1.100"
                  className="h-8 text-xs font-mono"
                />
              </div>
              <div className="space-y-1">
                <Label className="text-xs font-medium">Port</Label>
                <Input
                  type="number"
                  value={lanPort}
                  onChange={(e) => setLanPort(parseInt(e.target.value) || 9100)}
                  className="h-8 text-xs font-mono"
                />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-1">
                <Label className="text-xs font-medium">Speed (1-6)</Label>
                <Input
                  type="number"
                  min={1}
                  max={10}
                  value={lanSpeed}
                  onChange={(e) => setLanSpeed(parseInt(e.target.value) || 4)}
                  className="h-8 text-xs font-mono"
                />
              </div>
              <div className="space-y-1">
                <Label className="text-xs font-medium">ความเข้ม Darkness</Label>
                <Input
                  type="number"
                  min={1}
                  max={20}
                  value={lanDarkness}
                  onChange={(e) => setLanDarkness(parseInt(e.target.value) || 10)}
                  className="h-8 text-xs font-mono"
                />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setLanFormOpen(false)}>
              ยกเลิก
            </Button>
            <Button className="bg-[#007a87] hover:bg-[#00606b] text-white" onClick={handleSaveLanPrinter}>
              บันทึกเครื่องพิมพ์
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </main>
  );
}
