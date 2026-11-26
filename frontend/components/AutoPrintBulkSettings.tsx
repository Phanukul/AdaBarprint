import { labelFormatOptions } from '@/lib/labelFormatOptions';
import { previewLayout } from '@/lib/previewLayout';
import { LabelPreview } from '@/components/LabelPreview';
import type { LabelDesignLayout } from '@/types/design';
import React, { useMemo, useState } from 'react';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { SearchableSelect } from '@/components/ui/searchable-select';
import { resolveTemplateRoute } from '@/lib/printProfile';
import { Input } from '@/components/ui/input';
import {
  api,
  Field,
  selectClass,
  section,
  templateProblem,
  type Props,
  type ProfileTarget,
  type Rule,
  type Center,
  type Template,
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

function ProfileSampleSheet({ layout, size, single, legacy, promo }: {
  legacy: boolean;
  layout: LabelDesignLayout | null;
  size: Props['allLabelSizes'][number];
  single: boolean;
  promo: boolean;
}) {
  const barcode = '4006381333931';
  const line = {
    barcode, priType: promo ? '2' : '1',
    ctx: {
      product: { code: 'P0001', name: 'สินค้าตัวอย่าง', barcode, unit: 'ชิ้น' },
      price: { value: promo ? 79 : 99, old_value: promo ? 99 : undefined },
      promotion: { name: promo ? 'โปรโมชันตัวอย่าง' : '', is_discount: promo },
      tag: { selling_unit: 'ชิ้น' },
    },
  };
  const scale = 0.7;
  const mm = (n: number) => `${n * scale}mm`;
  const width = size.FCLbsWidth;
  const height = size.FCLbsHeight;
  const offsetX = size.FCLbsOffsetX ?? 0;
  const offsetY = size.FCLbsOffsetY ?? 0;
  const marginLeft = (size.FCLbsMagLeft ?? (legacy ? 0 : 3)) + (legacy ? 0 : offsetX);
  const marginTop = (size.FCLbsMagTop ?? (legacy ? 0 : 3)) + (legacy ? 0 : offsetY);
  return <div className="overflow-x-auto border-t bg-slate-100 p-3">
    <figure className="mx-auto w-fit">
        <div className="relative overflow-hidden bg-white shadow" style={{
          width: mm(single ? width : size.FCLbsPaperWidth || 210),
          height: mm(single ? height : size.FCLbsPaperHeight || 297),
        }}>
          <div className="absolute overflow-hidden" style={{
            width: mm(width), height: mm(height), left: mm(single ? 0 : marginLeft), top: mm(single ? 0 : marginTop),
          }}>
            <div style={{ transform: `translate(${mm(legacy || single ? offsetX : 0)}, ${mm(legacy || single ? offsetY : 0)})` }}>
              <LabelPreview lbsCode={size.FTLbsCode} labelW={width} labelH={height} customLayout={layout} scale={scale} line={line} />
            </div>
          </div>
        </div>
    </figure>
  </div>;
}

function nextProfileCode(existing: Rule[]): string {
  const used = new Set(existing.map((r) => r.code.trim().toUpperCase()));
  for (let n = 1; ; n++) {
    const cand = String(n).padStart(3, '0');
    if (!used.has(cand)) return cand;
  }
}

function targetDatePreview(offsetDays: number): string {
  const d = new Date();
  d.setDate(d.getDate() + (Number.isFinite(offsetDays) ? offsetDays : 0));
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
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
    return rule.targets.map((t) => ({ ...t, quantity: 1 }));
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
      quantity: 1,
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
  const [deleteTarget, setDeleteTarget] = useState<Rule | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [preview, setPreview] = useState<Rule | null>(null);

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
    () => labelFormatOptions(p.allLabelSizes.map(size => ({
      ...size,
      templates: p.templates.filter(t => t.FTLbsCode === size.FTLbsCode),
    })), true),
    [p.templates, p.allLabelSizes]
  );

  const startNewProfile = (source = 'PRICE_CHG') => {
    const code = nextProfileCode(p.center.rules);
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

  const openEditProfile = (rule: Rule) => {
    setError('');
    const list = normalizeTargets(rule);
    setPreview(null);
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
    if (p.center.rules.some((x) => x.id !== rule.id && x.code.trim().toUpperCase() === cleanCode.toUpperCase())) {
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
      const legacy = t.template === `legacy:${t.size}`;
      if (!legacy && !tmpl) return `${prefix}ไม่พบแบบป้ายที่เลือก`;
      const prob = tmpl && templateProblem(tmpl, !t.ip);
      if (prob) return `${prefix}${prob}`;
      if (!p.allLabelSizes.some((s) => s.FTLbsCode === t.size && s.FTLbsStaUse === '1')) {
        return `${prefix}ขนาดป้ายของแบบนี้ถูกปิดใช้งาน`;
      }
      if (!t.printer.trim()) return `${prefix}กรุณาเลือกเครื่องพิมพ์`;
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
      setDeleteTarget(null);
      if (editing?.id === rule.id) setEditing(null);
      await p.onSaved();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="space-y-4 min-w-0">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h2 className="font-semibold text-lg">โปรไฟล์พิมพ์อัตโนมัติ</h2>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {/* <span className="text-sm text-slate-500">สาขา {branch || '-'}</span> */}
          <Button size="sm" variant={p.profileAdmin ? 'outline' : 'teal'} disabled={busy} onClick={p.onProfileLockChange}>{p.profileAdmin ? 'ล็อกกลับ' : 'ปลดล็อกเพื่อแก้ไข'}</Button>
          {!editing && p.profileAdmin && <Button className="bg-teal-700 hover:bg-teal-800 text-white" disabled={busy || !branch} onClick={async () => { if (await p.checkProfileAdmin?.()) startNewProfile('PRICE_CHG'); }}>
            + เพิ่มโปรไฟล์
          </Button>}
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
            <Field label="เหตุการณ์ที่ต้องการพิมพ์ *">
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
            <Field label="ค้นหาเหตุการณ์ของวันไหน">
              <select
                className={selectClass}
                value={[-1, 0, 1].includes(editing.offsetDays) ? String(editing.offsetDays) : 'custom'}
                onChange={(e) => {
                  if (e.target.value === 'custom') {
                    setEditing({ ...editing, offsetDays: 2 });
                  } else {
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
          <p className="text-sm text-slate-600">ถ้ารันวันนี้ จะค้นหาเหตุการณ์วันที่ {targetDatePreview(editing.offsetDays)}</p>

          <div className="space-y-3 pt-2 border-t">
            <div className="flex flex-wrap items-center justify-between gap-2">
              <h4 className="font-semibold text-sm text-slate-800">
                แบบป้ายและเครื่องพิมพ์
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
                <article key={idx} tabIndex={-1} aria-label={`กระดาษ / แบบป้ายที่ ${idx + 1}`} className="rounded-lg border border-slate-200 bg-slate-50/60 p-3 space-y-3">
                  <div className="flex items-center justify-between gap-2">
                    <span className="text-xs font-semibold text-slate-700">
                      กระดาษ / แบบป้ายที่ {idx + 1}
                    </span>
                    <div className="flex items-center gap-2">
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

                  <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
                    <div className="min-w-0 space-y-1">
                    <Field label="แบบป้าย *">
                      <SearchableSelect
                        ariaLabel={`แบบป้าย ${idx + 1}`}
                        value={target.template}
                        options={target.template && !templateOptions.some(o => o.value === target.template)
                          ? [...templateOptions, { value: target.template, label: targetTemplate?.FTLtmName || target.template, badge: 'แบบที่บันทึกไว้' }]
                          : templateOptions}
                        dropdownMinWidth={420}
                        placeholder="เลือกแบบป้าย"
                        searchPlaceholder="ค้นหาชื่อหรือขนาดป้าย..."
                        onValueChange={(tmplCode) => {
                          const t = p.templates.find((x) => x.FTLtmCode === tmplCode);
                          if (tmplCode === target.template) return;
                          const size = t?.FTLbsCode || (tmplCode.startsWith('legacy:') ? tmplCode.slice(7) : '');
                          const mapping = p.driverMappings.find(item => item.lbsCode === size);
                          const route = mapping && resolveTemplateRoute(mapping, tmplCode, t?.FTLtmPriType || (editing.source === 'PROMO_START' ? '2' : '1'));
                          const printer = printerOptions.find(item => item.name === route?.printer);
                          updateEditingTarget(idx, {
                            ...emptyTarget(),
                            template: tmplCode, size, quantity: target.quantity,
                            ...(printer && route && {
                              printer: printer.name, tray: route.tray, protocol: printer.protocol,
                              ip: printer.ip, port: printer.port, dpi: printer.dpi,
                            }),
                          });
                        }}
                      />
                    </Field>
                      {p.profileAdmin && p.onOpenDesign && templateOptions.some(option => option.value === target.template) && (
                        <button
                          type="button"
                          className="text-xs font-medium text-teal-700 underline"
                          onClick={event => {
                            const row = event.currentTarget.closest('article');
                            p.onOpenDesign?.(
                              target.size, target.template,
                              targetTemplate ? targetTemplate.FTLtmPriType === '2' : editing.source === 'PROMO_START',
                              { printer: target.printer, tray: target.tray, protocol: target.ip ? 'LAN' : 'WINDOWS' },
                              (size, template) => {
                                if (size && template) {
                                  updateEditingTarget(idx, { size, template });
                                }
                                void p.onSaved()
                                  .catch(() => toast.error('โหลดรายการแบบป้ายใหม่ไม่สำเร็จ'))
                                  .finally(() => requestAnimationFrame(() => row?.focus()));
                              },
                            );
                          }}
                        >
                          {target.template.startsWith('legacy:') ? 'เปิดแบบดั้งเดิมใน Design' : 'แก้ไขแบบป้ายนี้'}
                        </button>
                      )}
                    </div>

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

                  </div>
                </article>
              );
            })}
          </div>

          <div className="flex flex-wrap justify-end gap-2 pt-2 border-t">
            {profiles.some(rule => rule.id === editing.id) && <Button variant="ghost" className="mr-auto text-red-600" disabled={busy} onClick={() => setDeleteTarget(profiles.find(rule => rule.id === editing.id)!)}>ลบโปรไฟล์</Button>}
            <Button variant="outline" disabled={busy} onClick={() => setPreview(editing)}>ดูตัวอย่าง</Button>
            <Button type="button" variant="outline" disabled={busy} onClick={() => setEditing(null)}>
              ยกเลิก
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
        {!editing && EVENT_CHOICES.map((ev) => {
          const eventProfiles = profiles.filter((r) => r.source === ev.value);
          return (
            <section key={ev.value} className={section}>
              <div className="flex flex-wrap items-center justify-between gap-2 border-b pb-2.5">
                <h3 className="font-semibold text-base text-slate-800">
                  {ev.shortLabel} <span className="text-xs font-normal text-slate-500">({eventProfiles.length} Profile)</span>
                </h3>
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
                            {p.profileAdmin && <Button size="sm" disabled={busy} onClick={async () => { if (await p.checkProfileAdmin?.()) openEditProfile(rule); }}>แก้ไขโปรไฟล์</Button>}
                          </div>
                        </div>

                        <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 text-xs">
                          {targets.map((t, i) => {
                            const format = templateOptions.find(o => o.value === t.template);
                            return (
                              <div key={i} className="rounded border border-slate-100 bg-slate-50 px-2.5 py-2 space-y-0.5">
                                <div className="font-medium text-slate-800">
                                  กระดาษ {i + 1}: {format?.label || t.template || '-'}
                                </div>
                                <div className="text-slate-600">
                                  เครื่องพิมพ์: {t.printer || '-'}
                                  {t.tray ? ` (${p.formatTrayLabel(t.tray)})` : ''}
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

      <Dialog open={Boolean(deleteTarget)} onOpenChange={open => { if (!open && !busy) setDeleteTarget(null); }}>
        <DialogContent><DialogHeader><DialogTitle>ลบโปรไฟล์พิมพ์อัตโนมัติ?</DialogTitle>
          <DialogDescription>โปรไฟล์ [{deleteTarget?.code}] {deleteTarget?.name} ของสาขา {deleteTarget?.branch} จะถูกลบ</DialogDescription>
        </DialogHeader>
        <div className="flex justify-end gap-2"><Button variant="outline" disabled={busy} onClick={() => setDeleteTarget(null)}>ยกเลิก</Button>
          <Button variant="destructive" disabled={busy} onClick={() => deleteTarget && void handleDeleteProfile(deleteTarget)}>ยืนยันลบโปรไฟล์</Button></div>
        </DialogContent>
      </Dialog>
      {preview && (
        <Dialog open onOpenChange={open => { if (!open) setPreview(null); }}>
        <DialogContent className="max-h-[85dvh] grid-rows-[auto_minmax(0,1fr)] sm:max-w-5xl">
          <DialogHeader className="pr-6">
            <DialogTitle>ตัวอย่างป้าย {preview.name}</DialogTitle>
            <DialogDescription>ข้อมูลตัวอย่างสำหรับดูแบบป้าย</DialogDescription>
          </DialogHeader>
          <div className="min-w-0 space-y-3 overflow-auto">
            {normalizeTargets(preview).map((target, index) => {
              const tmpl = p.templates.find(t => t.FTLtmCode === target.template);
              const size = p.allLabelSizes.find(s => s.FTLbsCode === target.size);
              const legacy = target.template.startsWith('legacy:');
              return (
                <section key={index} className="min-w-0 rounded-md border">
                  <h3 className="p-3 text-sm font-semibold">กระดาษ {index + 1} · {tmpl?.FTLtmName || target.template || 'ยังไม่ได้เลือกแบบป้าย'}</h3>
                  {size && (tmpl || legacy) ?
                    <ProfileSampleSheet size={size} single={Boolean(target.ip)} legacy={legacy} layout={previewLayout(tmpl?.FTLtmLayout, size)} promo={tmpl ? tmpl.FTLtmPriType === '2' : preview.source === 'PROMO_START'} /> :
                    <p className="p-3 text-sm" role="status">{target.template ? 'ไม่พบข้อมูลแบบป้ายนี้' : 'เลือกแบบป้ายเพื่อดูตัวอย่าง'}</p>}
                </section>
              );
            })}
          </div>
        </DialogContent>
        </Dialog>
      )}
    </div>
  );
}
