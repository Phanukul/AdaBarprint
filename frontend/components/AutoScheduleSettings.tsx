import React, { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { TablePagination } from './TablePagination';
import { AutoPrintBulkSettings } from './AutoPrintBulkSettings';
import { api, states, section, type Props, type Center, type Template } from './autoPrintShared';

interface JobDestination {
  printer: string | null;
  label: string | null;
  template: string | null;
  units: number;
}

export function AutoScheduleSettings(p:Props) {
  const [center,setCenter]=useState<Center|null>(null);
  const [templates,setTemplates]=useState<Template[]>([]);
  const [failure,setFailure]=useState('');
  const [busy,setBusy]=useState(false);
  const [decision,setDecision]=useState<{job:Center['jobs'][number];action:string}|null>(null);
  const [detailJob, setDetailJob] = useState<Center['jobs'][number] | null>(null);
  const [destinations, setDestinations] = useState<JobDestination[] | null>(null);
  const [detailError, setDetailError] = useState('');
  useEffect(() => {
    if (!detailJob) return;
    let active = true;
    api(`/api/print-center/jobs/${encodeURIComponent(detailJob.id)}/details`)
      .then(rows => { if (active) setDestinations(rows); })
      .catch(error => { if (active) setDetailError(String(error.message || error)); });
    return () => { active = false; };
  }, [detailJob]);
  function openDetails(job: Center['jobs'][number]) {
    setDestinations(null);
    setDetailError('');
    setDetailJob(job);
  }
  const refresh=useCallback(async()=>{const [c,t]=await Promise.all([api('/api/print-center'),api('/api/templates')]);setCenter(c);setTemplates(t.db||[]);setFailure('');},[]);
  useEffect(()=>{let active=true;const load=async()=>{try{const [c,t]=await Promise.all([api('/api/print-center'),api('/api/templates')]);if(active){setCenter(c);setTemplates(t.db||[]);setFailure('');}}catch(e){if(active)setFailure(e instanceof Error?e.message:String(e));}};void load();const timer=setInterval(load,5000);return()=>{active=false;clearInterval(timer);};},[]);
  const perform=async(fn:()=>Promise<void>)=>{setBusy(true);try{await fn();await refresh();}catch(e){toast.error(e instanceof Error?e.message:String(e));}finally{setBusy(false);}};
  const jobs = center?.jobs ?? [];
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(10);
  const currentPage = Math.min(page, Math.max(1, Math.ceil(jobs.length / pageSize)));
  const start = (currentPage - 1) * pageSize;
  const pageJobs = jobs.slice(start, start + pageSize);
  const selected = jobs.find(job => job.id === detailJob?.id);
  const preparationFailed = (job: Center['jobs'][number]) =>
    job.status === 'NEEDS_REVIEW' && (job.error === 'สร้างภาพป้ายเดิมไม่สำเร็จ' || job.error?.startsWith('สร้างภาพป้ายไม่สำเร็จ:'));
  const actionLabels: Record<string,string> = {RETRY:'ลองอีกครั้ง', CONFIRM_SENT:'ยืนยันว่าพิมพ์แล้ว', CANCEL:'ยกเลิกงาน'};
  function chooseAction(action: string) {
    if (!selected) return;
    setDetailJob(null);
    setDecision({job:selected, action});
  }
  return <div className="min-w-0 space-y-4">
    {failure&&<p role="alert" className="rounded-lg bg-red-50 p-4 text-red-700">{failure}</p>}
    {p.mode!=='jobs'?center?<AutoPrintBulkSettings {...p} center={center} templates={templates} onSaved={refresh}/>:<p className="p-4">กำลังโหลดการตั้งค่า…</p>:<>
    {<section className={section}>
      <header><h3 className="text-lg font-semibold">รายการงานพิมพ์</h3>
        <p className="mt-1 text-sm text-slate-600">งานในคิวทั้งหมดและประวัติที่จบแล้ว 100 งานล่าสุด</p>
      </header>
      <div className="overflow-x-auto rounded-lg border border-slate-200">
        <table className="w-full min-w-[760px] text-left text-sm">
          <thead className="bg-slate-100 text-slate-700"><tr>
            {['งานพิมพ์','เวลาสั่งงาน','วันเป้าหมาย','สถานะ','ส่งแล้ว / ทั้งหมด',''].map((title,i) =>
              <th key={i} scope="col" className="px-4 py-3 font-medium">{title || <span className="sr-only">จัดการ</span>}</th>)}
          </tr></thead>
          <tbody className="divide-y divide-slate-200">{pageJobs.map(job => {
            const review = ['BLOCKED','NEEDS_REVIEW'].includes(job.status);
            return <tr key={job.id} className={review ? 'bg-amber-50/60' : ''}>
              <td className="px-4 py-3"><strong className="font-medium">{job.name}</strong>
                <p className="text-xs text-slate-500">{job.source === 'MANUAL' ? 'สั่งพิมพ์เอง' : job.source === 'PROFILE' ? 'พิมพ์ตามโปรไฟล์' : 'พิมพ์ตามตาราง'}</p></td>
              <td className="whitespace-nowrap px-4 py-3 tabular-nums">{job.created}</td>
              <td className="px-4 py-3 tabular-nums">{job.targetDate || '—'}</td>
              <td className="px-4 py-3"><span className={review ? 'font-medium text-amber-900' : ''}>{states[job.status] || job.status}</span>
                {job.error && <p className="max-w-xs break-words text-xs text-red-700">{job.error}</p>}</td>
              <td className="px-4 py-3 tabular-nums">{job.completed} / {job.total}</td>
              <td className="px-4 py-3 text-right"><Button variant="outline" size="sm" onClick={() => openDetails(job)}>
                {review ? 'ตรวจสอบงาน' : 'ดูรายละเอียด'}</Button></td>
            </tr>;
          })}</tbody>
        </table>
      </div>
      {jobs.length > 0 && <TablePagination total={jobs.length} page={currentPage} pageSize={pageSize}
        onPageChange={setPage} onPageSizeChange={size => { setPageSize(size); setPage(1); }} />}
      {!jobs.length&&<p className="py-8 text-center text-sm text-slate-500">{center?'ยังไม่มีรายการงานพิมพ์':failure?'โหลดรายการไม่สำเร็จ':'กำลังโหลดรายการงานพิมพ์…'}</p>}
      <p className="text-xs text-slate-500">สถานะ “ส่งงานแล้ว” หมายถึงส่งให้ระบบเครื่องพิมพ์แล้ว กรุณาตรวจสอบป้ายที่เครื่องพิมพ์</p>
      <details className="text-xs text-slate-500"><summary className="cursor-pointer">สถานะบริการพิมพ์</summary><p className="mt-2">ติดต่อบริการล่าสุด: {center?.heartbeat||'ยังไม่ได้รับสถานะ'}</p></details>
      <Dialog open={!!detailJob} onOpenChange={open => { if (!open) setDetailJob(null); }}>
        <DialogContent className="max-h-[85dvh] overflow-auto sm:max-w-3xl">
          <DialogHeader>
            <DialogTitle>รายละเอียดงานพิมพ์</DialogTitle>
            <DialogDescription>{detailJob?.name} · {detailJob?.created}</DialogDescription>
          </DialogHeader>
          {selected?.error && <p className="rounded-md bg-amber-50 p-3 text-sm text-amber-900">{selected.error}</p>}
          {detailError ? <p role="alert">{detailError}</p> : destinations === null ? <p>กำลังโหลดรายละเอียด…</p> :
            destinations.length === 0 ? <p>งานนี้ไม่มีหน่วยพิมพ์ที่บันทึกไว้</p> :
            <div className="overflow-x-auto"><table className="w-full text-left text-sm">
              <thead className="bg-slate-100"><tr><th className="p-2">เครื่องพิมพ์</th><th className="p-2">แบบป้าย</th><th className="p-2 text-right">จำนวนหน่วยพิมพ์</th></tr></thead>
              <tbody>{destinations.map((row, index) => <tr key={index} className="border-t">
                <td className="p-2">{row.printer || 'ไม่ระบุ'}</td>
                <td className="p-2">{row.label || row.template || 'ไม่ระบุในประวัติ'}{row.label && row.template && <div className="text-xs text-slate-500">{row.template}</div>}</td>
                <td className="p-2 text-right">{row.units}</td>
              </tr>)}</tbody>
            </table></div>}
          {selected && ['WAIT_DATA','QUEUED','BLOCKED','NEEDS_REVIEW'].includes(selected.status) &&
            <footer className="space-y-3 border-t pt-4">
              {['BLOCKED','NEEDS_REVIEW'].includes(selected.status) && <p className="text-sm text-slate-600">
                พักงานนี้ไว้แล้ว ระบบจะดำเนินงานอื่นในคิวต่อ{selected.status === 'NEEDS_REVIEW' && !preparationFailed(selected) && ' กรุณาตรวจป้ายที่เครื่องพิมพ์ก่อนเลือกคำสั่ง'}
              </p>}
              <div className="flex flex-wrap justify-end gap-2">
                <Button variant="ghost" onClick={() => chooseAction('CANCEL')}>ยกเลิกงาน</Button>
                {selected.status === 'NEEDS_REVIEW' && !preparationFailed(selected) &&
                  <Button variant="outline" onClick={() => chooseAction('CONFIRM_SENT')}>ยืนยันว่าพิมพ์แล้ว</Button>}
                {['BLOCKED','NEEDS_REVIEW'].includes(selected.status) &&
                  <Button onClick={() => chooseAction('RETRY')}>ลองอีกครั้ง</Button>}
              </div>
            </footer>}
        </DialogContent>
      </Dialog>
      <Dialog open={!!decision} onOpenChange={open => { if (!open && !busy) setDecision(null); }}>
        {decision && <DialogContent>
          <DialogHeader><DialogTitle>{actionLabels[decision.action]}</DialogTitle>
            <DialogDescription>{decision.job.name}</DialogDescription></DialogHeader>
          <p className="text-sm">{decision.action === 'CANCEL' ? 'ระบบจะยกเลิกส่วนที่ยังไม่ได้ส่งของงานนี้'
            : decision.action === 'CONFIRM_SENT' ? 'ยืนยันเมื่อเห็นป้ายออกครบแล้ว ระบบจะข้ามส่วนที่รอยืนยันและทำงานต่อ'
            : preparationFailed(decision.job) ? 'ระบบจะลองสร้างภาพป้ายและส่งงานอีกครั้ง'
            : 'กรุณาตรวจป้ายที่เครื่องพิมพ์ก่อน หน้าที่ระบบยังไม่ทราบผลอาจพิมพ์ซ้ำ'}</p>
          <div className="flex justify-end gap-2">
            <Button variant="outline" disabled={busy} onClick={() => {openDetails(decision.job); setDecision(null);}}>กลับ</Button>
            <Button disabled={busy} onClick={() => perform(async () => {
              await api('/api/print-center/action',{id:decision.job.id,action:decision.action});
              setDecision(null);
            })}>{busy ? 'กำลังดำเนินการ…' : actionLabels[decision.action]}</Button>
          </div>
        </DialogContent>}
      </Dialog>
    </section>}
    </>}
  </div>;
}
