import { invoke } from '@tauri-apps/api/core';
import React, { useEffect, useState } from 'react';
import { Download, FolderOpen, RefreshCw, ShieldCheck } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { getStoredAuthToken } from '@/lib/auth';

type ProtectedAction = 'save' | 'check' | 'sync';

export function MasterDatabaseSettings({ onSaved }: { onSaved?: () => void }) {
  const [defaultPath, setDefaultPath] = useState('');
  const [dataRoot, setDataRoot] = useState('');
  const [pathWarning, setPathWarning] = useState('');
  const [dbPath, setDbPath] = useState('');
  const [branch, setBranch] = useState('');
  const [age, setAge] = useState(1440);
  const [apiConfigured, setApiConfigured] = useState(false);
  const [hasKey, setHasKey] = useState(false);
  const [dirty, setDirty] = useState(false);
  const [sync, setSync] = useState({
    baseUrl: '',
    headerName: 'X-API-Key',
    apiKey: '',
    agency: '',
    branch: '',
    pos: '',
    shop: '',
  });
  const [syncStatus, setSyncStatus] = useState('');
  const [lastSuccess, setLastSuccess] = useState('');
  const [branches, setBranches] = useState<Array<{ code: string; name?: string }>>([]);
  const [checkedPath, setCheckedPath] = useState('');
  const [operatorPassword, setOperatorPassword] = useState('');
  const [pin, setPin] = useState('');
  const [newPin, setNewPin] = useState('');
  const [pinConfirmation, setPinConfirmation] = useState('');
  const [pinChangeRequired, setPinChangeRequired] = useState(false);
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const [pendingAction, setPendingAction] = useState<ProtectedAction | null>(null);
  const [authorizationError, setAuthorizationError] = useState('');

  async function request(url: string, body?: unknown) {
    const r = await fetch(url, {
      method: body !== undefined ? 'POST' : 'GET',
      headers: { 'Content-Type': 'application/json' },
      ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
    });
    if (!r.headers.get('content-type')?.includes('application/json')) {
      throw new Error('บริการตั้งค่าแหล่งข้อมูลยังไม่พร้อม กรุณาตรวจรุ่นของบริการ');
    }
    const data = await r.json();
    if (!r.ok) throw new Error(data.error || 'ทำรายการไม่สำเร็จ');
    return data;
  }

  const refreshStatus = async () => {
    const s = await request('/api/storage');
    const appDefaultPath = String(s.defaultPath || s.standalonePath || '');
    setDefaultPath(appDefaultPath);
    setDataRoot(String(s.dataRoot || ''));
    if (s.dataRootWritable === false) {
      setPathWarning(
        `โฟลเดอร์โปรแกรม (${s.dataRoot}) ไม่มีสิทธิ์เขียนไฟล์ กรุณาย้ายโปรแกรมออกจาก Program Files เช่น D:\\AdaBarprint`,
      );
    }
    setPinChangeRequired(Boolean(s.adminPinChangeRequired));
    setDbPath(String(s.path || appDefaultPath || ''));
    setBranch(String(s.branch || ''));
    setAge(Number(s.maxAgeMinutes || 1440));
    setApiConfigured(Boolean(s.apiConfigured || s.sync?.config?.baseUrl));
    setHasKey(Boolean(s.sync?.hasApiKey));
    if (s.sync?.config) {
      setSync((prev) => ({
        ...prev,
        baseUrl: s.sync.config.baseUrl || '',
        headerName: s.sync.config.headerName || 'X-API-Key',
        agency: s.sync.config.agency || '',
        branch: s.sync.config.branch || s.branch || '',
        pos: s.sync.config.pos || '',
        shop: s.sync.config.shop || '',
        apiKey: '',
      }));
    }
    setSyncStatus(String(s.sync?.status || ''));
    setLastSuccess(String(s.sync?.lastSuccess || ''));
    setDirty(false);
    return s;
  };

  useEffect(() => {
    let active = true;
    refreshStatus()
      .then((s) => {
        if (!active) return;
        if (s.connected) {
          setMessage('เชื่อมต่อฐานข้อมูลสินค้าพร้อมใช้งานแล้ว');
        } else if (s.sync?.config?.baseUrl) {
          setMessage(String(s.sync?.status || 'ตั้งค่า API2PSMaster แล้ว สามารถกด “ดาวน์โหลดข้อมูลตอนนี้” ได้'));
        } else {
          setMessage(String(s.error || 'กำหนดไฟล์ AdaAccD.db หรือ API2PSMaster แล้วกดบันทึก'));
        }
      })
      .catch((e) => {
        if (active) setMessage(e instanceof Error ? e.message : String(e));
      });
    return () => {
      active = false;
    };
  }, []);

  function pinFields() {
    if ((pinChangeRequired || newPin) && (!newPin || newPin !== pinConfirmation)) {
      throw new Error('กรอก PIN ใหม่และยืนยันให้ตรงกัน');
    }
    return { pin, newPin };
  }

  const actionTitle =
    pendingAction === 'check'
      ? 'ตรวจสอบไฟล์ฐานข้อมูล AdaAccD.db'
      : pendingAction === 'sync'
        ? 'ดาวน์โหลดข้อมูลตอนนี้ (Manual Sync)'
        : 'บันทึกการตั้งค่าแหล่งข้อมูล';

  const runDirectSync = async () => {
    if (dirty) {
      setMessage('มีการแก้ไขค่าที่ยังไม่บันทึก กรุณากด “บันทึกการตั้งค่าแหล่งข้อมูล” ก่อนดาวน์โหลด');
      return;
    }
    if (!apiConfigured) {
      setMessage('ยังไม่ได้ตั้งค่า URL ของ API2PSMaster สำหรับดาวน์โหลด');
      return;
    }
    if (!getStoredAuthToken()) {
      beginAction('sync');
      return;
    }
    setBusy(true);
    setSyncStatus('กำลังดาวน์โหลด');
    try {
      const r = await request('/api/storage/sync', {});
      setMessage(String(r.message || 'ดาวน์โหลดข้อมูลครบแล้ว'));
      await refreshStatus();
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'ดาวน์โหลดข้อมูลไม่สำเร็จ';
      setMessage(msg);
      setSyncStatus(msg);
    } finally {
      setBusy(false);
    }
  };

  function beginAction(action: ProtectedAction) {
    let error = '';
    const trimmedPath = dbPath.trim();
    if (action === 'check') {
      if (!trimmedPath) error = 'กรุณาระบุพาธไฟล์ฐานข้อมูล AdaAccD.db';
    } else if (action === 'sync') {
      if (dirty) error = 'มีการแก้ไขค่าที่ยังไม่บันทึก กรุณากด “บันทึกการตั้งค่าแหล่งข้อมูล” ก่อนดาวน์โหลด';
      else if (!apiConfigured) error = 'ยังไม่ได้ตั้งค่า URL ของ API2PSMaster สำหรับดาวน์โหลด';
    } else if (action === 'save') {
      const hasUrl = Boolean(sync.baseUrl.trim());
      if (!trimmedPath && !defaultPath) {
        error = 'กรุณาระบุไฟล์ฐานข้อมูล AdaAccD.db';
      } else if (hasUrl) {
        if (!(sync.baseUrl.startsWith('https://') || sync.baseUrl.startsWith('http://'))) {
          error = 'URL ของ API2PSMaster ต้องขึ้นต้นด้วย http:// หรือ https://';
        } else if (!branch.trim() && !sync.branch.trim()) {
          error = 'กรุณาระบุรหัสสาขาสำหรับเชื่อมต่อ API2PSMaster';
        } else if (!sync.apiKey && !hasKey) {
          error = 'กรุณากรอก API Key สำหรับเชื่อมต่อ API2PSMaster';
        }
      }
    }
    if (error) {
      setMessage(error);
      return;
    }
    setPin('');
    setNewPin('');
    setPinConfirmation('');
    setAuthorizationError('');
    setPendingAction(action);
  }

  function closeAuthorization() {
    if (busy) return;
    setPendingAction(null);
    setPin('');
    setNewPin('');
    setPinConfirmation('');
    setAuthorizationError('');
  }

  async function confirmAction(event: React.FormEvent) {
    event.preventDefault();
    if (busy || !pendingAction) return;
    if (!pin.trim()) {
      setAuthorizationError('กรุณากรอก PIN ผู้ดูแลระบบเพื่อยืนยัน');
      return;
    }
    setBusy(true);
    setAuthorizationError('');
    try {
      if (pendingAction === 'check') {
        const r = await request('/api/storage/check', { path: dbPath.trim(), pin });
        const branchList = Array.isArray(r.branches) ? r.branches : [];
        setBranches(branchList);
        setCheckedPath(dbPath.trim());
        if (!branch.trim() && branchList.length === 1 && branchList[0]?.code) {
          setBranch(branchList[0].code);
          setSync((prev) => ({ ...prev, branch: branchList[0].code }));
        }
        setMessage(
          branchList.length === 0
            ? 'พบไฟล์ฐานข้อมูล แต่ยังไม่มีข้อมูลสาขา'
            : `ตรวจโครงสร้าง DB สำเร็จ · สินค้า ${r.productCount} รายการ · ผู้ใช้ ${r.userCount} ราย`,
        );
      } else if (pendingAction === 'sync') {
        const r = await request('/api/storage/sync', { pin });
        setMessage(String(r.message || 'ดาวน์โหลดข้อมูลครบแล้ว'));
        await refreshStatus();
      } else {
        const effectiveBranch = branch.trim() || sync.branch.trim();
        const hasUrl = Boolean(sync.baseUrl.trim());
        const payload = {
          operatorPassword: hasUrl ? '' : operatorPassword,
          mode: 'UNIFIED',
          path: dbPath.trim() || defaultPath,
          branch: effectiveBranch,
          maxAgeMinutes: age,
          sync: hasUrl
            ? {
                ...sync,
                baseUrl: sync.baseUrl.trim(),
                branch: effectiveBranch,
              }
            : null,
          ...pinFields(),
        };
        const r = await request('/api/storage', payload);
        setOperatorPassword('');
        if (newPin) setPinChangeRequired(false);
        setApiConfigured(hasUrl);
        setDirty(false);
        if (hasUrl) {
          setHasKey(true);
          setSync((prev) => ({ ...prev, apiKey: '' }));
        }
        setMessage(r.message);
        await refreshStatus();
        onSaved?.();
      }
      setPendingAction(null);
      setPin('');
      setNewPin('');
      setPinConfirmation('');
    } catch (e) {
      setAuthorizationError(e instanceof Error ? e.message : 'ทำรายการไม่สำเร็จ กรุณาลองใหม่');
    } finally {
      setBusy(false);
    }
  }

  async function act(fn: () => Promise<void>) {
    setBusy(true);
    try {
      await fn();
    } catch (e) {
      setMessage(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="min-w-0 rounded-xl border border-slate-200 bg-white p-4 sm:p-5 space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h3 className="font-semibold text-teal-800 flex items-center gap-1.5">
          <ShieldCheck className="h-4 w-4 text-[#007a87]" />
          แหล่งข้อมูลสินค้า &amp; API
        </h3>
        {dataRoot && (
          <span className="text-xs text-slate-500">
            Application Path: <code className="rounded bg-slate-100 px-1.5 py-0.5">{dataRoot}</code>
          </span>
        )}
      </div>

      {pathWarning && (
        <p role="alert" className="rounded-md border border-amber-300 bg-amber-50 p-3 text-sm text-amber-900">
          {pathWarning}
        </p>
      )}

      {/* ส่วนที่ 1: ไฟล์ฐานข้อมูลสินค้า AdaAccD.db */}
      <div className="rounded-lg border border-slate-200 bg-slate-50/60 p-4 space-y-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <span className="text-sm font-semibold text-slate-800">1. ฐานข้อมูลสินค้า (AdaAccD.db)</span>
          <div className="flex flex-wrap gap-2">
            {defaultPath && (
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="h-8 text-xs bg-white"
                disabled={busy}
                onClick={() => {
                  setDbPath(defaultPath);
                  setCheckedPath('');
                  setBranches([]);
                  setDirty(true);
                }}
              >
                <RefreshCw className="h-3.5 w-3.5 mr-1" />
                ใช้ Path มาตรฐานข้างโปรแกรม
              </Button>
            )}
            {'__TAURI_INTERNALS__' in window && (
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="h-8 text-xs bg-white"
                disabled={busy}
                onClick={() =>
                  act(async () => {
                    const selected = await invoke<string | null>('choose_master_database');
                    if (selected) {
                      setDbPath(selected);
                      setCheckedPath('');
                      setBranches([]);
                      setDirty(true);
                    }
                  })
                }
              >
                <FolderOpen className="h-3.5 w-3.5 mr-1" />
                เลือกไฟล์...
              </Button>
            )}
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="h-8 text-xs bg-white"
              disabled={busy}
              onClick={() => beginAction('check')}
            >
              ตรวจสอบไฟล์ DB
            </Button>
          </div>
        </div>

        <label className="block space-y-1.5 text-sm">
          <span className="text-slate-700">ไฟล์ฐานข้อมูล AdaAccD.db</span>
          <Input
            className="w-full min-w-0 bg-white font-mono text-xs"
            value={dbPath}
            disabled={busy}
            placeholder={defaultPath || 'เช่น D:\\AdaBarprint\\Data\\AdaAccD.db'}
            onChange={(e) => {
              setDbPath(e.target.value);
              setCheckedPath('');
              setBranches([]);
              setDirty(true);
            }}
          />
        </label>
      </div>

      {/* ส่วนที่ 2: ข้อมูลสาขา & เชื่อมต่อ API2PSMaster */}
      <div className="rounded-lg border border-slate-200 p-4 space-y-3">
        <span className="text-sm font-semibold text-slate-800 block">
          2. ข้อมูลสาขา &amp; เชื่อมต่อ API2PSMaster (ไม่บังคับ)
        </span>

        <div className="grid gap-3 sm:grid-cols-2">
          <label className="block space-y-1.5 text-sm">
            <span className="text-slate-700">รหัสสาขา</span>
            {checkedPath === dbPath.trim() && branches.length > 0 ? (
              <select
                className="block w-full h-9 rounded-md border border-slate-300 bg-white px-2 text-sm"
                value={branch}
                disabled={busy}
                onChange={(e) => {
                  setBranch(e.target.value);
                  setSync((prev) => ({ ...prev, branch: e.target.value }));
                  setDirty(true);
                }}
              >
                <option value="">-- เลือกสาขา --</option>
                {branches.map((b) => (
                  <option key={b.code} value={b.code}>
                    {b.code} · {b.name || b.code}
                  </option>
                ))}
              </select>
            ) : (
              <Input
                className="bg-white h-9 text-sm"
                value={branch}
                disabled={busy}
                placeholder="เช่น 00001"
                onChange={(e) => {
                  setBranch(e.target.value);
                  setSync((prev) => ({ ...prev, branch: e.target.value }));
                  setDirty(true);
                }}
              />
            )}
          </label>

          <label className="block space-y-1.5 text-sm">
            <span className="text-slate-700">รหัสตัวแทน (Agency)</span>
            <Input
              value={sync.agency}
              disabled={busy}
              placeholder="เช่น 00001"
              onChange={(e) => {
                setSync({ ...sync, agency: e.target.value });
                setDirty(true);
              }}
            />
          </label>

          <label className="block space-y-1.5 text-sm">
            <span className="text-slate-700">URL ของ API2PSMaster (ไม่บังคับ)</span>
            <Input
              value={sync.baseUrl}
              disabled={busy}
              placeholder="http://.../API2PSMaster/V6"
              autoComplete="off"
              onChange={(e) => {
                setSync({ ...sync, baseUrl: e.target.value });
                setDirty(true);
              }}
            />
          </label>

          <label className="block space-y-1.5 text-sm">
            <span className="text-slate-700">
              {hasKey ? 'API Key (เว้นว่างเพื่อใช้ค่าเดิม)' : 'API Key'}
            </span>
            <Input
              type="password"
              value={sync.apiKey}
              disabled={busy}
              autoComplete="off"
              onChange={(e) => {
                setSync({ ...sync, apiKey: e.target.value });
                setDirty(true);
              }}
            />
          </label>

          <label className="block space-y-1.5 text-sm">
            <span className="text-slate-700">รหัสเครื่อง POS (ไม่บังคับ)</span>
            <Input
              value={sync.pos}
              disabled={busy}
              placeholder="ปล่อยว่างหากไม่ใช่เครื่อง POS"
              onChange={(e) => {
                setSync({ ...sync, pos: e.target.value });
                setDirty(true);
              }}
            />
          </label>
        </div>

        <details className="pt-1">
          <summary className="text-xs text-slate-500 cursor-pointer">
            ตั้งค่าขั้นสูง (Header Name)
          </summary>
          <label className="block mt-2 space-y-1.5 text-sm max-w-xs">
            <span className="text-xs text-slate-600">ชื่อ Header สำหรับ API Key</span>
            <Input
              className="h-8 text-xs"
              value={sync.headerName}
              disabled={busy}
              onChange={(e) => {
                setSync({ ...sync, headerName: e.target.value });
                setDirty(true);
              }}
            />
          </label>
        </details>
      </div>

      {/* ปุ่มดำเนินการ: บันทึก + ดาวน์โหลดข้อมูลตอนนี้ (Manual Sync) */}
      <div className="flex flex-wrap items-center justify-end gap-2 pt-1">
        <Button
          type="button"
          variant="outline"
          disabled={busy || syncStatus === 'กำลังดาวน์โหลด'}
          onClick={() => void runDirectSync()}
        >
          <Download className="h-4 w-4 mr-1.5" />
          {syncStatus === 'กำลังดาวน์โหลด' ? 'กำลังดาวน์โหลด…' : 'ดาวน์โหลดข้อมูลตอนนี้ (Manual Sync)'}
        </Button>
        <Button
          type="button"
          className="bg-[#007a87] hover:bg-[#00606b] text-white"
          disabled={busy}
          onClick={() => beginAction('save')}
        >
          {busy ? 'กำลังบันทึก…' : 'บันทึกการตั้งค่าแหล่งข้อมูล'}
        </Button>
      </div>

      <div className="rounded-md bg-slate-50 p-3 text-xs text-slate-600 space-y-1" aria-live="polite">
        <p>
          โหมดเชื่อมต่อ:{' '}
          <strong className="text-slate-800">
            {apiConfigured ? 'API2PSMaster (Manual Sync)' : 'ฐานข้อมูลโดยตรง / นำเข้า Master'}
          </strong>
        </p>
        {syncStatus && <p>สถานะ: {syncStatus}</p>}
        {lastSuccess && (
          <p>
            อัปเดตล่าสุด:{' '}
            {lastSuccess.includes('T') || lastSuccess.includes('-')
              ? new Date(lastSuccess.replace(' ', 'T') + (lastSuccess.endsWith('Z') ? '' : 'Z')).toLocaleString('th-TH')
              : lastSuccess}
          </p>
        )}
      </div>

      <p className="text-sm break-words rounded-md bg-slate-50 border border-slate-200 p-3" role="status">
        {message}
      </p>

      <Dialog
        open={pendingAction !== null}
        onOpenChange={(open) => {
          if (!open) closeAuthorization();
        }}
      >
        <DialogContent
          className="sm:max-w-md"
          onEscapeKeyDown={(event) => {
            if (busy) event.preventDefault();
          }}
          onPointerDownOutside={(event) => {
            if (busy) event.preventDefault();
          }}
        >
          <DialogHeader>
            <DialogTitle>ยืนยันด้วย PIN ผู้ดูแลระบบ (IT / Admin)</DialogTitle>
            <DialogDescription>
              {actionTitle} · ใช้ PIN ผู้ดูแลของ AdaBarprint ไม่ใช่รหัสเข้าสู่ระบบ POS
            </DialogDescription>
          </DialogHeader>
          <form onSubmit={confirmAction} className="space-y-4">
            <label className="block space-y-2 text-sm">
              <span>PIN ผู้ดูแลระบบ</span>
              <Input
                autoFocus
                type="password"
                inputMode="numeric"
                autoComplete="off"
                value={pin}
                disabled={busy}
                onChange={(e) => setPin(e.target.value)}
                aria-invalid={Boolean(authorizationError)}
                aria-describedby={authorizationError ? 'source-pin-error' : undefined}
              />
            </label>
            {pendingAction === 'save' && !sync.baseUrl.trim() && <label className="grid gap-1 text-sm">
              ตั้งรหัสผ่านสำหรับผู้ใช้ LOCAL (เฉพาะเครื่องใหม่ที่ไม่มี POS/API)
              <input className="rounded border p-2" type="password" autoComplete="new-password" minLength={8} maxLength={64} value={operatorPassword} onChange={e=>setOperatorPassword(e.target.value)} placeholder="8–64 ตัวอักษร หรือเว้นว่างหากใช้บัญชีเดิม" />
            </label>}
            {pendingAction === 'save' && (
              <details open={pinChangeRequired || undefined}>
                <summary className="text-sm cursor-pointer">
                  {pinChangeRequired ? 'ตั้ง PIN ผู้ดูแลระบบใหม่ก่อนเริ่มใช้งาน' : 'เปลี่ยน PIN ผู้ดูแลระบบ (ถ้าต้องการ)'}
                </summary>
                <div className="space-y-3 mt-3">
                  <label className="block space-y-2 text-sm">
                    <span>PIN ใหม่ (ตัวเลข 6–12 หลัก)</span>
                    <Input
                      type="password"
                      inputMode="numeric"
                      autoComplete="new-password"
                      value={newPin}
                      disabled={busy}
                      onChange={(e) => setNewPin(e.target.value)}
                    />
                  </label>
                  <label className="block space-y-2 text-sm">
                    <span>ยืนยัน PIN ใหม่</span>
                    <Input
                      type="password"
                      inputMode="numeric"
                      autoComplete="new-password"
                      value={pinConfirmation}
                      disabled={busy}
                      onChange={(e) => setPinConfirmation(e.target.value)}
                    />
                  </label>
                </div>
              </details>
            )}
            {authorizationError && (
              <p id="source-pin-error" role="alert" className="text-sm text-red-700 bg-red-50 rounded-md p-3">
                {authorizationError}
              </p>
            )}
            <div className="flex justify-end gap-2">
              <Button type="button" variant="outline" disabled={busy} onClick={closeAuthorization}>
                ยกเลิก
              </Button>
              <Button type="submit" disabled={busy}>
                {busy ? 'กำลังดำเนินการ…' : pendingAction === 'sync' ? 'ยืนยันดาวน์โหลด' : pendingAction === 'check' ? 'ยืนยันตรวจสอบ' : 'ยืนยันบันทึก'}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>
    </section>
  );
}
