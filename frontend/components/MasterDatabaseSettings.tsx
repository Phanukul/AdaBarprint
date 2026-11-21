import { invoke } from '@tauri-apps/api/core';
import React, { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';

export function MasterDatabaseSettings({ onSaved }: { onSaved?: () => void }) {
  const [mode,setMode]=useState('POS_SHARED');
  const [version,setVersion]=useState(0);
  const [saved,setSaved]=useState(false);
  const [syncStatus,setSyncStatus]=useState('');
  const [lastSuccess,setLastSuccess]=useState('');
  const [sync,setSync]=useState({baseUrl:'',headerName:'X-API-Key',apiKey:'',agency:'',branch:'',pos:'',shop:'',intervalMinutes:60});
  const [hasKey,setHasKey]=useState(false);
  const [ownPath,setOwnPath]=useState('');
  const [path,setPath]=useState('');
  const [branch,setBranch]=useState('');
  const [age,setAge]=useState(1440);
  const [pin,setPin]=useState('');
  const [newPin,setNewPin]=useState('');
  const [pinConfirmation,setPinConfirmation]=useState('');
  const [pinChangeRequired,setPinChangeRequired]=useState(false);
  const [branches,setBranches]=useState<Array<{code:string;name?:string}>>([]);
  const [checkedPath,setCheckedPath]=useState('');
  const [message,setMessage]=useState('');
  const [busy,setBusy]=useState(false);
  async function request(url:string,body?:unknown) {
    const r=await fetch(url,{method:body?'POST':'GET',headers:{'Content-Type':'application/json'},...(body?{body:JSON.stringify(body)}:{})});
    if(!r.headers.get('content-type')?.includes('application/json'))throw new Error('บริการตั้งค่าแหล่งข้อมูลยังไม่พร้อม กรุณาตรวจรุ่นของบริการ');
    const data=await r.json();if(!r.ok)throw new Error(data.error||'ทำรายการไม่สำเร็จ');return data;
  }
  useEffect(()=>{let active=true;request('/api/storage').then(s=>{if(active){setVersion(s.sourceVersion||0);setPinChangeRequired(Boolean(s.adminPinChangeRequired));setMode(s.mode||'POS_SHARED');setOwnPath(s.standalonePath||'');setSaved(s.mode==='STANDALONE');setHasKey(Boolean(s.sync?.hasApiKey));if(s.sync?.config?.baseUrl)setSync(prev=>({...prev,...s.sync.config,apiKey:''}));setPath(s.path);setBranch(s.branch);setAge(s.maxAgeMinutes);setMessage(s.mode==='STANDALONE'?(s.sync?.status||'รอดาวน์โหลดข้อมูลครั้งแรก'):s.connected?'เชื่อมต่อฐานข้อมูล StoreFront แบบอ่านอย่างเดียวแล้ว':s.error||'ตั้งค่าแหล่งข้อมูลก่อนเริ่มใช้งาน');}}).catch(e=>{if(active)setMessage(e.message);});return()=>{active=false;};},[]);
  useEffect(()=>{
    if(mode!=='STANDALONE'||!saved)return;
    let active=true;
    const refresh=async()=>{try{const s=await request('/api/storage');if(active){setSyncStatus(s.sync?.status||'');setLastSuccess(s.sync?.lastSuccess||'');}}catch{ /* Keep the last known download status during a temporary connection failure. */ }};
    void refresh();const timer=window.setInterval(()=>{void refresh();},5000);
    return()=>{active=false;window.clearInterval(timer);};
  },[mode,saved]);
  function pinFields(){if((pinChangeRequired||newPin)&&(!newPin||newPin!==pinConfirmation))throw new Error('กรอก PIN ใหม่และยืนยันให้ตรงกัน');return {pin,newPin};}
  async function act(fn:()=>Promise<void>){setBusy(true);try{await fn();}catch(e){setMessage(e instanceof Error?e.message:String(e));}finally{setBusy(false);}}
  return <section className="min-w-0 rounded-xl border border-slate-200 bg-white p-4 sm:p-5 space-y-4">
    <div><h3 className="font-semibold text-teal-800">แหล่งข้อมูลสินค้า</h3><p className="mt-1 text-sm text-slate-600">เลือกใช้ข้อมูลจาก POS หรือดาวน์โหลดข้อมูลสำหรับเครื่องนี้</p></div>
    <label className="block space-y-2 text-sm"><span>การใช้งานเครื่องนี้</span><select className="block w-full h-10 rounded-md border bg-white px-2" value={mode} disabled={busy} onChange={e=>{setMode(e.target.value);setSaved(false);}}><option value="POS_SHARED">ใช้ฐานข้อมูลของ StoreFront</option><option value="STANDALONE" disabled={version<2}>เครื่องแยกจาก POS — ดาวน์โหลดข้อมูลเอง</option></select></label>
    <label className="block space-y-2 text-sm"><span>PIN ผู้ดูแลระบบ</span><Input type="password" autoComplete="off" value={pin} disabled={busy} onChange={e=>setPin(e.target.value)}/></label>
    <details open={pinChangeRequired||undefined}><summary className="text-sm cursor-pointer">{pinChangeRequired?'ตั้ง PIN ผู้ดูแลระบบใหม่ก่อนเริ่มใช้งาน':'เปลี่ยน PIN ผู้ดูแลระบบ'}</summary><div className="grid sm:grid-cols-2 gap-3 mt-3">
      <label className="space-y-2 text-sm"><span>PIN ใหม่ (ตัวเลข 6–12 หลัก)</span><Input type="password" autoComplete="new-password" value={newPin} disabled={busy} onChange={e=>setNewPin(e.target.value)}/></label>
      <label className="space-y-2 text-sm"><span>ยืนยัน PIN ใหม่</span><Input type="password" autoComplete="new-password" value={pinConfirmation} disabled={busy} onChange={e=>setPinConfirmation(e.target.value)}/></label>
    </div></details>
    {mode==='STANDALONE'?<>
      <label className="block space-y-2 text-sm"><span>ไฟล์ฐานข้อมูลสำหรับดาวน์โหลด</span><Input value={ownPath} disabled={busy||version<3} onChange={e=>{setOwnPath(e.target.value);setSaved(false);}}/><span className="block text-xs text-slate-500">ระบุพาธเต็มลงท้ายด้วย AdaAccD.db ระบบสร้างไฟล์ใหม่ให้ ห้ามเลือกฐานข้อมูลที่ POS ใช้งานอยู่</span></label>
      <div className="grid gap-4 sm:grid-cols-2">
        {([{key:'baseUrl',label:'URL ของ API2PSMaster'},{key:'agency',label:'รหัสตัวแทน (Agency)'},{key:'branch',label:'รหัสสาขา'},{key:'pos',label:'รหัสเครื่อง POS'},{key:'apiKey',label:hasKey?'API Key (เว้นว่างเพื่อใช้ค่าเดิม)':'API Key'}] as const).map(field=><label key={field.key} className="block space-y-2 text-sm"><span>{field.label}</span><Input value={sync[field.key]} type={field.key==='apiKey'?'password':'text'} autoComplete="off" disabled={busy} onChange={e=>{setSync({...sync,[field.key]:e.target.value});setSaved(false);}}/></label>)}
        <label className="block space-y-2 text-sm"><span>ดาวน์โหลดทุก (นาที)</span><Input type="number" min={1} max={1440} value={sync.intervalMinutes} disabled={busy} onChange={e=>{setSync({...sync,intervalMinutes:Number(e.target.value)});setSaved(false);}}/></label>
      </div>
      <details><summary className="text-sm cursor-pointer">ตั้งค่าขั้นสูงสำหรับผู้ดูแลระบบ</summary><p className="mt-2 text-xs text-slate-500">ใช้ค่าเดิมได้ เปลี่ยนเฉพาะเมื่อผู้ดูแล API แจ้งให้เปลี่ยน</p><label className="block mt-3 space-y-2 text-sm"><span>ชื่อ Header สำหรับ API Key</span><Input value={sync.headerName} disabled={busy} onChange={e=>{setSync({...sync,headerName:e.target.value});setSaved(false);}}/></label></details>

      <div className="flex flex-wrap gap-2"><Button disabled={busy||!pin||!sync.baseUrl||!sync.branch||(!sync.apiKey&&!hasKey)} onClick={()=>act(async()=>{const r=await request('/api/storage',{mode,path:ownPath,sync,maxAgeMinutes:age,...pinFields()});setSaved(true);setHasKey(true);if(newPin){setPin(newPin);setNewPin('');setPinConfirmation('');setPinChangeRequired(false);}setSync({...sync,apiKey:''});setMessage(r.message+' · ระบบจะดาวน์โหลดข้อมูลตามรอบที่ตั้งไว้');})}>บันทึกการเชื่อมต่อ</Button><Button variant="outline" disabled={busy||!saved||!pin||syncStatus==='กำลังดาวน์โหลด'} onClick={()=>act(async()=>{setMessage('กำลังดาวน์โหลดข้อมูล กรุณารอ');const r=await request('/api/storage/sync',{pin});setMessage(r.message);setSyncStatus(r.message);setPin('');})}>{busy?'กำลังดำเนินการ…':'ดาวน์โหลดข้อมูลตอนนี้'}</Button></div>
      <div className="rounded-md bg-slate-50 p-3 text-sm space-y-1" aria-live="polite">
        <p>สถานะดาวน์โหลด: {syncStatus||'ยังไม่ได้บันทึกการเชื่อมต่อ'}</p>
        <p>สำเร็จล่าสุด: {lastSuccess?new Date(lastSuccess.replace(' ','T')+'Z').toLocaleString('th-TH'):'ยังไม่มี'}</p>
        <p className="text-slate-600">ระบบตรวจรอบเมื่อบริการทำงาน และดาวน์โหลดซ้ำตามจำนวนนาทีที่บันทึกไว้ การดาวน์โหลดข้อมูลไม่ใช่การสั่งพิมพ์</p>
      </div>
    </>:<>
    <label className="block space-y-2 text-sm"><span>ไฟล์ฐานข้อมูล AdaAccD.db</span><Input className="w-full min-w-0" value={path} onChange={e=>{setPath(e.target.value);setCheckedPath('');setBranches([]);}} placeholder="C:\ProgramData\AdaPos6.0\AdaStoreFront\Data\AdaAccD.db"/><span className="block text-xs text-slate-500">วางพาธเต็มของไฟล์บนเครื่อง POS แล้วกดตรวจสอบ ระบบจะไม่สร้างไฟล์หรือเขียนข้อมูลลงฐานนี้</span></label>
    {'__TAURI_INTERNALS__' in window&&<Button variant="outline" disabled={busy} onClick={()=>act(async()=>{const selected=await invoke<string|null>('choose_master_database');if(selected){setPath(selected);setCheckedPath('');setBranches([]);}})}>เลือกไฟล์ฐานข้อมูล</Button>}
    <Button variant="outline" disabled={busy||!path.trim()||!pin} onClick={()=>act(async()=>{const r=await request('/api/storage/check',{path,pin});setBranches(r.branches);setCheckedPath(path);setMessage(r.branches.length===0?'พบฐานข้อมูล แต่ยังไม่มีข้อมูลสาขา กรุณาเปิด StoreFront และอัปเดตข้อมูลก่อน':`ตรวจโครงสร้างสำเร็จ · สินค้า ${r.productCount} รายการ · ผู้ใช้ ${r.userCount} ราย${r.productCount===0||r.userCount===0?' · ต้องอัปเดตข้อมูลจาก StoreFront ก่อนใช้งาน':' · กรุณายืนยันสาขาและบันทึก'}`);})}>ตรวจสอบการเชื่อมต่อ</Button>
    {checkedPath===path&&branches.length>0&&<div className="grid gap-4 sm:grid-cols-2">
      <label className="space-y-2 text-sm"><span>สาขาของเครื่องพิมพ์</span><select className="block w-full h-10 rounded-md border bg-white px-2" value={branch} onChange={e=>setBranch(e.target.value)}><option value="">เลือกสาขา</option>{branches.map(b=><option key={b.code} value={b.code}>{b.code} · {b.name||b.code}</option>)}</select></label>
      <label className="space-y-2 text-sm"><span>อายุข้อมูลสูงสุดสำหรับ Auto (นาที)</span><Input type="number" min={1} max={10080} value={age} onChange={e=>setAge(Number(e.target.value))}/></label>

      <div className="flex items-end"><Button className="bg-teal-700 text-white" disabled={busy||!branch||!pin} onClick={()=>act(async()=>{const r=await request('/api/storage',{mode,path,branch,maxAgeMinutes:age,...pinFields()});setPin('');setMessage(r.message);onSaved?.();})}>บันทึกแหล่งข้อมูล</Button></div>
    </div>}
    </>}
    <p className="text-sm break-words rounded-md bg-slate-50 p-3" role="status">{message}</p>
    <p className="text-xs text-slate-500">การเปลี่ยนฐานหรือสาขาต้องไม่มีงานค้าง ระบบจะปิดตาราง Auto เพื่อให้ตรวจสอบก่อนเปิดใช้งานอีกครั้ง</p>
  </section>;
}
