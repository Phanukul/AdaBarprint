import React from 'react';
import { getStoredAuthToken, type AuthSessionUser } from '@/lib/auth';
import type { ConfigLabelSizeRow, SettingsDriverMapping, SettingsWindowsPrinter, SysOptionRow } from './SettingsPanel';
import type { PrinterConfig } from '@/types/printer';

export interface Props {
  mode?: 'schedules' | 'jobs';
  onOpenConnection?: () => void;
  authUser: AuthSessionUser;
  allLabelSizes: ConfigLabelSizeRow[]; sysOptions: SysOptionRow[];
  branches: Array<{ FTBchCode: string; FTBchName: string | null }>;
  windowsPrinters: SettingsWindowsPrinter[]; printers: PrinterConfig[];
  driverMappings: SettingsDriverMapping[]; driverServerCode: string;
  formatTrayLabel: (tray: string) => string; onRefreshAll: () => Promise<void>;
}
export interface ProfileTarget {
  size:string; template:string; printer:string; tray:string; protocol:string; ip:string; port:number; dpi:number; quantity:number;
}
export interface Rule {
  id:string; code:string; name:string; enabled:boolean; branch:string; source:string; time:string; days:string;
  size:string; template:string; printer:string; tray:string; protocol:string; ip:string; port:number; dpi:number;
  quantity:number; catchUp:boolean; offsetDays:number; targets?:ProfileTarget[];
}
interface Job { id:string; seq:number; source:string; name:string; status:string; created:string; error?:string; completed:number; total:number }
export type EventProfile=Pick<Rule,'template'|'size'|'printer'|'tray'|'protocol'|'ip'|'port'|'dpi'>;
export interface Center { separateSettingsVersion?:number; master?:{connected:boolean;autoReady:boolean;branch:string;path:string;error?:string}; rules:Rule[]; jobs:Job[]; heartbeat:string; legacySchedules:number; bulkVersion?:number; bulkSettings?:unknown }
export interface Template { FTLtmCode:string; FTLtmName:string; FTLbsCode:string; FTLtmPriType:string; FTLtmLayout?:string | null }
export interface Candidate { pdtCode:string; barCode:string; pdtName:string; salePrice:number; qty:number; skippedDup:boolean; priType:string; context?:Record<string, unknown> }
function templateProblem(template:Template, windows:boolean):string {
  try {
    const layout=JSON.parse(template.FTLtmLayout||'null');
    if(!layout||!Array.isArray(layout.elements)||!layout.elements.length)return 'ยังไม่มีแบบพร้อมใช้งาน';
    const elements=layout.elements.filter((el:{visible?:boolean})=>el.visible!==false);
    if(!elements.length)return 'ไม่มีองค์ประกอบที่เปิดแสดงบนป้าย';
    if(!windows && layout.designMode==='preprint')return 'การพิมพ์อัตโนมัติยังไม่รองรับการชดเชยตำแหน่งบนกระดาษพิมพ์พื้นหลัง';
    const unsupported=elements.filter((el:{type?:string;iconName?:string})=>!(windows && (el.type==='image'||el.type==='qrcode'||(el.type==='icon'&&el.iconName==='heart'))) && !['text','static','price','date','barcode','rect','line'].includes(el.type||''));
    if(unsupported.length){const names:Record<string,string>={qrcode:'QR Code',qr:'QR Code',image:'รูปภาพ',icon:'ไอคอน'};return 'การพิมพ์อัตโนมัติยังไม่รองรับ: '+[...new Set(unsupported.map((el:{type:string})=>names[el.type]||el.type))].join(', ');}
    if(elements.some((el:{rotation?:number})=>(el.rotation??0)!==0))return 'การพิมพ์อัตโนมัติยังไม่รองรับองค์ประกอบที่หมุน';
    if(elements.some((el:{opacity?:number})=>(el.opacity??1)!==1))return 'การพิมพ์อัตโนมัติยังไม่รองรับองค์ประกอบโปร่งใส';
    return '';
  } catch {return 'ยังไม่มีแบบพร้อมใช้งาน';}
}
const sources:Record<string,string> = { PRICE_CHG:'สินค้าปรับราคา', PROMO_START:'สินค้าเริ่มโปรโมชั่น', PROMO_END_REVERT:'สินค้าสิ้นสุดโปรโมชั่น', NEW_PDT:'สินค้าใหม่' };
const purposes:Record<string,{title:string;description:string;name:string}>={
  PRICE_CHG:{title:'ราคาสินค้าเปลี่ยน',description:'พิมพ์ป้ายราคาทดแทน เมื่อราคาขายมีการปรับเปลี่ยน',name:'ป้ายราคาที่เปลี่ยน'},
  PROMO_START:{title:'เริ่มโปรโมชั่น',description:'พิมพ์ป้ายสำหรับสินค้าที่เริ่มรายการส่งเสริมการขาย',name:'ป้ายเริ่มโปรโมชั่น'},
  PROMO_END_REVERT:{title:'สิ้นสุดโปรโมชั่น',description:'พิมพ์ป้ายราคาปกติเพื่อเปลี่ยนแทนป้ายโปรโมชั่นที่หมดอายุ',name:'ป้ายคืนราคาปกติ'},
  NEW_PDT:{title:'มีสินค้าใหม่',description:'พิมพ์ป้ายให้สินค้าที่เพิ่มเข้าระบบ',name:'ป้ายสินค้าใหม่'},
};
const states:Record<string,string> = { WAIT_DATA:'รอรับข้อมูล', PREPARING:'กำลังเตรียมงาน', QUEUED:'รอพิมพ์', SENDING:'กำลังส่งงานพิมพ์', NEEDS_REVIEW:'รอตรวจสอบ', BLOCKED:'พักงาน', SUBMITTED:'ส่งงานแล้ว', EMPTY:'ไม่มีรายการใหม่', CANCELLED:'ยกเลิก' };
const selectClass='h-10 w-full min-w-0 max-w-full rounded-md border border-slate-300 bg-white px-2 text-sm';
const section='rounded-xl border border-slate-200 bg-white p-3 sm:p-5 min-w-0 space-y-4 shadow-sm';
async function api(path:string, body?:unknown) {
  const headers:Record<string,string>={Accept:'application/json'};
  const token=getStoredAuthToken();if(token)headers.Authorization=`Bearer ${token}`;
  if(body!==undefined)headers['Content-Type']='application/json';
  let res:Response;
  try {res=await fetch(path,{method:body===undefined?'GET':'POST',headers,...(body===undefined?{}:{body:JSON.stringify(body)})});}
  catch {throw new Error('ไม่สามารถเชื่อมต่อบริการพิมพ์ได้ กรุณาตรวจสอบการเชื่อมต่อแล้วลองใหม่');}
  if(!res.headers.get('content-type')?.includes('application/json'))throw new Error('บริการพิมพ์ไม่พร้อมใช้งาน หรือเป็นรุ่นที่ไม่รองรับหน้านี้ กรุณาให้ผู้ดูแลระบบตรวจสอบบริการที่เชื่อมต่อ');
  const data=await res.json().catch(()=>{throw new Error('ข้อมูลตอบกลับไม่ถูกต้อง กรุณาลองใหม่');});
  if(!res.ok)throw new Error(res.status===401?'กรุณาเข้าสู่ระบบอีกครั้ง':data.error||data.message||'ทำรายการไม่สำเร็จ');
  return data;
}
function Field({label,children}:{label:string;children:React.ReactNode}) { return <label className="min-w-0 space-y-1 text-sm block"><span className="font-medium text-slate-600">{label}</span>{children}</label>; }

export { templateProblem, api, Field, sources, purposes, states, selectClass, section };
