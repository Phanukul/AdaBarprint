import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';
import {
  AlertTriangle,
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  Clock,
  FileDown,
  History,
  Loader2,
  RefreshCw,
  RotateCcw,
  Search,
  Settings,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import type {
  ManualLoadLine,
  SchedQueueRecord,
  SchedRunRecord,
} from '@/components/MorningOpsBanner';

interface AutoScheduleHistoryProps {
  onLoadManualRows?: (lines: ManualLoadLine[]) => void;
  onOpenAutoPrintTab?: () => void;
  readOnly?: boolean;
}

function formatRunStatusTh(status: string): string {
  switch (status) {
    case 'SUCCESS':
      return 'พิมพ์ครบเรียบร้อย';
    case 'PARTIAL':
      return 'พิมพ์ได้บางส่วน (มีค้างพิมพ์)';
    case 'FAILED':
      return 'พิมพ์ไม่สำเร็จ';
    case 'SYNC_FAILED':
      return 'ดึงข้อมูลจากส่วนกลางไม่ได้';
    case 'SKIPPED_EMPTY':
      return 'ไม่มีรายการเปลี่ยน';
    case 'RUNNING':
      return 'กำลังทำงาน';
    default:
      return status;
  }
}

function formatTriggerByTh(triggerBy: string): string {
  switch (triggerBy) {
    case 'TIMER':
      return 'ตั้งเวลาอัตโนมัติ';
    case 'PRINT_NOW':
      return 'กดสั่งพิมพ์ทันที';
    case 'MISFIRE':
      return 'พิมพ์ชดเชย';
    case 'AUTO_RESUME':
      return 'พิมพ์ต่ออัตโนมัติ';
    case 'REPRINT_FAILED':
      return 'พิมพ์ซ้ำใบค้าง';
    default:
      return triggerBy || '-';
  }
}

function formatTriggerSrcTh(src: string): string {
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
      return src || '-';
  }
}

function formatQueueStatusTh(status: string): string {
  switch (status) {
    case 'PRINTED':
      return 'พิมพ์ออกแล้ว';
    case 'FAILED':
      return 'ค้างพิมพ์';
    case 'SKIPPED_DUP':
      return 'พิมพ์ราคานี้ไปแล้ว';
    case 'READY':
      return 'รอพิมพ์';
    default:
      return status || '-';
  }
}

function runStatusBadgeClass(status: string): string {
  if (status === 'SUCCESS') return 'bg-emerald-100 text-emerald-800 border-emerald-200';
  if (status === 'PARTIAL' || status === 'FAILED' || status === 'SYNC_FAILED') {
    return 'bg-red-100 text-red-800 border-red-200';
  }
  if (status === 'RUNNING') return 'bg-amber-100 text-amber-800 border-amber-200';
  return 'bg-slate-100 text-slate-700 border-slate-200';
}

function isIssueStatus(status?: string | null): boolean {
  return status === 'PARTIAL' || status === 'FAILED' || status === 'SYNC_FAILED';
}

function formatTimeShort(ts?: string | null): string {
  if (!ts) return '-';
  const parts = String(ts).trim().split(/\s+/);
  if (parts.length >= 2) return parts[1];
  return ts;
}

export function AutoScheduleHistory({
  onLoadManualRows,
  onOpenAutoPrintTab,
  readOnly = false,
}: AutoScheduleHistoryProps) {
  const [runs, setRuns] = useState<SchedRunRecord[]>([]);
  const [latestRun, setLatestRun] = useState<SchedRunRecord | null>(null);
  const [queueItems, setQueueItems] = useState<SchedQueueRecord[]>([]);
  const [selectedRunId, setSelectedRunId] = useState<string>('');
  const selectedRunIdRef = useRef<string>('');
  selectedRunIdRef.current = selectedRunId;

  const [loading, setLoading] = useState<boolean>(true);
  const [loadingDetail, setLoadingDetail] = useState<boolean>(false);
  const [reprintingRunId, setReprintingRunId] = useState<string | null>(null);
  const [retryingRunId, setRetryingRunId] = useState<string | null>(null);
  const [loadingManualRunId, setLoadingManualRunId] = useState<string | null>(null);

  // Run list filters & pagination
  const [runSearch, setRunSearch] = useState('');
  const [runStatusFilter, setRunStatusFilter] = useState<'all' | 'SUCCESS' | 'ISSUES' | 'SKIPPED_EMPTY'>('all');
  const [runTriggerFilter, setRunTriggerFilter] = useState<string>('all');
  const [runPage, setRunPage] = useState(1);
  const runPageSize = 10;

  // Detail item filters & pagination
  const [itemSearch, setItemSearch] = useState('');
  const [itemStatusFilter, setItemStatusFilter] = useState<'all' | 'PRINTED' | 'FAILED' | 'SKIPPED_DUP' | 'READY'>('all');
  const [itemPriFilter, setItemPriFilter] = useState<'all' | '1' | '2'>('all');
  const [itemPage, setItemPage] = useState(1);
  const itemPageSize = 15;

  const fetchRuns = useCallback(async (focusRunId?: string, isDetailSwitch = false) => {
    if (isDetailSwitch) setLoadingDetail(true);
    else setLoading(true);
    try {
      const targetId = focusRunId !== undefined ? focusRunId : selectedRunIdRef.current;
      const qs = targetId
        ? `limit=100&runId=${encodeURIComponent(targetId)}`
        : 'limit=100';
      const res = await fetch(`/api/schedules/runs?${qs}`);
      if (!res.ok) return;
      const data = await res.json();
      const nextRuns: SchedRunRecord[] = Array.isArray(data?.runs) ? data.runs : [];
      const nextLatest: SchedRunRecord | null = data?.latestRun || nextRuns[0] || null;
      setRuns(nextRuns);
      setLatestRun(nextLatest);
      setQueueItems(Array.isArray(data?.queueItems) ? data.queueItems : []);

      if (focusRunId !== undefined) {
        setSelectedRunId(focusRunId);
      } else if (!selectedRunIdRef.current && nextLatest?.FTLsrRunID) {
        setSelectedRunId(nextLatest.FTLsrRunID);
      } else if (
        selectedRunIdRef.current &&
        nextRuns.length > 0 &&
        !nextRuns.some((r) => r.FTLsrRunID === selectedRunIdRef.current)
      ) {
        setSelectedRunId(nextRuns[0].FTLsrRunID);
      }
    } catch {
      // ignore transient network errors
    } finally {
      if (isDetailSwitch) setLoadingDetail(false);
      else setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchRuns('');
  }, [fetchRuns]);

  const selectedRun = useMemo(
    () => runs.find((r) => r.FTLsrRunID === selectedRunId) || latestRun,
    [runs, selectedRunId, latestRun]
  );

  const filteredRuns = useMemo(() => {
    const q = runSearch.trim().toLowerCase();
    return runs.filter((r) => {
      if (runStatusFilter === 'SUCCESS' && r.FTLsrStatus !== 'SUCCESS') return false;
      if (runStatusFilter === 'ISSUES' && !isIssueStatus(r.FTLsrStatus)) return false;
      if (runStatusFilter === 'SKIPPED_EMPTY' && r.FTLsrStatus !== 'SKIPPED_EMPTY') return false;
      if (runTriggerFilter !== 'all' && r.FTLsrTriggerBy !== runTriggerFilter) return false;
      if (!q) return true;
      return (
        r.FDLsrBizDate.toLowerCase().includes(q) ||
        r.FTLsrRunID.toLowerCase().includes(q) ||
        r.FTLsrTriggerBy.toLowerCase().includes(q) ||
        (r.FTLsrErrMsg || '').toLowerCase().includes(q)
      );
    });
  }, [runs, runSearch, runStatusFilter, runTriggerFilter]);

  useEffect(() => {
    setRunPage(1);
  }, [runSearch, runStatusFilter, runTriggerFilter]);

  const totalRunPages = Math.max(1, Math.ceil(filteredRuns.length / runPageSize));
  const safeRunPage = Math.min(Math.max(1, runPage), totalRunPages);
  const pagedRuns = useMemo(() => {
    const start = (safeRunPage - 1) * runPageSize;
    return filteredRuns.slice(start, start + runPageSize);
  }, [filteredRuns, safeRunPage]);

  const filteredItems = useMemo(() => {
    const q = itemSearch.trim().toLowerCase();
    return queueItems.filter((item) => {
      if (itemStatusFilter !== 'all' && item.FTLsqStatus !== itemStatusFilter) return false;
      if (itemPriFilter !== 'all' && item.FTLsqPriType !== itemPriFilter) return false;
      if (!q) return true;
      return (
        item.FTBarCode.toLowerCase().includes(q) ||
        item.FTPdtCode.toLowerCase().includes(q) ||
        item.FTLbsCode.toLowerCase().includes(q) ||
        (item.FTLsqPromoDocNo || '').toLowerCase().includes(q) ||
        (item.FTLsqErrMsg || '').toLowerCase().includes(q)
      );
    });
  }, [queueItems, itemSearch, itemStatusFilter, itemPriFilter]);

  useEffect(() => {
    setItemPage(1);
  }, [selectedRunId, itemSearch, itemStatusFilter, itemPriFilter]);

  const totalItemPages = Math.max(1, Math.ceil(filteredItems.length / itemPageSize));
  const safeItemPage = Math.min(Math.max(1, itemPage), totalItemPages);
  const pagedItems = useMemo(() => {
    const start = (safeItemPage - 1) * itemPageSize;
    return filteredItems.slice(start, start + itemPageSize);
  }, [filteredItems, safeItemPage]);

  const handleSelectRun = (runId: string) => {
    setSelectedRunId(runId);
    fetchRuns(runId, true);
  };

  const handleReprintFailed = async (runId: string) => {
    setReprintingRunId(runId);
    try {
      const res = await fetch(`/api/schedules/runs/${encodeURIComponent(runId)}/reprint-failed`, {
        method: 'POST',
      });
      const data = await res.json();
      if (!res.ok) {
        toast.error(data?.error || 'พิมพ์ซ้ำรายการที่ค้างไม่สำเร็จ');
      } else if (data.ok) {
        toast.success(`พิมพ์ซ้ำเฉพาะรายการที่ค้างสำเร็จ (${data.norQty + data.pmtQty} รายการ)`);
      } else {
        toast.warning(`ยังค้างพิมพ์อีก ${data.failQty} รายการ (${data.errMsg || 'กรุณาตรวจเช็กเครื่องพิมพ์'})`);
      }
      await fetchRuns(runId);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setReprintingRunId(null);
    }
  };

  const handleRetrySyncRun = async (run: SchedRunRecord) => {
    setRetryingRunId(run.FTLsrRunID);
    try {
      const code = run.FTLscCode || 'SCH-0400';
      const res = await fetch(`/api/schedules/${encodeURIComponent(code)}/trigger`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode: 'PRINT_NOW', bizDate: run.FDLsrBizDate }),
      });
      const data = await res.json();
      if (!res.ok || data?.status === 'SYNC_FAILED' || data?.status === 'FAILED') {
        toast.error(data?.errMsg || data?.error || 'ลองซิงค์และพิมพ์ใหม่ยังไม่สำเร็จ');
      } else if (data?.status === 'SKIPPED_EMPTY') {
        toast.info('ไม่มีรายการเปลี่ยนแปลงสำหรับรอบนี้');
      } else {
        toast.success(`สั่งพิมพ์ใหม่เรียบร้อย: ปกติ ${data.norQty} | โปรฯ ${data.pmtQty}`);
      }
      await fetchRuns('');
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setRetryingRunId(null);
    }
  };

  const handleLoadToManual = async (runId: string) => {
    if (!onLoadManualRows) return;
    setLoadingManualRunId(runId);
    try {
      const res = await fetch(`/api/schedules/runs/${encodeURIComponent(runId)}/load-to-manual`, {
        method: 'POST',
      });
      const data = await res.json();
      if (!res.ok) {
        toast.error(data?.error || 'ดึงรายการที่ค้างไม่สำเร็จ');
        return;
      }
      const lines: ManualLoadLine[] = data?.lines || [];
      if (lines.length === 0) {
        toast.info('ไม่มีรายการที่ค้างพิมพ์ในรอบนี้');
        return;
      }
      onLoadManualRows(lines);
      toast.success(`นำเข้า ${lines.length} รายการที่ค้างไปที่หน้าพิมพ์หลักเรียบร้อยแล้ว`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setLoadingManualRunId(null);
    }
  };

  return (
    <div className="space-y-4">
      <Card className="border-slate-200 shadow-xs bg-white py-0 gap-0">
        <CardHeader className="bg-[#3d4a57] text-white py-2.5 px-4 flex flex-row flex-wrap items-center justify-between gap-2">
          <div>
            <CardTitle className="text-sm font-semibold flex items-center gap-2">
              <History className="h-4 w-4 text-teal-300" />
              <span>ประวัติการพิมพ์ป้ายอัตโนมัติ</span>
            </CardTitle>
            <p className="text-[11px] text-slate-300 mt-0.5">
              สรุปผลรอบล่าสุด รายการย้อนหลังแต่ละวัน รายละเอียดสินค้าที่ถูกคัดมาพิมพ์ และจัดการพิมพ์ซ้ำรายการที่ค้าง
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            {!readOnly && onOpenAutoPrintTab && (
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="h-8 text-xs bg-white/10 text-white border-white/25 hover:bg-white/20 hover:text-white"
                onClick={onOpenAutoPrintTab}
              >
                <Settings className="h-3.5 w-3.5 mr-1.5" />
                ตั้งค่าพิมพ์อัตโนมัติ
              </Button>
            )}
            <Button
              type="button"
              size="sm"
              className="h-8 text-xs bg-[#007a87] hover:bg-[#00606b] text-white"
              disabled={loading}
              onClick={() => fetchRuns()}
            >
              <RefreshCw className={`h-3.5 w-3.5 mr-1.5 ${loading ? 'animate-spin' : ''}`} />
              รีเฟรชประวัติ
            </Button>
          </div>
        </CardHeader>

        <CardContent className="p-4 space-y-4">
          {/* Top Highlight: Latest Run */}
          {latestRun ? (
            <div
              className={`rounded-md border px-3.5 py-2.5 text-xs flex flex-wrap items-center justify-between gap-2 ${
                isIssueStatus(latestRun.FTLsrStatus)
                  ? 'bg-red-50 border-red-200 text-red-900'
                  : 'bg-emerald-50/70 border-emerald-200 text-emerald-900'
              }`}
            >
              <div className="space-y-0.5 min-w-0">
                <div className="font-semibold flex flex-wrap items-center gap-2">
                  {isIssueStatus(latestRun.FTLsrStatus) ? (
                    <AlertTriangle className="h-4 w-4 text-red-600 shrink-0" />
                  ) : (
                    <CheckCircle2 className="h-4 w-4 text-emerald-600 shrink-0" />
                  )}
                  <span>
                    ผลรอบล่าสุด ({latestRun.FDLsrBizDate} • {formatTriggerByTh(latestRun.FTLsrTriggerBy)}):
                  </span>
                  <Badge variant="outline" className="bg-white text-[10px]">
                    {formatRunStatusTh(latestRun.FTLsrStatus)}
                  </Badge>
                </div>
                <div className="text-[11px] text-slate-700 pl-6">
                  ปกติ <b>{latestRun.FNLsrNorQty}</b> ใบ | โปรฯ <b>{latestRun.FNLsrPmtQty}</b> ใบ | ข้ามเดิม{' '}
                  <b>{latestRun.FNLsrSkippedDup}</b> | ค้าง{' '}
                  <b className="text-red-700">{latestRun.FNLsrFailQty}</b> ใบ
                  {latestRun.FDLsrFinishedAt ? ` • เสร็จเมื่อ ${latestRun.FDLsrFinishedAt}` : ''}
                </div>
                {latestRun.FTLsrErrMsg && (
                  <div className="text-[11px] text-red-700 font-medium break-all pl-6">
                    สาเหตุ: {latestRun.FTLsrErrMsg}
                  </div>
                )}
              </div>

              {!readOnly && (
                <div className="flex flex-wrap items-center gap-1.5">
                  {latestRun.FNLsrFailQty > 0 && (
                    <Button
                      type="button"
                      size="sm"
                      className="h-7 text-[11px] bg-red-600 hover:bg-red-700 text-white"
                      disabled={reprintingRunId === latestRun.FTLsrRunID}
                      onClick={() => handleReprintFailed(latestRun.FTLsrRunID)}
                    >
                      {reprintingRunId === latestRun.FTLsrRunID ? (
                        <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                      ) : (
                        <RotateCcw className="h-3 w-3 mr-1" />
                      )}
                      พิมพ์ซ้ำเฉพาะที่ค้าง ({latestRun.FNLsrFailQty})
                    </Button>
                  )}
                  {latestRun.FTLsrStatus === 'SYNC_FAILED' && (
                    <Button
                      type="button"
                      size="sm"
                      className="h-7 text-[11px] bg-red-600 hover:bg-red-700 text-white"
                      disabled={retryingRunId === latestRun.FTLsrRunID}
                      onClick={() => handleRetrySyncRun(latestRun)}
                    >
                      {retryingRunId === latestRun.FTLsrRunID ? (
                        <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                      ) : (
                        <RotateCcw className="h-3 w-3 mr-1" />
                      )}
                      ลองซิงค์และพิมพ์ใหม่
                    </Button>
                  )}
                </div>
              )}
            </div>
          ) : (
            <div className="rounded-md border border-slate-200 bg-slate-50 px-3.5 py-3 text-xs text-slate-600 flex items-center gap-2">
              <Clock className="h-4 w-4 text-slate-400 shrink-0" />
              <span>ยังไม่มีประวัติการพิมพ์อัตโนมัติ (เมื่อถึงเวลาที่ตั้งไว้หรือกด &ldquo;สั่งพิมพ์รอบนี้ทันที&rdquo; ประวัติจะแสดงที่นี่)</span>
            </div>
          )}

          {/* Master-Detail Split */}
          <div className="grid grid-cols-1 xl:grid-cols-12 gap-4 items-start">
            {/* Left Panel: Runs List */}
            <div className="xl:col-span-5 border border-slate-200 rounded-md overflow-hidden bg-white">
              <div className="bg-slate-50 border-b border-slate-200 p-3 space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-xs font-semibold text-slate-800">
                    รอบการทำงานย้อนหลัง ({filteredRuns.length})
                  </span>
                </div>
                <div className="flex flex-wrap gap-2">
                  <div className="relative flex-1 min-w-[140px]">
                    <Search className="h-3.5 w-3.5 text-slate-400 absolute left-2.5 top-2" />
                    <Input
                      value={runSearch}
                      onChange={(e) => setRunSearch(e.target.value)}
                      placeholder="ค้นวันที่ / Run ID / ข้อความ..."
                      className="h-8 pl-8 text-xs bg-white"
                    />
                  </div>
                  <Select
                    value={runStatusFilter}
                    onValueChange={(v) => setRunStatusFilter(v as typeof runStatusFilter)}
                  >
                    <SelectTrigger className="h-8 w-[135px] text-xs bg-white">
                      <SelectValue placeholder="สถานะ" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">ทุกสถานะ</SelectItem>
                      <SelectItem value="SUCCESS">พิมพ์ครบเรียบร้อย</SelectItem>
                      <SelectItem value="ISSUES">มีปัญหา / ค้างพิมพ์</SelectItem>
                      <SelectItem value="SKIPPED_EMPTY">ไม่มีรายการเปลี่ยน</SelectItem>
                    </SelectContent>
                  </Select>
                  <Select value={runTriggerFilter} onValueChange={setRunTriggerFilter}>
                    <SelectTrigger className="h-8 w-[125px] text-xs bg-white">
                      <SelectValue placeholder="รูปแบบ" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">ทุกรูปแบบ</SelectItem>
                      <SelectItem value="TIMER">ตั้งเวลา (TIMER)</SelectItem>
                      <SelectItem value="PRINT_NOW">สั่งพิมพ์ทันที</SelectItem>
                      <SelectItem value="MISFIRE">พิมพ์ชดเชย</SelectItem>
                      <SelectItem value="REPRINT_FAILED">พิมพ์ซ้ำใบค้าง</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow className="bg-slate-50/70">
                      <TableHead className="text-[11px]">วันที่ / เวลา</TableHead>
                      <TableHead className="text-[11px]">รูปแบบ / สถานะ</TableHead>
                      <TableHead className="text-[11px] text-right">ปกติ / โปร / ค้าง</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {loading && runs.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={3} className="text-center py-8 text-xs text-slate-400">
                          <Loader2 className="h-4 w-4 animate-spin inline mr-1.5" />
                          กำลังโหลดประวัติ...
                        </TableCell>
                      </TableRow>
                    ) : pagedRuns.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={3} className="text-center py-8 text-xs text-slate-400">
                          ไม่พบรอบการทำงานที่ตรงกับเงื่อนไข
                        </TableCell>
                      </TableRow>
                    ) : (
                      pagedRuns.map((r) => {
                        const isSelected = r.FTLsrRunID === selectedRun?.FTLsrRunID;
                        return (
                          <TableRow
                            key={r.FTLsrRunID}
                            onClick={() => handleSelectRun(r.FTLsrRunID)}
                            className={`cursor-pointer transition-colors ${
                              isSelected
                                ? 'bg-teal-50/80 hover:bg-teal-50 border-l-2 border-l-[#007a87]'
                                : 'hover:bg-slate-50'
                            }`}
                          >
                            <TableCell className="text-xs align-top">
                              <div className="font-mono font-semibold text-slate-800">
                                {r.FDLsrBizDate}
                              </div>
                              <div className="text-[10px] text-slate-500 font-mono">
                                {formatTimeShort(r.FDLsrStartedAt)}
                                {r.FDLsrFinishedAt ? ` → ${formatTimeShort(r.FDLsrFinishedAt)}` : ''}
                              </div>
                            </TableCell>
                            <TableCell className="text-xs align-top">
                              <div className="text-[11px] font-medium text-slate-700">
                                {formatTriggerByTh(r.FTLsrTriggerBy)}
                              </div>
                              <Badge
                                variant="outline"
                                className={`mt-0.5 text-[10px] ${runStatusBadgeClass(r.FTLsrStatus)}`}
                              >
                                {formatRunStatusTh(r.FTLsrStatus)}
                              </Badge>
                            </TableCell>
                            <TableCell className="text-xs text-right align-top font-mono">
                              <div>
                                <span className="text-emerald-700 font-semibold">{r.FNLsrNorQty}</span>
                                {' / '}
                                <span className="text-red-700 font-semibold">{r.FNLsrPmtQty}</span>
                                {' / '}
                                <span className={r.FNLsrFailQty > 0 ? 'text-red-600 font-bold' : 'text-slate-400'}>
                                  {r.FNLsrFailQty}
                                </span>
                              </div>
                              {r.FNLsrSkippedDup > 0 && (
                                <div className="text-[10px] text-slate-400">
                                  ข้ามเดิม {r.FNLsrSkippedDup}
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

              {filteredRuns.length > runPageSize && (
                <div className="border-t border-slate-200 px-3 py-2 flex items-center justify-between text-xs text-slate-600 bg-slate-50/50">
                  <span>
                    หน้า {safeRunPage} / {totalRunPages}
                  </span>
                  <div className="flex items-center gap-1">
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      className="h-7 px-2 text-xs bg-white"
                      disabled={safeRunPage <= 1}
                      onClick={() => setRunPage((p) => p - 1)}
                    >
                      <ChevronLeft className="h-3.5 w-3.5" />
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      className="h-7 px-2 text-xs bg-white"
                      disabled={safeRunPage >= totalRunPages}
                      onClick={() => setRunPage((p) => p + 1)}
                    >
                      <ChevronRight className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </div>
              )}
            </div>

            {/* Right Panel: Selected Run Details & Queue Items */}
            <div className="xl:col-span-7 border border-slate-200 rounded-md overflow-hidden bg-white">
              <div className="bg-slate-50 border-b border-slate-200 p-3 space-y-2.5">
                {selectedRun ? (
                  <>
                    <div className="flex flex-wrap items-start justify-between gap-2">
                      <div>
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="text-xs font-semibold text-slate-900">
                            รายการสินค้าในรอบ {selectedRun.FDLsrBizDate}
                          </span>
                          <Badge
                            variant="outline"
                            className={`text-[10px] ${runStatusBadgeClass(selectedRun.FTLsrStatus)}`}
                          >
                            {formatRunStatusTh(selectedRun.FTLsrStatus)}
                          </Badge>
                          <span className="text-[11px] text-slate-500 font-mono">
                            #{selectedRun.FTLsrRunID}
                          </span>
                        </div>
                        <div className="text-[11px] text-slate-500 mt-0.5">
                          รูปแบบ: {formatTriggerByTh(selectedRun.FTLsrTriggerBy)} &bull; เริ่ม{' '}
                          <span className="font-mono">{selectedRun.FDLsrStartedAt}</span>
                          {selectedRun.FDLsrFinishedAt ? (
                            <>
                              {' '}
                              &bull; เสร็จ <span className="font-mono">{selectedRun.FDLsrFinishedAt}</span>
                            </>
                          ) : null}
                        </div>
                      </div>

                      {!readOnly && (
                        <div className="flex flex-wrap items-center gap-1.5">
                          {selectedRun.FNLsrFailQty > 0 && (
                            <>
                              <Button
                                type="button"
                                size="sm"
                                className="h-7 px-2.5 text-[11px] bg-red-600 hover:bg-red-700 text-white"
                                disabled={reprintingRunId === selectedRun.FTLsrRunID}
                                onClick={() => handleReprintFailed(selectedRun.FTLsrRunID)}
                              >
                                {reprintingRunId === selectedRun.FTLsrRunID ? (
                                  <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                                ) : (
                                  <RotateCcw className="h-3 w-3 mr-1" />
                                )}
                                พิมพ์ซ้ำเฉพาะที่ค้าง ({selectedRun.FNLsrFailQty})
                              </Button>
                              {onLoadManualRows && (
                                <Button
                                  type="button"
                                  size="sm"
                                  variant="outline"
                                  className="h-7 px-2.5 text-[11px] border-red-300 text-red-800 bg-white hover:bg-red-50"
                                  disabled={loadingManualRunId === selectedRun.FTLsrRunID}
                                  onClick={() => handleLoadToManual(selectedRun.FTLsrRunID)}
                                >
                                  {loadingManualRunId === selectedRun.FTLsrRunID ? (
                                    <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                                  ) : (
                                    <FileDown className="h-3 w-3 mr-1" />
                                  )}
                                  ส่งไปหน้าพิมพ์หลัก
                                </Button>
                              )}
                            </>
                          )}
                          {selectedRun.FTLsrStatus === 'SYNC_FAILED' && (
                            <Button
                              type="button"
                              size="sm"
                              className="h-7 px-2.5 text-[11px] bg-red-600 hover:bg-red-700 text-white"
                              disabled={retryingRunId === selectedRun.FTLsrRunID}
                              onClick={() => handleRetrySyncRun(selectedRun)}
                            >
                              {retryingRunId === selectedRun.FTLsrRunID ? (
                                <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                              ) : (
                                <RotateCcw className="h-3 w-3 mr-1" />
                              )}
                              ลองซิงค์และพิมพ์ใหม่
                            </Button>
                          )}
                        </div>
                      )}
                    </div>

                    {selectedRun.FTLsrErrMsg && (
                      <div className="rounded border border-red-200 bg-red-50 px-2.5 py-1.5 text-[11px] text-red-700 break-all">
                        สาเหตุ: {selectedRun.FTLsrErrMsg}
                      </div>
                    )}

                    <div className="flex flex-wrap gap-1.5 text-[11px]">
                      <Badge variant="outline" className="bg-white text-slate-700 font-normal">
                        คัดมารวม <b>{selectedRun.FNLsrCandTotal}</b>
                      </Badge>
                      <Badge variant="outline" className="bg-white border-emerald-200 text-emerald-800 font-normal">
                        ป้ายปกติ <b>{selectedRun.FNLsrNorQty}</b> ใบ
                      </Badge>
                      <Badge variant="outline" className="bg-white border-red-200 text-red-700 font-normal">
                        ป้ายโปรฯ <b>{selectedRun.FNLsrPmtQty}</b> ใบ
                      </Badge>
                      <Badge variant="outline" className="bg-white text-slate-600 font-normal">
                        ข้ามรายการเดิม <b>{selectedRun.FNLsrSkippedDup}</b>
                      </Badge>
                      <Badge variant="outline" className="bg-white text-red-700 font-normal">
                        ค้างพิมพ์ <b>{selectedRun.FNLsrFailQty}</b> ใบ
                      </Badge>
                    </div>

                    <div className="flex flex-wrap gap-2">
                      <div className="relative flex-1 min-w-[160px]">
                        <Search className="h-3.5 w-3.5 text-slate-400 absolute left-2.5 top-2" />
                        <Input
                          value={itemSearch}
                          onChange={(e) => setItemSearch(e.target.value)}
                          placeholder="ค้นหาบาร์โค้ด / รหัสสินค้า / เลขโปรฯ / ขนาด..."
                          className="h-8 pl-8 text-xs bg-white"
                        />
                      </div>
                      <Select
                        value={itemPriFilter}
                        onValueChange={(v) => setItemPriFilter(v as typeof itemPriFilter)}
                      >
                        <SelectTrigger className="h-8 w-[125px] text-xs bg-white">
                          <SelectValue placeholder="ประเภทป้าย" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="all">ทุกประเภทป้าย</SelectItem>
                          <SelectItem value="1">ป้ายปกติ</SelectItem>
                          <SelectItem value="2">ป้ายโปรโมชัน</SelectItem>
                        </SelectContent>
                      </Select>
                      <Select
                        value={itemStatusFilter}
                        onValueChange={(v) => setItemStatusFilter(v as typeof itemStatusFilter)}
                      >
                        <SelectTrigger className="h-8 w-[135px] text-xs bg-white">
                          <SelectValue placeholder="ผลลัพธ์" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="all">ทุกผลลัพธ์</SelectItem>
                          <SelectItem value="PRINTED">พิมพ์ออกแล้ว</SelectItem>
                          <SelectItem value="FAILED">ค้างพิมพ์</SelectItem>
                          <SelectItem value="SKIPPED_DUP">พิมพ์ราคานี้ไปแล้ว</SelectItem>
                          <SelectItem value="READY">รอพิมพ์</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </>
                ) : (
                  <div className="text-xs text-slate-500 py-1">
                    เลือกรอบการทำงานทางด้านซ้ายเพื่อดูรายละเอียดรายการสินค้า
                  </div>
                )}
              </div>

              <div className="overflow-x-auto">
                <Table>
                  <TableHeader>
                    <TableRow className="bg-slate-50/70">
                      <TableHead className="text-[11px] w-11">ลำดับ</TableHead>
                      <TableHead className="text-[11px]">บาร์โค้ด / รหัสสินค้า</TableHead>
                      <TableHead className="text-[11px]">สาเหตุที่พิมพ์</TableHead>
                      <TableHead className="text-[11px]">ประเภทป้าย</TableHead>
                      <TableHead className="text-[11px] text-right">ราคาขาย</TableHead>
                      <TableHead className="text-[11px] text-center">แผ่นที่</TableHead>
                      <TableHead className="text-[11px]">ผลลัพธ์</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {loadingDetail ? (
                      <TableRow>
                        <TableCell colSpan={7} className="text-center py-8 text-xs text-slate-400">
                          <Loader2 className="h-4 w-4 animate-spin inline mr-1.5" />
                          กำลังโหลดรายการสินค้า...
                        </TableCell>
                      </TableRow>
                    ) : pagedItems.length === 0 ? (
                      <TableRow>
                        <TableCell colSpan={7} className="text-center py-8 text-xs text-slate-400">
                          ไม่มีรายการสินค้าในรอบนี้
                        </TableCell>
                      </TableRow>
                    ) : (
                      pagedItems.map((q) => (
                        <TableRow key={`${q.FTLsrRunID}-${q.FNLsqSeq}`}>
                          <TableCell className="text-xs font-mono">{q.FNLsqSeq}</TableCell>
                          <TableCell className="text-xs">
                            <div className="font-mono font-medium text-slate-800">{q.FTBarCode}</div>
                            <div className="text-[11px] text-slate-500">
                              รหัส: {q.FTPdtCode} &bull; {q.FTLbsCode}
                              {q.FTLsqPromoDocNo ? ` • โปรฯ ${q.FTLsqPromoDocNo}` : ''}
                            </div>
                          </TableCell>
                          <TableCell className="text-xs">{formatTriggerSrcTh(q.FTLsqTriggerSrc)}</TableCell>
                          <TableCell className="text-xs">
                            {q.FTLsqPriType === '2' ? (
                              <Badge className="bg-red-100 text-red-800 border-red-200 text-[10px]">
                                ป้ายโปรโมชัน
                              </Badge>
                            ) : (
                              <Badge variant="secondary" className="text-[10px]">
                                ป้ายปกติ
                              </Badge>
                            )}
                          </TableCell>
                          <TableCell className="text-xs text-right font-mono">
                            <div className="font-semibold text-slate-800">
                              {q.FCLsqSalePrice.toFixed(2)} ฿
                            </div>
                            {q.FCLsqOldPrice != null &&
                              Math.abs(q.FCLsqOldPrice - q.FCLsqSalePrice) > 0.001 && (
                                <div className="text-[10px] text-slate-400 line-through">
                                  {q.FCLsqOldPrice.toFixed(2)} ฿
                                </div>
                              )}
                            {q.FNLsqQty > 1 && (
                              <div className="text-[10px] text-[#007a87] font-semibold">
                                x{q.FNLsqQty} ดวง
                              </div>
                            )}
                          </TableCell>
                          <TableCell className="text-xs text-center font-mono">
                            {q.FNLsqBatchPage ?? '-'}
                          </TableCell>
                          <TableCell className="text-xs">
                            <Badge
                              className={`text-[10px] ${
                                q.FTLsqStatus === 'PRINTED'
                                  ? 'bg-emerald-100 text-emerald-800'
                                  : q.FTLsqStatus === 'FAILED'
                                    ? 'bg-red-100 text-red-800'
                                    : 'bg-slate-100 text-slate-600'
                              }`}
                            >
                              {formatQueueStatusTh(q.FTLsqStatus)}
                            </Badge>
                            {q.FTLsqErrMsg && (
                              <div className="text-[10px] text-red-600 mt-0.5 max-w-[200px] break-words">
                                {q.FTLsqErrMsg}
                              </div>
                            )}
                          </TableCell>
                        </TableRow>
                      ))
                    )}
                  </TableBody>
                </Table>
              </div>

              {filteredItems.length > itemPageSize && (
                <div className="border-t border-slate-200 px-3 py-2 flex items-center justify-between text-xs text-slate-600 bg-slate-50/50">
                  <span>
                    แสดง {(safeItemPage - 1) * itemPageSize + 1}–
                    {Math.min(filteredItems.length, safeItemPage * itemPageSize)} จาก {filteredItems.length} รายการ
                  </span>
                  <div className="flex items-center gap-1">
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      className="h-7 px-2 text-xs bg-white"
                      disabled={safeItemPage <= 1}
                      onClick={() => setItemPage((p) => p - 1)}
                    >
                      <ChevronLeft className="h-3.5 w-3.5" />
                    </Button>
                    <span className="px-2 font-mono text-[11px]">
                      {safeItemPage} / {totalItemPages}
                    </span>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      className="h-7 px-2 text-xs bg-white"
                      disabled={safeItemPage >= totalItemPages}
                      onClick={() => setItemPage((p) => p + 1)}
                    >
                      <ChevronRight className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                </div>
              )}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
