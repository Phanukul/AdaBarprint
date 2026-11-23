import { useEffect, useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from './ui/dialog';
import { Button } from './ui/button';
import { Input } from './ui/input';

export function AdminAuthorization() {
  const [request, setRequest] = useState<{ finish: (allowed: boolean) => void } | null>(null);
  const [pin, setPin] = useState('');
  const [newPin, setNewPin] = useState('');
  const [confirmNewPin, setConfirmNewPin] = useState('');
  const [pinChangeRequired, setPinChangeRequired] = useState(false);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    const receive = (event: Event) => {
      setPin(''); setNewPin(''); setConfirmNewPin(''); setPinChangeRequired(false); setError(''); setBusy(false);
      setRequest((event as CustomEvent).detail);
    };
    window.addEventListener('adabarprint-admin-required', receive);
    return () => window.removeEventListener('adabarprint-admin-required', receive);
  }, []);
  const close = (allowed: boolean) => {
    request?.finish(allowed); setRequest(null); setPin(''); setNewPin(''); setConfirmNewPin(''); setPinChangeRequired(false); setError('');
  };
  async function verify() {
    if (!pin || busy) return;
    if (pinChangeRequired && (!newPin.trim() || newPin.trim() !== confirmNewPin.trim())) {
      setError('กรุณากรอก PIN ใหม่ (ตัวเลข 6–12 หลัก) และยืนยันให้ตรงกัน');
      return;
    }
    setBusy(true); setError('');
    try {
      const response = await fetch('/api/config/admin-verify', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ pin, ...(pinChangeRequired ? { newPin: newPin.trim() } : {}) }),
      });
      const result = await response.json();
      if (!response.ok) {
        if (result?.pinChangeRequired) setPinChangeRequired(true);
        throw new Error(result.error || 'ยืนยันสิทธิ์ไม่สำเร็จ');
      }
      close(true);
    } catch (e) { setError(e instanceof Error ? e.message : 'ไม่สามารถเชื่อมต่อบริการได้'); }
    finally { setBusy(false); }
  }
  return <Dialog open={!!request} onOpenChange={open => { if (!open && !busy) close(false); }}>
    <DialogContent onInteractOutside={event => { if (busy) event.preventDefault(); }}>
      <DialogHeader>
        <DialogTitle>ยืนยันสิทธิ์ผู้ดูแล</DialogTitle>
        <DialogDescription>กรอก PIN ผู้ดูแลเพื่อบันทึกการเปลี่ยนแปลง สิทธิ์นี้ใช้ได้ 10 นาที และสิ้นสุดเมื่อล็อกหน้าจอ</DialogDescription>
      </DialogHeader>
      <form onSubmit={event => { event.preventDefault(); void verify(); }} className="space-y-3">
        <div>
          <label htmlFor="admin-authorization-pin" className="block mb-2 text-sm">
            {pinChangeRequired ? 'PIN ผู้ดูแลระบบปัจจุบัน (ค่าเริ่มต้น 1234)' : 'PIN ผู้ดูแลระบบ'}
          </label>
          <Input id="admin-authorization-pin" type="password" autoComplete="off" inputMode="numeric" autoFocus
            value={pin} disabled={busy} onChange={event => setPin(event.target.value)} />
        </div>
        {pinChangeRequired && (
          <>
            <div>
              <label htmlFor="admin-authorization-new-pin" className="block mb-1.5 text-sm">ตั้ง PIN ใหม่ (ตัวเลข 6–12 หลัก)</label>
              <Input id="admin-authorization-new-pin" type="password" autoComplete="new-password" inputMode="numeric"
                value={newPin} disabled={busy} onChange={event => setNewPin(event.target.value)} />
            </div>
            <div>
              <label htmlFor="admin-authorization-confirm-pin" className="block mb-1.5 text-sm">ยืนยัน PIN ใหม่</label>
              <Input id="admin-authorization-confirm-pin" type="password" autoComplete="new-password" inputMode="numeric"
                value={confirmNewPin} disabled={busy} onChange={event => setConfirmNewPin(event.target.value)} />
            </div>
          </>
        )}
        {error && <p role="alert" className="text-sm text-red-600 mt-3">{error}</p>}
        <DialogFooter className="mt-5">
          <Button type="button" variant="outline" disabled={busy} onClick={() => close(false)}>ยกเลิก</Button>
          <Button type="submit" disabled={!pin || (pinChangeRequired && (!newPin || !confirmNewPin)) || busy}>{busy ? 'กำลังตรวจสอบ' : 'ยืนยันและดำเนินการ'}</Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>;
}
