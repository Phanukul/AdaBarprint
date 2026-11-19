import React, { useCallback, useEffect, useRef, useState } from 'react';
import { toast } from 'sonner';
import {
  AlertTriangle,
  CheckCircle2,
  Clock,
  Eye,
  FileDown,
  Loader2,
  RefreshCw,
  RotateCcw,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

export interface SchedRunRecord {
  FTLsrRunID: string;
  FTLscCode: string;
  FDLsrBizDate: string;
  FTLsrTriggerBy: string;
  FTLsrStatus: string;
  FDLsrStartedAt: string;
  FDLsrSyncedAt?: string | null;
  FNLsrSyncAttempts: number;
  FNLsrCandTotal: number;
  FNLsrSkippedDup: number;
  FNLsrNorQty: number;
  FNLsrPmtQty: number;
  FNLsrFailQty: number;
  FTLjbJobID?: string | null;
  FTLsrErrMsg?: string | null;
  FDLsrFinishedAt?: string | null;
}

export interface SchedQueueRecord {
  FTLsrRunID: string;
  FNLsqSeq: number;
  FTPdtCode: string;
  FTBarCode: string;
  FTLsqTriggerSrc: string;
  FTLsqPriType: string;
  FTLbsCode: string;
  FCLsqOldPrice?: number | null;
  FCLsqSalePrice: number;
  FTLsqPromoDocNo?: string | null;
  FNLsqQty: number;
  FNLsqBatchPage?: number | null;
  FTLsqStatus: string;
  FTLsqErrMsg?: string | null;
}

export interface ManualLoadLine {
  pdtCode: string;
  barCode: string;
  pdtName: string;
  price: number;
  priType: '1' | '2';
  qty: number;
  lbsCode: string;
  tmplCode: string;
  promoName?: string | null;
  statusText: string;
  ctx: Record<string, any>;
}

interface MorningOpsBannerProps {
  onLoadManualRows: (lines: ManualLoadLine[]) => void;
  onOpenScheduleSettings: () => void;
}

function formatTriggerLabel(src: string): string {
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

function formatRunStatusLabel(status: string): string {
  switch (status) {
    case 'SUCCESS':
      return 'พิมพ์ครบเรียบร้อย';
    case 'PARTIAL':
      return 'พิมพ์ได้บางส่วน (เครื่องพิมพ์ขัดข้อง)';
    case 'FAILED':
      return 'พิมพ์ไม่สำเร็จ';
    case 'SYNC_FAILED':
      return 'ดึงข้อมูลจากส่วนกลางไม่ได้';
    case 'SKIPPED_EMPTY':
      return 'วันนี้ไม่มีรายการเปลี่ยน';
    case 'RUNNING':
      return 'กำลังทำงาน';
    default:
      return status;
  }
}

function formatQueueStatusLabel(status: string): string {
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
      return status;
  }
}

export function MorningOpsBanner({
  onLoadManualRows,
  onOpenScheduleSettings,
}: MorningOpsBannerProps) {
  const [runs, setRuns] = useState<SchedRunRecord[]>([]);
  const [latestRun, setLatestRun] = useState<SchedRunRecord | null>(null);
  const [queueItems, setQueueItems] = useState<SchedQueueRecord[]>([]);
  const [inspectedRunId, setInspectedRunId] = useState<string>('');
  const inspectedRunIdRef = useRef<string>('');
  inspectedRunIdRef.current = inspectedRunId;
  const [schedActive, setSchedActive] = useState<boolean>(false);
  const [schedTime, setSchedTime] = useState<string>('04:00');
  const [loading, setLoading] = useState<boolean>(false);
  const [reprinting, setReprinting] = useState<boolean>(false);
  const [retryingRun, setRetryingRun] = useState<boolean>(false);
  const [loadingManual, setLoadingManual] = useState<boolean>(false);
  const [detailOpen, setDetailOpen] = useState<boolean>(false);
  const detailOpenRef = useRef<boolean>(false);
  detailOpenRef.current = detailOpen;

  const fetchStatus = useCallback(async (focusRunId?: string) => {
    try {
      const targetId =
        focusRunId !== undefined
          ? focusRunId
          : detailOpenRef.current
            ? inspectedRunIdRef.current
            : '';
      const runQs = targetId ? `limit=10&runId=${encodeURIComponent(targetId)}` : 'limit=10';
      const [sRes, rRes] = await Promise.all([
        fetch('/api/schedules'),
        fetch(`/api/schedules/runs?${runQs}`),
      ]);
      if (sRes.ok) {
        const sData = await sRes.json();
        const firstSched = sData?.schedules?.[0];
        if (firstSched) {
          setSchedActive(firstSched.FTLscStaUse === '1');
          setSchedTime(firstSched.FTLscRunTime || '04:00');
        }
      }
      if (rRes.ok) {
        const rData = await rRes.json();
        setRuns(Array.isArray(rData?.runs) ? rData.runs : []);
        setLatestRun(rData?.latestRun || null);
        setQueueItems(rData?.queueItems || []);
        if (!targetId && rData?.latestRun?.FTLsrRunID) {
          setInspectedRunId(rData.latestRun.FTLsrRunID);
        }
      }
    } catch {
      // Ignore silent polling errors while backend restarts
    }
  }, []);

  useEffect(() => {
    fetchStatus('');
    const timer = window.setInterval(() => fetchStatus(), 30000);
    return () => window.clearInterval(timer);
  }, [fetchStatus]);

  const handleManualRefresh = async () => {
    setLoading(true);
    await fetchStatus();
    setLoading(false);
  };

  const handleRetryRun = async () => {
    setRetryingRun(true);
    try {
      const res = await fetch('/api/schedules/SCH-0400/trigger', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ mode: 'PRINT_NOW', bizDate: latestRun?.FDLsrBizDate }),
      });
      const data = await res.json();
      if (!res.ok || data?.status === 'SYNC_FAILED' || data?.status === 'FAILED') {
        toast.error(data?.errMsg || data?.error || 'ลองซิงค์และพิมพ์ใหม่ยังไม่สำเร็จ');
      } else if (data?.status === 'SKIPPED_EMPTY') {
        toast.info('ไม่มีรายการเปลี่ยนแปลงสำหรับรอบนี้');
      } else {
        toast.success(`สั่งพิมพ์ใหม่เรียบร้อย: ปกติ ${data.norQty} | โปรฯ ${data.pmtQty}`);
      }
      await fetchStatus('');
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setRetryingRun(false);
    }
  };

  const handleReprintFailed = async () => {
    if (!latestRun) return;
    setReprinting(true);
    try {
      const res = await fetch(`/api/schedules/runs/${encodeURIComponent(latestRun.FTLsrRunID)}/reprint-failed`, {
        method: 'POST',
      });
      const data = await res.json();
      if (!res.ok) {
        toast.error(data?.error || 'พิมพ์ซ้ำรายการที่ค้างไม่สำเร็จ');
      } else if (data.ok) {
        toast.success(`พิมพ์ซ้ำเฉพาะรายการที่ค้างสำเร็จ (${data.norQty + data.pmtQty} รายการ)`);
      } else {
        toast.warning(`ยังค้างพิมพ์อีก ${data.failQty} รายการ กรุณาตรวจเช็กเครื่องพิมพ์`);
      }
      await fetchStatus();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setReprinting(false);
    }
  };

  const handleLoadToManual = async () => {
    if (!latestRun) return;
    setLoadingManual(true);
    try {
      const res = await fetch(`/api/schedules/runs/${encodeURIComponent(latestRun.FTLsrRunID)}/load-to-manual`, {
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
      setDetailOpen(false);
      toast.success(`นำเข้า ${lines.length} รายการที่ค้างมาไว้ในตารางพิมพ์ด้านล่างแล้ว`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setLoadingManual(false);
    }
  };

  const hasIssue =
    latestRun &&
    (latestRun.FTLsrStatus === 'PARTIAL' ||
      latestRun.FTLsrStatus === 'FAILED' ||
      latestRun.FTLsrStatus === 'SYNC_FAILED');

  return (
    <>
      <div
        className={`rounded-lg border px-3.5 py-2.5 flex flex-wrap items-center justify-between gap-2 text-xs shadow-xs ${
          hasIssue
            ? 'bg-red-50 border-red-200 text-red-900'
            : latestRun
              ? 'bg-emerald-50/90 border-emerald-200 text-emerald-900'
              : 'bg-slate-50 border-slate-200 text-slate-700'
        }`}
      >
        <div className="flex flex-wrap items-center gap-2 min-w-0">
          {hasIssue ? (
            <AlertTriangle className="h-4 w-4 text-red-600 shrink-0" />
          ) : latestRun ? (
            <CheckCircle2 className="h-4 w-4 text-emerald-600 shrink-0" />
          ) : (
            <Clock className="h-4 w-4 text-[#007a87] shrink-0" />
          )}

          {!latestRun ? (
            <span>
              <strong>พิมพ์ป้ายอัตโนมัติ (เวลา {schedTime} น.):</strong>{' '}
              {schedActive ? (
                <Badge className="bg-emerald-600 text-white text-[10px]">เปิดทำงานอัตโนมัติ</Badge>
              ) : (
                <Badge variant="secondary" className="text-[10px]">ยังไม่เปิดใช้งาน</Badge>
              )}{' '}
              <span className="text-slate-500 ml-1">กด &ldquo;ตั้งค่าพิมพ์อัตโนมัติ&rdquo; ด้านขวาเพื่อเปิดใช้งาน</span>
            </span>
          ) : hasIssue ? (
            <div className="flex flex-wrap items-center gap-1.5">
              <strong>
                แจ้งเตือนรอบวันที่ {latestRun.FDLsrBizDate} &bull; {formatRunStatusLabel(latestRun.FTLsrStatus)}:
              </strong>
              {latestRun.FTLsrStatus === 'SYNC_FAILED' ? (
                <span>ไม่สามารถดึงราคาใหม่จากส่วนกลางได้ ระบบจึงหยุดพิมพ์เพื่อป้องกันราคาผิดพลาด</span>
              ) : (
                <span>
                  พิมพ์ออกแล้ว <b>{latestRun.FNLsrNorQty + latestRun.FNLsrPmtQty}</b> ใบ | ยังค้าง{' '}
                  <b className="text-red-700">{latestRun.FNLsrFailQty}</b> ใบ
                </span>
              )}
              {latestRun.FTLsrErrMsg && (
                <span className="text-[11px] text-red-700 font-medium">({latestRun.FTLsrErrMsg})</span>
              )}
            </div>
          ) : (
            <div className="flex flex-wrap items-center gap-2">
              <strong>สรุปพิมพ์อัตโนมัติรอบ {latestRun.FDLsrBizDate}:</strong>
              <Badge variant="outline" className="bg-white border-emerald-300 text-emerald-800 text-[10px]">
                ป้ายปกติ {latestRun.FNLsrNorQty} ใบ
              </Badge>
              <Badge variant="outline" className="bg-white border-red-200 text-red-700 text-[10px]">
                ป้ายโปรฯ {latestRun.FNLsrPmtQty} ใบ
              </Badge>
              <Badge variant="outline" className="bg-white border-slate-200 text-slate-600 text-[10px]">
                ข้ามรายการเดิม {latestRun.FNLsrSkippedDup}
              </Badge>
              <span className="text-[11px] text-emerald-700">({formatRunStatusLabel(latestRun.FTLsrStatus)})</span>
            </div>
          )}
        </div>

        <div className="flex flex-wrap items-center gap-1.5">
          {latestRun && latestRun.FNLsrFailQty > 0 && (
            <>
              <Button
                type="button"
                size="sm"
                className="h-7 px-2.5 text-[11px] bg-red-600 hover:bg-red-700 text-white"
                disabled={reprinting}
                onClick={handleReprintFailed}
              >
                {reprinting ? (
                  <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                ) : (
                  <RotateCcw className="h-3 w-3 mr-1" />
                )}
                พิมพ์ซ้ำเฉพาะใบที่ค้าง ({latestRun.FNLsrFailQty})
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="h-7 px-2.5 text-[11px] border-red-300 text-red-800 bg-white hover:bg-red-50"
                disabled={loadingManual}
                onClick={handleLoadToManual}
              >
                {loadingManual ? (
                  <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                ) : (
                  <FileDown className="h-3 w-3 mr-1" />
                )}
                ส่งไปพิมพ์แบบกำหนดเอง
              </Button>
            </>
          )}

          {latestRun && latestRun.FTLsrStatus === 'SYNC_FAILED' && (
            <Button
              type="button"
              size="sm"
              className="h-7 px-2.5 text-[11px] bg-red-600 hover:bg-red-700 text-white"
              disabled={retryingRun}
              onClick={handleRetryRun}
            >
              {retryingRun ? <Loader2 className="h-3 w-3 mr-1 animate-spin" /> : <RotateCcw className="h-3 w-3 mr-1" />}
              ลองซิงค์และพิมพ์ใหม่
            </Button>
          )}

          {latestRun && (
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="h-7 px-2.5 text-[11px] bg-white"
              onClick={() => setDetailOpen(true)}
            >
              <Eye className="h-3 w-3 mr-1" />
              ดูรายการทั้งหมด
            </Button>
          )}

          <Button
            type="button"
            size="sm"
            variant="ghost"
            className="h-7 px-2 text-[11px]"
            onClick={handleManualRefresh}
            title="อัปเดตสถานะล่าสุด"
          >
            <RefreshCw className={`h-3 w-3 ${loading ? 'animate-spin' : ''}`} />
          </Button>

          <Button
            type="button"
            size="sm"
            variant="link"
            className="h-7 px-1.5 text-[11px] text-[#007a87]"
            onClick={onOpenScheduleSettings}
          >
            ตั้งค่าพิมพ์อัตโนมัติ
          </Button>
        </div>
      </div>

      <Dialog open={detailOpen} onOpenChange={setDetailOpen}>
        <DialogContent className="sm:max-w-4xl max-h-[85vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="text-base">
              รายละเอียดการพิมพ์อัตโนมัติ รอบวันที่ {(runs.find((r) => r.FTLsrRunID === inspectedRunId) || latestRun)?.FDLsrBizDate}
            </DialogTitle>
            <DialogDescription className="text-xs">
              สถานะรอบนี้: <strong>{latestRun ? formatRunStatusLabel((runs.find((r) => r.FTLsrRunID === inspectedRunId) || latestRun)!.FTLsrStatus) : '-'}</strong> &bull; พิมพ์ออกแล้วจะไม่ถูกพิมพ์ซ้ำในรอบถัดไป ส่วนรายการที่ค้างสามารถกดพิมพ์ต่อได้ทันที
            </DialogDescription>
          </DialogHeader>

          {runs.length > 1 && (
            <div className="flex flex-wrap items-center gap-2 text-xs">
              <span className="text-slate-500">เลือกดูรอบรันย้อนหลัง:</span>
              <select
                value={inspectedRunId || latestRun?.FTLsrRunID || ''}
                onChange={(e) => {
                  const nextId = e.target.value;
                  setInspectedRunId(nextId);
                  fetchStatus(nextId);
                }}
                className="h-7 rounded border border-slate-200 bg-white px-2 text-xs"
              >
                {runs.map((r) => (
                  <option key={r.FTLsrRunID} value={r.FTLsrRunID}>
                    {r.FDLsrBizDate} ({r.FTLsrTriggerBy}) — {formatRunStatusLabel(r.FTLsrStatus)} [ปกติ {r.FNLsrNorQty} / โปร {r.FNLsrPmtQty} / ค้าง {r.FNLsrFailQty}]
                  </option>
                ))}
              </select>
            </div>
          )}

          <div className="border rounded-md overflow-hidden">
            <Table>
              <TableHeader>
                <TableRow className="bg-slate-100">
                  <TableHead className="text-xs w-12">ลำดับ</TableHead>
                  <TableHead className="text-xs">บาร์โค้ด / รหัสสินค้า</TableHead>
                  <TableHead className="text-xs">สาเหตุที่พิมพ์</TableHead>
                  <TableHead className="text-xs">ประเภทป้าย</TableHead>
                  <TableHead className="text-xs text-right">ราคาขาย</TableHead>
                  <TableHead className="text-xs">แผ่นที่</TableHead>
                  <TableHead className="text-xs">ผลลัพธ์</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {queueItems.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={7} className="text-center text-xs text-slate-400 py-6">
                      ไม่มีรายการในรอบนี้ (ยังไม่มีสินค้าเปลี่ยนราคา หรือการเชื่อมต่อส่วนกลางขัดข้อง)
                    </TableCell>
                  </TableRow>
                ) : (
                  queueItems.map((q) => (
                    <TableRow key={q.FNLsqSeq}>
                      <TableCell className="text-xs font-mono">{q.FNLsqSeq}</TableCell>
                      <TableCell className="text-xs">
                        <div className="font-mono font-medium">{q.FTBarCode}</div>
                        <div className="text-[11px] text-slate-500">รหัส: {q.FTPdtCode} &bull; ขนาด {q.FTLbsCode}</div>
                      </TableCell>
                      <TableCell className="text-xs">{formatTriggerLabel(q.FTLsqTriggerSrc)}</TableCell>
                      <TableCell className="text-xs">
                        {q.FTLsqPriType === '2' ? (
                          <Badge className="bg-red-100 text-red-800 border-red-200 text-[10px]">ป้ายโปรโมชัน</Badge>
                        ) : (
                          <Badge variant="secondary" className="text-[10px]">ป้ายปกติ</Badge>
                        )}
                      </TableCell>
                      <TableCell className="text-xs text-right font-mono font-semibold">
                        {q.FCLsqSalePrice.toFixed(2)} ฿
                      </TableCell>
                      <TableCell className="text-xs font-mono">
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
                          {formatQueueStatusLabel(q.FTLsqStatus)}
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

          <DialogFooter>
            {latestRun && latestRun.FNLsrFailQty > 0 && (
              <Button variant="outline" size="sm" onClick={handleLoadToManual}>
                <FileDown className="h-3.5 w-3.5 mr-1" />
                ส่ง {latestRun.FNLsrFailQty} รายการที่ค้างไปหน้าพิมพ์หลัก
              </Button>
            )}
            <Button size="sm" onClick={() => setDetailOpen(false)}>
              ปิด
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
