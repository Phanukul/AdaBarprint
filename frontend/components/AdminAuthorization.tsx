import { useEffect, useState } from 'react';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from './ui/dialog';
import { Button } from './ui/button';
import { Input } from './ui/input';

export function AdminAuthorization() {
  const [request, setRequest] = useState<{ finish: (allowed: boolean) => void } | null>(null);
  const [pin, setPin] = useState('');
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    const receive = (event: Event) => {
      setPin(''); setError(''); setBusy(false);
      setRequest((event as CustomEvent).detail);
    };
    window.addEventListener('adabarprint-admin-required', receive);
    return () => window.removeEventListener('adabarprint-admin-required', receive);
  }, []);
  const close = (allowed: boolean) => {
    request?.finish(allowed); setRequest(null); setPin(''); setError('');
  };
  async function verify() {
    if (!pin || busy) return;
    setBusy(true); setError('');
    try {
      const response = await fetch('/api/config/admin-verify', {
        method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ pin }),
      });
      const result = await response.json();
      if (!response.ok) throw new Error(result.error || 'ยืนยันสิทธิ์ไม่สำเร็จ');
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
      <form onSubmit={event => { event.preventDefault(); void verify(); }}>
        <label htmlFor="admin-authorization-pin" className="block mb-2 text-sm">PIN ผู้ดูแลระบบ</label>
        <Input id="admin-authorization-pin" type="password" autoComplete="off" inputMode="numeric" autoFocus
          value={pin} disabled={busy} onChange={event => setPin(event.target.value)} />
        {error && <p role="alert" className="text-sm text-red-600 mt-3">{error}</p>}
        <DialogFooter className="mt-5">
          <Button type="button" variant="outline" disabled={busy} onClick={() => close(false)}>ยกเลิก</Button>
          <Button type="submit" disabled={!pin || busy}>{busy ? 'กำลังตรวจสอบ' : 'ยืนยันและดำเนินการ'}</Button>
        </DialogFooter>
      </form>
    </DialogContent>
  </Dialog>;
}
