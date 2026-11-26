import { invoke } from '@tauri-apps/api/core';
import React, { useEffect, useState } from 'react';
import { Download, FolderOpen, ShieldCheck } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { getStoredAuthToken } from '@/lib/auth';

type ProtectedAction = 'save' | 'check' | 'sync';

export function MasterDatabaseSettings({ onSaved, readOnly = false, lockControl }: { onSaved?: () => void; readOnly?: boolean; lockControl?: React.ReactNode }) {
  const [defaultPath, setDefaultPath] = useState('');
  const [pathWarning, setPathWarning] = useState('');
  const [useApi, setUseApi] = useState(false);
  const [savedSource, setSavedSource] = useState('');
  const [dbPath, setDbPath] = useState('');
  const [editingPath, setEditingPath] = useState(false);
  const [databaseStatus, setDatabaseStatus] = useState('กำลังตรวจสอบฐานข้อมูล…');
  const [branch, setBranch] = useState('');
  const [age, setAge] = useState(1440);
  const [apiConfigured, setApiConfigured] = useState(false);
  const [hasKey, setHasKey] = useState(false);
  const [savedEndpoint, setSavedEndpoint] = useState({ url: '', branch: '', agency: '' });
  const [showErrors, setShowErrors] = useState(false);
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
    if (s.dataRootWritable === false) {
      setPathWarning(
        `โฟลเดอร์โปรแกรม (${s.dataRoot}) ไม่มีสิทธิ์เขียนไฟล์ กรุณาย้ายโปรแกรมออกจาก Program Files เช่น D:\\AdaBarprint`,
      );
    }
    setPinChangeRequired(Boolean(s.adminPinChangeRequired));
    const mode = s.configured && s.mode === 'POS_SHARED' ? 'POS_SHARED' : 'STANDALONE';
    setSavedSource(s.configured ? mode : '');
    setUseApi(Boolean(s.apiConfigured || s.sync?.config?.baseUrl));
    const path = String(s.path || appDefaultPath || '');
    setDbPath(path);
    setEditingPath(Boolean(s.error) || !path);
    setDatabaseStatus(s.connected ? 'พร้อมใช้งาน' : s.error || (path ? 'พบฐานข้อมูลแล้ว' : 'กรุณาเลือกไฟล์ฐานข้อมูล'));
    setBranch(String(s.branch || ''));
    setAge(Number(s.maxAgeMinutes || 1440));
    setApiConfigured(Boolean(s.apiConfigured || s.sync?.config?.baseUrl));
    setHasKey(Boolean(s.sync?.hasApiKey));
    setShowErrors(false);
    setSavedEndpoint({
      url: (s.sync?.config?.baseUrl || '').trim(),
      branch: (s.sync?.config?.branch || s.branch || '').trim(),
      agency: (s.sync?.config?.agency || '').trim(),
    });
    const config = s.sync?.config;
    setSync({
      baseUrl: config?.baseUrl || '',
      headerName: config?.headerName || 'X-API-Key',
      agency: config?.agency || '',
      branch: config?.branch || s.branch || '',
      pos: config?.pos || '',
      shop: config?.shop || '',
      apiKey: '',
    });
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
          setMessage(s.error ? '' : 'กดบันทึกการตั้งค่าเพื่อใช้ฐานข้อมูลที่พบ');
        }
      })
      .catch((e) => {
        if (active) {
          setDatabaseStatus('โหลดสถานะฐานข้อมูลไม่สำเร็จ');
          setEditingPath(true);
          setMessage(e instanceof Error ? e.message : String(e));
        }
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

  const canReuseKey = hasKey && savedEndpoint.url === sync.baseUrl.trim()
    && savedEndpoint.branch === branch.trim() && savedEndpoint.agency === sync.agency.trim();
  const needsBranch = useApi || (checkedPath === dbPath.trim() && branches.length > 1);
  type Field = 'path' | 'branch' | 'url' | 'key';
  const errors: Partial<Record<Field, string>> = {};
  if (!dbPath.trim()) errors.path = 'กรุณาระบุไฟล์ AdaAccD.db';
  if (needsBranch && !branch.trim()) errors.branch = 'กรุณาระบุสาขา';
  if (useApi) {
    if (!sync.baseUrl.trim()) errors.url = 'กรุณากรอก URL ของ API2PSMaster';
    else if (!/^https?:\/\/[^/\s]+/i.test(sync.baseUrl.trim())) errors.url = 'ระบุ URL ที่ขึ้นต้นด้วย http:// หรือ https:// พร้อมชื่อเซิร์ฟเวอร์';
    if (!sync.apiKey.trim() && !(canReuseKey && sync.apiKey === '')) errors.key = 'กรุณากรอก API Key';
  }

  function fieldProps(field: Field, required: boolean) {
    const invalid = showErrors && Boolean(errors[field]);
    return {
      id: 'source-' + field, 'aria-required': required, 'aria-invalid': invalid,
      'aria-describedby': invalid ? 'source-error-' + field : undefined,
    };
  }

  function fieldError(field: Field) {
    return showErrors && errors[field] &&
      <span id={'source-error-' + field} className="block text-xs text-red-700">{errors[field]}</span>;
  }

  function beginAction(action: ProtectedAction) {
    let error = '';
    const trimmedPath = dbPath.trim();
    if (action === 'check') {
      if (!trimmedPath) error = 'กรุณาระบุพาธไฟล์ฐานข้อมูล AdaAccD.db';
    } else if (action === 'sync') {
      if (dirty) error = 'มีการแก้ไขค่าที่ยังไม่บันทึก กรุณากด “บันทึกการตั้งค่าแหล่งข้อมูล” ก่อนดาวน์โหลด';
      else if (!apiConfigured) error = 'ยังไม่ได้ตั้งค่า URL ของ API2PSMaster สำหรับดาวน์โหลด';
    } else if (action === 'save') {
      setShowErrors(true);
      const first = Object.keys(errors)[0];
      if (first) {
        document.getElementById('source-' + first)?.focus();
        return;
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
        setDatabaseStatus('ตรวจสอบไฟล์ผ่านแล้ว');
        if (branchList.length === 1 && branchList[0]?.code) {
          setBranch(branchList[0].code);
        } else if (!branchList.some((b: {code: string}) => b.code === branch)) {
          setBranch('');
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
        const effectiveBranch = branch.trim();
        const hasUrl = useApi;
        const payload = {
          operatorPassword: !hasUrl ? operatorPassword : '',
          mode: 'UNIFIED',
          path: dbPath.trim() || defaultPath,
          branch: effectiveBranch,
          maxAgeMinutes: age,
          sync: hasUrl
            ? {
                ...sync,
                baseUrl: sync.baseUrl.trim(),
                headerName: sync.headerName.trim() || 'X-API-Key',
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

  const pathFields = (
    <div className="rounded-lg border border-slate-200 bg-slate-50/60 p-4 space-y-3">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <span className="text-sm font-semibold text-slate-800">ฐานข้อมูลที่ใช้พิมพ์ป้าย</span>
          <div className="flex flex-wrap gap-2">
            {!editingPath && dbPath && (
              <Button type="button" size="sm" variant="outline" disabled={busy}
                onClick={() => setEditingPath(true)}>เปลี่ยนไฟล์ฐานข้อมูล</Button>
            )}
            {editingPath && '__TAURI_INTERNALS__' in window && (
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
                      setDatabaseStatus('รอตรวจสอบไฟล์ที่เลือก');
                      setBranch('');
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
              disabled={busy || !dbPath.trim()}
              onClick={() => beginAction('check')}
            >
              ตรวจสอบไฟล์ DB
            </Button>
          </div>
        </div>

        <p role="status" className="text-sm font-medium">{databaseStatus}</p>
        {editingPath ? <label className="block space-y-1.5 text-sm">
          <span className="text-slate-700">ไฟล์ฐานข้อมูล AdaAccD.db</span>
          <Input
            className="w-full min-w-0 bg-white font-mono text-xs"
            {...fieldProps('path', true)}
            value={dbPath}
            disabled={busy}
            placeholder={defaultPath || 'เช่น D:\\AdaBarprint\\Data\\AdaAccD.db'}
            onChange={(e) => {
              setDbPath(e.target.value);
              setDatabaseStatus('รอตรวจสอบไฟล์ที่ระบุ');
              setBranch('');
              setCheckedPath('');
              setBranches([]);
              setDirty(true);
            }}
          />
        {fieldError('path')}
          </label> : <p className="break-all font-mono text-xs text-slate-600">{dbPath}</p>}
      </div>
  );

  return (
    <section className="min-w-0 rounded-xl border border-slate-200 bg-white p-4 sm:p-5 space-y-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <h3 className="font-semibold text-teal-800 flex items-center gap-1.5">
          <ShieldCheck className="h-4 w-4 text-[#007a87]" />
          แหล่งข้อมูลสินค้า
        </h3>
        {lockControl}
      </div>

      {pathWarning && (
        <p role="alert" className="rounded-md border border-amber-300 bg-amber-50 p-3 text-sm text-amber-900">
          {pathWarning}
        </p>
      )}

      {readOnly ? (
        <dl className="grid gap-4 rounded-lg border border-slate-200 p-4 text-sm sm:grid-cols-2">
          {[
            ['ฐานข้อมูลที่ใช้พิมพ์ป้าย', dbPath],
            ['สถานะฐานข้อมูล', databaseStatus],
            ['รหัสสาขา', branch],
            ['ดาวน์โหลดจากส่วนกลาง (API2PSMaster)', useApi ? 'เปิดใช้งาน' : 'ปิดใช้งาน'],
            ...(useApi ? [
              ['URL ของ API2PSMaster', sync.baseUrl],
              ['รหัสตัวแทน Agency', sync.agency],
              ['รหัสเครื่อง POS', sync.pos],
              ['ชื่อ Header สำหรับ API Key', sync.headerName],
            ] : []),
          ].map(([label, value]) => (
            <div key={label} className="min-w-0">
              <dt className="text-slate-600">{label}</dt>
              <dd className="mt-1 break-all font-medium">{value || '—'}</dd>
            </div>
          ))}
        </dl>
      ) : <>
      {pathFields}

        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" checked={useApi} disabled={busy}
            onChange={e => {
              setUseApi(e.target.checked);
              setDirty(true);
              setMessage('');
            }} />
          เปิดใช้การดาวน์โหลดจากส่วนกลาง (API2PSMaster)
        </label>

      {/* ส่วนที่ 2: ข้อมูลสาขา & เชื่อมต่อ API2PSMaster */}
      <div className="rounded-lg border border-slate-200 p-4 space-y-3">
        <span className="text-sm font-semibold text-slate-800 block">
          {useApi ? 'การเชื่อมต่อ API2PSMaster' : 'สาขาสำหรับพิมพ์ป้าย'}
        </span>

        <div className="grid gap-3 sm:grid-cols-2">
          <label className="block space-y-1.5 text-sm">
            <span className="text-slate-700">รหัสสาขา</span>
            {checkedPath === dbPath.trim() && branches.length > 0 ? (
              <select
                className="block w-full h-9 rounded-md border border-slate-300 aria-invalid:border-red-600 bg-white px-2 text-sm"
                {...fieldProps('branch', needsBranch)}
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
                {...fieldProps('branch', needsBranch)}
                value={branch}
                disabled={busy}
                placeholder={useApi ? 'ระบุรหัสสาขา' : 'ตรวจสอบไฟล์เพื่อดูสาขา'}
                onChange={(e) => {
                  setBranch(e.target.value);
                  setSync((prev) => ({ ...prev, branch: e.target.value }));
                  setDirty(true);
                }}
              />
            )}
          {fieldError('branch')}
          </label>

          {useApi && <>
          <label className="block space-y-1.5 text-sm">
            <span className="text-slate-700">URL ของ API2PSMaster</span>
            <Input
              {...fieldProps('url', true)}
              value={sync.baseUrl}
              disabled={busy}
              placeholder="http://.../API2PSMaster/V6"
              autoComplete="off"
              onChange={(e) => {
                setSync({ ...sync, baseUrl: e.target.value });
                setDirty(true);
              }}
            />
          {fieldError('url')}
          </label>

          <label className="block space-y-1.5 text-sm">
            <span className="text-slate-700">
              {canReuseKey ? 'API Key (เว้นว่างเพื่อใช้ค่าเดิม)' : 'API Key'}
            </span>
            <Input
              type="password"
              {...fieldProps('key', !canReuseKey)}
              value={sync.apiKey}
              disabled={busy}
              autoComplete="off"
              onChange={(e) => {
                setSync({ ...sync, apiKey: e.target.value });
                setDirty(true);
              }}
            />
          {fieldError('key')}
          </label>

          <label className="block space-y-1.5 text-sm">
            <span className="text-slate-700">รหัสตัวแทน Agency (ไม่บังคับ)</span>
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
          <label className="block space-y-1.5 text-sm">
            <span className="text-slate-700">ชื่อ Header สำหรับ API Key (ไม่บังคับ)</span>
            <Input
              placeholder="X-API-Key"
              value={sync.headerName}
              disabled={busy}
              onChange={(e) => {
                setSync({ ...sync, headerName: e.target.value });
                setDirty(true);
              }}
            />
          </label>
          </>}
        </div>
      </div>

      {/* ปุ่มดำเนินการ: บันทึก + ดาวน์โหลดข้อมูลตอนนี้ (Manual Sync) */}
      <div className="flex flex-wrap items-center justify-end gap-2 pt-1">
        {useApi && <Button
          type="button"
          variant="outline"
          disabled={busy || dirty || !apiConfigured || syncStatus === 'กำลังดาวน์โหลด'}
          onClick={() => void runDirectSync()}
        >
          <Download className="h-4 w-4 mr-1.5" />
          {syncStatus === 'กำลังดาวน์โหลด' ? 'กำลังดาวน์โหลด…' : 'ดาวน์โหลดข้อมูลตอนนี้ (Manual Sync)'}
        </Button>}
        <Button
          type="button"
          className="bg-[#007a87] hover:bg-[#00606b] text-white"
          disabled={busy}
          onClick={() => beginAction('save')}
        >
          {busy ? 'กำลังบันทึก…' : 'บันทึกการตั้งค่าแหล่งข้อมูล'}
        </Button>
      </div>

      </>}

      <div className="rounded-md bg-slate-50 p-3 text-xs text-slate-600 space-y-1" aria-live="polite">
        <p>
          แหล่งข้อมูลที่บันทึกไว้:{' '}
          <strong className="text-slate-800">
            {apiConfigured ? 'ฐานข้อมูลบนเครื่องนี้ · เปิดใช้การดาวน์โหลดจากส่วนกลาง' : savedSource === 'POS_SHARED' ? 'ฐานข้อมูลของ POS' : savedSource ? 'ฐานข้อมูลบนเครื่องนี้' : 'ยังไม่ได้ตั้งค่า'}
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
            {pendingAction === 'save' && !useApi && <details><summary className="cursor-pointer text-sm">ตั้งผู้ใช้ LOCAL สำหรับฐานใหม่</summary><label className="grid gap-1 text-sm">
              ตั้งรหัสผ่านสำหรับผู้ใช้ LOCAL (เฉพาะเครื่องใหม่ที่ไม่มี POS/API)
              <input className="rounded border p-2" type="password" autoComplete="new-password" minLength={8} maxLength={64} value={operatorPassword} onChange={e=>setOperatorPassword(e.target.value)} placeholder="8–64 ตัวอักษร หรือเว้นว่างหากใช้บัญชีเดิม" />
            </label></details>}
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
