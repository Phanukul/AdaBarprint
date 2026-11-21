import React, { useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { SearchableSelect } from '@/components/ui/searchable-select';
import {resolveTemplateRoute} from '@/lib/printProfile';
import { Input } from '@/components/ui/input';
import { api, Field, purposes, selectClass, section, templateProblem, type Props, type Rule, type Center, type Template, type Candidate } from './autoPrintShared';

type Row = { id:string; source:string; selected:boolean; template:string; time:string; printerOverride:boolean; days?:string; quantity?:number; catchUp?:boolean; original?:Rule; route?:Pick<Rule,'printer'|'tray'|'protocol'|'ip'|'port'|'dpi'> };
type Preferences = { branch:string; enabled:boolean; time:string; days:string; quantity:number; catchUp:boolean; normal:string; promo:string; rows:Row[] };
const standardSources=['PRICE_CHG','PROMO_START','PROMO_END_REVERT','NEW_PDT'];
const dayNames=['จ.','อ.','พ.','พฤ.','ศ.','ส.','อา.'];
const timePattern=/^([01]\d|2[0-3]):[0-5]\d$/;
function TimeInput({value,onChange,label}:{value:string;onChange:(v:string)=>void;label:string}) {
  return <span className="flex items-center gap-1"><select aria-label={`${label} ชั่วโมง`} className={selectClass} value={value.split(':')[0]} onChange={e=>onChange(`${e.target.value}:${value.split(':')[1]||'00'}`)}>{Array.from({length:24},(_,i)=>String(i).padStart(2,'0')).map(v=><option key={v}>{v}</option>)}</select><span>:</span><select aria-label={`${label} นาที`} className={selectClass} value={value.split(':')[1]} onChange={e=>onChange(`${value.split(':')[0]||'01'}:${e.target.value}`)}>{Array.from({length:60},(_,i)=>String(i).padStart(2,'0')).map(v=><option key={v}>{v}</option>)}</select></span>;
}
export function AutoPrintBulkSettings(p:Props & {center:Center;templates:Template[];onSaved:()=>Promise<void>}) {
  const branch=p.center.master?.branch||p.sysOptions.find(o=>o.FTSopCode==='DefaultBranch')?.FTSopValue||'';
  const existing=p.center.rules.filter(r=>r.branch===branch);
  const [form,setForm]=useState<Preferences>(()=>{
    const saved=p.center.bulkSettings as Preferences|undefined;
    const previous=saved?.branch===branch&&Array.isArray(saved.rows)?saved:undefined;
    const first=existing[0];
    const defaults={branch,enabled:existing.some(r=>r.enabled),time:first?.time||'01:00',days:first?.days||'1111111',quantity:first?.quantity||1,catchUp:first?.catchUp??true,normal:'',promo:''};
    return {...defaults,...previous,rows:standardSources.map<Row>(source=>{
      const candidates=existing.filter(r=>r.source===source);
      const prior=previous?.rows.find(row=>row.source===source&&candidates.some(r=>r.id===row.id&&r.enabled))||previous?.rows.find(row=>row.source===source);
      const original=candidates.find(r=>r.id===prior?.id)||candidates.find(r=>r.enabled)||candidates[0];
      if(prior)return {...prior,id:original?.id||prior.id,original,selected:original?.enabled||Boolean(prior.selected&&(prior.template||original?.template))};
      if(original)return {id:original.id,source,selected:original.enabled,template:original.template,time:original.time===defaults.time?'':original.time,printerOverride:true,days:original.days===defaults.days?undefined:original.days,quantity:original.quantity===defaults.quantity?undefined:original.quantity,catchUp:original.catchUp===defaults.catchUp?undefined:original.catchUp,original};
      return {id:crypto.randomUUID(),source,selected:false,template:'',time:'',printerOverride:false};
    })};
  });
  const [busy,setBusy]=useState(false);
  const [error,setError]=useState('');
  const [dirty,setDirty]=useState(false);
  const [preview,setPreview]=useState<Array<{name:string;items:Candidate[]}>|null>(null);
  const [confirmEnable,setConfirmEnable]=useState(false);
  const mismatch=!branch||form.branch!==branch||Boolean(p.authUser.branchId&&p.authUser.branchId!==branch);
  const update=(patch:Partial<Preferences>)=>{setForm(f=>({...f,...patch}));setDirty(true);setError('');setPreview(null);setConfirmEnable(false);};
  const updateRow=(id:string,patch:Partial<Row>)=>update({rows:form.rows.map(r=>r.id===id?{...r,...patch}:r)});
  const printerOptions=[...p.windowsPrinters.map(w=>({name:w.name,protocol:'WINDOWS',ip:'',port:9100,dpi:203,trays:w.trays||[]})),...p.printers.filter(x=>x.enabled&&x.ip!=='Windows'&&x.brand!=='WINDOWS'&&x.brand!=='SAMSUNG').map(x=>({name:x.name,protocol:x.brand,ip:x.ip,port:x.port,dpi:x.dpi,trays:[] as string[]}))];
  const templateOptions=p.templates.map(t=>{const size=p.allLabelSizes.find(s=>s.FTLbsCode===t.FTLbsCode);return {value:t.FTLtmCode,label:t.FTLtmName,keywords:`${t.FTLtmCode} ${t.FTLbsCode} ${size?.FTLbsName||''}`,badge:size?`${size.FCLbsWidth}×${size.FCLbsHeight} มม.`:undefined};});
  const resolve=(row:Row):Rule=>{
    const code=row.template||row.original?.template||(row.source==='PROMO_START'?form.promo:form.normal)||'';
    const template=p.templates.find(t=>t.FTLtmCode===code);
    const route=row.route||row.original||{printer:'',tray:'',protocol:'WINDOWS',ip:'',port:9100,dpi:203};
    return {id:row.id,name:row.original?.name||purposes[row.source]?.name||row.source,source:row.source,branch,enabled:form.enabled&&row.selected,
      time:row.time||form.time,days:row.days||form.days,quantity:row.quantity??form.quantity,catchUp:row.catchUp??form.catchUp,
      template:code,size:template?.FTLbsCode||'',printer:route.printer,tray:route.tray,protocol:route.protocol,ip:route.ip,port:route.port,dpi:route.dpi};
  };
  const validate=()=>{
    if(mismatch)return 'สาขาของการตั้งค่าไม่ตรงกับแหล่งข้อมูล กรุณาเปิดหน้านี้ใหม่หลังตรวจสอบสาขา';
    if(!timePattern.test(form.time)||!form.days.includes('1'))return 'กรุณาระบุเวลาและเลือกวันพิมพ์อย่างน้อยหนึ่งวัน';
    if(!Number.isInteger(form.quantity)||form.quantity<1||form.quantity>999)return 'จำนวนป้ายต้องเป็นจำนวนเต็ม 1–999';
    if(form.enabled&&!form.rows.some(r=>r.selected))return 'เลือกประเภทงานอย่างน้อยหนึ่งรายการก่อนเปิดใช้งาน';
    for(const row of form.rows.filter(r=>r.selected)){
      const rule=resolve(row);const template=p.templates.find(t=>t.FTLtmCode===rule.template);const name=purposes[row.source]?.title||row.source;
      if(!template)return `${name}: กรุณาเลือกแบบป้ายสำหรับงานนี้`;
      if(templateProblem(template))return `${name}: ${templateProblem(template)}`;
      if(!p.allLabelSizes.some(s=>s.FTLbsCode===rule.size&&s.FTLbsStaUse==='1'))return `${name}: ขนาดป้ายถูกปิดใช้งาน`;
      if(!rule.printer)return `${name}: กรุณาเลือกเครื่องพิมพ์สำหรับงานนี้`;
      if(!timePattern.test(rule.time))return `${name}: เวลาไม่ถูกต้อง`;
    }
    return '';
  };
  const check=async()=>{
    const message=validate();if(message){setError(message);return;}
    setBusy(true);setError('');setPreview(null);
    try{const result:Array<{name:string;items:Candidate[]}>=[];for(const row of form.rows.filter(r=>r.selected)){const d=await api('/api/print-center/preview',resolve(row));result.push({name:purposes[row.source]?.title||row.source,items:d.items});}setPreview(result);}catch(e){setError(e instanceof Error?e.message:String(e));}finally{setBusy(false);}
  };
  const save=async(confirmed=false)=>{
    const message=validate();if(message){setError(message);return;}
    if(form.enabled&&!confirmed){setConfirmEnable(true);return;}
    setBusy(true);setError('');
    try{
      // Keep old IDs/watermarks. Unselected old rules are disabled, never deleted.
      const rules=[...form.rows.flatMap(row=>row.selected?[resolve(row)]:existing.some(r=>r.id===row.id)?[{...existing.find(r=>r.id===row.id)!,enabled:false}]:[]),...existing.filter(r=>!form.rows.some(row=>row.id===r.id)).map(r=>({...r,enabled:false}))];
      await api('/api/print-center/rules',{branch,rules,preferences:{...form,rows:form.rows.map(row=>({...row,original:rules.find(r=>r.id===row.id)}))}});
      setForm(f=>({...f,rows:f.rows.map(row=>({...row,original:rules.find(r=>r.id===row.id)||row.original}))}));
      setDirty(false);setConfirmEnable(false);toast.success('บันทึกการตั้งค่าแล้ว');
      await p.onSaved();
    }catch(e){setError(e instanceof Error?e.message:String(e));}finally{setBusy(false);}
  };
  return <div className="space-y-4 min-w-0">
    <div className="flex flex-wrap items-center justify-between gap-3"><div><h2 className="font-semibold text-lg">ตั้งค่าพิมพ์อัตโนมัติ</h2><p className="text-sm text-slate-500 mt-1">เลือกเหตุการณ์และแบบป้ายในหน้าเดียว ระบบค้นหารายการสินค้าให้ตามวันที่มีผล</p></div><span className="text-sm text-slate-500">สาขา {branch}</span></div>
    <div className="flex flex-wrap items-center gap-2 text-sm text-slate-500"><span className={`h-2 w-2 rounded-full ${p.center.master?.autoReady?'bg-emerald-500':'bg-slate-400'}`}/><span>{p.center.master?.autoReady?'ข้อมูลพร้อมใช้งาน':p.center.master?.connected?'เชื่อมต่อ StoreFront แล้ว':'ยังไม่เชื่อมต่อ StoreFront'}</span><Button variant="link" size="sm" className="text-teal-700" onClick={p.onOpenConnection}>รายละเอียดแหล่งข้อมูล</Button></div>
    <details className="text-sm text-slate-600"><summary className="cursor-pointer">การทำงานเมื่อปิดหน้าจอโปรแกรม</summary><p className="mt-2">ตั้งครั้งเดียวบนเครื่องที่ติดตั้ง ให้บริการดาวน์โหลดและพิมพ์เริ่มเมื่อผู้ใช้เข้าสู่ Windows</p><Button variant="outline" className="mt-2" disabled={busy} onClick={async()=>{try{const result=await api('/api/agent/register',{});toast.success(result.message);}catch(e){toast.error(e instanceof Error?e.message:String(e));}}}>ตั้งให้เริ่มพร้อม Windows</Button></details>
    {existing.some(r=>r.enabled&&!form.rows.some(row=>row.id===r.id))&&<p className="text-sm text-amber-800">พบตารางเดิมเพิ่มเติม เมื่อบันทึกจะปิดตารางที่ไม่ได้แสดง เพื่อให้เหลือหนึ่งตารางต่อเหตุการณ์ ประวัติเดิมยังอยู่</p>}
    {error&&<p role="alert" className="bg-red-50 text-red-700 p-4 rounded-lg">{error}</p>}
    <fieldset disabled={busy||mismatch} className="min-w-0 space-y-4">
    <section className={section}><h3 className="font-semibold">ค่าร่วมสำหรับทุกประเภท</h3><div className="grid gap-4 md:grid-cols-2 xl:grid-cols-2">
      <Field label="เวลาเริ่มพิมพ์ (24 ชม.)"><TimeInput value={form.time} label="เวลาเริ่มพิมพ์" onChange={time=>update({time})}/></Field>


      <Field label="จำนวนป้ายต่อรายการ"><Input type="number" min={1} max={999} value={form.quantity} onChange={e=>update({quantity:Number(e.target.value)})}/></Field>
    </div><div className="flex flex-wrap gap-2 items-center text-sm"><span className="font-medium mr-2">วันพิมพ์</span><Button variant="outline" size="sm" onClick={()=>update({days:'1111111'})}>ทุกวัน</Button><Button variant="outline" size="sm" onClick={()=>update({days:'1111100'})}>จันทร์–ศุกร์</Button>{dayNames.map((d,i)=><label key={d} className="flex items-center gap-1 rounded border px-2 py-2"><input type="checkbox" className="accent-teal-700" checked={form.days[i]==='1'} onChange={()=>update({days:form.days.split('').map((v,j)=>i===j?(v==='1'?'0':'1'):v).join('')})}/>{d}</label>)}</div>
      <label className="flex items-start gap-2 text-sm"><input type="checkbox" className="mt-1 accent-teal-700" checked={form.catchUp} onChange={e=>update({catchUp:e.target.checked})}/>เมื่อเปิดเครื่องหลังเวลาที่ตั้ง ให้พิมพ์รอบที่พลาดของวันนี้</label>
    </section>
    <section className={section}>
    <h3 className="font-semibold">งานพิมพ์ตามเหตุการณ์</h3><p className="text-sm text-slate-500">เลือกเฉพาะเหตุการณ์ที่ต้องการใช้งาน ไม่ต้องตั้งค่าครบทั้ง 4 เหตุการณ์</p>
    {form.rows.map(row=>{const rule=resolve(row);const template=p.templates.find(t=>t.FTLtmCode===rule.template);const printer=printerOptions.find(x=>x.name===rule.printer);return <article key={row.id} className="rounded-lg border p-3 space-y-3 text-sm">
      <div className="flex flex-wrap items-center justify-between gap-2"><label className="flex items-center gap-2"><input type="checkbox" className="accent-teal-700" checked={row.selected} onChange={e=>updateRow(row.id,{selected:e.target.checked})}/><b>{purposes[row.source]?.title||row.source}</b></label></div>
      {row.selected&&<>
      <div className="grid sm:grid-cols-2 xl:grid-cols-4 gap-3">
        <Field label="แบบป้าย"><SearchableSelect ariaLabel={`แบบป้าย ${purposes[row.source]?.title}`} value={rule.template} options={templateOptions} placeholder="เลือกแบบป้าย" searchPlaceholder="ค้นหาชื่อหรือขนาดป้าย…" onValueChange={template=>updateRow(row.id,{template})}/></Field>
        <Field label="เครื่องพิมพ์"><SearchableSelect ariaLabel={`เครื่องพิมพ์ ${purposes[row.source]?.title}`} value={rule.printer} options={[...printerOptions.map(x=>({value:x.name,label:x.name})),...(rule.printer&&!printer?[{value:rule.printer,label:rule.printer+' (ที่บันทึกไว้)'}]:[])]} placeholder="เลือกเครื่องพิมพ์" onValueChange={name=>{const pr=printerOptions.find(x=>x.name===name);if(pr)updateRow(row.id,{route:{printer:pr.name,tray:'',protocol:pr.protocol,ip:pr.ip,port:pr.port,dpi:pr.dpi}});}}/></Field>
        <Field label="ถาดกระดาษ"><SearchableSelect ariaLabel={`ถาด ${purposes[row.source]?.title}`} disabled={!rule.printer} value={rule.tray} options={[{value:'',label:'ถาดเริ่มต้น'},...(printer?.trays||[]).map(t=>({value:t,label:p.formatTrayLabel(t)})),...(rule.tray&&!printer?.trays.includes(rule.tray)?[{value:rule.tray,label:p.formatTrayLabel(rule.tray)}]:[])]} onValueChange={tray=>updateRow(row.id,{route:{...rule,tray}})}/></Field>
        <Field label="เวลา"><select className={selectClass} value={row.time?'custom':'shared'} onChange={e=>updateRow(row.id,{time:e.target.value==='custom'?form.time:''})}><option value="shared">ใช้เวลาหลัก {form.time}</option><option value="custom">กำหนดเวลาเฉพาะ</option></select>{row.time&&<TimeInput label={purposes[row.source]?.title||row.source} value={row.time} onChange={time=>updateRow(row.id,{time})}/>}</Field>
      </div>
      <Button variant="link" size="sm" disabled={!rule.template} onClick={()=>{const m=p.driverMappings.find(x=>x.lbsCode===rule.size);if(!m){setError('ยังไม่ได้ตั้งค่าเครื่องพิมพ์ของแบบป้ายนี้');return;}const route=resolveTemplateRoute(m,rule.template,template?.FTLtmPriType||'1');const pr=printerOptions.find(x=>x.name===route.printer);if(!pr){setError('ไม่พบเครื่องพิมพ์ของแบบป้ายนี้ กรุณาเลือกเครื่องพิมพ์');return;}updateRow(row.id,{route:{...route,ip:pr.ip,port:pr.port,dpi:pr.dpi}});}}>นำค่าเครื่องพิมพ์จากแบบป้ายมาใช้</Button>
      {template&&templateProblem(template)&&<p className="text-amber-800 text-xs">{templateProblem(template)}</p>}
      {(row.days!==undefined||row.quantity!==undefined||row.catchUp!==undefined)&&<div className="text-xs text-slate-500">ค่าจากตารางเดิม: วัน {dayNames.filter((_,i)=>(row.days||form.days)[i]==='1').join(' ')} · {rule.quantity} ป้าย · ชดเชยรอบ {rule.catchUp?'เปิด':'ปิด'} <Button variant="link" size="sm" onClick={()=>updateRow(row.id,{days:undefined,quantity:undefined,catchUp:undefined})}>ใช้ค่าหลัก</Button></div>}
      </>}
    </article>;})}
    <p className="text-sm text-slate-500">งานก่อนหน้าทำจนจบก่อนเริ่มงานถัดไป</p></section>
    <section className={section}><label className="flex items-start gap-3"><input type="checkbox" className="mt-1 accent-teal-700" checked={form.enabled} onChange={e=>update({enabled:e.target.checked})}/><span className="text-sm"><b>เปิดพิมพ์อัตโนมัติหลังบันทึก</b><span className="block mt-1 text-slate-500">{form.enabled?'จะเปิดใช้งานเฉพาะประเภทที่เลือก และอาจเริ่มรอบที่พลาดของวันนี้':'บันทึกการตั้งค่าไว้ก่อน โดยยังไม่พิมพ์อัตโนมัติ'}</span></span></label></section>
    </fieldset>
    {preview&&<section className={section}><h3 className="font-semibold">ผลตรวจรายการสินค้า — ยังไม่ได้สั่งพิมพ์</h3>{preview.map((group,i)=><details key={i} className="border rounded-lg p-3"><summary className="cursor-pointer text-sm">{group.name}: เข้าเงื่อนไข {group.items.filter(x=>!x.skippedDup).length} จาก {group.items.length} รายการ</summary><div className="max-h-60 overflow-auto text-sm">{group.items.map((x,j)=><p key={j} className="border-t py-2">{x.barCode} · {x.pdtName} · {x.salePrice.toFixed(2)} บาท{x.skippedDup?' · เคยส่งข้อมูลชุดนี้แล้ว':''}</p>)}</div></details>)}<p className="text-xs text-slate-500">ผลตรวจขึ้นกับข้อมูลและวันที่มีผลในขณะนี้ กรุณาตรวจสอบเวลาที่ดาวน์โหลดข้อมูลสำเร็จล่าสุด</p></section>}
    {confirmEnable&&<div role="alert" className="rounded-lg border border-amber-300 bg-amber-50 p-4 space-y-3 text-sm"><p>ยืนยันเปิดพิมพ์อัตโนมัติสำหรับ {form.rows.filter(r=>r.selected).length} ประเภท? หากเปิดชดเชยรอบที่พลาด ระบบอาจเริ่มพิมพ์หลังบันทึก</p><Button disabled={busy} className="bg-teal-700" onClick={()=>void save(true)}>ยืนยันบันทึกและเปิดใช้งาน</Button> <Button disabled={busy} variant="outline" onClick={()=>setConfirmEnable(false)}>กลับไปตรวจสอบ</Button></div>}
    <div className="sticky bottom-0 z-10 rounded-lg border bg-white p-3 flex flex-wrap items-center justify-between gap-3"><div className="text-sm text-slate-500">{(p.center.bulkVersion!==1||p.center.separateSettingsVersion!==1)?<details><summary className="cursor-pointer">ยังบันทึกไม่ได้ · ตรวจสอบบริการ</summary><p className="mt-2 max-w-lg">บริการที่เปิดอยู่ยังเป็นรุ่นเดิม ต้องอัปเดตบริการเพื่อบันทึกการตั้งค่า Auto ที่แยกจากพิมพ์เอง</p></details>:dirty?'มีการเปลี่ยนแปลงที่ยังไม่บันทึก':'เลือกใช้เฉพาะเหตุการณ์ที่ต้องการ แล้วบันทึกครั้งเดียว'}</div><div className="flex flex-wrap gap-2"><Button variant="outline" disabled={busy||mismatch||!form.rows.some(r=>r.selected)} onClick={()=>void check()}>ตรวจรายการสินค้า (ไม่พิมพ์)</Button><Button className="bg-teal-700 hover:bg-teal-800 text-white" disabled={busy||mismatch||(p.center.bulkVersion!==1||p.center.separateSettingsVersion!==1)} onClick={()=>void save()}>{busy?'กำลังดำเนินการ…':'บันทึกการตั้งค่า'}</Button></div></div>
  </div>;
}
