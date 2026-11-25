import type { SearchableOption } from '@/components/ui/searchable-select';

interface Format {
  FTPprCode?: string | null;
  FTLtmCode: string;
  FTLtmName: string;
  FTLtmPriType: string;
  FTLtmLayout?: string | null;
}

interface Size {
  FTPprCode?: string | null;
  FTLbsCode: string;
  FTLbsName: string | null;
  FTLbsStaUse: string;
  FCLbsWidth: number;
  FCLbsHeight: number;
  templates?: Format[];
}

export interface LabelFormatOption extends SearchableOption { lbsCode: string }

export function sizePreprintKeywords(size: Size, aliases: ReadonlyMap<string, string>): string {
  const codes = new Set([size.FTPprCode, ...(size.templates || []).map(t => t.FTPprCode)]);
  return Array.from(codes, code => code && (aliases.get(code) || code)).filter(Boolean).join(' ');
}

export function labelFormatOptions(sizes: Size[], includeLegacy: boolean, aliases?: ReadonlyMap<string, string>): LabelFormatOption[] {
  const options: LabelFormatOption[] = [];
  for (const size of sizes) {
    if (size.FTLbsStaUse !== '1') continue;
    const badge = `${size.FCLbsWidth}×${size.FCLbsHeight} มม.`;
    const formats = (size.templates || []).filter(t => {
      try { return Array.isArray(JSON.parse(t.FTLtmLayout || 'null')?.elements); }
      catch { return false; }
    });
    for (const t of formats) {
      const layout = JSON.parse(t.FTLtmLayout!);
      const formCode = t.FTPprCode || layout.preprint?.preprintCode;
      const formKeywords = aliases?.get(formCode) || formCode || '';
      const mode = layout.designMode === 'preprint' ? 'Pre-Print' : 'พิมพ์ครบจบ';
      options.push({ lbsCode: size.FTLbsCode, value: t.FTLtmCode, label: `${t.FTLtmName || size.FTLbsCode} · ${t.FTLtmPriType === '2' ? 'โปรโมชัน' : 'ปกติ'} · ${mode} · ${t.FTLtmCode}`, badge,
        keywords: `${size.FTLbsCode} ${size.FTLbsName || ''} ${size.FCLbsWidth}x${size.FCLbsHeight} ${formKeywords}` });
    }
    if (includeLegacy && !formats.length) {
      options.push({ lbsCode: size.FTLbsCode, value: `legacy:${size.FTLbsCode}`, label: `แบบดั้งเดิม · ${size.FTLbsName || size.FTLbsCode} · ${size.FTLbsCode}`,
        badge, keywords: `${badge} ${size.FCLbsWidth}x${size.FCLbsHeight}` });
    }
  }
  return options.sort((a, b) => a.label.localeCompare(b.label, 'th', { numeric: true }));
}
