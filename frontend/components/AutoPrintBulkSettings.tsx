import React, { useMemo, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { SearchableSelect } from '@/components/ui/searchable-select';
import { resolveTemplateRoute } from '@/lib/printProfile';
import { Input } from '@/components/ui/input';
import {
  api,
  Field,
  purposes,
  selectClass,
  section,
  templateProblem,
  type Props,
  type ProfileTarget,
  type Rule,
  type Center,
  type Template,
  type Candidate,
} from './autoPrintShared';

const EVENT_CHOICES = [
  {
    value: 'PRICE_CHG',
    label: 'ราคาสินค้าเปลี่ยน',
    shortLabel: 'ราคาสินค้าเปลี่ยน',
  },
  {
    value: 'PROMO_START',
    label: 'เริ่มโปรโมชัน',
    shortLabel: 'เริ่มโปรโมชัน',
  },
  {
    value: 'PROMO_END_REVERT',
    label: 'สิ้นสุดโปรโมชัน',
    shortLabel: 'สิ้นสุดโปรโมชัน',
  },
  {
    value: 'NEW_PDT',
    label: 'มีสินค้าใหม่',
    shortLabel: 'มีสินค้าใหม่',
  },
] as const;

const OFFSET_CHOICES = [
  { value: 0, label: 'วันนี้ (0)' },
  { value: 1, label: 'พรุ่งนี้ (+1)' },
  { value: -1, label: 'เมื่อวาน (-1)' },
] as const;

function nextProfileCode(existing: Rule[]): string {
  const used = new Set(existing.map((r) => r.code.trim().toUpperCase()));
  for (let n = 1; n <= 999; n++) {
    const cand = String(n).padStart(3, '0');
    if (!used.has(cand)) return cand;
  }
  return String(existing.length + 1).padStart(3, '0');
}

function targetDatePreview(offsetDays: number): string {
  const d = new Date();
  d.setDate(d.getDate() + (Number.isFinite(offsetDays) ? offsetDays : 0));
  return d.toISOString().slice(0, 10);
}

function emptyTarget(): ProfileTarget {
  return {
    size: '',
    template: '',
    printer: '',
    tray: '',
    protocol: 'WINDOWS',
    ip: '',
    port: 9100,
    dpi: 203,
    quantity: 1,
  };
}

export function normalizeTargets(rule: Rule): ProfileTarget[] {
  if (Array.isArray(rule.targets) && rule.targets.length > 0) {
    return rule.targets.map((t) => ({ ...t }));
  }
  return [
    {
      size: rule.size || '',
      template: rule.template || '',
      printer: rule.printer || '',
      tray: rule.tray || '',
      protocol: rule.protocol || 'WINDOWS',
      ip: rule.ip || '',
      port: rule.port || 9100,
      dpi: rule.dpi || 203,
      quantity: rule.quantity || 1,
    },
  ];
}

export function AutoPrintBulkSettings(
  p: Props & { center: Center; templates: Template[]; onSaved: () => Promise<void> }
) {
  const branch =
    p.center.master?.branch ||
    p.sysOptions.find((o) => o.FTSopCode === 'DefaultBranch')?.FTSopValue ||
    '';

  const profiles = useMemo(
    () =>
      p.center.rules
        .filter((r) => !branch || r.branch === branch)
        .sort((a, b) => (a.code || a.id).localeCompare(b.code || b.id)),
    [p.center.rules, branch]
  );

  const [editing, setEditing] = useState<Rule | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [preview, setPreview] = useState<{
    title: string;
    targetDate: string;
    groups: Array<{ template: string; size: string; printer: string; items: Candidate[] }>;
  } | null>(null);

  const printerOptions = useMemo(
    () => [
      ...p.windowsPrinters.map((w) => ({
        name: w.name,
        protocol: 'WINDOWS',
        ip: '',
        port: 9100,
        dpi: 203,
        trays: w.trays || [],
      })),
      ...p.printers
        .filter((x) => x.enabled && x.ip !== 'Windows' && x.brand !== 'WINDOWS' && x.brand !== 'SAMSUNG')
        .map((x) => ({
          name: x.name,
          protocol: x.brand,
          ip: x.ip,
          port: x.port,
          dpi: x.dpi,
          trays: [] as string[],
        })),
    ],
    [p.windowsPrinters, p.printers]
  );

  const templateOptions = useMemo(
    () =>
      p.templates.map((t) => {
        const size = p.allLabelSizes.find((s) => s.FTLbsCode === t.FTLbsCode);
        return {
          value: t.FTLtmCode,
          label: t.FTLtmName,
          keywords: `${t.FTLtmCode} ${t.FTLbsCode} ${size?.FTLbsName || ''}`,
          badge: size ? `${size.FCLbsWidth}×${size.FCLbsHeight} มม.` : undefined,
        };
      }),
    [p.templates, p.allLabelSizes]
  );

  const startNewProfile = (source = 'PRICE_CHG') => {
    const code = nextProfileCode(profiles);
    const ev = EVENT_CHOICES.find((e) => e.value === source);
    setError('');
    setPreview(null);
    setEditing({
      id: crypto.randomUUID(),
      code,
      name: `${ev?.shortLabel || 'โปรไฟล์'} (${code})`,
      enabled: true,
      branch,
      source,
      time: '00:00',
      days: '1111111',
      size: '',
      template: '',
      printer: '',
      tray: '',
      protocol: 'WINDOWS',
      ip: '',
      port: 9100,
      dpi: 203,
      quantity: 1,
      catchUp: false,
      offsetDays: source === 'PROMO_END_REVERT' ? -1 : 0,
      targets: [emptyTarget()],
    });
  };

  const openEditProfile = (rule: Rule, appendExtraPaper = false) => {
    setError('');
    const list = normalizeTargets(rule);
    if (appendExtraPaper) {
      list.push(emptyTarget());
    }
    setEditing({
      ...rule,
      targets: list,
    });
  };

  const updateEditingTarget = (index: number, patch: Partial<ProfileTarget>) => {
    if (!editing) return;
    const list = normalizeTargets(editing);
    list[index] = { ...list[index], ...patch };
    const first = list[0];
    setEditing({
      ...editing,
      size: first.size,
      template: first.template,
      printer: first.printer,
      tray: first.tray,
      protocol: first.protocol,
      ip: first.ip,
      port: first.port,
      dpi: first.dpi,
      quantity: first.quantity,
      targets: list,
    });
  };

  const addPaperTarget = () => {
    if (!editing) return;
    const list = normalizeTargets(editing);
    list.push(emptyTarget());
    setEditing({ ...editing, targets: list });
  };

  const removePaperTarget = (index: number) => {
    if (!editing) return;
    const list = normalizeTargets(editing);
    if (list.length <= 1) return;
    list.splice(index, 1);
    const first = list[0];
    setEditing({
      ...editing,
      size: first.size,
      template: first.template,
      printer: first.printer,
      tray: first.tray,
      protocol: first.protocol,
      ip: first.ip,
      port: first.port,
      dpi: first.dpi,
      quantity: first.quantity,
      targets: list,
    });
  };

  const validateProfile = (rule: Rule): string => {
    if (!branch) return 'กรุณาตั้งค่าแหล่งข้อมูลและสาขาก่อนสร้าง Profile';
    const cleanCode = rule.code.trim();
    if (!cleanCode || cleanCode.length > 24 || !/^[A-Za-z0-9_-]+$/.test(cleanCode)) {
      return 'กรุณาระบุรหัส Profile เป็นตัวเลขหรือภาษาอังกฤษ 1–24 ตัว (ห้ามเว้นวรรค)';
    }
    if (profiles.some((x) => x.id !== rule.id && x.code.trim().toUpperCase() === cleanCode.toUpperCase())) {
      return `รหัส Profile "${cleanCode}" ถูกใช้งานแล้ว กรุณาเปลี่ยนรหัสใหม่`;
    }
    if (!rule.name.trim()) return 'กรุณาระบุชื่อ Profile';
    const targets = normalizeTargets(rule);
    if (!targets.length) return 'กรุณาเพิ่มแบบป้ายและเครื่องพิมพ์อย่างน้อย 1 รายการ';

    for (let i = 0; i < targets.length; i++) {
      const t = targets[i];
      const prefix = targets.length > 1 ? `กระดาษ/แบบป้ายที่ ${i + 1}: ` : '';
      if (!t.template) return `${prefix}กรุณาเลือกแบบป้าย`;
      const tmpl = p.templates.find((x) => x.FTLtmCode === t.template);
      if (!tmpl) return `${prefix}ไม่พบแบบป้ายที่เลือก`;
      const prob = templateProblem(tmpl);
      if (prob) return `${prefix}${prob}`;
      if (!p.allLabelSizes.some((s) => s.FTLbsCode === t.size && s.FTLbsStaUse === '1')) {
        return `${prefix}ขนาดป้ายของแบบนี้ถูกปิดใช้งาน`;
      }
      if (!t.printer.trim()) return `${prefix}กรุณาเลือกเครื่องพิมพ์`;
      if (!Number.isInteger(t.quantity) || t.quantity < 1 || t.quantity > 999) {
        return `${prefix}จำนวนป้ายต้องเป็นจำนวนเต็ม 1–999`;
      }
    }
    return '';
  };

  const buildPayload = (rule: Rule): Rule => {
    const targets = normalizeTargets(rule);
    const first = targets[0] || emptyTarget();
    return {
      ...rule,
      code: rule.code.trim(),
      branch,
      enabled: true,
      size: first.size,
      template: first.template,
      printer: first.printer,
      tray: first.tray,
      protocol: first.protocol,
      ip: first.ip,
      port: first.port,
      dpi: first.dpi,
      quantity: first.quantity,
      targets,
    };
  };

  const handleSaveProfile = async () => {
    if (!editing) return;
    const msg = validateProfile(editing);
    if (msg) {
      setError(msg);
      return;
    }
    setBusy(true);
    setError('');
    try {
      const payload = buildPayload(editing);
      await api('/api/print-center/rule', payload);
      toast.success(`บันทึก Profile "${payload.code}" เรียบร้อยแล้ว`);
      setEditing(null);
      await p.onSaved();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleDeleteProfile = async (rule: Rule) => {
    setBusy(true);
    setError('');
    try {
      await api('/api/print-center/profile/delete', { id: rule.id });
      toast.success(`ลบ Profile "${rule.code}" แล้ว`);
      if (editing?.id === rule.id) setEditing(null);
      await p.onSaved();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const handlePreview = async (rule: Rule) => {
    const msg = validateProfile(rule);
    if (msg) {
      setError(msg);
      return;
    }
    setBusy(true);
    setError('');
    try {
      const payload = buildPayload(rule);
      const res = await api('/api/print-center/preview', payload);
      const groups = Array.isArray(res.groups)
        ? res.groups
        : [{ template: payload.template, size: payload.size, printer: payload.printer, items: Array.isArray(res.items) ? res.items : [] }];
      setPreview({
        title: `[${payload.code}] ${payload.name} — ${purposes[payload.source]?.title || payload.source}`,
        targetDate: targetDatePreview(payload.offsetDays),
        groups,
      });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const handleRunProfileNow = async (rule: Rule) => {
    setBusy(true);
    setError('');
    try {
      const res = await api('/api/print-center/run-profile', { code: rule.code, id: rule.id });
      if (res.status === 'EMPTY') {
        toast.info(`Profile [${rule.code}]: ไม่พบรายการสินค้าที่เข้าเงื่อนไข ณ วันที่ ${res.targetDate}`);
      } else {
        toast.success(`ส่งพิมพ์ Profile [${rule.code}] สำเร็จ (${res.itemCount} รายการ / ${res.printedUnits} ชุด)`);
      }
      await p.onSaved();
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-4 min-w-0">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <h2 className="font-semibold text-lg">จัดการ Profile การพิมพ์</h2>
        <div className="flex items-center gap-2">
          <span className="text-sm text-slate-500">สาขา {branch || '-'}</span>
          <Button className="bg-teal-700 hover:bg-teal-800 text-white" disabled={busy || !branch} onClick={() => startNewProfile('PRICE_CHG')}>
            + เพิ่ม Profile ใหม่
          </Button>
        </div>
      </div>

      {error && (
        <p role="alert" className="bg-red-50 border border-red-200 text-red-700 p-3 rounded-lg text-sm">
          {error}
        </p>
      )}

      {editing && (
        <section className={section}>
          <div className="flex flex-wrap items-center justify-between gap-2 border-b pb-2">
            <h3 className="font-semibold text-base text-teal-800">
              {profiles.some((x) => x.id === editing.id) ? `แก้ไข Profile [${editing.code}]` : 'สร้าง Profile ใหม่'}
            </h3>
          </div>

          <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
            <Field label="รหัส Profile *">
              <Input
                value={editing.code}
                maxLength={24}
                placeholder="001"
                className="font-mono font-bold"
                onChange={(e) => setEditing({ ...editing, code: e.target.value })}
              />
            </Field>
            <Field label="ชื่อ Profile *">
              <Input
                value={editing.name}
                placeholder="เช่น ราคาเปลี่ยน"
                onChange={(e) => setEditing({ ...editing, name: e.target.value })}
              />
            </Field>
            <Field label="ประเภทราคา *">
              <select
                className={selectClass}
                value={editing.source}
                onChange={(e) => {
                  const source = e.target.value;
                  const suggestedOffset = source === 'PROMO_END_REVERT' ? -1 : editing.offsetDays;
                  setEditing({ ...editing, source, offsetDays: suggestedOffset });
                }}
              >
                {EVENT_CHOICES.map((opt) => (
                  <option key={opt.value} value={opt.value}>
                    {opt.label}
                  </option>
                ))}
              </select>
            </Field>
            <Field label="Offset วัน (n)">
              <select
                className={selectClass}
                value={[-1, 0, 1].includes(editing.offsetDays) ? String(editing.offsetDays) : 'custom'}
                onChange={(e) => {
                  if (e.target.value !== 'custom') {
                    setEditing({ ...editing, offsetDays: Number(e.target.value) });
                  }
                }}
              >
                {OFFSET_CHOICES.map((opt) => (
                  <option key={opt.value} value={String(opt.value)}>
                    {opt.label}
                  </option>
                ))}
                <option value="custom">กำหนดวันชดเชยเอง...</option>
              </select>
              {![-1, 0, 1].includes(editing.offsetDays) && (
                <Input
                  type="number"
                  min={-30}
                  max={30}
                  value={editing.offsetDays}
                  onChange={(e) => setEditing({ ...editing, offsetDays: Number(e.target.value) || 0 })}
                  className="mt-1"
                />
              )}
            </Field>
          </div>

          <div className="space-y-3 pt-2 border-t">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <h4 className="font-semibold text-sm text-slate-800">
                กระดาษ / แบบป้าย ({normalizeTargets(editing).length} รายการ)
              </h4>
              <Button type="button" size="sm" variant="outline" className="border-teal-600 text-teal-700" onClick={addPaperTarget}>
                + เพิ่มกระดาษ / แบบป้าย
              </Button>
            </div>

            {normalizeTargets(editing).map((target, idx) => {
              const targetTemplate = p.templates.find((x) => x.FTLtmCode === target.template);
              const targetPrinter = printerOptions.find((x) => x.name === target.printer);
              const totalTargets = normalizeTargets(editing).length;
              return (
                <article key={idx} className="rounded-lg border border-slate-200 bg-slate-50/60 p-3 space-y-3">
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-xs font-semibold text-slate-700">
                      กระดาษ / แบบป้ายที่ {idx + 1}
                    </span>
                    <div className="flex items-center gap-2">
                      <Button
                        type="button"
                        variant="link"
                        size="sm"
                        className="h-auto p-0 text-xs text-teal-700"
                        disabled={!target.template}
                        onClick={() => {
                          const m = p.driverMappings.find((x) => x.lbsCode === target.size);
                          if (!m) {
                            setError('ยังไม่ได้ตั้งค่าเครื่องพิมพ์ของแบบป้ายนี้ในหน้า เครื่องพิมพ์ตามแบบป้าย');
                            return;
                          }
                          const route = resolveTemplateRoute(m, target.template, targetTemplate?.FTLtmPriType || '1');
                          const pr = printerOptions.find((x) => x.name === route.printer);
                          if (!pr) {
                            setError('ไม่พบเครื่องพิมพ์ที่ผูกไว้กับแบบป้ายนี้');
                            return;
                          }
                          updateEditingTarget(idx, {
                            printer: pr.name,
                            tray: route.tray,
                            protocol: pr.protocol,
                            ip: pr.ip,
                            port: pr.port,
                            dpi: pr.dpi,
                          });
                          toast.success('ดึงค่าเครื่องพิมพ์และถาดจากแบบป้ายแล้ว');
                        }}
                      >
                        ดึงค่าจาก &ldquo;เครื่องพิมพ์ตามแบบป้าย&rdquo;
                      </Button>
                      {totalTargets > 1 && (
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          className="h-6 px-2 text-xs text-red-600 hover:bg-red-50"
                          onClick={() => removePaperTarget(idx)}
                        >
                          ลบแถวนี้
                        </Button>
                      )}
                    </div>
                  </div>

                  <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
                    <Field label="แบบป้าย *">
                      <SearchableSelect
                        ariaLabel={`แบบป้าย ${idx + 1}`}
                        value={target.template}
                        options={templateOptions}
                        placeholder="เลือกแบบป้าย"
                        searchPlaceholder="ค้นหาชื่อหรือขนาดป้าย..."
                        onValueChange={(tmplCode) => {
                          const t = p.templates.find((x) => x.FTLtmCode === tmplCode);
                          updateEditingTarget(idx, {
                            template: tmplCode,
                            size: t?.FTLbsCode || '',
                          });
                        }}
                      />
                    </Field>

                    <Field label="เครื่องพิมพ์ *">
                      <SearchableSelect
                        ariaLabel={`เครื่องพิมพ์ ${idx + 1}`}
                        value={target.printer}
                        options={[
                          ...printerOptions.map((x) => ({ value: x.name, label: x.name })),
                          ...(target.printer && !targetPrinter
                            ? [{ value: target.printer, label: `${target.printer} (ที่บันทึกไว้)` }]
                            : []),
                        ]}
                        placeholder="เลือกเครื่องพิมพ์"
                        onValueChange={(name) => {
                          const pr = printerOptions.find((x) => x.name === name);
                          if (pr) {
                            updateEditingTarget(idx, {
                              printer: pr.name,
                              tray: '',
                              protocol: pr.protocol,
                              ip: pr.ip,
                              port: pr.port,
                              dpi: pr.dpi,
                            });
                          }
                        }}
                      />
                    </Field>

                    <Field label="ถาดกระดาษ">
                      <SearchableSelect
                        ariaLabel={`ถาดกระดาษ ${idx + 1}`}
                        disabled={!target.printer}
                        value={target.tray}
                        options={[
                          { value: '', label: 'ถาดเริ่มต้น' },
                          ...(targetPrinter?.trays || []).map((t) => ({ value: t, label: p.formatTrayLabel(t) })),
                          ...(target.tray && !targetPrinter?.trays.includes(target.tray)
                            ? [{ value: target.tray, label: p.formatTrayLabel(target.tray) }]
                            : []),
                        ]}
                        onValueChange={(tray) => updateEditingTarget(idx, { tray })}
                      />
                    </Field>

                    <Field label="จำนวนป้ายต่อรายการ">
                      <Input
                        type="number"
                        min={1}
                        max={999}
                        value={target.quantity}
                        onChange={(e) =>
                          updateEditingTarget(idx, { quantity: Math.max(1, Number(e.target.value) || 1) })
                        }
                      />
                    </Field>
                  </div>
                </article>
              );
            })}
          </div>

          <div className="flex flex-wrap justify-end gap-2 pt-2 border-t">
            <Button type="button" variant="outline" disabled={busy} onClick={() => setEditing(null)}>
              ยกเลิก
            </Button>
            <Button type="button" variant="outline" disabled={busy} onClick={() => void handlePreview(editing)}>
              ตรวจรายการสินค้า (ไม่พิมพ์)
            </Button>
            <Button
              type="button"
              className="bg-teal-700 hover:bg-teal-800 text-white"
              disabled={busy}
              onClick={() => void handleSaveProfile()}
            >
              {busy ? 'กำลังบันทึก...' : 'บันทึก Profile'}
            </Button>
          </div>
        </section>
      )}

      {/* แสดง Profile แบ่งตาม 4 เหตุการณ์ราคา เพื่อให้เพิ่มกระดาษ/Profile ในราคานั้นๆ ได้ทันที */}
      <div className="space-y-4">
        {EVENT_CHOICES.map((ev) => {
          const eventProfiles = profiles.filter((r) => r.source === ev.value);
          return (
            <section key={ev.value} className={section}>
              <div className="flex flex-wrap items-center justify-between gap-2 border-b pb-2.5">
                <h3 className="font-semibold text-base text-slate-800">
                  {ev.shortLabel} <span className="text-xs font-normal text-slate-500">({eventProfiles.length} Profile)</span>
                </h3>
                <Button
                  size="sm"
                  variant="outline"
                  className="border-teal-700 text-teal-700 hover:bg-teal-50"
                  disabled={busy || !branch}
                  onClick={() => startNewProfile(ev.value)}
                >
                  + เพิ่ม Profile ในราคานี้
                </Button>
              </div>

              {eventProfiles.length === 0 ? (
                <p className="text-xs text-slate-400 py-3">ยังไม่มี Profile</p>
              ) : (
                <div className="space-y-3">
                  {eventProfiles.map((rule) => {
                    const targets = normalizeTargets(rule);
                    const offsetLabel =
                      rule.offsetDays > 0 ? `+${rule.offsetDays}` : String(rule.offsetDays || 0);
                    return (
                      <article key={rule.id} className="rounded-lg border border-slate-200 p-3 space-y-2.5 bg-white">
                        <div className="flex flex-wrap items-center justify-between gap-2">
                          <div className="flex flex-wrap items-center gap-2.5">
                            <span className="rounded bg-teal-50 border border-teal-200 px-2 py-0.5 font-mono font-bold text-teal-800 text-sm">
                              รหัส {rule.code}
                            </span>
                            <span className="font-semibold text-sm text-slate-800">{rule.name}</span>
                            <span className="rounded bg-slate-100 px-2 py-0.5 font-mono text-xs text-slate-700">
                              Offset {offsetLabel} วัน
                            </span>
                          </div>
                          <div className="flex flex-wrap items-center gap-1.5">
                            <Button
                              size="sm"
                              variant="outline"
                              className="border-teal-600 text-teal-700 text-xs"
                              disabled={busy}
                              onClick={() => openEditProfile(rule, true)}
                            >
                              + เพิ่มกระดาษ/แบบป้าย
                            </Button>
                            <Button size="sm" variant="outline" className="text-xs" disabled={busy} onClick={() => void handlePreview(rule)}>
                              ตรวจสินค้า
                            </Button>
                            <Button
                              size="sm"
                              variant="outline"
                              className="text-xs text-teal-700"
                              disabled={busy}
                              onClick={() => void handleRunProfileNow(rule)}
                            >
                              พิมพ์ตอนนี้
                            </Button>
                            <Button size="sm" variant="outline" className="text-xs" disabled={busy} onClick={() => openEditProfile(rule, false)}>
                              แก้ไข
                            </Button>
                            <Button
                              size="sm"
                              variant="ghost"
                              className="text-xs text-red-600 hover:bg-red-50"
                              disabled={busy}
                              onClick={() => void handleDeleteProfile(rule)}
                            >
                              ลบ
                            </Button>
                          </div>
                        </div>

                        <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 text-xs">
                          {targets.map((t, i) => {
                            const tmpl = p.templates.find((x) => x.FTLtmCode === t.template);
                            return (
                              <div key={i} className="rounded border border-slate-100 bg-slate-50 px-2.5 py-2 space-y-0.5">
                                <div className="font-medium text-slate-800">
                                  กระดาษ {i + 1}: {tmpl?.FTLtmName || t.template || '-'}
                                </div>
                                <div className="text-slate-600">
                                  เครื่องพิมพ์: {t.printer || '-'}
                                  {t.tray ? ` (${p.formatTrayLabel(t.tray)})` : ''} · {t.quantity} ดวง/รายการ
                                </div>
                              </div>
                            );
                          })}
                        </div>
                      </article>
                    );
                  })}
                </div>
              )}
            </section>
          );
        })}
      </div>

      {preview && (
        <section className={section}>
          <div className="flex flex-wrap items-center justify-between gap-2">
            <h3 className="font-semibold">
              ผลตรวจรายการสินค้า: {preview.title} (วันเป้าหมาย {preview.targetDate})
            </h3>
            <Button size="sm" variant="ghost" onClick={() => setPreview(null)}>
              ปิดตัวอย่าง
            </Button>
          </div>
          <div className="space-y-3">
            {preview.groups.map((grp, gIdx) => {
              const tmpl = p.templates.find((x) => x.FTLtmCode === grp.template);
              return (
                <div key={gIdx} className="border rounded-md overflow-hidden">
                  <div className="bg-slate-100 px-3 py-2 text-xs font-semibold flex flex-wrap justify-between gap-2">
                    <span>
                      แบบป้าย: {tmpl?.FTLtmName || grp.template} · เครื่องพิมพ์: {grp.printer}
                    </span>
                    <span>เข้าเงื่อนไข {grp.items.length} รายการ</span>
                  </div>
                  {grp.items.length === 0 ? (
                    <p className="text-xs text-slate-500 p-4 text-center">
                      ไม่พบรายการสินค้าที่เข้าเงื่อนไขในวันเป้าหมาย {preview.targetDate}
                    </p>
                  ) : (
                    <div className="max-h-56 overflow-auto divide-y text-sm">
                      {grp.items.map((item, idx) => (
                        <div key={idx} className="px-3 py-2 flex flex-wrap items-center justify-between gap-2">
                          <span>
                            <strong className="font-mono">{item.barCode}</strong> · {item.pdtName}
                          </span>
                          <span className="font-mono font-semibold text-teal-800">
                            {item.salePrice.toFixed(2)} บาท
                          </span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </section>
      )}
    </div>
  );
}
