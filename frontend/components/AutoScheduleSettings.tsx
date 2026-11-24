import React, { useCallback, useEffect, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { TablePagination } from './TablePagination';
import { AutoPrintBulkSettings } from './AutoPrintBulkSettings';
import { api, states, section, type Props, type Center, type Template } from './autoPrintShared';

export function AutoScheduleSettings(p:Props) {
  const [center,setCenter]=useState<Center|null>(null);
  const [templates,setTemplates]=useState<Template[]>([]);
  const [failure,setFailure]=useState('');
  const [busy,setBusy]=useState(false);
  const [decision,setDecision]=useState<{job:Center['jobs'][number];action:string}|null>(null);
  const refresh=useCallback(async()=>{const [c,t]=await Promise.all([api('/api/print-center'),api('/api/templates')]);setCenter(c);setTemplates(t.db||[]);setFailure('');},[]);
  useEffect(()=>{let active=true;const load=async()=>{try{const [c,t]=await Promise.all([api('/api/print-center'),api('/api/templates')]);if(active){setCenter(c);setTemplates(t.db||[]);setFailure('');}}catch(e){if(active)setFailure(e instanceof Error?e.message:String(e));}};void load();const timer=setInterval(load,5000);return()=>{active=false;clearInterval(timer);};},[]);
  const perform=async(fn:()=>Promise<void>)=>{setBusy(true);try{await fn();await refresh();}catch(e){toast.error(e instanceof Error?e.message:String(e));}finally{setBusy(false);}};
  const jobs = center?.jobs ?? [];
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(10);
  const currentPage = Math.min(page, Math.max(1, Math.ceil(jobs.length / pageSize)));
  const start = (currentPage - 1) * pageSize;
  const pageJobs = jobs.slice(start, start + pageSize);
  return <div className="min-w-0 space-y-4">
    {failure&&<p role="alert" className="rounded-lg bg-red-50 p-4 text-red-700">{failure}</p>}
    {p.mode!=='jobs'?center?<AutoPrintBulkSettings {...p} center={center} templates={templates} onSaved={refresh}/>:<p className="p-4">กำลังโหลดการตั้งค่า…</p>:<>
    {<section className={section}><h3 className="font-semibold text-lg">รายการงานพิมพ์</h3><p className="text-sm text-slate-600">แสดงงานที่ยังดำเนินการไม่จบทั้งหมด และประวัติที่จบแล้ว 100 งานล่าสุด ระบบจะทำงานตามลำดับ หากงานหยุดรอตรวจสอบ ต้องแก้ไขหรือยกเลิกก่อนเริ่มงานถัดไป</p>
      <div className="overflow-x-auto"><table className="w-full min-w-[720px] text-sm text-left"><thead className="bg-[#344b60] text-white"><tr>{['ลำดับ','งาน','สถานะ','ส่งแล้ว / ทั้งหมด','รายละเอียด / จัดการ'].map(x=><th key={x} className="p-2">{x}</th>)}</tr></thead><tbody>{pageJobs.map((j,index)=><tr key={j.id} className="border-t"><td className="p-2">{start + index + 1}</td><td className="p-2">{j.name}<p className="text-xs text-slate-500">{j.source==='MANUAL'?'สั่งพิมพ์เอง':j.source==='PROFILE'?'พิมพ์ตาม Profile':'พิมพ์ตามตาราง'} · {j.created}</p></td><td className="p-2">{states[j.status]||j.status}</td><td className="p-2">{j.completed}/{j.total}</td><td className="p-2"><p className="text-red-700 max-w-md">{j.error}</p>{['BLOCKED','NEEDS_REVIEW'].includes(j.status)&&<><Button variant="outline" size="sm" onClick={()=>setDecision({job:j,action:'RETRY'})}>ดำเนินการต่อ</Button> {j.status==='NEEDS_REVIEW'&&<Button variant="outline" size="sm" onClick={()=>setDecision({job:j,action:'CONFIRM_SENT'})}>ยืนยันว่าพิมพ์แล้ว</Button>}</>}{['WAIT_DATA','QUEUED','BLOCKED','NEEDS_REVIEW'].includes(j.status)&&<Button variant="ghost" size="sm" onClick={()=>setDecision({job:j,action:'CANCEL'})}>ยกเลิกงาน</Button>}</td></tr>)}</tbody></table></div>
      {jobs.length > 0 && <TablePagination total={jobs.length} page={currentPage} pageSize={pageSize}
        onPageChange={setPage} onPageSizeChange={size => { setPageSize(size); setPage(1); }} />}
      {!jobs.length&&<p className="py-8 text-center text-sm text-slate-500">{center?'ยังไม่มีรายการงานพิมพ์':'ไม่สามารถแสดงรายการงานพิมพ์ได้'}</p>}
      <p className="text-xs text-slate-500">สถานะ “ส่งงานแล้ว” หมายถึงส่งให้ระบบเครื่องพิมพ์แล้ว กรุณาตรวจสอบป้ายที่เครื่องพิมพ์</p>
      <details className="text-xs text-slate-500"><summary className="cursor-pointer">สถานะบริการพิมพ์</summary><p className="mt-2">ติดต่อบริการล่าสุด: {center?.heartbeat||'ยังไม่ได้รับสถานะ'}</p></details>
      {decision&&<div role="dialog" className="border-2 border-amber-500 bg-amber-50 rounded p-4 space-y-3"><b>{decision.job.name}</b><p>{decision.action==='RETRY'?'ตรวจสอบกระดาษและรายการพิมพ์ของเครื่องก่อน: หน้าที่ไม่ทราบผลจะส่งใหม่และอาจออกซ้ำ หน้าที่ส่งสำเร็จแล้วจะไม่ส่งซ้ำ':decision.action==='CONFIRM_SENT'?'ยืนยันว่าหน้าล่าสุดที่ไม่ทราบผลออกครบแล้ว ระบบจะข้ามหน้านั้นและทำหน้าถัดไป':'ยกเลิกหน้าที่ยังไม่ได้ส่ง เพื่อให้งานถัดไปทำต่อ ไม่สามารถดึงกระดาษหรือคำสั่งที่ส่งแล้วกลับได้'}</p><Button className="bg-teal-700 hover:bg-teal-800 text-white" disabled={busy} onClick={()=>perform(async()=>{await api('/api/print-center/action',{id:decision.job.id,action:decision.action});setDecision(null);})}>ยืนยัน</Button> <Button variant="outline" onClick={()=>setDecision(null)}>กลับ</Button></div>}
    </section>}
    </>}
  </div>;
}
