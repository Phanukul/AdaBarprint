import type { SearchableOption } from '@/components/ui/searchable-select';

interface Format {
  FTLtmCode: string;
  FTLtmName: string;
  FTLtmPriType: string;
  FTLtmLayout?: string | null;
}

interface Size {
  FTLbsCode: string;
  FTLbsName: string | null;
  FTLbsStaUse: string;
  FCLbsWidth: number;
  FCLbsHeight: number;
  templates?: Format[];
}

export function labelFormatOptions(sizes: Size[], includeLegacy: boolean): SearchableOption[] {
  const options: SearchableOption[] = [];
  for (const size of sizes) {
    if (size.FTLbsStaUse !== '1') continue;
    const badge = `${size.FCLbsWidth}×${size.FCLbsHeight} มม.`;
    const formats = (size.templates || []).filter(t => {
      try { return JSON.parse(t.FTLtmLayout || 'null')?.elements?.length > 0; }
      catch { return false; }
    });
    for (const t of formats) {
      options.push({ value: t.FTLtmCode, label: t.FTLtmName || t.FTLtmCode, badge,
        keywords: `${size.FTLbsCode} ${size.FTLbsName || ''} ${size.FCLbsWidth}x${size.FCLbsHeight}` });
    }
    if (!formats.length && includeLegacy) {
      options.push({ value: `legacy:${size.FTLbsCode}`, label: size.FTLbsName || size.FTLbsCode,
        badge: 'แบบเดิม', keywords: `${badge} ${size.FCLbsWidth}x${size.FCLbsHeight}` });
    }
  }
  const counts = new Map<string, number>();
  const key = (o: SearchableOption) => JSON.stringify([o.label, o.badge]);
  for (const o of options) counts.set(key(o), (counts.get(key(o)) || 0) + 1);
  for (const o of options) {
    if (counts.get(key(o))! > 1) o.label += ` · รหัส ${o.value}`;
  }
  return options.sort((a, b) => a.label.localeCompare(b.label, 'th', { numeric: true }));
}
