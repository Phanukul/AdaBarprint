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

function ProfileLabelSheets({ items, layout, size, single }: {
  items: Candidate[];
  layout: LabelDesignLayout | null;
  size?: Props['allLabelSizes'][number];
  single: boolean;
}) {
  if (!size) return <p role="status" className="p-3">ไม่มีข้อมูลสำหรับแสดงตัวอย่างป้าย</p>;
  const cols = single ? 1 : Math.max(1, size.FNLbsCol || 1);
  const rows = single ? 1 : Math.max(1, size.FNLbsRow || 1);
  const capacity = cols * rows;
  const labels = items.flatMap(item => Array<Candidate>(item.qty).fill(item));
  const count = Math.ceil(labels.length / capacity);
  const scale = 0.7;
  const mm = (n: number) => `${n * scale}mm`;
  const width = size.FCLbsWidth;
  const height = size.FCLbsHeight;
  return <div className="overflow-x-auto border-t bg-slate-100 p-3">
    <p className="mb-3 text-sm">{labels.length} ดวง · {count} {single ? 'ป้ายเดี่ยว' : 'หน้า'}</p>
    {Array.from({ length: count }, (_, page) => (
      <figure key={page} className="mx-auto mb-4 w-fit" style={{ contentVisibility: 'auto' }}>
        <figcaption className="mb-1 text-xs text-slate-600">หน้า {page + 1} / {count}</figcaption>
        <div className="relative overflow-hidden bg-white shadow" style={{
          width: mm(single ? width : size.FCLbsPaperWidth || 210),
          height: mm(single ? height : size.FCLbsPaperHeight || 297),
        }}>
          {labels.slice(page * capacity, (page + 1) * capacity).map((item, slot) => (
            <div key={slot} className="absolute overflow-hidden" style={{
              width: mm(width), height: mm(height),
              left: mm(single ? 0 : (size.FCLbsMagLeft || 0) + (slot % cols) * (width + (size.FCLbsGapH || 0))),
              top: mm(single ? 0 : (size.FCLbsMagTop || 0) + Math.floor(slot / cols) * (height + (size.FCLbsGapV || 0))),
            }}>
              {item.context
                ? <div style={{ transform: `translate(${mm(size.FCLbsOffsetX || 0)}, ${mm(size.FCLbsOffsetY || 0)})` }}>
                    <LabelPreview lbsCode={size.FTLbsCode} labelW={width} labelH={height} customLayout={layout} scale={scale} line={{ barcode: item.barCode, priType: item.priType, ctx: item.context }} />
                  </div>
                : <span>ไม่มีข้อมูลป้าย {item.barCode}</span>}
            </div>
          ))}
        </div>
      </figure>
    ))}
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
    view: 'products' | 'labels';
    title: string;
    targetDate: string;
    loading: boolean;
    error?: string;
    groups: Array<{ template: string; size: string; printer: string; items: Candidate[]; single?: boolean }>;
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

  const handlePreview = async (rule: Rule, view: 'products' | 'labels' = 'products') => {
    const msg = validateProfile(rule);
    const pending = {
      view,
      title: `[${rule.code}] ${rule.name}`,
      targetDate: targetDatePreview(rule.offsetDays),
      groups: [],
      loading: !msg,
      error: msg,
    };
    setPreview(pending);
    setError('');
    if (msg) return;
    try {
      const payload = buildPayload(rule);
      const res = await api('/api/print-center/preview', payload);
      const groups = Array.isArray(res.groups)
        ? res.groups
        : [{ template: payload.template, size: payload.size, printer: payload.printer, items: Array.isArray(res.items) ? res.items : [] }];
      setPreview(current => current === pending ? { ...pending, groups: groups.map((group: NonNullable<typeof preview>['groups'][number], index: number) => ({ ...group, single: Boolean(payload.targets?.[index]?.ip ?? payload.ip) })), loading: false } : current);
    } catch (e) {
      setPreview(current => current === pending ? { ...pending, error: String(e instanceof Error ? e.message : e), loading: false } : current);
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
        toast.success(`เพิ่ม Profile [${rule.code}] เข้าคิวแล้ว (${res.itemCount} รายการ / ${res.totalUnits} ชุด) ตรวจสถานะการส่งได้ในคิวงานพิมพ์`);
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
        <div>
          <h2 className="font-semibold text-lg">โปรไฟล์พิมพ์อัตโนมัติ</h2>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-sm text-slate-500">สาขา {branch || '-'}</span>
          {!editing && <Button className="bg-teal-700 hover:bg-teal-800 text-white" disabled={busy || !branch} onClick={() => startNewProfile('PRICE_CHG')}>
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
                        options={target.template && !templateOptions.some(o => o.value === target.template)
                          ? [...templateOptions, { value: target.template, label: targetTemplate?.FTLtmName || target.template, badge: 'แบบที่บันทึกไว้' }]
                          : templateOptions}
                        dropdownMinWidth={420}
                        placeholder="เลือกแบบป้าย"
                        searchPlaceholder="ค้นหาชื่อหรือขนาดป้าย..."
                        onValueChange={(tmplCode) => {
                          const t = p.templates.find((x) => x.FTLtmCode === tmplCode);
                          updateEditingTarget(idx, {
                            template: tmplCode,
                            size: t?.FTLbsCode || (tmplCode.startsWith('legacy:') ? tmplCode.slice(7) : ''),
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
                            <Button size="sm" variant="outline" className="text-xs" disabled={busy} onClick={() => void handlePreview(rule)}>
                              ตรวจสินค้า
                            </Button>
                            <Button size="sm" variant="outline" className="text-xs" disabled={busy} onClick={() => void handlePreview(rule, 'labels')}>
                              ดูตัวอย่างป้าย
                            </Button>
                            <Button size="sm" variant="outline" className="text-xs" disabled={busy} onClick={() => openEditProfile(rule)}>
                              แก้ไข
                            </Button>
                            <details className="relative">
                              <summary className="cursor-pointer rounded border px-3 py-2 text-xs" aria-label={`ตัวเลือกเพิ่มเติมสำหรับ ${rule.code}`}>เพิ่มเติม</summary>
                              <div className="absolute right-0 z-10 grid min-w-28 rounded border bg-white p-1 shadow">
                                <Button size="sm" variant="ghost" disabled={busy} onClick={() => void handleRunProfileNow(rule)}>พิมพ์ตอนนี้</Button>
                                <Button size="sm" variant="ghost" className="text-red-600" disabled={busy} onClick={() => void handleDeleteProfile(rule)}>ลบโปรไฟล์</Button>
                              </div>
                            </details>
                          </div>
                        </div>

                        <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 text-xs">
                          {targets.map((t, i) => {
                            const tmpl = p.templates.find((x) => x.FTLtmCode === t.template);
                            return (
                              <div key={i} className="rounded border border-slate-100 bg-slate-50 px-2.5 py-2 space-y-0.5">
                                <div className="font-medium text-slate-800">
                                  กระดาษ {i + 1}: {tmpl?.FTLtmName || templateOptions.find(o => o.value === t.template)?.label || t.template || '-'}
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
        <Dialog open onOpenChange={open => { if (!open) setPreview(null); }}>
        <DialogContent className="max-h-[85dvh] grid-rows-[auto_minmax(0,1fr)] sm:max-w-5xl">
          <DialogHeader className="pr-6">
            <DialogTitle>{preview.view === 'labels' ? 'ตัวอย่างป้าย' : 'ผลตรวจรายการสินค้า'} {preview.title}</DialogTitle>
            <DialogDescription>วันเป้าหมาย {preview.targetDate}</DialogDescription>
          </DialogHeader>
          <div className="min-w-0 space-y-3 overflow-auto">
            {preview.loading && <p role="status">กำลังตรวจสินค้า…</p>}
            {preview.error && <p role="alert" className="text-red-700">{preview.error}</p>}
            {preview.groups.map((grp, gIdx) => {
              const tmpl = p.templates.find((x) => x.FTLtmCode === grp.template);
              const size = p.allLabelSizes.find(s => s.FTLbsCode === grp.size);
              return (
                <section key={gIdx} className="min-w-0 rounded-md border">
                  <div className="flex flex-wrap items-start justify-between gap-3 p-3 text-sm">
                    <dl className="flex flex-wrap gap-x-8 gap-y-2">
                      <div><dt className="text-xs text-slate-600">แบบป้าย</dt><dd>{tmpl?.FTLtmName || size?.FTLbsName || grp.template}</dd></div>
                      <div><dt className="text-xs text-slate-600">เครื่องพิมพ์</dt><dd>{grp.printer}</dd></div>
                    </dl>
                    <span className="font-semibold">เข้าเงื่อนไข {grp.items.length} รายการ</span>
                  </div>
                  {grp.items.length === 0 ? (
                    <p className="text-xs text-slate-500 p-4 text-center">
                      ไม่พบรายการสินค้าที่เข้าเงื่อนไขในวันเป้าหมาย {preview.targetDate}
                    </p>
                  ) : preview.view === 'labels' ? (
                    <ProfileLabelSheets items={grp.items} size={size} single={Boolean(grp.single)} layout={previewLayout(tmpl?.FTLtmLayout, size)} />
                  ) : (
                    <div className="max-h-[52dvh] overflow-auto">
                      <table className="w-full min-w-[40rem] border-separate border-spacing-0 text-left text-sm">
                        <caption className="sr-only">สินค้าที่เข้าเงื่อนไข {preview.title}</caption>
                        <thead className="sticky top-0 bg-slate-100 text-slate-700">
                          <tr>
                            <th scope="col" className="px-3 py-2">รหัสสินค้า</th>
                            <th scope="col" className="px-3 py-2">บาร์โค้ด</th>
                            <th scope="col" className="px-3 py-2">ชื่อสินค้า</th>
                            <th scope="col" className="whitespace-nowrap px-3 py-2 text-right">ราคาบนป้าย (บาท)</th>
                            <th scope="col" className="whitespace-nowrap px-3 py-2 text-right">จำนวน (ดวง)</th>
                          </tr>
                        </thead>
                        <tbody>
                          {grp.items.map((item, idx) => (
                            <tr key={idx} className="even:bg-slate-50 [&>td]:border-t [&>td]:px-3 [&>td]:py-2">
                              <td className="font-mono">{item.pdtCode}</td>
                              <td className="font-mono">{item.barCode}</td>
                              <td className="break-words">{item.pdtName}</td>
                              <td className="text-right font-semibold tabular-nums">{item.salePrice.toFixed(2)}</td>
                              <td className="text-right tabular-nums">{item.qty}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  )}
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
