import React, {useEffect,useState} from 'react';
import type {ConfigLabelSizeRow,SettingsDriverMapping,SettingsWindowsPrinter} from './SettingsPanel';
import type {PrinterConfig} from '@/types/printer';
import {api,Field,selectClass,type Template} from './autoPrintShared';
import {resolveTemplateRoute,withManualRoute} from '@/lib/printProfile';
interface Props {value:SettingsDriverMapping;onChange:(value:SettingsDriverMapping)=>void;windowsPrinters:SettingsWindowsPrinter[];printers:PrinterConfig[];formatTrayLabel:(tray:string)=>string;paper?:ConfigLabelSizeRow;disabled?:boolean;onReadyChange?:(ready:boolean)=>void}
export function LabelPrinterRoutesEditor({value,onChange,windowsPrinters,printers,formatTrayLabel,paper,disabled,onReadyChange}:Props){
 const [templates,setTemplates]=useState<Template[]>([]);const [code,setCode]=useState('');const [error,setError]=useState('');const [loading,setLoading]=useState(true);const [retry,setRetry]=useState(0);
 useEffect(()=>{let active=true;setLoading(true);setError('');setTemplates([]);setCode('');onReadyChange?.(false);
 api('/api/templates').then(data=>{if(!Array.isArray(data.db))throw new Error('ข้อมูลแบบป้ายไม่ถูกต้อง กรุณาลองใหม่');if(active){const rows=data.db.filter((t:Template)=>t.FTLbsCode===value.lbsCode);setTemplates(rows);setCode(rows[0]?.FTLtmCode||'');if(!rows.length)onChange(withManualRoute(value,resolveTemplateRoute(value,'',value.normalPrinter?'1':'2')));setLoading(false);onReadyChange?.(true);}}).catch(e=>{if(active){setError(String(e.message));setLoading(false);onReadyChange?.(false);}});return()=>{active=false;};},[value.lbsCode,retry,onReadyChange]);
 const template=templates.find(t=>t.FTLtmCode===code);const route=resolveTemplateRoute(value,code,template?.FTLtmPriType||'1');
 const choices=[...windowsPrinters.map(p=>({name:p.name,protocol:'WINDOWS',trays:p.trays||[]})),...printers.filter(p=>p.enabled&&p.ip!=='Windows'&&p.brand!=='WINDOWS'&&p.brand!=='SAMSUNG').map(p=>({name:p.name,protocol:p.brand,trays:[] as string[]}))];
 const trays=choices.find(p=>p.name===route.printer)?.trays||[];
 const update=(patch:Partial<typeof route>)=>onChange({...value,templateRoutes:{...value.templateRoutes,[code]:{...route,...patch}}});
 const trayLabel=(t:string)=>/automatically select/i.test(t)?'เลือกถาดอัตโนมัติ':formatTrayLabel(t);
 const legacyFields=()=>{
   const printer=value.normalPrinter;const tray=value.normalTray;
   const list=choices.find(p=>p.name===printer)?.trays||[];
   const change=(patch:Partial<typeof route>)=>onChange(withManualRoute(value,{printer,tray,protocol:value.normalProtocol||'WINDOWS',...patch}));
   return <div className="grid sm:grid-cols-2 gap-4"><Field label="เครื่องพิมพ์"><select className={selectClass} value={printer} onChange={e=>{const p=choices.find(p=>p.name===e.target.value);if(p)change({printer:p.name,protocol:p.protocol,tray:''});}}><option value="" disabled>เลือกเครื่องพิมพ์</option>{printer&&!choices.some(p=>p.name===printer)&&<option value={printer}>{printer} (ที่บันทึกไว้)</option>}{choices.map((p,i)=><option key={i} value={p.name}>{p.name}</option>)}</select></Field><Field label="ถาดกระดาษ"><select className={selectClass} disabled={!printer} value={tray} onChange={e=>change({tray:e.target.value})}><option value="">ถาดเริ่มต้นของเครื่องพิมพ์</option>{tray&&!list.includes(tray)&&<option value={tray}>{trayLabel(tray)}</option>}{list.map(t=><option key={t} value={t}>{trayLabel(t)}</option>)}</select></Field></div>;
 };
 if(loading)return <p role="status" className="py-4 text-sm text-slate-500">กำลังโหลดแบบป้าย…</p>;
 if(error)return <div role="alert" className="space-y-3 text-sm"><p className="text-red-700">{error}</p><button type="button" className="rounded-md border px-3 py-2" onClick={()=>setRetry(n=>n+1)}>ลองใหม่</button></div>;
 if(!templates.length)return <fieldset disabled={disabled} className="space-y-4 min-w-0">
   <div><h3 className="font-medium text-sm">{paper?.FTLbsName||value.lbsName}</h3><p className="text-xs text-slate-500 mt-1">กำหนดเครื่องพิมพ์และถาดสำหรับพิมพ์เองด้วยแบบป้ายนี้</p></div>
   {legacyFields()}
   <p className="text-xs text-slate-500">เมื่อบันทึก การพิมพ์เองด้วยแบบป้ายนี้จะใช้เครื่องพิมพ์และถาดชุดเดียวกันทุกประเภทราคา การตั้งค่า Auto ไม่เปลี่ยนแปลง</p>
 </fieldset>;
 return <fieldset disabled={disabled} className="space-y-4 min-w-0">
 <p className="text-sm text-slate-600">เลือกแบบป้าย แล้วกำหนดเครื่องพิมพ์และถาด สำหรับพิมพ์เอง โดยไม่เปลี่ยนการตั้งค่า Auto</p>
 {error&&<p role="alert" className="text-red-700">{error}</p>}
 <Field label="แบบป้าย"><select className={selectClass} value={code} onChange={e=>setCode(e.target.value)}><option value="" disabled>เลือกแบบป้าย</option>{templates.map(t=><option key={t.FTLtmCode} value={t.FTLtmCode}>{t.FTLtmName}</option>)}</select></Field>
 {template&&<><p className="text-xs text-slate-500">กระดาษ {paper?.FTLbsPaperSize||'ตามขนาดป้าย'} · {paper?.FCLbsWidth} × {paper?.FCLbsHeight} มม.</p>
 <div className="grid sm:grid-cols-2 gap-4"><Field label="เครื่องพิมพ์"><select className={selectClass} value={route.printer} onChange={e=>{const p=choices.find(p=>p.name===e.target.value);if(p)update({printer:p.name,protocol:p.protocol,tray:''});}}><option value="" disabled>เลือกเครื่องพิมพ์</option>{route.printer&&!choices.some(p=>p.name===route.printer)&&<option value={route.printer}>{route.printer} (ที่บันทึกไว้)</option>}{choices.map((p,i)=><option key={i} value={p.name}>{p.name}</option>)}</select></Field><Field label="ถาดกระดาษ"><select className={selectClass} value={route.tray} disabled={!route.printer} onChange={e=>update({tray:e.target.value})}><option value="">ถาดเริ่มต้นของเครื่องพิมพ์</option>{route.tray&&!trays.includes(route.tray)&&<option value={route.tray}>{trayLabel(route.tray)}</option>}{trays.map(t=><option key={t} value={t}>{trayLabel(t)}</option>)}</select></Field></div>
 <p className="text-xs text-slate-500">{value.templateRoutes?.[code]?'กำหนดเครื่องพิมพ์สำหรับแบบนี้':'ยังใช้ค่าการพิมพ์เดิมของแบบนี้ เปลี่ยนเครื่องหรือถาดเพื่อกำหนดเฉพาะแบบ'}</p></>}
 <p className="text-xs text-slate-500">การพิมพ์อัตโนมัติกำหนดแบบป้าย เครื่องพิมพ์ และถาดแยกในแท็บตั้งเวลาพิมพ์</p>
 </fieldset>;
}
