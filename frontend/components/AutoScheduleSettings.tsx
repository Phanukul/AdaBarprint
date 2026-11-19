import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';
import {
  AlertTriangle,
  Calendar,
  CheckCircle2,
  ChevronDown,
  ChevronUp,
  Clock,
  FlaskConical,
  KeyRound,
  Loader2,
  Lock,
  Play,
  Printer,
  RefreshCw,
  Save,
  Settings2,
  Unlock,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { SearchableSelect } from '@/components/ui/searchable-select';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import type {
  ConfigLabelSizeRow,
  SettingsDriverMapping,
  SettingsWindowsPrinter,
  SysOptionRow,
} from '@/components/SettingsPanel';
import type { PrinterConfig } from '@/types/printer';

interface AutoScheduleSettingsProps {
  allLabelSizes: ConfigLabelSizeRow[];
  sysOptions: SysOptionRow[];
  branches: Array<{ FTBchCode: string; FTBchName: string | null }>;
  windowsPrinters: SettingsWindowsPrinter[];
  printers: PrinterConfig[];
  driverMappings: SettingsDriverMapping[];
  driverServerCode: string;
  formatTrayLabel: (tray: string) => string;
  onRefreshAll: () => Promise<void>;
}

const DAY_LABELS = ['จ.', 'อ.', 'พ.', 'พฤ.', 'ศ.', 'ส.', 'อา.'];
const RULE_OPTIONS = [
  { key: 'PRICE_CHG', label: '1. สินค้าปรับราคาขายวันนี้' },
  { key: 'PROMO_START', label: '2. สินค้าเริ่มโปรโมชันวันนี้' },
  { key: 'PROMO_END_REVERT', label: '3. โปรโมชันหมดเมื่อวาน (คืนป้ายปกติ)' },
  { key: 'NEW_PDT', label: '4. สินค้าเพิ่มใหม่เข้าร้าน' },
];

function triggerSourceLabel(src: string): string {
  switch (src) {
    case 'PRICE_CHG':
      return 'ปรับราคา';
    case 'PROMO_START':
      return 'เริ่มโปรโมชัน';
    case 'PROMO_END_REVERT':
      return 'หมดโปรฯ คืนราคาปกติ';
    case 'NEW_PDT':
      return 'สินค้าใหม่';
    default:
      return src;
  }
}

function getLocalIsoDate(offsetDays = 0): string {
  const d = new Date();
  if (offsetDays !== 0) d.setDate(d.getDate() + offsetDays);
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

function getLocalTimePlusMin(plusMin = 0): string {
  const d = new Date(Date.now() + plusMin * 60000);
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
}

function normalizeTime24(raw: string, fallback = '04:00'): string {
  const digits = raw.replace(/\D/g, '');
  let hhNum = -1;
  let mmNum = -1;
  if (raw.includes(':')) {
    const [hStr, mStr] = raw.split(':');
    hhNum = parseInt(hStr || '0', 10);
    mmNum = parseInt(mStr || '0', 10);
  } else if (digits.length === 4) {
    hhNum = parseInt(digits.slice(0, 2), 10);
    mmNum = parseInt(digits.slice(2, 4), 10);
  } else if (digits.length === 3) {
    hhNum = parseInt(digits.slice(0, 1), 10);
    mmNum = parseInt(digits.slice(1, 3), 10);
  } else if (digits.length >= 1 && digits.length <= 2) {
    hhNum = parseInt(digits, 10);
    mmNum = 0;
  }
  if (Number.isNaN(hhNum) || Number.isNaN(mmNum) || hhNum < 0 || hhNum > 23 || mmNum < 0 || mmNum > 59) {
    return fallback;
  }
  return `${String(hhNum).padStart(2, '0')}:${String(mmNum).padStart(2, '0')}`;
}

export function AutoScheduleSettings({
  allLabelSizes,
  sysOptions,
  branches,
  windowsPrinters,
  printers,
  driverMappings,
  driverServerCode,
  formatTrayLabel,
  onRefreshAll,
}: AutoScheduleSettingsProps) {
  const getOpt = useCallback(
    (code: string, fallback = '') =>
      sysOptions.find((o) => o.FTSopCode === code)?.FTSopValue ?? fallback,
    [sysOptions]
  );

  // Store User Main Settings
  const [staUse, setStaUse] = useState<'1' | '2'>('2');
  const [runTime, setRunTime] = useState('04:00');
  const [daysMask, setDaysMask] = useState('1111111');
  const [selectedRules, setSelectedRules] = useState<string[]>([
    'PRICE_CHG',
    'PROMO_START',
    'PROMO_END_REVERT',
    'NEW_PDT',
  ]);
  const [lbsCode, setLbsCode] = useState('ADA-100x38');
  const [bchCode, setBchCode] = useState('');

  // Store User Secondary Settings (collapsed under "ตั้งค่าเพิ่มเติม" so main screen stays uncluttered)
  const [moreOptionsOpen, setMoreOptionsOpen] = useState(false);
  const [qtyMode, setQtyMode] = useState('1');
  const [staCoverSheet, setStaCoverSheet] = useState('1');
  const [onPrinterBack, setOnPrinterBack] = useState<'WAIT_CONFIRM' | 'AUTO_RESUME'>('WAIT_CONFIRM');
  const [onSyncFail, setOnSyncFail] = useState<'ABORT' | 'USE_LOCAL_WARN'>('ABORT');
  const [testBizDate, setTestBizDate] = useState(() => getLocalIsoDate());
  const testDateInputRef = useRef<HTMLInputElement | null>(null);

  const [runHour, runMinute] = useMemo(() => {
    const parts = (runTime || '04:00').split(':');
    const hh = String(Math.min(23, Math.max(0, parseInt(parts[0] || '4', 10) || 0))).padStart(2, '0');
    const mm = String(Math.min(59, Math.max(0, parseInt(parts[1] || '0', 10) || 0))).padStart(2, '0');
    return [hh, mm];
  }, [runTime]);

  const formattedTestDateTh = useMemo(() => {
    const parts = (testBizDate || '').split('-');
    if (parts.length !== 3) return testBizDate || '';
    return `${parts[2]}/${parts[1]}/${parts[0]}`;
  }, [testBizDate]);

  // Target Printer & Tray for Auto-Print (reads real mapping from Tab 3 for selected lbsCode; never silently fills DefaultPrinter)
  const [normalPrinter, setNormalPrinter] = useState('');
  const [normalTray, setNormalTray] = useState('');
  const [normalProto, setNormalProto] = useState('WINDOWS');
  const [promoPrinter, setPromoPrinter] = useState('');
  const [promoTray, setPromoTray] = useState('');
  const [promoProto, setPromoProto] = useState('WINDOWS');
  const [printerDirty, setPrinterDirty] = useState(false);

  // IT Admin Connection Settings (API2PSMaster POS6)
  const [pinUnlocked, setPinUnlocked] = useState(false);
  const [pinInput, setPinInput] = useState('');
  const [verifyingPin, setVerifyingPin] = useState(false);
  const [apiUrl, setApiUrl] = useState(() => getOpt('MasterApiUrl', ''));
  const [apiKey, setApiKey] = useState(() => getOpt('MasterApiKey', ''));
  const [agnCode, setAgnCode] = useState(() => getOpt('MasterAgnCode', ''));
  const [masterBch, setMasterBch] = useState(() => getOpt('MasterBchCode', ''));
  const [preSyncMin, setPreSyncMin] = useState(30);
  const [jitterSec, setJitterSec] = useState(900);
  const [syncRetry, setSyncRetry] = useState(3);
  const [retryWaitSec, setRetryWaitSec] = useState(300);
  const [misfireMin, setMisfireMin] = useState(300);
  const [retentionDays, setRetentionDays] = useState(() => getOpt('AutoPrintRetentionDays', '60'));
  const [newAdminPin, setNewAdminPin] = useState(() => getOpt('AutoPrintAdminPin', '1234'));

  const [lastSyncInfo, setLastSyncInfo] = useState<string>('');
  const [computerName, setComputerName] = useState<string>('');
  const [schedulerActive, setSchedulerActive] = useState<boolean>(true);
  const [savingSched, setSavingSched] = useState(false);
  const [savingAdmin, setSavingAdmin] = useState(false);
  const [testingSync, setTestingSync] = useState(false);
  const [runningTrigger, setRunningTrigger] = useState<'DRY_RUN' | 'PRINT_NOW' | null>(null);
  const [dryRunResult, setDryRunResult] = useState<any | null>(null);

  const printerChoices = useMemo(() => {
    const list: Array<{ value: string; label: string; proto: string; trays: string[] }> = [];
    for (const wp of windowsPrinters) {
      list.push({
        value: wp.name,
        label: `${wp.name}${wp.isDefault ? ' (ค่าเริ่มต้น Windows)' : ''}`,
        proto: 'WINDOWS',
        trays: wp.trays || [],
      });
    }
    for (const lp of printers) {
      if (lp.ip !== 'Windows' && !list.some((x) => x.value === lp.name)) {
        list.push({
          value: lp.name,
          label: `${lp.name} [LAN ${lp.ip}:${lp.port}]`,
          proto: lp.brand || 'ZEBRA',
          trays: [],
        });
      }
    }
    return list;
  }, [windowsPrinters, printers]);

  useEffect(() => {
    const mapped = driverMappings.find((m) => m.lbsCode === lbsCode);
    setNormalPrinter(mapped?.normalPrinter || '');
    setNormalTray(mapped?.normalTray || '');
    setNormalProto(mapped?.normalProtocol || 'WINDOWS');
    setPromoPrinter(mapped?.promoPrinter || '');
    setPromoTray(mapped?.promoTray || '');
    setPromoProto(mapped?.promoProtocol || 'WINDOWS');
    setPrinterDirty(false);
  }, [lbsCode, driverMappings]);

  const loadScheduleData = useCallback(async () => {
    try {
      const [sRes, syncRes] = await Promise.all([
        fetch('/api/schedules'),
        fetch('/api/sync/status'),
      ]);
      if (sRes.ok) {
        const sData = await sRes.json();
        if (sData?.computerName) setComputerName(String(sData.computerName));
        if (typeof sData?.schedulerActive === 'boolean') setSchedulerActive(sData.schedulerActive);
        const sc = sData?.schedules?.[0];
        if (sc) {
          setStaUse(sc.FTLscStaUse === '1' ? '1' : '2');
          setRunTime(sc.FTLscRunTime || '04:00');
          setDaysMask(sc.FTLscDaysMask || '1111111');
          setSelectedRules(
            (sc.FTLscTriggerRules || 'PRICE_CHG,PROMO_START,PROMO_END_REVERT,NEW_PDT')
              .split(',')
              .map((s: string) => s.trim())
              .filter(Boolean)
          );
          const fallbackLbs = allLabelSizes.find((s) => s.FTLbsStaUse === '1')?.FTLbsCode || 'ADA-100x38';
          setLbsCode(sc.FTLscLbsCode || fallbackLbs);
          setBchCode(sc.FTLscBchCode || '');
          setQtyMode(String(sc.FNLscQtyMode || 1));
          setStaCoverSheet(sc.FTLscStaCoverSheet === '2' ? '2' : '1');
          setOnPrinterBack(sc.FTLscOnPrinterBack === 'AUTO_RESUME' ? 'AUTO_RESUME' : 'WAIT_CONFIRM');
          setOnSyncFail(sc.FTLscOnSyncFail === 'USE_LOCAL_WARN' ? 'USE_LOCAL_WARN' : 'ABORT');
          setPreSyncMin(sc.FNLscPreSyncMin ?? 30);
          setJitterSec(sc.FNLscJitterSec ?? 900);
          setSyncRetry(sc.FNLscSyncRetry ?? 3);
          setRetryWaitSec(sc.FNLscRetryWaitSec ?? 300);
          setMisfireMin(sc.FNLscMisfireMin ?? 300);
        }
      }
      if (syncRes.ok) {
        const syncData = await syncRes.json();
        const st = syncData?.states?.[0];
        if (st) {
          setLastSyncInfo(`${st.FDLastSyncAt} (${st.FTSynLastStatus === 'OK' ? 'สำเร็จ' : 'ขัดข้อง'})`);
        } else {
          setLastSyncInfo('ยังไม่เคยดึงข้อมูลจาก API2PSMaster');
        }
      }
    } catch {
      // ignore initial error
    }
  }, []);

  useEffect(() => {
    loadScheduleData();
  }, [loadScheduleData]);

  useEffect(() => {
    setApiUrl(getOpt('MasterApiUrl', ''));
    setApiKey(getOpt('MasterApiKey', ''));
    setAgnCode(getOpt('MasterAgnCode', ''));
    setMasterBch(getOpt('MasterBchCode', ''));
    setRetentionDays(getOpt('AutoPrintRetentionDays', '60'));
    setNewAdminPin(getOpt('AutoPrintAdminPin', '1234'));
  }, [getOpt]);

  const toggleDay = (idx: number) => {
    const arr = daysMask.padEnd(7, '1').split('').slice(0, 7);
    arr[idx] = arr[idx] === '1' ? '0' : '1';
    setDaysMask(arr.join(''));
  };

  const toggleRule = (key: string) => {
    setSelectedRules((prev) =>
      prev.includes(key) ? prev.filter((k) => k !== key) : [...prev, key]
    );
  };

  const persistAdminOptionsOnly = async (): Promise<boolean> => {
    const res = await fetch('/api/config/options', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        options: [
          { code: 'MasterApiUrl', value: apiUrl.trim() },
          { code: 'MasterApiKey', value: apiKey.trim() },
          { code: 'MasterAgnCode', value: agnCode.trim() },
          { code: 'MasterBchCode', value: masterBch.trim() },
          { code: 'AutoPrintRetentionDays', value: String(retentionDays).trim() || '60' },
          { code: 'AutoPrintRetryWaitSec', value: String(retryWaitSec) },
          { code: 'AutoPrintAdminPin', value: newAdminPin.trim() || '1234' },
        ],
      }),
    });
    const data = await res.json();
    if (!res.ok || !data?.ok) {
      toast.error(data?.error || 'บันทึกค่าเชื่อมต่อ API2PSMaster ไม่สำเร็จ');
      return false;
    }
    return true;
  };

  const persistScheduleAndPrinters = async (requirePrinterWhenEnabled: boolean): Promise<boolean> => {
    if (selectedRules.length === 0) {
      toast.error('กรุณาเลือกเหตุผลที่ต้องการพิมพ์อย่างน้อย 1 ข้อ');
      return false;
    }
    if (requirePrinterWhenEnabled && staUse === '1' && (!normalPrinter.trim() || !promoPrinter.trim())) {
      toast.error('กรุณาเลือกเครื่องพิมพ์สำหรับทั้ง "ป้ายราคาปกติ" และ "ป้ายโปรโมชัน" ให้ครบก่อนเปิดใช้งาน');
      return false;
    }

    const cleanTime = normalizeTime24(runTime, '04:00');
    setRunTime(cleanTime);

    if (printerDirty) {
      const prtRes = await fetch('/api/label-printers', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          lbsCode,
          serverCode: driverServerCode || 'LOCAL',
          normalPrinter: normalPrinter.trim(),
          normalTray: normalTray.trim(),
          normalProtocol: normalProto || 'WINDOWS',
          promoPrinter: promoPrinter.trim(),
          promoTray: promoTray.trim(),
          promoProtocol: promoProto || 'WINDOWS',
        }),
      });
      if (!prtRes.ok) {
        const pErr = await prtRes.json().catch(() => ({}));
        toast.error(pErr?.error || 'บันทึกเครื่องพิมพ์ไม่สำเร็จ');
        return false;
      }
      setPrinterDirty(false);
    }

    const res = await fetch('/api/schedules/save', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        code: 'SCH-0400',
        name: `พิมพ์ป้ายอัตโนมัติประจำวัน (${cleanTime} น.)`,
        staUse,
        runTime: cleanTime,
        preSyncMin,
        jitterSec,
        daysMask,
        triggerRules: selectedRules.join(','),
        bchCode: bchCode || null,
        lbsCode,
        qtyMode: Number(qtyMode) || 1,
        syncMode: 'DIRECT_API',
        syncRetry,
        retryWaitSec,
        onSyncFail,
        onPrinterBack,
        misfireMin,
        staCoverSheet,
      }),
    });
    const data = await res.json();
    if (!res.ok || !data?.ok) {
      toast.error(data?.error || 'บันทึกตั้งค่าไม่สำเร็จ');
      return false;
    }
    return true;
  };

  const handleSaveSchedule = async () => {
    setSavingSched(true);
    try {
      const ok = await persistScheduleAndPrinters(true);
      if (!ok) return;
      await loadScheduleData();
      await onRefreshAll();
      toast.success('บันทึกการตั้งค่าพิมพ์อัตโนมัติเรียบร้อยแล้ว');
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setSavingSched(false);
    }
  };

  const handleTrigger = async (mode: 'DRY_RUN' | 'PRINT_NOW') => {
    if (mode === 'PRINT_NOW' && (!normalPrinter.trim() || !promoPrinter.trim())) {
      toast.error('กรุณาเลือกเครื่องพิมพ์สำหรับทั้ง "ป้ายราคาปกติ" และ "ป้ายโปรโมชัน" ให้ครบก่อนสั่งพิมพ์จริง');
      return;
    }
    if (selectedRules.length === 0) {
      toast.error('กรุณาเลือกเหตุผลที่ต้องการพิมพ์อย่างน้อย 1 ข้อ');
      return;
    }
    setRunningTrigger(mode);
    try {
      if (mode === 'PRINT_NOW') {
        const saved = await persistScheduleAndPrinters(false);
        if (!saved) return;
      }

      const payload =
        mode === 'DRY_RUN'
          ? {
              mode,
              bizDate: testBizDate || undefined,
              lbsCode,
              triggerRules: selectedRules.join(','),
              bchCode: bchCode || '',
              qtyMode: Number(qtyMode) || 1,
            }
          : { mode, bizDate: testBizDate || undefined };

      const res = await fetch('/api/schedules/SCH-0400/trigger', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      if (!res.ok) {
        toast.error(data?.error || 'สั่งทำงานไม่สำเร็จ');
        return;
      }
      if (mode === 'DRY_RUN') {
        setDryRunResult(data);
        toast.success(`จำลองรายการวันที่ ${data.bizDate} เรียบร้อย: พบ ${data.totalCandidates} รายการ`);
      } else {
        if (data.status === 'SYNC_FAILED') {
          toast.error(data.errMsg || 'ดึงข้อมูลจาก API2PSMaster ไม่ได้ ระบบจึงหยุดไม่พิมพ์ราคาเก่า');
        } else if (data.status === 'FAILED') {
          toast.error(data.errMsg || `สั่งพิมพ์ไม่สำเร็จ (ค้าง ${data.failQty} รายการ)`);
        } else if (data.status === 'PARTIAL' || (data.failQty ?? 0) > 0) {
          toast.warning(`พิมพ์ออก ${data.norQty + data.pmtQty} ใบ, ยังค้าง ${data.failQty} ใบ (${data.errMsg || 'ตรวจเช็กเครื่องพิมพ์'})`);
        } else if (data.status === 'SKIPPED_EMPTY') {
          toast.info(`ไม่พบสินค้าที่ต้องพิมพ์ใหม่สำหรับวันที่ ${data.bizDate} (หรือเคยพิมพ์ราคานี้ไปแล้ว)`);
        } else {
          toast.success(`สั่งพิมพ์เสร็จสิ้น: ป้ายปกติ ${data.norQty} ใบ, โปรฯ ${data.pmtQty} ใบ`);
        }
        await loadScheduleData();
        await onRefreshAll();
      }
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setRunningTrigger(null);
    }
  };

  const handleVerifyPin = async (e: React.FormEvent) => {
    e.preventDefault();
    setVerifyingPin(true);
    try {
      const res = await fetch('/api/config/admin-verify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ pin: pinInput }),
      });
      const data = await res.json();
      if (!res.ok || !data?.ok) {
        toast.error(data?.error || 'รหัส PIN ไม่ถูกต้อง');
        return;
      }
      setPinUnlocked(true);
      setPinInput('');
      toast.success('ปลดล็อกส่วนเชื่อมต่อ API2PSMaster แล้ว');
    } catch (e2) {
      toast.error(e2 instanceof Error ? e2.message : String(e2));
    } finally {
      setVerifyingPin(false);
    }
  };

  const handleSaveAdminOptions = async () => {
    setSavingAdmin(true);
    try {
      if (!(await persistAdminOptionsOnly())) return;
      await handleSaveSchedule();
      toast.success('บันทึกค่าเชื่อมต่อ API2PSMaster และรอบเวลาเรียบร้อยแล้ว');
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setSavingAdmin(false);
    }
  };

  const handleTestSync = async () => {
    if (!apiUrl.trim()) {
      toast.error('กรุณากรอก URL ของ API2PSMaster ในช่องด้านบนก่อนกดทดสอบ');
      return;
    }
    setTestingSync(true);
    try {
      // Save current URL / Key in the input fields automatically first so user doesn't have to click Save separately!
      const saved = await persistAdminOptionsOnly();
      if (!saved) return;

      const res = await fetch('/api/sync/trigger', { method: 'POST' });
      const data = await res.json();
      if (!res.ok || !data?.ok) {
        toast.error(data?.message || data?.error || 'เชื่อมต่อ API2PSMaster ไม่สำเร็จ');
      } else {
        toast.success(data.message || 'ดึงข้อมูลล่าสุดจาก API2PSMaster สำเร็จ');
      }
      await loadScheduleData();
      await onRefreshAll();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setTestingSync(false);
    }
  };

  const normalChoiceObj = printerChoices.find((p) => p.value === normalPrinter);
  const promoChoiceObj = printerChoices.find((p) => p.value === promoPrinter);
  const defaultSysPrinter = getOpt('DefaultPrinter', '');
  const hasPrinterReady = Boolean(normalPrinter.trim() && promoPrinter.trim());

  return (
    <div className="space-y-4">
      {/* ── Store User Section (Clean 3-Part Layout) ── */}
      <Card className="border-slate-200 shadow-xs bg-white py-0 gap-0">
        <CardHeader className="bg-[#3d4a57] text-white py-2.5 px-4 flex flex-row flex-wrap items-center justify-between gap-2">
          <div>
            <CardTitle className="text-sm font-semibold flex items-center gap-2">
              <Clock className="h-4 w-4 text-teal-300" />
              <span>ตั้งเวลาพิมพ์ป้ายอัตโนมัติ (สำหรับหน้าร้าน)</span>
            </CardTitle>
            <p className="text-[11px] text-slate-300">
              ตั้ง 3 อย่าง: <b>1) เวลา &amp; วัน</b> &rarr; <b>2) ขนาดป้าย &amp; เหตุผล</b> &rarr; <b>3) เครื่องพิมพ์ปกติ / โปรโมชัน</b> แล้วกดบันทึกได้เลย
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Badge className="bg-slate-700 text-slate-100 text-[11px]">
              เครื่องที่ตั้งเวลา: {computerName || 'เครื่องนี้'} {schedulerActive ? '• ตัวจับเวลาพร้อม' : '• ปิดตัวจับเวลา'}
            </Badge>
            <Badge className={staUse === '1' ? 'bg-emerald-500 text-white' : 'bg-slate-600 text-slate-200'}>
              {staUse === '1' ? `เปิดพิมพ์อัตโนมัติ (${runTime} น.)` : 'ปิดอยู่ (กดพิมพ์เอง)'}
            </Badge>
          </div>
        </CardHeader>

        <CardContent className="p-4 space-y-4">
          {/* Block 1: When (Switch + Time + Days) */}
          <div className="rounded-lg border border-slate-200 bg-slate-50/70 p-3.5">
            <div className="grid grid-cols-1 lg:grid-cols-12 gap-4 items-end">
              <div className="lg:col-span-3 space-y-1.5 min-w-0">
                <Label className="text-xs font-semibold text-slate-800">1. สถานะการทำงาน</Label>
                <Select value={staUse} onValueChange={(v) => setStaUse(v as '1' | '2')}>
                  <SelectTrigger className="h-9 w-full min-w-0 overflow-hidden text-xs bg-white">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="1">เปิดใช้งาน (พิมพ์อัตโนมัติตามเวลาที่ตั้ง)</SelectItem>
                    <SelectItem value="2">ปิดไว้ (ให้พนักงานกดพิมพ์เอง)</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              <div className="lg:col-span-3 space-y-1.5 min-w-0">
                <div className="flex items-center justify-between gap-2">
                  <Label className="text-xs font-semibold text-slate-800">เวลาเริ่มพิมพ์ (24 ชม.)</Label>
                  <div className="flex items-center gap-1.5">
                    <button
                      type="button"
                      onClick={() => setRunTime('04:00')}
                      className="text-[10px] text-[#007a87] hover:underline"
                    >
                      04:00
                    </button>
                    <span className="text-[10px] text-slate-300">|</span>
                    <button
                      type="button"
                      onClick={() => setRunTime(getLocalTimePlusMin(2))}
                      className="text-[10px] text-[#007a87] hover:underline"
                    >
                      +2 นาที
                    </button>
                  </div>
                </div>
                <div className="flex items-center gap-1.5">
                  <div className="flex-1 min-w-0">
                    <SearchableSelect
                      value={runHour}
                      onValueChange={(hh) => setRunTime(`${hh}:${runMinute}`)}
                      searchPlaceholder="พิมพ์ชั่วโมง (00-23)..."
                      className="h-9 bg-white text-xs font-mono"
                      dropdownMinWidth={150}
                      options={Array.from({ length: 24 }, (_, i) => {
                        const h = String(i).padStart(2, '0');
                        return { value: h, label: h, keywords: `${h} ${i}` };
                      })}
                    />
                  </div>
                  <span className="text-sm font-bold text-slate-500">:</span>
                  <div className="flex-1 min-w-0">
                    <SearchableSelect
                      value={runMinute}
                      onValueChange={(mm) => setRunTime(`${runHour}:${mm}`)}
                      searchPlaceholder="พิมพ์นาที (00-59)..."
                      className="h-9 bg-white text-xs font-mono"
                      dropdownMinWidth={150}
                      options={Array.from({ length: 60 }, (_, i) => {
                        const m = String(i).padStart(2, '0');
                        return { value: m, label: m, keywords: `${m} ${i}` };
                      })}
                    />
                  </div>
                  <span className="text-xs font-medium text-slate-600 shrink-0">น.</span>
                </div>
              </div>

              <div className="lg:col-span-6 space-y-1.5">
                <Label className="text-xs font-semibold text-slate-800">วันที่ให้พิมพ์อัตโนมัติ</Label>
                <div className="flex flex-wrap gap-1.5">
                  {DAY_LABELS.map((lbl, idx) => {
                    const active = daysMask.padEnd(7, '1')[idx] === '1';
                    return (
                      <Button
                        key={lbl}
                        type="button"
                        size="sm"
                        variant={active ? 'default' : 'outline'}
                        className={`h-9 px-3 text-xs ${active ? 'bg-[#007a87] hover:bg-[#00606b] text-white' : 'bg-white'}`}
                        onClick={() => toggleDay(idx)}
                      >
                        {lbl}
                      </Button>
                    );
                  })}
                </div>
              </div>
            </div>
          </div>

          {/* Block 2 & 3 Side-by-Side: Left = Label & Rules, Right = Printers & Trays */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            {/* Left Box: Label Size + Branch + 4 Rules */}
            <div className="rounded-lg border border-slate-200 p-3.5 space-y-3 bg-white min-w-0">
              <div className="text-xs font-bold text-slate-800">2. ขนาดป้าย &amp; รายการที่จะคัดมาพิมพ์</div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                <div className="space-y-1 min-w-0">
                  <Label className="text-[11px] text-slate-600">ขนาดป้ายเริ่มต้น</Label>
                  <SearchableSelect
                    value={lbsCode}
                    onValueChange={setLbsCode}
                    placeholder="เลือกขนาดป้าย..."
                    searchPlaceholder="ค้นหารหัส หรือชื่อขนาดป้าย..."
                    className="h-9 text-xs"
                    dropdownMinWidth={320}
                    options={allLabelSizes
                      .filter((s) => s.FTLbsStaUse === '1')
                      .map((s) => ({
                        value: s.FTLbsCode,
                        label: s.FTLbsName || s.FTLbsCode,
                        keywords: `${s.FTLbsCode} ${s.FTLbsName || ''} ${s.FCLbsWidth}x${s.FCLbsHeight}`,
                        badge: s.FTLbsGroup === '2' ? 'ม้วน' : 'A4',
                        badgeClassName:
                          s.FTLbsGroup === '2'
                            ? 'bg-sky-100 text-sky-800'
                            : 'bg-slate-100 text-slate-700',
                      }))}
                  />
                </div>

                <div className="space-y-1 min-w-0">
                  <Label className="text-[11px] text-slate-600">สาขาของเครื่องนี้</Label>
                  <SearchableSelect
                    value={bchCode || '__AUTO__'}
                    onValueChange={(v) => setBchCode(v === '__AUTO__' ? '' : v)}
                    placeholder="เลือกสาขา..."
                    searchPlaceholder="ค้นหารหัส หรือชื่อสาขา..."
                    className="h-9 text-xs"
                    options={[
                      { value: '__AUTO__', label: 'ใช้สาขาหลักที่ตั้งไว้ในระบบ', badge: 'AUTO', badgeClassName: 'bg-teal-100 text-teal-800' },
                      ...branches.map((b) => ({
                        value: b.FTBchCode,
                        label: `${b.FTBchName || b.FTBchCode} (${b.FTBchCode})`,
                        keywords: `${b.FTBchCode} ${b.FTBchName || ''}`,
                        badge: b.FTBchCode,
                      })),
                    ]}
                  />
                </div>
              </div>

              <div className="space-y-1.5 pt-1">
                <Label className="text-[11px] text-slate-600">เหตุผลที่ให้พิมพ์ (ไม่พิมพ์ทั้งร้าน)</Label>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2 bg-slate-50 p-2.5 rounded-md border border-slate-200">
                  {RULE_OPTIONS.map((r) => (
                    <label key={r.key} className="flex items-center gap-2 text-xs cursor-pointer">
                      <Checkbox
                        checked={selectedRules.includes(r.key)}
                        onCheckedChange={() => toggleRule(r.key)}
                      />
                      <span>{r.label}</span>
                    </label>
                  ))}
                </div>
              </div>
            </div>

            {/* Right Box: Target Printers & Trays */}
            <div className={`rounded-lg border p-3.5 space-y-3 min-w-0 ${hasPrinterReady ? 'bg-blue-50/40 border-blue-200' : 'bg-amber-50 border-amber-300'}`}>
              <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="flex items-center gap-2 text-xs font-bold text-slate-800">
                  <Printer className="h-4 w-4 text-[#007a87] shrink-0" />
                  <span>3. เครื่องพิมพ์ &amp; ถาดกระดาษ (สำหรับขนาดป้ายที่เลือก)</span>
                </div>
                {!hasPrinterReady && (
                  <Badge className="bg-amber-200 text-amber-900 text-[10px] flex items-center gap-1">
                    <AlertTriangle className="h-3 w-3" />
                    <span>กรุณาเลือกเครื่องพิมพ์</span>
                  </Badge>
                )}
              </div>

              <div className="bg-white rounded-md border border-slate-200 p-2.5 space-y-2 min-w-0">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="text-xs font-semibold text-slate-800">ป้ายราคาปกติ</div>
                  {!normalPrinter.trim() && defaultSysPrinter && (
                    <button
                      type="button"
                      onClick={() => {
                        const found = printerChoices.find((c) => c.value === defaultSysPrinter);
                        setNormalPrinter(defaultSysPrinter);
                        setNormalProto(found?.proto || 'WINDOWS');
                        setNormalTray('');
                        setPrinterDirty(true);
                      }}
                      className="text-[10px] text-[#007a87] hover:underline"
                    >
                      ใช้ค่าเริ่มต้นระบบ ({defaultSysPrinter})
                    </button>
                  )}
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                  <div className="min-w-0">
                    <SearchableSelect
                      value={normalPrinter}
                      onValueChange={(v) => {
                        const val = v === '__NONE__' ? '' : v;
                        const found = printerChoices.find((c) => c.value === val);
                        setNormalPrinter(val);
                        setNormalProto(found?.proto || 'WINDOWS');
                        if (!found?.trays?.includes(normalTray)) setNormalTray('');
                        setPrinterDirty(true);
                      }}
                      placeholder="เลือกเครื่องพิมพ์..."
                      searchPlaceholder="ค้นหาชื่อเครื่องพิมพ์..."
                      className="h-8 text-xs"
                      options={[
                        ...(normalPrinter ? [{ value: '__NONE__', label: '-- ล้างการเลือก (ยังไม่เลือก) --' }] : []),
                        ...printerChoices.map((c) => ({
                          value: c.value,
                          label: c.label,
                          keywords: `${c.value} ${c.label} ${c.proto}`,
                          badge: c.proto === 'WINDOWS' ? 'WIN' : 'LAN',
                          badgeClassName:
                            c.proto === 'WINDOWS'
                              ? 'bg-slate-100 text-slate-700'
                              : 'bg-emerald-100 text-emerald-800',
                        })),
                      ]}
                    />
                  </div>

                  <div className="min-w-0">
                    <Select
                      value={normalTray || '__DEF__'}
                      disabled={!normalChoiceObj || normalChoiceObj.trays.length === 0}
                      onValueChange={(v) => {
                        setNormalTray(v === '__DEF__' ? '' : v);
                        setPrinterDirty(true);
                      }}
                    >
                      <SelectTrigger className="h-8 w-full min-w-0 overflow-hidden text-xs">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="__DEF__">ถาดเริ่มต้น (Auto)</SelectItem>
                        {(normalChoiceObj?.trays || []).map((tr) => (
                          <SelectItem key={`nt-${tr}`} value={tr}>
                            {formatTrayLabel(tr)}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>
              </div>

              <div className="bg-white rounded-md border border-red-200 p-2.5 space-y-2 min-w-0">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <div className="text-xs font-semibold text-red-700">ป้ายโปรโมชัน</div>
                  {normalPrinter.trim() && promoPrinter !== normalPrinter && (
                    <button
                      type="button"
                      onClick={() => {
                        setPromoPrinter(normalPrinter);
                        setPromoProto(normalProto);
                        setPromoTray(normalTray);
                        setPrinterDirty(true);
                      }}
                      className="text-[10px] text-[#007a87] hover:underline"
                    >
                      ใช้เครื่องเดียวกับป้ายปกติ
                    </button>
                  )}
                </div>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
                  <div className="min-w-0">
                    <SearchableSelect
                      value={promoPrinter}
                      onValueChange={(v) => {
                        const val = v === '__NONE__' ? '' : v;
                        const found = printerChoices.find((c) => c.value === val);
                        setPromoPrinter(val);
                        setPromoProto(found?.proto || 'WINDOWS');
                        if (!found?.trays?.includes(promoTray)) setPromoTray('');
                        setPrinterDirty(true);
                      }}
                      placeholder="เลือกเครื่องพิมพ์..."
                      searchPlaceholder="ค้นหาชื่อเครื่องพิมพ์..."
                      className="h-8 text-xs"
                      options={[
                        ...(promoPrinter ? [{ value: '__NONE__', label: '-- ล้างการเลือก (ยังไม่เลือก) --' }] : []),
                        ...printerChoices.map((c) => ({
                          value: c.value,
                          label: c.label,
                          keywords: `${c.value} ${c.label} ${c.proto}`,
                          badge: c.proto === 'WINDOWS' ? 'WIN' : 'LAN',
                          badgeClassName:
                            c.proto === 'WINDOWS'
                              ? 'bg-slate-100 text-slate-700'
                              : 'bg-emerald-100 text-emerald-800',
                        })),
                      ]}
                    />
                  </div>

                  <div className="min-w-0">
                    <Select
                      value={promoTray || '__DEF__'}
                      disabled={!promoChoiceObj || promoChoiceObj.trays.length === 0}
                      onValueChange={(v) => {
                        setPromoTray(v === '__DEF__' ? '' : v);
                        setPrinterDirty(true);
                      }}
                    >
                      <SelectTrigger className="h-8 w-full min-w-0 overflow-hidden text-xs">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="__DEF__">ถาดเริ่มต้น (Auto)</SelectItem>
                        {(promoChoiceObj?.trays || []).map((tr) => (
                          <SelectItem key={`pt-${tr}`} value={tr}>
                            {formatTrayLabel(tr)}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>
              </div>
            </div>
          </div>

          {/* Collapsible Advanced Store Options */}
          <div className="border border-slate-200 rounded-lg overflow-hidden">
            <button
              type="button"
              onClick={() => setMoreOptionsOpen((o) => !o)}
              className="w-full px-3.5 py-2 bg-slate-50 hover:bg-slate-100 flex items-center justify-between text-xs font-medium text-slate-700"
            >
              <span className="flex items-center gap-1.5">
                <Settings2 className="h-3.5 w-3.5 text-slate-500" />
                <span>ตั้งค่าเพิ่มเติม &amp; วันที่สำหรับทดสอบ (ระบบตั้งค่ามาตรฐานไว้ให้แล้ว)</span>
              </span>
              {moreOptionsOpen ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
            </button>

            {moreOptionsOpen && (
              <div className="p-3.5 bg-white border-t border-slate-200 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
                <div className="space-y-1 min-w-0">
                  <Label className="text-[11px] font-medium">จำนวนป้ายต่อ 1 สินค้า</Label>
                  <Select value={qtyMode} onValueChange={setQtyMode}>
                    <SelectTrigger className="h-8 w-full min-w-0 overflow-hidden text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1">พิมพ์สินค้าละ 1 ดวงเสมอ</SelectItem>
                      <SelectItem value="2">ใช้จำนวนป้ายประจำสินค้า (ค่าเริ่มต้น 1 ดวง)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-1 min-w-0">
                  <Label className="text-[11px] font-medium">ใบสรุปหน้าปกตอนเปิดร้าน</Label>
                  <Select value={staCoverSheet} onValueChange={setStaCoverSheet}>
                    <SelectTrigger className="h-8 w-full min-w-0 overflow-hidden text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1">พิมพ์ใบสรุปยอด 1 แผ่น (แนะนำ)</SelectItem>
                      <SelectItem value="2">ไม่ต้องพิมพ์ใบสรุป</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-1 min-w-0">
                  <div className="flex items-center justify-between gap-2">
                    <Label className="text-[11px] font-medium">วันที่ใช้จำลอง / ทดสอบ</Label>
                    <div className="flex items-center gap-1.5">
                      <button
                        type="button"
                        onClick={() => setTestBizDate(getLocalIsoDate(0))}
                        className="text-[10px] text-[#007a87] hover:underline"
                      >
                        วันนี้
                      </button>
                      <span className="text-[10px] text-slate-300">|</span>
                      <button
                        type="button"
                        onClick={() => setTestBizDate(getLocalIsoDate(-1))}
                        className="text-[10px] text-[#007a87] hover:underline"
                      >
                        เมื่อวาน
                      </button>
                      <span className="text-[10px] text-slate-300">|</span>
                      <button
                        type="button"
                        onClick={() => setTestBizDate('2026-01-01')}
                        className="text-[10px] text-[#007a87] hover:underline"
                      >
                        วันตัวอย่าง (01/01/2026)
                      </button>
                    </div>
                  </div>
                  <div className="relative">
                    <button
                      type="button"
                      onClick={() => {
                        const el = testDateInputRef.current as (HTMLInputElement & { showPicker?: () => void }) | null;
                        if (!el) return;
                        if (typeof el.showPicker === 'function') {
                          el.showPicker();
                        } else {
                          el.focus();
                          el.click();
                        }
                      }}
                      className="h-8 w-full rounded-md border border-input bg-white px-2.5 flex items-center justify-between text-xs font-mono text-slate-800 shadow-xs hover:border-slate-400"
                    >
                      <span>{formattedTestDateTh}</span>
                      <Calendar className="h-3.5 w-3.5 text-slate-500 shrink-0" />
                    </button>
                    <input
                      ref={testDateInputRef}
                      type="date"
                      value={testBizDate}
                      onChange={(e) => setTestBizDate(e.target.value)}
                      tabIndex={-1}
                      aria-hidden="true"
                      className="absolute inset-0 w-full h-full opacity-0 pointer-events-none"
                    />
                  </div>
                </div>

                <div className="space-y-1 lg:col-span-2 min-w-0">
                  <Label className="text-[11px] font-medium">หากเครื่องพิมพ์ดับกลางคัน แล้วกลับมาติดทีหลัง ให้ทำอย่างไร?</Label>
                  <Select
                    value={onPrinterBack}
                    onValueChange={(v) => setOnPrinterBack(v as 'WAIT_CONFIRM' | 'AUTO_RESUME')}
                  >
                    <SelectTrigger className="h-8 w-full min-w-0 overflow-hidden text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="WAIT_CONFIRM">
                        รอพนักงานมากดปุ่มพิมพ์ซ้ำเองตอนเช้า (แนะนำ — ปลอดภัยสุด)
                      </SelectItem>
                      <SelectItem value="AUTO_RESUME">
                        พิมพ์ต่อเฉพาะใบที่ค้างให้อัตโนมัติทันทีในวันเดียวกัน
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-1 min-w-0">
                  <Label className="text-[11px] font-medium">หากเน็ตล่ม ดึงราคาใหม่ไม่ได้</Label>
                  <Select
                    value={onSyncFail}
                    onValueChange={(v) => setOnSyncFail(v as 'ABORT' | 'USE_LOCAL_WARN')}
                  >
                    <SelectTrigger className="h-8 w-full min-w-0 overflow-hidden text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="ABORT">หยุดพิมพ์ทันที (แนะนำ)</SelectItem>
                      <SelectItem value="USE_LOCAL_WARN">ใช้ราคาในเครื่อง + แจ้งเตือน</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            )}
          </div>

          {/* Primary Action Bar */}
          <div className="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-slate-100">
            <div className="flex flex-wrap items-center gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="text-xs"
                disabled={runningTrigger !== null}
                onClick={() => handleTrigger('DRY_RUN')}
              >
                {runningTrigger === 'DRY_RUN' ? (
                  <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                ) : (
                  <FlaskConical className="h-3.5 w-3.5 mr-1 text-amber-600" />
                )}
                ทดลองจำลองรายการ (ไม่ใช้กระดาษ)
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="text-xs border-[#007a87] text-[#007a87]"
                disabled={runningTrigger !== null}
                onClick={() => handleTrigger('PRINT_NOW')}
              >
                {runningTrigger === 'PRINT_NOW' ? (
                  <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                ) : (
                  <Play className="h-3.5 w-3.5 mr-1" />
                )}
                สั่งพิมพ์รอบนี้ทันที
              </Button>
            </div>

            <Button
              type="button"
              className="bg-[#007a87] hover:bg-[#00606b] text-white"
              disabled={savingSched}
              onClick={handleSaveSchedule}
            >
              {savingSched ? <Loader2 className="h-4 w-4 mr-1.5 animate-spin" /> : <Save className="h-4 w-4 mr-1.5" />}
              บันทึกการตั้งค่า
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* ── IT Admin Section (API2PSMaster POS6 Direct Sync) ── */}
      <Card className="border-slate-200 shadow-xs bg-white py-0 gap-0">
        <CardHeader className="bg-slate-800 text-white py-2.5 px-4 flex flex-row flex-wrap items-center justify-between gap-2">
          <div>
            <CardTitle className="text-sm font-semibold flex items-center gap-2">
              {pinUnlocked ? <Unlock className="h-4 w-4 text-emerald-400" /> : <Lock className="h-4 w-4 text-amber-300" />}
              <span>ตั้งค่าดึงข้อมูลจาก API2PSMaster (เฉพาะผู้ดูแลระบบ / ไอที)</span>
            </CardTitle>
            <p className="text-[11px] text-slate-300">
              สถานะดึงข้อมูลล่าสุด: <span className="font-mono text-teal-300">{lastSyncInfo || '-'}</span>
            </p>
          </div>
          {pinUnlocked && (
            <Button
              type="button"
              size="sm"
              variant="secondary"
              className="h-7 text-xs"
              onClick={() => setPinUnlocked(false)}
            >
              <Lock className="h-3 w-3 mr-1" />ล็อกหน้านี้
            </Button>
          )}
        </CardHeader>

        <CardContent className="p-4">
          {!pinUnlocked ? (
            <form onSubmit={handleVerifyPin} className="flex flex-wrap items-end gap-3 max-w-md">
              <div className="space-y-1 flex-1">
                <Label className="text-xs font-medium flex items-center gap-1">
                  <KeyRound className="h-3.5 w-3.5 text-amber-600" />
                  <span>รหัสผ่านไอที 4 หลัก (ค่าเริ่มต้น 1234)</span>
                </Label>
                <Input
                  type="password"
                  value={pinInput}
                  onChange={(e) => setPinInput(e.target.value)}
                  placeholder="กรอกรหัส PIN"
                  className="h-9 text-xs font-mono"
                />
              </div>
              <Button type="submit" disabled={verifyingPin} className="h-9 bg-slate-800 text-white">
                {verifyingPin && <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />}
                ปลดล็อก
              </Button>
            </form>
          ) : (
            <div className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-1.5">
                  <Label className="text-xs font-medium">API2PSMaster URL (ส่วนกลาง POS6)</Label>
                  <Input
                    value={apiUrl}
                    onChange={(e) => setApiUrl(e.target.value)}
                    placeholder="https://hq-api.adasoft.com/API2PSMaster/V6"
                    className="h-9 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1.5">
                  <Label className="text-xs font-medium">API Key (X-API-Key)</Label>
                  <Input
                    type="password"
                    value={apiKey}
                    onChange={(e) => setApiKey(e.target.value)}
                    placeholder="ระบุ X-API-Key จากเซิร์ฟเวอร์"
                    className="h-9 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1.5">
                  <Label className="text-xs font-medium">Agency Code (รหัสบริษัท/ตัวแทน)</Label>
                  <Input
                    value={agnCode}
                    onChange={(e) => setAgnCode(e.target.value)}
                    placeholder="เช่น 001"
                    className="h-9 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1.5">
                  <Label className="text-xs font-medium">Branch Code (รหัสสาขา)</Label>
                  <Input
                    value={masterBch}
                    onChange={(e) => setMasterBch(e.target.value)}
                    placeholder="เช่น 00001"
                    className="h-9 text-xs font-mono"
                  />
                </div>
              </div>

              {/* Uncramped 2-row Grid for timing & retention */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3 pt-2 border-t border-slate-100">
                <div className="space-y-1">
                  <Label className="text-[11px]">ดึงข้อมูลล่วงหน้า (นาที)</Label>
                  <Input
                    type="number"
                    value={preSyncMin}
                    onChange={(e) => setPreSyncMin(Number(e.target.value) || 30)}
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[11px]">กระจายเวลาเริ่ม (วินาที)</Label>
                  <Input
                    type="number"
                    value={jitterSec}
                    onChange={(e) => setJitterSec(Number(e.target.value) || 0)}
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[11px]">ลองใหม่เมื่อเน็ตหลุด (ครั้ง)</Label>
                  <Input
                    type="number"
                    min={1}
                    max={5}
                    value={syncRetry}
                    onChange={(e) => setSyncRetry(Number(e.target.value) || 3)}
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[11px]">รอระหว่างลองใหม่ (วินาที)</Label>
                  <Input
                    type="number"
                    value={retryWaitSec}
                    onChange={(e) => setRetryWaitSec(Number(e.target.value) || 300)}
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[11px]">ชดเชยหลังไฟดับ (นาที)</Label>
                  <Input
                    type="number"
                    value={misfireMin}
                    onChange={(e) => setMisfireMin(Number(e.target.value) || 300)}
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[11px]">เก็บประวัติย้อนหลัง (วัน)</Label>
                  <Input
                    type="number"
                    value={retentionDays}
                    onChange={(e) => setRetentionDays(e.target.value)}
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1 md:col-span-2">
                  <Label className="text-[11px]">รหัส PIN ผู้ดูแลระบบใหม่ (ถ้าต้องการเปลี่ยน)</Label>
                  <Input
                    type="password"
                    value={newAdminPin}
                    onChange={(e) => setNewAdminPin(e.target.value)}
                    className="h-8 text-xs font-mono"
                  />
                </div>
              </div>

              <div className="flex flex-wrap items-center justify-between gap-2 pt-2 border-t border-slate-100">
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  disabled={testingSync}
                  onClick={handleTestSync}
                >
                  {testingSync ? (
                    <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                  ) : (
                    <RefreshCw className="h-3.5 w-3.5 mr-1 text-purple-600" />
                  )}
                  ทดสอบดึงข้อมูล API2PSMaster ตอนนี้
                </Button>

                <Button
                  type="button"
                  size="sm"
                  className="bg-slate-800 hover:bg-slate-900 text-white"
                  disabled={savingAdmin}
                  onClick={handleSaveAdminOptions}
                >
                  {savingAdmin ? (
                    <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                  ) : (
                    <CheckCircle2 className="h-3.5 w-3.5 mr-1" />
                  )}
                  บันทึกค่าเชื่อมต่อทั้งหมด
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Dry-Run Preview Modal */}
      <Dialog open={Boolean(dryRunResult)} onOpenChange={(o) => { if (!o) setDryRunResult(null); }}>
        <DialogContent className="sm:max-w-3xl max-h-[82vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="text-base">ตัวอย่างรายการที่จะพิมพ์รอบอัตโนมัติ (จำลองเท่านั้น ยังไม่ได้พิมพ์จริง)</DialogTitle>
            <DialogDescription className="text-xs">
              วันที่คำนวณ: {dryRunResult?.bizDate} &bull; เข้าข่ายทั้งหมด {dryRunResult?.totalCandidates ?? 0} รายการ &bull; ป้ายปกติ {dryRunResult?.norQty ?? 0} ใบ &bull; ป้ายโปรฯ {dryRunResult?.pmtQty ?? 0} ใบ &bull; ข้ามรายการเดิม {dryRunResult?.skippedDup ?? 0}
            </DialogDescription>
          </DialogHeader>
          <div className="text-xs space-y-1.5 max-h-96 overflow-y-auto">
            {(dryRunResult?.dryRunDelta?.items || []).length === 0 ? (
              <div className="text-center text-slate-400 py-8">ไม่มีสินค้าที่มีการเปลี่ยนแปลงราคา/โปรฯ สำหรับวันที่เลือก</div>
            ) : (
              (dryRunResult?.dryRunDelta?.items || []).slice(0, 200).map((it: any) => (
                <div
                  key={`${it.pdtCode}-${it.barCode}`}
                  className="flex items-center justify-between border rounded px-3 py-1.5 bg-slate-50"
                >
                  <div>
                    <span className="font-mono font-semibold">{it.barCode}</span>{' '}
                    <span className="text-slate-600">{it.pdtName}</span>
                    <span className="ml-2 text-[11px] text-[#007a87] font-medium">
                      ({triggerSourceLabel(it.triggerSrc)})
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="font-mono font-bold">{Number(it.salePrice).toFixed(2)} ฿</span>
                    <Badge className={`text-[10px] ${it.skippedDup ? 'bg-slate-200 text-slate-600' : it.priType === '2' ? 'bg-red-100 text-red-800' : 'bg-emerald-100 text-emerald-800'}`}>
                      {it.skippedDup ? 'พิมพ์ราคานี้แล้ว (ข้าม)' : it.priType === '2' ? 'ป้ายโปรฯ' : 'ป้ายปกติ'}
                    </Badge>
                  </div>
                </div>
              ))
            )}
          </div>
          <DialogFooter>
            <Button size="sm" onClick={() => setDryRunResult(null)}>ปิดหน้าต่าง</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
