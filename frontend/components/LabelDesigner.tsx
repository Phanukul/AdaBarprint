import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { toast } from 'sonner';
import {
  AlertTriangle,
  AlignCenter,
  AlignLeft,
  AlignRight,
  ArrowDown,
  ArrowUp,
  Calculator,
  ChevronDown,
  ChevronUp,
  Copy,
  Eye,
  EyeOff,
  FilePlus2,
  FileSpreadsheet,
  FileUp,
  Grid3X3,
  Image as ImageIcon,
  Layers,
  Loader2,
  Lock,
  Maximize2,
  MoreHorizontal,
  Move,
  Palette,
  Plus,
  Redo2,
  RotateCcw,
  Save,
  Shapes,
  Sparkles,
  Trash2,
  Type,
  Undo2,
  Unlock,
  Upload,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Checkbox } from '@/components/ui/checkbox';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { sizePreprintKeywords, labelFormatOptions } from '@/lib/labelFormatOptions';
import { previewLayout } from '@/lib/previewLayout';
import { SearchableSelect } from '@/components/ui/searchable-select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { CustomDesignRenderer, type LabelPreviewLine } from '@/components/LabelPreview';
import {
  buildStarterLayout,
  DESIGN_ADVANCED_FIELDS,
  DESIGN_BADGE_PRESETS,
  DESIGN_COLOR_SWATCHES,
  DESIGN_DATA_FIELDS,
  DESIGN_ICON_LIST,
  mergePaperSizeOptions,
  PAPER_SIZE_PRESETS,
  roundMm,
  uidDesign,
  type CompanyInfoRecord,
  type DesignElement,
  type DesignElementType,
  type DesignIconName,
  type LabelDesignLayout,
  type LabelFieldRecord,
  type PaperSizeCode,
  type PaperSizeRecord,
  type PreprintRecord,
} from '@/types/design';
import {
  computeFirstCellCropBox,
  cropSheetCellToDataUrl,
  detectPaperCodeFromFilename,
  detectPaperPresetFromDimensions,
  inferPreprintGridFromFile,
  parsePreprintFile,
  type ParsedPreprintResult,
} from '@/lib/preprintImport';
import {
  PreviewMediaToolbar,
  PreviewReferenceFrameWrapper,
} from '@/components/PreviewMediaToolbar';
import {
  computeEffectivePreviewLayout,
  getDefaultPreviewMediaConfig,
  getPaperSizeDisplayName,
  type PreviewMediaConfig,
} from '@/lib/previewMedia';

const PX_PER_MM = 3.779527559;

export interface SheetConfigMeta {
  paperSize?: PaperSizeCode;
  paperWidthMm?: number;
  paperHeightMm?: number;
  cols?: number;
  rows?: number;
  magTop?: number;
  magLeft?: number;
  magBottom?: number;
  magRight?: number;
  gapH?: number;
  gapV?: number;
  widthMm?: number;
  heightMm?: number;
}

export interface DesignerLabelSizeItem {
  templates?: Array<{ FTLtmCode: string; FTLtmName: string; FTLtmPriType: string; FTLtmLayout?: string | null; hasOriginal?: boolean; userCreated?: boolean }>;
  FTLbsCode: string;
  FTLbsGroup?: string;
  FTLbsName: string | null;
  FCLbsWidth: number;
  FCLbsHeight: number;
  FNLbsCol?: number | null;
  FNLbsRow?: number | null;
  FCLbsMagTop?: number | null;
  FCLbsMagLeft?: number | null;
  FCLbsMagBottom?: number | null;
  FCLbsMagRight?: number | null;
  FCLbsGapH?: number | null;
  FCLbsGapV?: number | null;
  FNLbsQtyPageNor?: number | null;
  FNLbsQtyPagePmt?: number | null;
  FTLbsHhtCode?: string | null;
  FTLbsStaUse: string;
  FTLbsPaperSize?: string | null;
  FCLbsPaperWidth?: number | null;
  FCLbsPaperHeight?: number | null;
  FCLbsOffsetX?: number | null;
  FCLbsOffsetY?: number | null;
}

export interface DesignerProductItem {
  FTPdtCode: string;
  FTPdtName: string;
  FTPdtNameOth?: string | null;
  FTPdtUnit?: string | null;
  FTPdtDim?: string | null;
  FCPdtPrice?: number | null;
  FTPdtImg?: string | null;
  bars?: Array<{ FTPdbBarCode: string }>;
}

export interface SaveAsPayload {
  reuseSize: boolean;
  sourceLbsCode: string;
  newLbsCode?: string;
  newLbsName: string;
  priType: '1' | '2';
  widthMm?: number;
  heightMm?: number;
  cols?: number;
  rows?: number;
  magTop?: number;
  magLeft?: number;
  magBottom?: number;
  magRight?: number;
  gapH?: number;
  gapV?: number;
  paperSize?: PaperSizeCode;
  paperWidthMm?: number;
  paperHeightMm?: number;
  layout: LabelDesignLayout;
}

interface LabelDesignerProps {
  labelSizes: DesignerLabelSizeItem[];
  products: DesignerProductItem[];
  paperSizes?: PaperSizeRecord[];
  preprints?: PreprintRecord[];
  labelFields?: LabelFieldRecord[];
  companyInfo?: CompanyInfoRecord;
  initialTemplateCode?: string;
  initialPriceType?: '1' | '2';
  selectedLabelCode: string;
  onChangeLabelCode: (code: string) => void;
  customLayouts: Record<string, LabelDesignLayout>;
  onSaveLayout: (layout: LabelDesignLayout, meta?: SheetConfigMeta, templateCode?: string) => Promise<boolean>;
  onSaveAsLayout: (payload: SaveAsPayload) => Promise<boolean>;
  onDeleteCustomFormat?: (lbsCode: string) => Promise<boolean>;
  onResetLayout: (lbsCode: string, priType: '1' | '2', templateCode?: string) => Promise<boolean>;
}

async function compressImageFileToDataUrl(file: File, maxDim = 600): Promise<string> {
  const rawUrl = await new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ''));
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
  return await new Promise<string>((resolve) => {
    const img = new Image();
    img.onload = () => {
      const ratio = Math.min(1, maxDim / Math.max(img.width || 1, img.height || 1));
      const w = Math.max(1, Math.round(img.width * ratio));
      const h = Math.max(1, Math.round(img.height * ratio));
      const canvas = document.createElement('canvas');
      canvas.width = w;
      canvas.height = h;
      const ctx = canvas.getContext('2d');
      if (!ctx) {
        resolve(rawUrl);
        return;
      }
      ctx.drawImage(img, 0, 0, w, h);
      const isPng = file.type.includes('png') || file.type.includes('svg');
      resolve(canvas.toDataURL(isPng ? 'image/png' : 'image/jpeg', 0.88));
    };
    img.onerror = () => resolve(rawUrl);
    img.src = rawUrl;
  });
}

type HandleDir = 'nw' | 'n' | 'ne' | 'e' | 'se' | 's' | 'sw' | 'w';

interface DragSession {
  mode: 'move' | 'resize' | 'resize-board';
  handle?: HandleDir;
  elId: string;
  startClientX: number;
  startClientY: number;
  origX: number;
  origY: number;
  origW: number;
  origH: number;
  pxPerMm: number;
}

function defaultZoomForSize(wMm: number, hMm: number): number {
  const maxDim = Math.max(wMm, hMm);
  if (maxDim <= 55) return 2.8;
  if (maxDim <= 105) return 1.85;
  if (maxDim <= 150) return 1.35;
  return 1.05;
}

function ColorSwatchPicker({
  label,
  value,
  onChange,
}: {
  label: string;
  value: string;
  onChange: (val: string) => void;
}) {
  const pickerVal = value && value.startsWith('#') && value.length >= 7 ? value.slice(0, 7) : '#000000';
  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between gap-2">
        <Label className="text-[11px] font-medium text-slate-600">{label}</Label>
        <div className="flex items-center gap-1.5">
          <input
            type="color"
            value={pickerVal}
            onChange={(e) => onChange(e.target.value.toUpperCase())}
            className="h-6 w-7 rounded border border-slate-300 cursor-pointer bg-white p-0.5"
            title="เลือกสีอิสระ"
          />
          <Input
            value={value}
            onChange={(e) => onChange(e.target.value)}
            className="h-6 w-20 text-[11px] font-mono px-1.5"
          />
        </div>
      </div>
      <div className="flex flex-wrap gap-1">
        {DESIGN_COLOR_SWATCHES.map((sw) => {
          const active = (value || '').toUpperCase() === sw.toUpperCase();
          return (
            <button
              key={sw}
              type="button"
              onClick={() => onChange(sw)}
              title={sw === 'transparent' ? 'โปร่งใส (ไม่มีสี)' : sw}
              className={`w-4.5 h-4.5 rounded-sm border transition-transform hover:scale-110 ${
                active ? 'ring-2 ring-[#007a87] ring-offset-1 border-slate-800' : 'border-slate-300'
              }`}
              style={{
                background:
                  sw === 'transparent'
                    ? 'repeating-conic-gradient(#cbd5e1 0% 25%, #ffffff 0% 50%) 50% / 8px 8px'
                    : sw,
              }}
            />
          );
        })}
      </div>
    </div>
  );
}

export function LabelDesigner({
  labelSizes,
  products,
  paperSizes = [],
  preprints = [],
  labelFields = [],
  companyInfo,
  initialTemplateCode,
  initialPriceType = '1',
  selectedLabelCode,
  onChangeLabelCode,
  customLayouts,
  onSaveLayout,
  onSaveAsLayout,
  onDeleteCustomFormat,
  onResetLayout,
}: LabelDesignerProps) {
  const activeSizes = useMemo(
    () => labelSizes.filter((l) => l.FTLbsStaUse === '1'),
    [labelSizes]
  );

  const effectiveLbsCode = selectedLabelCode || '';
  const [requestedTemplate, setRequestedTemplate] = useState(initialTemplateCode || '');
  const blankSizeSelected = requestedTemplate === `blank:${effectiveLbsCode}`;
  const isBlankWorkspace = !effectiveLbsCode || blankSizeSelected;
  const currentSize = useMemo(
    () =>
      (effectiveLbsCode ? activeSizes.find((l) => l.FTLbsCode === effectiveLbsCode) : undefined) || {
        FTLbsCode: effectiveLbsCode || 'NEW-BLANK',
        FTLbsName: effectiveLbsCode || 'หน้ากระดาษเปล่า (แบบใหม่)',
        FCLbsWidth: 100,
        FCLbsHeight: 60,
        FNLbsCol: 2,
        FNLbsRow: 4,
        FCLbsMagTop: 3,
        FCLbsMagLeft: 3,
        FCLbsMagBottom: 3,
        FCLbsMagRight: 3,
        FCLbsGapH: 2,
        FCLbsGapV: 2,
        FTLbsPaperSize: 'A4',
        FCLbsPaperWidth: 210,
        FCLbsPaperHeight: 297,
        FTLbsStaUse: '1',
      },
    [activeSizes, effectiveLbsCode]
  );

  const buildEmptyBlankDoc = useCallback(
    (priceType: '1' | '2'): LabelDesignLayout => ({
      lbsCode: effectiveLbsCode,
      name: 'ป้ายเปล่า',
      priType: priceType,
      designMode: 'full',
      width: currentSize.FCLbsWidth,
      height: currentSize.FCLbsHeight,
      backgroundColor: '#FFFFFF',
      borderWidth: 0.3,
      borderColor: '#CBD5E1',
      borderRadius: 0,
      customDesign: true,
      elements: [],
    }),
    [effectiveLbsCode, currentSize.FCLbsWidth, currentSize.FCLbsHeight]
  );

  const [priType, setPriType] = useState<'1' | '2'>(() => {
    const exact = currentSize.templates?.find(t => t.FTLtmCode === initialTemplateCode);
    if (exact) return exact.FTLtmPriType === '2' ? '2' : '1';
    if (!initialTemplateCode && currentSize.templates?.length && !currentSize.templates.some(t => t.FTLtmPriType === initialPriceType))
      return currentSize.templates[0].FTLtmPriType === '2' ? '2' : '1';
    return initialPriceType;
  });
  const preprintAliases = useMemo(() => new Map(preprints.map(form => [
    form.FTPprCode, [form.FTPprCode, form.FTPprName, form.FTPprFileName].filter(Boolean).join(' '),
  ])), [preprints]);
  const formatOptions = useMemo(() => labelFormatOptions(activeSizes, true, preprintAliases), [activeSizes, preprintAliases]);
  const sizeTemplates = activeSizes.find(size => size.FTLbsCode === effectiveLbsCode)?.templates;
  useEffect(() => {
    const exact = sizeTemplates?.find(t => t.FTLtmCode === requestedTemplate);
    if (exact) setPriType(exact.FTLtmPriType === '2' ? '2' : '1');
  }, [sizeTemplates, requestedTemplate]);
  const modeTemplates = sizeTemplates?.filter(t => (t.FTLtmPriType === '2' ? '2' : '1') === priType &&
    formatOptions.some(option => option.value === t.FTLtmCode));
  const designCode = `${effectiveLbsCode}-DESIGN-${priType === '2' ? 'PMT' : 'NOR'}`;
  const legacySelected = requestedTemplate === `legacy:${effectiveLbsCode}`;
  const startingLayout = requestedTemplate === `starter:${effectiveLbsCode}`;
  const selectedTemplate = startingLayout || isBlankWorkspace ? undefined : modeTemplates?.find(t => t.FTLtmCode === (legacySelected ? `${effectiveLbsCode}-STD-${priType === '2' ? 'PMT' : 'NOR'}` : requestedTemplate))
    || modeTemplates?.find(t => t.FTLtmCode === designCode) || modeTemplates?.[0];
  function selectFormat(value: string) {
    if (value.startsWith('legacy:')) {
      setRequestedTemplate(value);
      onChangeLabelCode(value.slice(7));
      return;
    }
    for (const size of activeSizes) {
      const template = size.templates?.find(t => t.FTLtmCode === value);
      if (template) {
        setRequestedTemplate(value);
        setPriType(template.FTLtmPriType === '2' ? '2' : '1');
        onChangeLabelCode(size.FTLbsCode);
        return;
      }
    }
  }
  const [doc, setDoc] = useState<LabelDesignLayout>(() =>
    effectiveLbsCode
      ? buildStarterLayout(
          effectiveLbsCode,
          currentSize.FCLbsWidth,
          currentSize.FCLbsHeight,
          '1',
          currentSize.FTLbsName || effectiveLbsCode
        )
      : buildEmptyBlankDoc('1')
  );
  const [past, setPast] = useState<LabelDesignLayout[]>([]);
  const [future, setFuture] = useState<LabelDesignLayout[]>([]);
  const [dirty, setDirty] = useState(false);
  const [saving, setSaving] = useState(false);

  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [zoom, setZoom] = useState<number>(() =>
    defaultZoomForSize(currentSize.FCLbsWidth, currentSize.FCLbsHeight)
  );
  const [showGrid, setShowGrid] = useState(true);
  const [snapToGrid, setSnapToGrid] = useState(true);
  const [leftTab, setLeftTab] = useState<'fields' | 'shapes' | 'icons' | 'layers'>('fields');
  const [sampleProductCode, setSampleProductCode] = useState<string>('__SAMPLE__');
  const [a4PreviewOpen, setA4PreviewOpen] = useState(false);
  const [a4Scale, setA4Scale] = useState<number>(0.5);
  const [previewMedia, setPreviewMedia] = useState<PreviewMediaConfig>(() =>
    getDefaultPreviewMediaConfig({
      savedPaperSize: currentSize.FTLbsPaperSize,
      savedPaperWidthMm: currentSize.FCLbsPaperWidth ?? undefined,
      savedPaperHeightMm: currentSize.FCLbsPaperHeight ?? undefined,
      lbsGroup: currentSize.FTLbsGroup,
      savedCols: currentSize.FNLbsCol, cellWidthMm: currentSize.FCLbsWidth, gapHMm: currentSize.FCLbsGapH,
      gapVMm: currentSize.FCLbsGapV ?? 2,
    })
  );
  const [saveAsOpen, setSaveAsOpen] = useState(false);
  const [newPrice, setNewPrice] = useState<'1'|'2'>('1');
  const [newMode, setNewMode] = useState<'full'|'preprint'>('full');
  const [saveAsBlank, setSaveAsBlank] = useState(false);
  const [overwriteConfirmOpen, setOverwriteConfirmOpen] = useState(false);
  const [resetConfirmOpen, setResetConfirmOpen] = useState(false);
  const [moreActionsOpen, setMoreActionsOpen] = useState(false);
  const [advancedPropsOpen, setAdvancedPropsOpen] = useState(false);
  const [saveAsName, setSaveAsName] = useState('');
  const [saveAsCode, setSaveAsCode] = useState('');
  const [saveSizeCode, setSaveSizeCode] = useState('');
  const existingSaveSize = activeSizes.find(size => size.FTLbsCode === saveSizeCode);
  const [saveAsWidth, setSaveAsWidth] = useState(100);
  const [saveAsHeight, setSaveAsHeight] = useState(38);
  const [sheetPreviewMode, setSheetPreviewMode] = useState<'guide' | 'ink'>('guide');
  const [sheetConfigDraft, setSheetConfigDraft] = useState<SheetConfigMeta | null>(null);
  const [importPreprintOpen, setImportPreprintOpen] = useState(false);
  const [impLoading, setImpLoading] = useState(false);
  const [impParsed, setImpParsed] = useState<ParsedPreprintResult | null>(null);
  const [impPageNum, setImpPageNum] = useState(1);
  const [impScope, setImpScope] = useState<'sheet' | 'label'>('sheet');
  const [impPaperSize, setImpPaperSize] = useState<PaperSizeCode>('A4');
  const [impPaperW, setImpPaperW] = useState(210);
  const [impPaperH, setImpPaperH] = useState(297);
  const [impLabelW, setImpLabelW] = useState(100);
  const [impLabelH, setImpLabelH] = useState(38);
  const [impCols, setImpCols] = useState(2);
  const [impRows, setImpRows] = useState(7);
  const [impMagTop, setImpMagTop] = useState(3);
  const [impMagLeft, setImpMagLeft] = useState(3);
  const [impMagBottom, setImpMagBottom] = useState(3);
  const [impMagRight, setImpMagRight] = useState(3);
  const [impGapH, setImpGapH] = useState(2);
  const [impGapV, setImpGapV] = useState(2);
  const [impOpacity, setImpOpacity] = useState(0.72);
  const [impContrast, setImpContrast] = useState(110);
  const [impStripDecor, setImpStripDecor] = useState(true);
  const [impAutoFit, setImpAutoFit] = useState(false);
  const [snapGuideX, setSnapGuideX] = useState<number | null>(null);
  const [snapGuideY, setSnapGuideY] = useState<number | null>(null);
  const [boardResizing, setBoardResizing] = useState(false);
  const [boardWInput, setBoardWInput] = useState(String(doc.width));
  const [boardHInput, setBoardHInput] = useState(String(doc.height));

  useEffect(() => {
    setBoardWInput(String(doc.width));
  }, [doc.width]);

  useEffect(() => {
    setBoardHInput(String(doc.height));
  }, [doc.height]);

  const uploadImgInputRef = useRef<HTMLInputElement>(null);
  const replaceImgInputRef = useRef<HTMLInputElement>(null);
  const preprintFileInputRef = useRef<HTMLInputElement>(null);
  const dragRef = useRef<DragSession | null>(null);
  const dragSnapRef = useRef<LabelDesignLayout | null>(null);
  const dragChangedRef = useRef(false);
  const docRef = useRef<LabelDesignLayout>(doc);
  docRef.current = doc;

  // Load saved custom layout or blank/starter preset when switching format or price mode
  const layoutKey = effectiveLbsCode ? `${effectiveLbsCode}:${priType}` : '';
  const savedLayoutForKey = layoutKey && !isBlankWorkspace && !startingLayout && !legacySelected ? customLayouts[layoutKey] : undefined;

  const templateDoc = useMemo(() => {
    const layout = previewLayout(selectedTemplate?.FTLtmLayout, currentSize);
    return layout && selectedTemplate ? {
      ...layout, lbsCode: effectiveLbsCode, priType, designMode: layout.designMode || 'full',
      name: selectedTemplate.FTLtmName,
      backgroundColor: layout.backgroundColor ?? '#FFFFFF',
      borderColor: layout.borderColor ?? '#CBD5E1',
      borderWidth: layout.borderWidth ?? 0,
      borderRadius: layout.borderRadius ?? 0,
    } : null;
  }, [selectedTemplate, currentSize, effectiveLbsCode, priType]);

  useEffect(() => {
    const nextDoc: LabelDesignLayout = isBlankWorkspace
      ? buildEmptyBlankDoc(priType)
      : templateDoc ? structuredClone(templateDoc) : savedLayoutForKey
        ? JSON.parse(JSON.stringify(savedLayoutForKey))
        : buildStarterLayout(
            effectiveLbsCode,
            currentSize.FCLbsWidth,
            currentSize.FCLbsHeight,
            priType,
            currentSize.FTLbsName || effectiveLbsCode
          );
    setDoc(nextDoc);
    setPast([]);
    setFuture([]);
    setSelectedId(nextDoc.elements[0]?.id || null);
    setDirty(false);
    setSheetConfigDraft(null);
    setZoom(defaultZoomForSize(nextDoc.width, nextDoc.height));
  }, [
    layoutKey,
    isBlankWorkspace,
    savedLayoutForKey,
    templateDoc,
    effectiveLbsCode,
    priType,
    currentSize.FCLbsWidth,
    currentSize.FCLbsHeight,
    currentSize.FTLbsName,
    buildEmptyBlankDoc,
  ]);

  const commitDoc = useCallback((next: LabelDesignLayout, recordHistory = true) => {
    if (recordHistory) {
      setPast((prev) => [...prev.slice(-39), docRef.current]);
      setFuture([]);
    }
    setDoc(next);
    setDirty(true);
  }, []);

  const updateDocField = useCallback(
    (patch: Partial<LabelDesignLayout>, recordHistory = true) => {
      commitDoc({ ...docRef.current, ...patch }, recordHistory);
    },
    [commitDoc]
  );

  const updateElement = useCallback(
    (elId: string, patch: Partial<DesignElement>, recordHistory = true) => {
      const cur = docRef.current;
      const next: LabelDesignLayout = {
        ...cur,
        elements: cur.elements.map((e) => (e.id === elId ? { ...e, ...patch } : e)),
      };
      commitDoc(next, recordHistory);
    },
    [commitDoc]
  );

  const handleUndo = useCallback(() => {
    setPast((prev) => {
      if (prev.length === 0) return prev;
      const previous = prev[prev.length - 1];
      setFuture((f) => [docRef.current, ...f.slice(0, 39)]);
      setDoc(previous);
      setDirty(true);
      return prev.slice(0, -1);
    });
  }, []);

  const handleRedo = useCallback(() => {
    setFuture((prev) => {
      if (prev.length === 0) return prev;
      const next = prev[0];
      setPast((p) => [...p.slice(-39), docRef.current]);
      setDoc(next);
      setDirty(true);
      return prev.slice(1);
    });
  }, []);

  const selectedElement = useMemo(
    () => doc.elements.find((e) => e.id === selectedId) || null,
    [doc.elements, selectedId]
  );

  const addElement = useCallback(
    (partial: Omit<DesignElement, 'id'>) => {
      const cur = docRef.current;
      const w = Math.min(partial.width, Math.max(4, cur.width - 2));
      const h = Math.min(partial.height, Math.max(1, cur.height - 2));
      const el: DesignElement = {
        ...partial,
        id: uidDesign(partial.type),
        x: partial.x ?? roundMm(Math.max(1, (cur.width - w) / 2)),
        y: partial.y ?? roundMm(Math.max(1, (cur.height - h) / 2)),
        width: w,
        height: h,
        visible: true,
        locked: false,
      };
      commitDoc({ ...cur, elements: [...cur.elements, el] });
      setSelectedId(el.id);
    },
    [commitDoc]
  );

  const deleteElement = useCallback(
    (elId: string) => {
      const cur = docRef.current;
      const remaining = cur.elements.filter((e) => e.id !== elId);
      commitDoc({ ...cur, elements: remaining });
      if (selectedId === elId) {
        setSelectedId(remaining[remaining.length - 1]?.id || null);
      }
    },
    [commitDoc, selectedId]
  );

  const duplicateElement = useCallback(
    (elId: string) => {
      const cur = docRef.current;
      const found = cur.elements.find((e) => e.id === elId);
      if (!found) return;
      const copy: DesignElement = {
        ...JSON.parse(JSON.stringify(found)),
        id: uidDesign(found.type),
        label: `${found.label} (สำเนา)`,
        x: roundMm(Math.min(cur.width - found.width, found.x + 2)),
        y: roundMm(Math.min(cur.height - found.height, found.y + 2)),
      };
      commitDoc({ ...cur, elements: [...cur.elements, copy] });
      setSelectedId(copy.id);
    },
    [commitDoc]
  );

  const moveLayer = useCallback(
    (elId: string, dir: 'up' | 'down' | 'top' | 'bottom') => {
      const cur = docRef.current;
      const idx = cur.elements.findIndex((e) => e.id === elId);
      if (idx === -1) return;
      const arr = [...cur.elements];
      const [item] = arr.splice(idx, 1);
      if (dir === 'up') arr.splice(Math.min(arr.length, idx + 1), 0, item);
      else if (dir === 'down') arr.splice(Math.max(0, idx - 1), 0, item);
      else if (dir === 'top') arr.push(item);
      else arr.unshift(item);
      commitDoc({ ...cur, elements: arr });
    },
    [commitDoc]
  );

  // Pointer drag/resize handlers
  const pxPerMm = PX_PER_MM * zoom;

  const startMove = (e: React.PointerEvent, el: DesignElement) => {
    if (el.locked) {
      setSelectedId(el.id);
      return;
    }
    e.stopPropagation();
    setSelectedId(el.id);
    dragSnapRef.current = docRef.current;
    dragChangedRef.current = false;
    dragRef.current = {
      mode: 'move',
      elId: el.id,
      startClientX: e.clientX,
      startClientY: e.clientY,
      origX: el.x,
      origY: el.y,
      origW: el.width,
      origH: el.height,
      pxPerMm,
    };
  };

  const startResize = (e: React.PointerEvent, el: DesignElement, handle: HandleDir) => {
    if (el.locked) return;
    e.stopPropagation();
    setSelectedId(el.id);
    dragSnapRef.current = docRef.current;
    dragChangedRef.current = false;
    dragRef.current = {
      mode: 'resize',
      handle,
      elId: el.id,
      startClientX: e.clientX,
      startClientY: e.clientY,
      origX: el.x,
      origY: el.y,
      origW: el.width,
      origH: el.height,
      pxPerMm,
    };
  };

  const startBoardResize = (e: React.PointerEvent, handle: 'e' | 's' | 'se') => {
    e.stopPropagation();
    const cur = docRef.current;
    dragSnapRef.current = cur;
    dragChangedRef.current = false;
    setBoardResizing(true);
    dragRef.current = {
      mode: 'resize-board',
      handle,
      elId: '__board__',
      startClientX: e.clientX,
      startClientY: e.clientY,
      origX: 0,
      origY: 0,
      origW: cur.width,
      origH: cur.height,
      pxPerMm,
    };
  };

  const resizeBoardTo = (nextW: number, nextH: number, scaleElements = false) => {
    const clampedW = roundMm(Math.max(10, Math.min(420, nextW || doc.width)), 0.1);
    const clampedH = roundMm(Math.max(10, Math.min(420, nextH || doc.height)), 0.1);
    if (clampedW === doc.width && clampedH === doc.height) return;
    const scaleX = clampedW / Math.max(1, doc.width);
    const scaleY = clampedH / Math.max(1, doc.height);
    const fontScale = Math.min(scaleX, scaleY);
    const nextElements = scaleElements
      ? doc.elements.map((el) => ({
          ...el,
          x: roundMm(el.x * scaleX, 0.1),
          y: roundMm(el.y * scaleY, 0.1),
          width: roundMm(Math.max(2, el.width * scaleX), 0.1),
          height: roundMm(Math.max(0.5, el.height * scaleY), 0.1),
          fontSize: el.fontSize
            ? roundMm(Math.max(4, Math.min(96, el.fontSize * fontScale)), 0.5)
            : el.fontSize,
        }))
      : doc.elements;
    commitDoc({
      ...doc,
      width: clampedW,
      height: clampedH,
      elements: nextElements,
    });
  };

  const fitElementsToBoard = () => {
    const pad = Math.min(2, roundMm(doc.width * 0.08, 0.5));
    const maxW = Math.max(4, doc.width - pad * 2);
    const maxH = Math.max(4, doc.height - pad * 2);
    const nextElements = doc.elements.map((el) => {
      const nextWidth = Math.min(el.width, el.x === 0 && el.width >= doc.width ? doc.width : maxW);
      const nextHeight = Math.min(el.height, maxH);
      const nextX =
        el.x === 0 && nextWidth >= doc.width - 0.5
          ? 0
          : roundMm(Math.max(pad, Math.min(doc.width - nextWidth - pad, el.x)), 0.5);
      const nextY = roundMm(Math.max(0.5, Math.min(doc.height - nextHeight - 0.5, el.y)), 0.5);
      const widthRatio = el.width > 0 ? nextWidth / el.width : 1;
      const nextFont =
        el.fontSize && widthRatio < 0.95
          ? roundMm(Math.max(4, el.fontSize * widthRatio), 0.5)
          : el.fontSize;
      return {
        ...el,
        x: nextX,
        y: nextY,
        width: roundMm(Math.max(2, nextWidth), 0.1),
        height: roundMm(Math.max(0.5, nextHeight), 0.1),
        fontSize: nextFont,
      };
    });
    commitDoc({ ...doc, elements: nextElements });
    toast.success('จัดขนาดและตำแหน่งองค์ประกอบให้พอดีกรอบป้ายแล้ว');
  };

  useEffect(() => {
    const onMove = (e: PointerEvent) => {
      const d = dragRef.current;
      if (!d) return;
      const dxMm = (e.clientX - d.startClientX) / d.pxPerMm;
      const dyMm = (e.clientY - d.startClientY) / d.pxPerMm;
      const step = snapToGrid && !e.altKey ? 0.5 : 0.1;
      const cur = docRef.current;

      if (d.mode === 'move') {
        let nx = roundMm(d.origX + dxMm, step);
        let ny = roundMm(d.origY + dyMm, step);

        // Snap center guide lines within 0.8mm
        const centerX = cur.width / 2;
        const centerY = cur.height / 2;
        const elCenterX = nx + d.origW / 2;
        const elCenterY = ny + d.origH / 2;
        if (Math.abs(elCenterX - centerX) <= 0.8) {
          nx = roundMm(centerX - d.origW / 2, 0.1);
          setSnapGuideX(centerX);
        } else {
          setSnapGuideX(null);
        }
        if (Math.abs(elCenterY - centerY) <= 0.8) {
          ny = roundMm(centerY - d.origH / 2, 0.1);
          setSnapGuideY(centerY);
        } else {
          setSnapGuideY(null);
        }

        if (nx !== d.origX || ny !== d.origY) {
          dragChangedRef.current = true;
          setDoc((prev) => ({
            ...prev,
            elements: prev.elements.map((el) =>
              el.id === d.elId ? { ...el, x: nx, y: ny } : el
            ),
          }));
          setDirty(true);
        }
      } else if (d.mode === 'resize' && d.handle) {
        let x = d.origX;
        let y = d.origY;
        let w = d.origW;
        let h = d.origH;
        const minW = 2;
        const minH = 0.4;

        if (d.handle.includes('e')) {
          w = Math.max(minW, roundMm(d.origW + dxMm, step));
        }
        if (d.handle.includes('s')) {
          h = Math.max(minH, roundMm(d.origH + dyMm, step));
        }
        if (d.handle.includes('w')) {
          const nextW = Math.max(minW, roundMm(d.origW - dxMm, step));
          x = roundMm(d.origX + (d.origW - nextW), step);
          w = nextW;
        }
        if (d.handle.includes('n')) {
          const nextH = Math.max(minH, roundMm(d.origH - dyMm, step));
          y = roundMm(d.origY + (d.origH - nextH), step);
          h = nextH;
        }

        if (x !== d.origX || y !== d.origY || w !== d.origW || h !== d.origH) {
          dragChangedRef.current = true;
          setDoc((prev) => ({
            ...prev,
            elements: prev.elements.map((el) =>
              el.id === d.elId ? { ...el, x, y, width: w, height: h } : el
            ),
          }));
          setDirty(true);
        }
      } else if (d.mode === 'resize-board' && d.handle) {
        let w = d.origW;
        let h = d.origH;
        if (d.handle.includes('e')) {
          w = Math.max(10, Math.min(420, roundMm(d.origW + dxMm, step)));
        }
        if (d.handle.includes('s')) {
          h = Math.max(10, Math.min(420, roundMm(d.origH + dyMm, step)));
        }
        if (w !== d.origW || h !== d.origH) {
          dragChangedRef.current = true;
          setDoc((prev) => ({ ...prev, width: w, height: h }));
          setDirty(true);
        }
      }
    };

    const onUp = () => {
      if (dragRef.current) {
        if (dragChangedRef.current && dragSnapRef.current) {
          const snap = dragSnapRef.current;
          setPast((p) => [...p.slice(-39), snap]);
          setFuture([]);
        }
        dragRef.current = null;
        dragSnapRef.current = null;
        dragChangedRef.current = false;
        setSnapGuideX(null);
        setSnapGuideY(null);
        setBoardResizing(false);
      }
    };

    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
    return () => {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
    };
  }, [snapToGrid]);

  // Keyboard shortcuts
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const tag = target?.tagName?.toLowerCase();
      if (tag === 'input' || tag === 'textarea' || tag === 'select' || target?.isContentEditable) {
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'z') {
        e.preventDefault();
        if (e.shiftKey) handleRedo();
        else handleUndo();
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'y') {
        e.preventDefault();
        handleRedo();
        return;
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'd' && selectedId) {
        e.preventDefault();
        duplicateElement(selectedId);
        return;
      }
      if (!selectedId) return;
      const el = docRef.current.elements.find((item) => item.id === selectedId);
      if (!el || el.locked) return;

      if (e.key === 'Delete' || e.key === 'Backspace') {
        e.preventDefault();
        deleteElement(selectedId);
        return;
      }

      const step = e.shiftKey ? 2 : 0.5;
      if (e.key === 'ArrowLeft') {
        e.preventDefault();
        updateElement(selectedId, { x: roundMm(el.x - step, 0.5) });
      } else if (e.key === 'ArrowRight') {
        e.preventDefault();
        updateElement(selectedId, { x: roundMm(el.x + step, 0.5) });
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        updateElement(selectedId, { y: roundMm(el.y - step, 0.5) });
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        updateElement(selectedId, { y: roundMm(el.y + step, 0.5) });
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, [selectedId, deleteElement, duplicateElement, handleRedo, handleUndo, updateElement]);

  // Live preview sample data (uses DB products & DB company/branch when available)
  const sampleLine: LabelPreviewLine = useMemo(() => {
    const picked =
      sampleProductCode === '__SAMPLE__'
        ? products[0]
        : products.find((p) => p.FTPdtCode === sampleProductCode);
    const pdtName = picked?.FTPdtName || 'สินค้าตัวอย่าง';
    const pdtNameOth = picked?.FTPdtNameOth || '';
    const pdtCode = picked?.FTPdtCode || '';
    const bar = picked?.bars?.[0]?.FTPdbBarCode || '8850123456789';
    const basePrice = picked?.FCPdtPrice ?? 159.0;
    const isPromo = priType === '2';
    const promoPrice = isPromo ? Math.max(1, Math.round(basePrice * 0.8 * 100) / 100) : basePrice;

    const promoRate = isPromo ? '20%' : '';

    return {
      barcode: bar,
      priType,
      lbsCode: effectiveLbsCode,
      ctx: {
        product: {
          code: pdtCode,
          name: pdtName,
          name_oth: pdtNameOth,
          barcode: bar,
          dim: picked?.FTPdtDim || '',
          unit: picked?.FTPdtUnit || '',
          sub_dept: '',
          image: picked?.FTPdtImg || '',
        },
        price: {
          value: promoPrice,
          old_value: isPromo ? basePrice : undefined,
        },
        promotion: {
          text: isPromo ? `ลดพิเศษ ${promoRate}` : '',
          name: isPromo ? `ลดพิเศษ ${promoRate}` : '',
          interval: '',
          is_discount: isPromo,
          isDiscount: isPromo,
        },
        tag: {
          selling_unit: picked?.FTPdtUnit || '',
          pri_per_unit: '',
          pri_status: 'AC',
          sub_dept: '',
          print_date: new Date().toISOString(),
        },
        store: {
          name: companyInfo?.shopName || companyInfo?.cmpName || companyInfo?.defaultBchName || '',
        },
      },
    };
  }, [products, sampleProductCode, priType, effectiveLbsCode, companyInfo]);

  const dbDataFields = useMemo(() => {
    const mapWeight = (w?: string | null): DesignElement['fontWeight'] => {
      if (w === 'medium' || w === 'semibold' || w === 'bold' || w === 'extrabold' || w === 'black') {
        return w;
      }
      return 'normal';
    };
    const toItem = (r: LabelFieldRecord) => ({
      field: r.FTLbfCode,
      label: r.FTLbfName,
      type: (r.FTLbfType || 'text') as DesignElementType,
      defaultW: r.FCLbfDefW ?? 40,
      defaultH: r.FCLbfDefH ?? 8,
      fontSize: r.FNLbfDefFont ?? 10,
      fontWeight: mapWeight(r.FTLbfDefWeight),
      color: r.FTLbfDefColor || '#111111',
      align: 'left' as const,
      multiLine: r.FTLbfCode === 'product.name',
      strike: r.FTLbfCode === 'price.old_value',
      prefix: r.FTLbfCode === 'price.old_value' ? 'ปกติ ' : undefined,
      suffix: undefined,
    });
    const active = labelFields.filter((f) => f.FTLbfStaUse === '1');
    if (active.length === 0) {
      return { main: DESIGN_DATA_FIELDS, adv: DESIGN_ADVANCED_FIELDS };
    }
    const main = active.filter((f) => f.FTLbfStaAdv !== '1').map(toItem);
    const adv = active.filter((f) => f.FTLbfStaAdv === '1').map(toItem);
    return { main: main.length > 0 ? main : DESIGN_DATA_FIELDS, adv };
  }, [labelFields]);

  const allPaperSizes = React.useMemo(() => mergePaperSizeOptions(paperSizes), [paperSizes]);

  const findPaperPreset = (code?: string | null) => {
    const c = (code || 'A4').trim();
    const dbP = allPaperSizes.find((p) => p.FTPpsCode === c);
    if (dbP) {
      return {
        code: dbP.FTPpsCode,
        label: dbP.FTPpsName,
        widthMm: dbP.FCPpsWidth,
        heightMm: dbP.FCPpsHeight,
        isCustom: dbP.FTPpsStaCustom === '1',
      };
    }
    const fb = PAPER_SIZE_PRESETS[c] || PAPER_SIZE_PRESETS.A4;
    return {
      code: c,
      label: fb.label,
      widthMm: fb.widthMm,
      heightMm: fb.heightMm,
      isCustom: c === 'CUSTOM',
    };
  };

  const resolvePaperMeta = (): SheetConfigMeta => {
    if (sheetConfigDraft) return sheetConfigDraft;
    if (doc.designMode === 'preprint' && doc.preprint) {
      return {
        paperSize: doc.preprint.paperSize,
        paperWidthMm: doc.preprint.paperWidthMm,
        paperHeightMm: doc.preprint.paperHeightMm,
        cols: savedMetrics.cols,
        rows: savedMetrics.rows,
        magTop: savedMetrics.magTop,
        magLeft: savedMetrics.magLeft,
        magBottom: savedMetrics.magBottom,
        magRight: savedMetrics.magRight,
        gapH: savedMetrics.gapH,
        gapV: savedMetrics.gapV,
        widthMm: doc.width,
        heightMm: doc.height,
      };
    }
    return {
      paperSize: savedMetrics.paperSize,
      paperWidthMm: savedMetrics.paperWidthMm,
      paperHeightMm: savedMetrics.paperHeightMm,
      cols: savedMetrics.cols,
      rows: savedMetrics.rows,
      magTop: savedMetrics.magTop,
      magLeft: savedMetrics.magLeft,
      magBottom: savedMetrics.magBottom,
      magRight: savedMetrics.magRight,
      gapH: savedMetrics.gapH,
      gapV: savedMetrics.gapV,
      widthMm: doc.width,
      heightMm: doc.height,
    };
  };

  const handleSave = async (): Promise<boolean> => {
    setSaving(true);
    try {
      const ok = await onSaveLayout({ ...doc, customDesign: true }, resolvePaperMeta(), selectedTemplate?.FTLtmCode);
      if (ok) {
        setRequestedTemplate(selectedTemplate?.FTLtmCode || designCode);
        setDirty(false);
      }
      return ok;
    } finally {
      setSaving(false);
    }
  };

  const openSaveAsModal = () => {
    const suffix = doc.designMode === 'preprint' ? ' (Pre-Print)' : ' (แบบใหม่)';
    setSaveAsBlank(false);
    setSaveAsName(
      isBlankWorkspace ? '' : `${currentSize.FTLbsName || effectiveLbsCode}${suffix}`
    );
    setSaveSizeCode(effectiveLbsCode || activeSizes[0]?.FTLbsCode || '');
    setSaveAsCode('');
    setSaveAsWidth(doc.width);
    setSaveAsHeight(doc.height);
    setNewPrice(priType);
    setNewMode(doc.designMode || 'full');
    setSaveAsOpen(true);
  };

  const openSaveAsBlankModal = () => {
    setSaveAsBlank(true);
    setSaveAsName('');
    setSaveSizeCode(effectiveLbsCode || activeSizes[0]?.FTLbsCode || '');
    setSaveAsCode('');
    setSaveAsWidth(doc.width);
    setSaveAsHeight(doc.height);
    setNewPrice(priType);
    setNewMode(doc.designMode || 'full');
    setSaveAsOpen(true);
  };

  const handleConfirmSaveAs = async () => {
    if (!saveAsName.trim()) {
      toast.error('กรุณาระบุชื่อแบบป้ายใหม่');
      return;
    }
    const meta = resolvePaperMeta();
    const maxW = meta.paperWidthMm || 210;
    const maxH = meta.paperHeightMm || 297;
    const wMm = existingSaveSize?.FCLbsWidth ?? Math.max(10, Math.min(maxW, Number(saveAsWidth) || doc.width));
    const hMm = existingSaveSize?.FCLbsHeight ?? Math.max(10, Math.min(maxH, Number(saveAsHeight) || doc.height));
    const targetLayout: LabelDesignLayout = saveAsBlank
      ? {
          ...doc,
          width: wMm,
          height: hMm,
          backgroundColor: doc.designMode === 'preprint' ? 'transparent' : '#FFFFFF',
          borderWidth: doc.designMode === 'preprint' ? 0 : 0.3,
          elements: [],
          preprint: doc.designMode === 'preprint' ? doc.preprint : undefined,
          customDesign: true,
        }
      : { ...doc, width: wMm, height: hMm, customDesign: true };
    targetLayout.priType = newPrice;
    targetLayout.designMode = newMode;
    if (newMode !== doc.designMode) {
      targetLayout.preprint = undefined;
      if (newMode === 'preprint') {
        targetLayout.backgroundColor = 'transparent';
        targetLayout.borderWidth = 0;
        targetLayout.elements = stripDecorativeElements(targetLayout.elements);
      }
    }
    setSaving(true);
    try {
      const ok = await onSaveAsLayout({
        reuseSize: Boolean(existingSaveSize),
        sourceLbsCode: existingSaveSize?.FTLbsCode || effectiveLbsCode || activeSizes[0]?.FTLbsCode || 'ADA-100x38',
        newLbsCode: existingSaveSize ? undefined : saveAsCode.trim() || undefined,
        newLbsName: saveAsName.trim(),
        priType: newPrice,
        widthMm: wMm,
        heightMm: hMm,
        cols: meta.cols,
        rows: meta.rows,
        magTop: meta.magTop,
        magLeft: meta.magLeft,
        magBottom: meta.magBottom,
        magRight: meta.magRight,
        gapH: meta.gapH,
        gapV: meta.gapV,
        paperSize: meta.paperSize,
        paperWidthMm: meta.paperWidthMm,
        paperHeightMm: meta.paperHeightMm,
        layout: targetLayout,
      });
      if (ok) {
        setDirty(false);
        setSaveAsOpen(false);
        setSaveAsBlank(false);
      }
    } finally {
      setSaving(false);
    }
  };

  const stripDecorativeElements = (elements: DesignElement[]): DesignElement[] =>
    elements.filter((el) => {
      if (el.type === 'rect' || el.type === 'circle' || el.type === 'line' || el.type === 'icon') {
        return false;
      }
      if (
        el.type === 'static' &&
        (el.label?.includes('หัวป้าย') ||
          el.label?.includes('แบรนด์') ||
          (el.text || '').includes('ราคาสินค้า'))
      ) {
        return false;
      }
      return true;
    });

  const handleToggleDesignMode = (mode: 'full' | 'preprint') => {
    if ((doc.designMode || 'full') === mode) return;
    if (mode === 'preprint') {
      const psInfo = findPaperPreset(currentSize.FTLbsPaperSize);
      const existingPp = doc.preprint;
      const hasBg = Boolean(existingPp?.cellBgDataUrl || existingPp?.sheetBgDataUrl);
      const strippedElements = stripDecorativeElements(doc.elements);
      commitDoc({
        ...doc,
        designMode: 'preprint',
        backgroundColor: 'transparent',
        borderWidth: 0,
        elements: strippedElements,
        preprint: existingPp || {
          scope: 'sheet',
          opacity: 0.72,
          paperSize: psInfo.code,
          paperWidthMm: currentSize.FCLbsPaperWidth ?? psInfo.widthMm,
          paperHeightMm: currentSize.FCLbsPaperHeight ?? psInfo.heightMm,
        },
      });
      setSelectedId(null);
      if (!hasBg) {
        toast.info('สลับเป็นโหมด Pre-Print และซ่อนแถบสี/กรอบเดิมแล้ว — กดนำเข้าไฟล์ Pre-Print หรือเริ่มวางข้อมูลบนหน้าเปล่าได้เลย');
      }
    } else {
      commitDoc({
        ...doc,
        designMode: 'full',
      });
    }
  };

  const handleStartBlankPreprint = () => {
    commitDoc({
      ...doc,
      designMode: 'preprint',
      backgroundColor: 'transparent',
      borderWidth: 0,
      elements: [],
    });
    setSelectedId(null);
    toast.success('ล้างหน้ากระดาษเป็นหน้าเปล่าแล้ว — คลิกเลือกข้อมูลจากแถบซ้ายเพื่อวางจุดพิมพ์ได้เลย');
  };

  const openImportPreprintModal = () => {
    const meta = resolvePaperMeta();
    const psInfo = findPaperPreset(meta.paperSize);
    setImpPaperSize(psInfo.code);
    setImpPaperW(meta.paperWidthMm || psInfo.widthMm);
    setImpPaperH(meta.paperHeightMm || psInfo.heightMm);
    setImpLabelW(doc.width);
    setImpLabelH(doc.height);
    setImpCols(meta.cols ?? currentSize.FNLbsCol ?? 2);
    setImpRows(meta.rows ?? currentSize.FNLbsRow ?? 4);
    setImpMagTop(meta.magTop ?? currentSize.FCLbsMagTop ?? 3);
    setImpMagLeft(meta.magLeft ?? currentSize.FCLbsMagLeft ?? 3);
    setImpMagBottom(meta.magBottom ?? currentSize.FCLbsMagBottom ?? meta.magTop ?? currentSize.FCLbsMagTop ?? 3);
    setImpMagRight(meta.magRight ?? currentSize.FCLbsMagRight ?? meta.magLeft ?? currentSize.FCLbsMagLeft ?? 3);
    setImpGapH(meta.gapH ?? currentSize.FCLbsGapH ?? 2);
    setImpGapV(meta.gapV ?? currentSize.FCLbsGapV ?? 2);
    setImpPageNum(doc.preprint?.pageIndex || 1);
    setImpScope(doc.preprint?.scope || 'sheet');
    setImpOpacity(doc.preprint?.opacity ?? 0.72);
    setImpContrast(doc.preprint?.contrast ?? 110);
    setImpAutoFit(false);
    setImportPreprintOpen(true);
  };

  const handleSelectSavedPreprint = (pp: PreprintRecord) => {
    let cropBox: { xPct: number; yPct: number; wPct: number; hPct: number } | undefined;
    if (pp.FTPprCropJson) {
      try {
        cropBox = JSON.parse(pp.FTPprCropJson);
      } catch {
        // ignore invalid crop JSON
      }
    }
    const wMm = pp.FCPprLabelW ?? doc.width;
    const hMm = pp.FCPprLabelH ?? doc.height;
    let nextElements = doc.elements.map((el) => ({ ...el }));
    if (impStripDecor) {
      nextElements = stripDecorativeElements(nextElements);
    }
    commitDoc({
      ...doc,
      width: wMm,
      height: hMm,
      designMode: 'preprint',
      preprint: {
        preprintCode: pp.FTPprCode,
        sourceName: pp.FTPprFileName || pp.FTPprName,
        scope: pp.FTPprScope === 'label' ? 'label' : 'sheet',
        sheetBgDataUrl: pp.FTPprSheetImg || undefined,
        cellBgDataUrl: pp.FTPprCellImg || undefined,
        pageIndex: pp.FNPprPageNo || 1,
        opacity: pp.FCPprOpacity ?? 0.72,
        paperSize: pp.FTPpsCode,
        paperWidthMm: pp.FCPprWidth,
        paperHeightMm: pp.FCPprHeight,
        cropBox,
      },
      elements: nextElements,
    });
    setSheetConfigDraft({
      paperSize: pp.FTPpsCode,
      paperWidthMm: pp.FCPprWidth,
      paperHeightMm: pp.FCPprHeight,
      cols: pp.FNPprCol,
      rows: pp.FNPprRow,
      magTop: pp.FCPprMagTop ?? 3,
      magLeft: pp.FCPprMagLeft ?? 3,
      magBottom: pp.FCPprMagBottom ?? pp.FCPprMagTop ?? 3,
      magRight: pp.FCPprMagRight ?? pp.FCPprMagLeft ?? 3,
      gapH: pp.FCPprGapH ?? 2,
      gapV: pp.FCPprGapV ?? 2,
      widthMm: wMm,
      heightMm: hMm,
    });
    setImportPreprintOpen(false);
    toast.success(`เลือกฟอร์ม Pre-Print "${pp.FTPprName}" จากฐานข้อมูลเรียบร้อย`);
  };

  const handlePreprintFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    e.target.value = '';
    if (!file) return;
    setImpLoading(true);
    try {
      const parsed = await parsePreprintFile(file, 6);
      setImpParsed(parsed);
      setImpPageNum(1);
      const firstPage = parsed.pages[0];

      let nextPaperCode = impPaperSize;
      let nextPaperW = impPaperW;
      let nextPaperH = impPaperH;

      if (firstPage?.mmWidth && firstPage?.mmHeight) {
        const detectedCode = detectPaperPresetFromDimensions(
          firstPage.mmWidth,
          firstPage.mmHeight,
          allPaperSizes
        );
        if (detectedCode !== 'CUSTOM') {
          const detected = findPaperPreset(detectedCode);
          nextPaperCode = detected.code;
          nextPaperW = detected.widthMm;
          nextPaperH = detected.heightMm;
          toast.info(`ตรวจพบขนาดหน้า PDF (${firstPage.mmWidth}×${firstPage.mmHeight} mm) → ${detected.label}`);
        } else {
          nextPaperCode = 'CUSTOM';
          nextPaperW = firstPage.mmWidth;
          nextPaperH = firstPage.mmHeight;
          toast.info(`ตรวจพบขนาดหน้า PDF แบบกำหนดเอง (${firstPage.mmWidth}×${firstPage.mmHeight} mm)`);
        }
      } else {
        const hintCode = detectPaperCodeFromFilename(file.name);
        if (hintCode) {
          const detected = findPaperPreset(hintCode);
          nextPaperCode = detected.code;
          nextPaperW = detected.widthMm;
          nextPaperH = detected.heightMm;
          toast.info(`ตรวจพบขนาดจากชื่อไฟล์ → ${detected.label}`);
        }
      }

      setImpPaperSize(nextPaperCode);
      setImpPaperW(nextPaperW);
      setImpPaperH(nextPaperH);

      const grid = inferPreprintGridFromFile({
        fileName: file.name,
        paperWidthMm: nextPaperW,
        paperHeightMm: nextPaperH,
      });
      if (grid) {
        setImpAutoFit(false);
        setImpCols(grid.cols);
        setImpRows(grid.rows);
        setImpLabelW(grid.labelW);
        setImpLabelH(grid.labelH);
        setImpMagTop(grid.magTop);
        setImpMagBottom(grid.magBottom);
        setImpMagLeft(grid.magLeft);
        setImpMagRight(grid.magRight);
        setImpGapH(grid.gapH);
        setImpGapV(grid.gapV);
        toast.info(
          `ปรับตารางอัตโนมัติ: ${grid.cols}×${grid.rows} ช่อง (${grid.labelW}×${grid.labelH} mm · ขอบ ${grid.magTop}/${grid.magLeft} mm)`
        );
      }

      toast.success(`โหลดไฟล์ ${parsed.fileName} สำเร็จ (${parsed.pages.length} หน้า)`);
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'ไม่สามารถอ่านไฟล์ PDF/รูป Pre-Print ได้');
    } finally {
      setImpLoading(false);
    }
  };

  const computeAutoFitLabelSize = React.useCallback(() => {
    const c = Math.max(1, impCols);
    const r = Math.max(1, impRows);
    const usableW = Math.max(10, impPaperW - impMagLeft - impMagRight - (c - 1) * impGapH);
    const usableH = Math.max(10, impPaperH - impMagTop - impMagBottom - (r - 1) * impGapV);
    return {
      w: roundMm(usableW / c, 0.1),
      h: roundMm(usableH / r, 0.1),
    };
  }, [impCols, impRows, impPaperW, impPaperH, impMagLeft, impMagRight, impMagTop, impMagBottom, impGapH, impGapV]);

  useEffect(() => {
    if (!importPreprintOpen || !impAutoFit) return;
    const { w, h } = computeAutoFitLabelSize();
    setImpLabelW(w);
    setImpLabelH(h);
  }, [importPreprintOpen, impAutoFit, computeAutoFitLabelSize]);

  const handleAutoFitPreprintCell = () => {
    const { w, h } = computeAutoFitLabelSize();
    setImpLabelW(w);
    setImpLabelH(h);
    setImpAutoFit(true);
  };

  const handleApplyImportedPreprint = async () => {
    const chosenPage = impParsed?.pages.find((p) => p.pageNumber === impPageNum) || impParsed?.pages[0];
    if (!chosenPage && !doc.preprint?.sheetBgDataUrl && !doc.preprint?.cellBgDataUrl) {
      toast.error('กรุณาเลือกไฟล์ PDF หรือรูปภาพฟอร์ม Pre-Print ก่อน');
      return;
    }

    setSaving(true);
    try {
      const c = Math.max(1, impCols);
      const r = Math.max(1, impRows);
      const wMm = Math.max(10, impLabelW);
      const hMm = Math.max(10, impLabelH);
      const sheetUrl =
        impScope === 'sheet'
          ? chosenPage?.dataUrl || doc.preprint?.sheetBgDataUrl
          : undefined;
      const crop = computeFirstCellCropBox({
        paperWidthMm: impPaperW,
        paperHeightMm: impPaperH,
        magTopMm: impMagTop,
        magLeftMm: impMagLeft,
        labelWidthMm: wMm,
        labelHeightMm: hMm,
      });
      const cellUrl =
        impScope === 'sheet' && sheetUrl
          ? await cropSheetCellToDataUrl(sheetUrl, crop)
          : chosenPage?.dataUrl || doc.preprint?.cellBgDataUrl;

      let nextElements = doc.elements.map((el) => ({ ...el }));
      if (impStripDecor) {
        nextElements = stripDecorativeElements(nextElements);
        // Nudge data items away from top/bottom Pre-print header strips if they are very close to the top edge
        nextElements = nextElements.map((el) =>
          el.y < 6 ? { ...el, y: roundMm(Math.min(Math.max(10, hMm * 0.16), Math.max(6, hMm - el.height - 4)), 0.5) } : el
        );
      }

      const nextDoc: LabelDesignLayout = {
        ...doc,
        width: wMm,
        height: hMm,
        designMode: 'preprint',
        preprint: {
          sourceName: impParsed?.fileName || doc.preprint?.sourceName,
          scope: impScope,
          sheetBgDataUrl: sheetUrl,
          cellBgDataUrl: cellUrl,
          pageIndex: impPageNum,
          opacity: impOpacity,
          contrast: impContrast,
          paperSize: impPaperSize,
          paperWidthMm: impPaperW,
          paperHeightMm: impPaperH,
          cropBox: impScope === 'sheet' ? crop : undefined,
        },
        elements: nextElements,
      };

      commitDoc(nextDoc);
      setSheetConfigDraft({
        paperSize: impPaperSize,
        paperWidthMm: impPaperW,
        paperHeightMm: impPaperH,
        cols: c,
        rows: r,
        magTop: impMagTop,
        magLeft: impMagLeft,
        magBottom: impMagBottom,
        magRight: impMagRight,
        gapH: impGapH,
        gapV: impGapV,
        widthMm: wMm,
        heightMm: hMm,
      });
      setSelectedId(nextElements[0]?.id || null);
      setImportPreprintOpen(false);
      toast.success('นำเข้าฟอร์ม Pre-Print เรียบร้อย — ลากปรับตำแหน่งข้อความให้ตรงช่อง แล้วกดบันทึกหรือบันทึกเป็นแบบใหม่ได้เลย');
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'ตัดภาพช่องป้ายไม่สำเร็จ');
    } finally {
      setSaving(false);
    }
  };

  const handleUploadNewImage = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    e.target.value = '';
    if (!file) return;
    try {
      const dataUrl = await compressImageFileToDataUrl(file);
      const sizeMm = roundMm(Math.max(10, Math.min(doc.width * 0.35, doc.height * 0.45)), 0.5);
      addElement({
        type: 'image',
        label: `รูป ${file.name.replace(/\.[^.]+$/, '')}`,
        src: dataUrl,
        objectFit: 'contain',
        x: roundMm(Math.max(1, doc.width - sizeMm - 2), 0.5),
        y: 2,
        width: sizeMm,
        height: sizeMm,
        borderRadius: 1,
      });
      toast.success('วางรูปภาพลงบนป้ายเรียบร้อย — สามารถลากปรับตำแหน่งได้เลย');
    } catch {
      toast.error('ไม่สามารถอ่านไฟล์รูปภาพได้');
    }
  };

  const handleReplaceSelectedImage = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    e.target.value = '';
    if (!file || !selectedId) return;
    try {
      const dataUrl = await compressImageFileToDataUrl(file);
      updateElement(selectedId, { src: dataUrl, field: undefined });
      toast.success('เปลี่ยนรูปภาพเรียบร้อย');
    } catch {
      toast.error('ไม่สามารถอ่านไฟล์รูปภาพได้');
    }
  };

  const isCustomCreatedFormat = !isBlankWorkspace && (selectedTemplate?.userCreated || currentSize.FTLbsHhtCode === 'CUSTOM');

  const executeReset = async (callServer: boolean) => {
    setSaving(true);
    try {
      if (callServer) {
        const ok = await onResetLayout(effectiveLbsCode, priType, selectedTemplate?.FTLtmCode);
        if (!ok) return;
        if (!selectedTemplate?.hasOriginal) setRequestedTemplate('');
        setResetConfirmOpen(false);
        return;
      }
      toast.info('ยกเลิกการแก้ไขที่ยังไม่บันทึกแล้ว');
      const fresh = templateDoc ? structuredClone(templateDoc) : isBlankWorkspace
        ? buildEmptyBlankDoc(priType)
        : buildStarterLayout(
            effectiveLbsCode,
            currentSize.FCLbsWidth,
            currentSize.FCLbsHeight,
            priType,
            currentSize.FTLbsName || effectiveLbsCode
          );
      setDoc(fresh);
      setPast([]);
      setFuture([]);
      setSelectedId(fresh.elements[0]?.id || null);
      setDirty(false);
      setResetConfirmOpen(false);
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    if (hasSavedCustom) {
      setResetConfirmOpen(true);
      return;
    }
    void executeReset(false);
  };

  const handleConfirmOverwriteStd = async () => {
    const ok = await handleSave();
    if (ok) {
      setOverwriteConfirmOpen(false);
    }
  };

  // Sheet/Roll layout calculation (supports dynamic preview media override + saved A4/B5/Letter/Roll/Custom)
  const savedPaperSizeRaw = sheetConfigDraft?.paperSize || doc.preprint?.paperSize || currentSize.FTLbsPaperSize || 'A4';
  const savedPaperSize = (
    savedPaperSizeRaw in PAPER_SIZE_PRESETS || allPaperSizes.some((p) => p.FTPpsCode === savedPaperSizeRaw)
      ? savedPaperSizeRaw
      : 'A4'
  ) as PaperSizeCode;

  useEffect(() => {
    setPreviewMedia(
      getDefaultPreviewMediaConfig({
        savedPaperSize,
        savedPaperWidthMm: sheetConfigDraft?.paperWidthMm || doc.preprint?.paperWidthMm || currentSize.FCLbsPaperWidth || undefined,
        savedPaperHeightMm: sheetConfigDraft?.paperHeightMm || doc.preprint?.paperHeightMm || currentSize.FCLbsPaperHeight || undefined,
        lbsGroup: currentSize.FTLbsGroup,
        savedCols: sheetConfigDraft?.cols ?? currentSize.FNLbsCol, cellWidthMm: doc.width, gapHMm: sheetConfigDraft?.gapH ?? currentSize.FCLbsGapH,
        gapVMm: sheetConfigDraft?.gapV ?? currentSize.FCLbsGapV ?? 2,
      })
    );
  }, [
    effectiveLbsCode,
    savedPaperSize,
    currentSize.FTLbsGroup,
    currentSize.FCLbsPaperWidth,
    currentSize.FCLbsPaperHeight,
    currentSize.FCLbsGapV, currentSize.FCLbsGapH, currentSize.FNLbsCol, doc.width,
    sheetConfigDraft?.cols, sheetConfigDraft?.gapH, sheetConfigDraft?.gapV,
    sheetConfigDraft?.paperWidthMm,
    sheetConfigDraft?.paperHeightMm,
    doc.preprint?.paperWidthMm,
    doc.preprint?.paperHeightMm,
  ]);

  const savedMetrics = useMemo(() => {
    const preset = PAPER_SIZE_PRESETS[savedPaperSize];
    const dbP = allPaperSizes.find((p) => p.FTPpsCode === savedPaperSize);
    const paperWidthMm =
      sheetConfigDraft?.paperWidthMm ||
      doc.preprint?.paperWidthMm ||
      currentSize.FCLbsPaperWidth ||
      dbP?.FCPpsWidth ||
      preset?.widthMm ||
      210;
    const paperHeightMm =
      sheetConfigDraft?.paperHeightMm ||
      doc.preprint?.paperHeightMm ||
      currentSize.FCLbsPaperHeight ||
      dbP?.FCPpsHeight ||
      preset?.heightMm ||
      297;
    const magTop = sheetConfigDraft?.magTop ?? currentSize.FCLbsMagTop ?? 3;
    const magLeft = sheetConfigDraft?.magLeft ?? currentSize.FCLbsMagLeft ?? 3;
    const magBottom = sheetConfigDraft?.magBottom ?? currentSize.FCLbsMagBottom ?? magTop;
    const magRight = sheetConfigDraft?.magRight ?? currentSize.FCLbsMagRight ?? magLeft;
    const gapH = sheetConfigDraft?.gapH ?? currentSize.FCLbsGapH ?? 2;
    const gapV = sheetConfigDraft?.gapV ?? currentSize.FCLbsGapV ?? 2;
    const usableW = Math.max(10, paperWidthMm - magLeft - magRight);
    const usableH = Math.max(10, paperHeightMm - magTop - magBottom);
    const calcCols = Math.max(1, Math.floor((usableW + gapH) / (doc.width + gapH)));
    const calcRows = Math.max(1, Math.floor((usableH + gapV) / (doc.height + gapV)));
    const dimsChanged =
      Math.abs(doc.width - (currentSize.FCLbsWidth || doc.width)) > 0.2 ||
      Math.abs(doc.height - (currentSize.FCLbsHeight || doc.height)) > 0.2;
    const cols = sheetConfigDraft?.cols || (!dimsChanged && currentSize.FNLbsCol ? currentSize.FNLbsCol : calcCols);
    const rows = sheetConfigDraft?.rows || (!dimsChanged && currentSize.FNLbsRow ? currentSize.FNLbsRow : calcRows);
    return {
      paperSize: savedPaperSize,
      paperWidthMm,
      paperHeightMm,
      cols,
      rows,
      totalPerSheet: cols * rows,
      magTop,
      magLeft,
      magBottom,
      magRight,
      gapH,
      gapV,
    };
  }, [
    doc.width,
    doc.height,
    savedPaperSize,
    allPaperSizes,
    currentSize.FCLbsWidth,
    currentSize.FCLbsHeight,
    doc.preprint?.paperWidthMm,
    doc.preprint?.paperHeightMm,
    sheetConfigDraft,
    currentSize.FCLbsPaperWidth,
    currentSize.FCLbsPaperHeight,
    currentSize.FNLbsCol,
    currentSize.FNLbsRow,
    currentSize.FCLbsMagTop,
    currentSize.FCLbsMagLeft,
    currentSize.FCLbsMagBottom,
    currentSize.FCLbsMagRight,
    currentSize.FCLbsGapH,
    currentSize.FCLbsGapV,
  ]);

  const a4Metrics = useMemo(
    () =>
      computeEffectivePreviewLayout(
        {
          lbsGroup: currentSize.FTLbsGroup,
          labelWidthMm: doc.width,
          labelHeightMm: doc.height,
          savedPaperSize: savedMetrics.paperSize,
          savedPaperWidthMm: savedMetrics.paperWidthMm,
          savedPaperHeightMm: savedMetrics.paperHeightMm,
          savedCols: savedMetrics.cols,
          savedRows: savedMetrics.rows,
          magTopMm: savedMetrics.magTop,
          magLeftMm: savedMetrics.magLeft,
          magBottomMm: savedMetrics.magBottom,
          magRightMm: savedMetrics.magRight,
          gapHMm: savedMetrics.gapH,
          gapVMm: savedMetrics.gapV,
        },
        previewMedia
      ),
    [doc.width, doc.height, currentSize.FTLbsGroup, savedMetrics, previewMedia]
  );

  const updateSheetSpacingDraft = (patch: Partial<SheetConfigMeta>) => {
    const pw = a4Metrics.paperWidthMm;
    const ph = a4Metrics.paperHeightMm;
    const magTop = patch.magTop ?? a4Metrics.magTopMm;
    const magBottom = patch.magBottom ?? a4Metrics.magBottomMm;
    const magLeft = patch.magLeft ?? a4Metrics.magLeftMm;
    const magRight = patch.magRight ?? a4Metrics.magRightMm;
    const gapH = patch.gapH ?? a4Metrics.gapHMm;
    const gapV = patch.gapV ?? a4Metrics.gapVMm;
    const usableW = Math.max(doc.width, pw - magLeft - magRight);
    const usableH = Math.max(doc.height, ph - magTop - magBottom);
    const cols = Math.max(1, Math.floor((usableW + gapH + 0.05) / (doc.width + gapH)));
    const rows = Math.max(1, Math.floor((usableH + gapV + 0.05) / (doc.height + gapV)));
    setSheetConfigDraft({
      paperSize: a4Metrics.paperSize,
      paperWidthMm: pw,
      paperHeightMm: ph,
      cols,
      rows,
      magTop,
      magBottom,
      magLeft,
      magRight,
      gapH,
      gapV,
      widthMm: doc.width,
      heightMm: doc.height,
    });
    if (previewMedia.mode !== 'AUTO') {
      setPreviewMedia((prev) => ({ ...prev, mode: 'AUTO' }));
    }
    setDirty(true);
  };

  const centerGridOnSheet = () => {
    const pw = a4Metrics.paperWidthMm;
    const ph = a4Metrics.paperHeightMm;
    const gapH = a4Metrics.gapHMm;
    const gapV = a4Metrics.gapVMm;
    const c = Math.max(1, a4Metrics.cols);
    const r = Math.max(1, a4Metrics.rows);
    const gridW = c * doc.width + Math.max(0, c - 1) * gapH;
    const gridH = r * doc.height + Math.max(0, r - 1) * gapV;
    const ml = Math.max(0, roundMm((pw - gridW) / 2, 0.1));
    const mt = Math.max(0, roundMm((ph - gridH) / 2, 0.1));
    updateSheetSpacingDraft({
      magLeft: ml,
      magRight: ml,
      magTop: mt,
      magBottom: mt,
    });
    toast.success(`จัดกึ่งกลางแผ่นแล้ว (ขอบซ้าย/ขวา ${ml} mm · ขอบบน/ล่าง ${mt} mm)`);
  };

  const hasSavedCustom = !isBlankWorkspace && !startingLayout && (selectedTemplate ? Boolean(selectedTemplate.hasOriginal || selectedTemplate.FTLtmCode === designCode) : Boolean(customLayouts[layoutKey]));

  const formatPicker = (
    <SearchableSelect
      value={selectedTemplate?.FTLtmCode || ''}
      onValueChange={selectFormat}
      placeholder={!isBlankWorkspace && effectiveLbsCode ? `แบบตั้งต้น (ยังไม่บันทึก) · ${effectiveLbsCode}` : 'เลือกรูปแบบป้าย...'}
      searchPlaceholder="ค้นหารหัสหรือชื่อแบบป้าย..."
      className="h-8 w-[340px] max-w-[42vw] text-xs"
      dropdownMinWidth={380}
      options={formatOptions}
    />
  );
  return (
    <div className="flex flex-col h-[calc(100vh-58px)] min-h-[640px] bg-slate-100 select-none">
      <input
        ref={uploadImgInputRef}
        type="file"
        accept="image/*"
        className="hidden"
        onChange={handleUploadNewImage}
      />
      <input
        ref={replaceImgInputRef}
        type="file"
        accept="image/*"
        className="hidden"
        onChange={handleReplaceSelectedImage}
      />
      <input
        ref={preprintFileInputRef}
        type="file"
        accept=".pdf,application/pdf,image/*,.png,.jpg,.jpeg,.webp"
        className="hidden"
        onChange={handlePreprintFileChange}
      />
      {/* Top Figma/Canva Toolbar (2 clean rows) */}
      <div className="bg-white border-b border-slate-200 shrink-0">
        {/* Row 1: Format, Price Mode, Dimensions & Sample Product */}
        <div className="px-4 py-2 border-b border-slate-100 flex flex-wrap items-center justify-between gap-3">
          <div className="flex flex-wrap items-center gap-3">
            <div className="flex flex-wrap items-center gap-2">
              <Label className="text-xs font-semibold text-slate-600">ขนาดป้าย</Label>
              <SearchableSelect
                value={blankSizeSelected ? effectiveLbsCode : ''}
                onValueChange={code => {
                  if (dirty && !window.confirm('เปิดป้ายเปล่าและละทิ้งการแก้ไขที่ยังไม่ได้บันทึก?')) return;
                  setRequestedTemplate(`blank:${code}`);
                  onChangeLabelCode(code);
                }}
                options={activeSizes.map(size => ({
                  value: size.FTLbsCode,
                  label: `${size.FTLbsCode} · ${size.FCLbsWidth} × ${size.FCLbsHeight} มม.`,
                  keywords: [size.FTLbsName, sizePreprintKeywords(size, preprintAliases)].join(' '),
                }))}
                placeholder="เลือกขนาดป้าย..."
                searchPlaceholder="ค้นหารหัส ชื่อ หรือขนาดป้าย..."
                className="h-8 w-[230px] max-w-[80vw] text-xs"
              />
              <Label className="text-xs font-semibold text-slate-600 whitespace-nowrap">รูปแบบป้าย:</Label>
              {formatPicker}
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="h-8 text-xs px-2.5 border-[#007a87] text-[#007a87] hover:bg-teal-50 shrink-0"
                onClick={openSaveAsBlankModal}
                title="สร้างรูปแบบป้ายใหม่เป็นหน้ากระดาษเปล่า 100%"
              >
                <Plus className="h-3.5 w-3.5 mr-1" />
                สร้างใหม่ (หน้าว่าง)
              </Button>
            </div>

            {selectedTemplate ? <span className="text-xs text-slate-600">{priType === '2' ? 'ราคาโปรโมชัน' : 'ราคาปกติ'} · {doc.designMode === 'preprint' ? 'Pre-Print' : 'พิมพ์ครบจบ'}</span> : <>
            <div className="inline-flex rounded-md border border-slate-200 bg-slate-50 p-0.5">
              <button
                type="button"
                onClick={() => setPriType('1')}
                className={`px-3 py-1 text-xs font-medium rounded-sm transition-colors ${
                  priType === '1'
                    ? 'bg-white text-[#007a87] shadow-2xs'
                    : 'text-slate-600 hover:text-slate-900'
                }`}
              >
                ราคาปกติ
              </button>
              <button
                type="button"
                onClick={() => setPriType('2')}
                className={`px-3 py-1 text-xs font-medium rounded-sm transition-colors ${
                  priType === '2'
                    ? 'bg-red-600 text-white shadow-2xs'
                    : 'text-slate-600 hover:text-slate-900'
                }`}
              >
                ราคาโปรโมชัน
              </button>
            </div>

            <div className="inline-flex rounded-md border border-slate-200 bg-slate-50 p-0.5" title="สลับระหว่างออกแบบพิมพ์ครบจบ กับพิมพ์ลงกระดาษ Pre-Print">
              <button
                type="button"
                onClick={() => handleToggleDesignMode('full')}
                className={`px-2.5 py-1 text-xs font-medium rounded-sm transition-colors ${
                  (doc.designMode || 'full') === 'full'
                    ? 'bg-white text-slate-800 shadow-2xs'
                    : 'text-slate-500 hover:text-slate-900'
                }`}
              >
                พิมพ์ครบจบ
              </button>
              <button
                type="button"
                onClick={() => handleToggleDesignMode('preprint')}
                className={`px-2.5 py-1 text-xs font-medium rounded-sm transition-colors ${
                  doc.designMode === 'preprint'
                    ? 'bg-amber-600 text-white shadow-2xs'
                    : 'text-slate-500 hover:text-slate-900'
                }`}
              >
                Pre-Print
              </button>
            </div>

            </>}
            <Badge variant="secondary" className="text-[11px] font-mono px-2.5 py-0.5">
              {doc.width}×{doc.height} mm · {savedMetrics.cols}×{savedMetrics.rows} ({savedMetrics.totalPerSheet} ดวง/{getPaperSizeDisplayName(savedMetrics.paperSize, savedMetrics.paperWidthMm, savedMetrics.paperHeightMm)})
            </Badge>

            {isCustomCreatedFormat && (
              <Badge className="bg-sky-100 text-sky-800 hover:bg-sky-100 border border-sky-300 text-[10px]">
                แบบที่สร้างใหม่
              </Badge>
            )}
            {hasSavedCustom && !isCustomCreatedFormat && (
              <Badge className="bg-emerald-100 text-emerald-800 hover:bg-emerald-100 border border-emerald-300 text-[10px]">
                บันทึกทับแบบเดิมแล้ว
              </Badge>
            )}
            {dirty && (
              <Badge className="bg-amber-100 text-amber-800 hover:bg-amber-100 border border-amber-300 text-[10px]">
                ยังไม่บันทึก*
              </Badge>
            )}
          </div>
        </div>

        {/* Row 2: Canvas Tools & Save Actions */}
        <div className="px-4 py-1.5 bg-slate-50/70 flex flex-wrap items-center justify-between gap-2">
          <div className="flex flex-wrap items-center gap-1.5">
            <Button
              size="icon"
              variant="outline"
              className="h-8 w-8 bg-white"
              disabled={past.length === 0}
              onClick={handleUndo}
              title="ย้อนกลับ (Ctrl+Z)"
            >
              <Undo2 className="h-3.5 w-3.5" />
            </Button>
            <Button
              size="icon"
              variant="outline"
              className="h-8 w-8 bg-white"
              disabled={future.length === 0}
              onClick={handleRedo}
              title="ทำซ้ำ (Ctrl+Y)"
            >
              <Redo2 className="h-3.5 w-3.5" />
            </Button>

            <div className="inline-flex items-center border border-slate-200 rounded-md bg-white h-8 px-1">
              <Button
                size="icon"
                variant="ghost"
                className="h-6 w-6"
                onClick={() => setZoom((z) => Math.max(0.5, Number((z - 0.2).toFixed(2))))}
                title="ย่อ"
              >
                <ZoomOut className="h-3.5 w-3.5" />
              </Button>
              <span className="text-[11px] font-mono w-11 text-center font-medium">
                {Math.round(zoom * 100)}%
              </span>
              <Button
                size="icon"
                variant="ghost"
                className="h-6 w-6"
                onClick={() => setZoom((z) => Math.min(4, Number((z + 0.2).toFixed(2))))}
                title="ขยาย"
              >
                <ZoomIn className="h-3.5 w-3.5" />
              </Button>
              <Button
                size="icon"
                variant="ghost"
                className="h-6 w-6"
                onClick={() => setZoom(defaultZoomForSize(doc.width, doc.height))}
                title="ขนาดพอดีจอ"
              >
                <Maximize2 className="h-3 w-3" />
              </Button>
            </div>

            <Button
              size="sm"
              variant={showGrid ? 'secondary' : 'outline'}
              className="h-8 text-xs px-2.5 bg-white"
              onClick={() => setShowGrid((g) => !g)}
              title="แสดง/ซ่อนตารางช่วยเล็งระยะ (mm)"
            >
              <Grid3X3 className="h-3.5 w-3.5 mr-1" />
              ตารางเล็ง
            </Button>

            <label
              className="inline-flex items-center gap-1.5 px-2.5 h-8 bg-white border border-slate-200 rounded-md text-xs text-slate-700 cursor-pointer hover:bg-slate-50"
              title="ดูดองค์ประกอบให้ตรงเส้นทุก 0.5 mm"
            >
              <input
                type="checkbox"
                checked={snapToGrid}
                onChange={(e) => setSnapToGrid(e.target.checked)}
                className="accent-[#007a87]"
              />
              ดูดชิดเส้น 0.5mm
            </label>

            <div className="flex items-center gap-1.5">
              <span className="text-xs text-slate-500 whitespace-nowrap hidden md:inline">สินค้าตัวอย่าง:</span>
              <SearchableSelect
                value={sampleProductCode}
                onValueChange={setSampleProductCode}
                placeholder="สินค้าตัวอย่าง..."
                searchPlaceholder="ค้นหาชื่อ หรือรหัสสินค้า..."
                className="h-8 w-48 text-xs"
                dropdownMinWidth={260}
                options={[
                  { value: '__SAMPLE__', label: 'สินค้าตัวอย่างมาตรฐาน' },
                  ...products.map((p) => ({
                    value: p.FTPdtCode,
                    label: p.FTPdtName,
                    keywords: `${p.FTPdtCode} ${p.FTPdtName}`,
                  })),
                ]}
              />
            </div>

            {doc.designMode === 'preprint' && (doc.preprint?.cellBgDataUrl || doc.preprint?.sheetBgDataUrl) && (
              <div className="hidden xl:flex items-center gap-1.5 px-2 h-8 bg-white border border-slate-200 rounded-md">
                <span className="text-[10px] text-slate-500">ความชัดพื้นทาบ:</span>
                <input
                  type="range"
                  min={15}
                  max={100}
                  step={5}
                  value={Math.round((doc.preprint?.opacity ?? 0.72) * 100)}
                  onChange={(e) => {
                    const op = Math.max(0.15, Math.min(1, Number(e.target.value) / 100));
                    updateDocField(
                      {
                        preprint: {
                          ...(doc.preprint || {
                            scope: 'sheet',
                            paperSize: a4Metrics.paperSize,
                            paperWidthMm: a4Metrics.paperWidthMm,
                            paperHeightMm: a4Metrics.paperHeightMm,
                            opacity: 0.72,
                          }),
                          opacity: op,
                        },
                      },
                      false
                    );
                  }}
                  className="w-16 accent-amber-600 cursor-pointer"
                />
                <span className="text-[10px] font-mono text-slate-600 w-7">
                  {Math.round((doc.preprint?.opacity ?? 0.72) * 100)}%
                </span>
              </div>
            )}

          </div>

          <div className="flex flex-wrap items-center gap-2">
            <Button
              size="sm"
              variant="outline"
              className="h-8 text-xs border-[#007a87] text-[#007a87] bg-white hover:bg-teal-50"
              onClick={() => {
                setSheetPreviewMode('guide');
                setA4PreviewOpen(true);
              }}
            >
              <FileSpreadsheet className="h-3.5 w-3.5 mr-1" />
              Preview เทียบกระดาษ
            </Button>

            {isCustomCreatedFormat ? (
              <Button
                size="sm"
                className="h-8 text-xs bg-[#007a87] hover:bg-[#00606b] text-white"
                onClick={handleSave}
                disabled={saving}
                title="บันทึกการแก้ไขในแบบปัจจุบัน"
              >
                {saving ? (
                  <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                ) : (
                  <Save className="h-3.5 w-3.5 mr-1" />
                )}
                บันทึก
              </Button>
            ) : (
              <Button
                size="sm"
                className="h-8 text-xs bg-[#007a87] hover:bg-[#00606b] text-white"
                onClick={openSaveAsModal}
                disabled={saving}
                title="ตั้งชื่อใหม่และบันทึกเป็นแบบใหม่ (ไม่ทับแบบมาตรฐานเดิม)"
              >
                {saving ? (
                  <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />
                ) : (
                  <FilePlus2 className="h-3.5 w-3.5 mr-1" />
                )}
                บันทึกเป็นแบบใหม่
              </Button>
            )}

            {(!isBlankWorkspace || dirty || hasSavedCustom) && (
            <div className="relative">
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="h-8 text-xs bg-white text-slate-700"
                onClick={() => setMoreActionsOpen((o) => !o)}
                disabled={saving}
              >
                <MoreHorizontal className="h-3.5 w-3.5 mr-1" />
                เพิ่มเติม
                <ChevronDown className="h-3 w-3 ml-1" />
              </Button>

              {moreActionsOpen && (
                <div
                  className="absolute right-0 top-9 z-50 w-52 rounded-md border border-slate-200 bg-white shadow-lg py-1 text-xs"
                  onMouseLeave={() => setMoreActionsOpen(false)}
                >
                  {isCustomCreatedFormat ? (
                    <button
                      type="button"
                      className="w-full text-left px-3 py-2 hover:bg-slate-50 flex items-center gap-2 text-slate-700"
                      onClick={() => {
                        setMoreActionsOpen(false);
                        openSaveAsModal();
                      }}
                    >
                      <FilePlus2 className="h-3.5 w-3.5 text-[#007a87]" />
                      บันทึกเป็นแบบใหม่ (ทำสำเนา)
                    </button>
                  ) : !isBlankWorkspace && !startingLayout ? (
                    <button
                      type="button"
                      className="w-full text-left px-3 py-2 hover:bg-amber-50 flex items-center gap-2 text-amber-800"
                      onClick={() => {
                        setMoreActionsOpen(false);
                        setOverwriteConfirmOpen(true);
                      }}
                    >
                      <Save className="h-3.5 w-3.5" />
                      บันทึกทับแบบ STD เดิม
                    </button>
                  ) : null}

                  {(dirty || hasSavedCustom) && (
                    <button
                      type="button"
                      className="w-full text-left px-3 py-2 hover:bg-slate-50 flex items-center gap-2 text-slate-700"
                      onClick={() => {
                        setMoreActionsOpen(false);
                        handleReset();
                      }}
                    >
                      <RotateCcw className="h-3.5 w-3.5" />
                      {isBlankWorkspace
                        ? 'ล้างหน้ากระดาษให้ว่าง'
                        : !isCustomCreatedFormat && hasSavedCustom
                          ? 'คืนค่า STD เดิม'
                          : 'รีเซ็ตเป็นค่าเริ่มต้น'}
                    </button>
                  )}

                  {!isBlankWorkspace && currentSize.FTLbsHhtCode === 'CUSTOM' && (sizeTemplates?.length ?? 0) <= 1 && onDeleteCustomFormat && (
                    <button
                      type="button"
                      className="w-full text-left px-3 py-2 hover:bg-red-50 flex items-center gap-2 text-red-600 border-t border-slate-100 mt-1"
                      onClick={async () => {
                        setMoreActionsOpen(false);
                        setSaving(true);
                        try {
                          await onDeleteCustomFormat(effectiveLbsCode);
                        } finally {
                          setSaving(false);
                        }
                      }}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                      ลบแบบที่สร้างนี้
                    </button>
                  )}
                </div>
              )}
            </div>
            )}
          </div>
        </div>
      </div>

      {/* Main 3-Pane Studio Workspace */}
      <div className="flex-1 min-h-0 flex overflow-hidden">
        {/* Left Panel: Canva Elements / Shapes / Icons / Layers */}
        <div className="w-72 bg-white border-r border-slate-200 flex flex-col shrink-0 min-h-0">
          <Tabs
            value={leftTab}
            onValueChange={(v) => setLeftTab(v as typeof leftTab)}
            className="flex flex-col flex-1 min-h-0 gap-0"
          >
            <TabsList className="grid grid-cols-4 rounded-none border-b border-slate-200 bg-slate-50 p-1 h-10 shrink-0">
              <TabsTrigger value="fields" className="text-[11px] px-1 gap-1">
                <Type className="h-3.5 w-3.5" />
                ข้อมูล
              </TabsTrigger>
              <TabsTrigger value="shapes" className="text-[11px] px-1 gap-1">
                <Shapes className="h-3.5 w-3.5" />
                รูปทรง
              </TabsTrigger>
              <TabsTrigger value="icons" className="text-[11px] px-1 gap-1">
                <Sparkles className="h-3.5 w-3.5" />
                ไอคอน
              </TabsTrigger>
              <TabsTrigger value="layers" className="text-[11px] px-1 gap-1">
                <Layers className="h-3.5 w-3.5" />
                เลเยอร์
              </TabsTrigger>
            </TabsList>

            <TabsContent value="fields" className="flex-1 min-h-0 overflow-y-auto p-3 space-y-2 mt-0">
              <p className="text-[11px] text-slate-500">
                คลิกเพื่อวางฟิลด์ข้อมูลสินค้าจากฐานข้อมูลลงบนป้าย:
              </p>
              <div className="space-y-1.5">
                {dbDataFields.main.map((f) => (
                  <button
                    key={f.field}
                    type="button"
                    onClick={() =>
                      addElement({
                        type: f.type,
                        field: f.field,
                        label: f.label.split(' (')[0],
                        x: roundMm(Math.max(1, (doc.width - Math.min(f.defaultW, doc.width - 2)) / 2)),
                        y: roundMm(Math.max(1, (doc.height - Math.min(f.defaultH, doc.height - 2)) / 2)),
                        width: f.defaultW,
                        height: f.defaultH,
                        fontSize: f.fontSize,
                        fontWeight: f.fontWeight || 'normal',
                        color: f.color || '#111111',
                        align: f.align || 'left',
                        prefix: f.prefix,
                        suffix: f.suffix,
                        strike: f.strike,
                        multiLine: f.multiLine,
                        showText: f.type === 'barcode' ? true : undefined,
                      })
                    }
                    className="w-full text-left px-2.5 py-2 rounded-md border border-slate-200 hover:border-[#007a87] hover:bg-teal-50/40 transition-colors flex items-center justify-between gap-2 group"
                  >
                    <div>
                      <div className="text-xs font-medium text-slate-800">{f.label}</div>
                      <div className="text-[10px] font-mono text-slate-400">{f.field}</div>
                    </div>
                    <Plus className="h-3.5 w-3.5 text-slate-400 group-hover:text-[#007a87] shrink-0" />
                  </button>
                ))}
              </div>

              {dbDataFields.adv.length > 0 && (
              <details className="pt-2 border-t border-slate-100">
                <summary className="text-[11px] font-medium text-slate-500 cursor-pointer hover:text-slate-800">
                  ฟิลด์เสริมอื่นๆ ({dbDataFields.adv.length})
                </summary>
                <div className="space-y-1.5 mt-2">
                  {dbDataFields.adv.map((f) => (
                    <button
                      key={f.field}
                      type="button"
                      onClick={() =>
                        addElement({
                          type: f.type,
                          field: f.field,
                          label: f.label.split(' (')[0],
                          x: roundMm(Math.max(1, (doc.width - Math.min(f.defaultW, doc.width - 2)) / 2)),
                          y: roundMm(Math.max(1, (doc.height - Math.min(f.defaultH, doc.height - 2)) / 2)),
                          width: f.defaultW,
                          height: f.defaultH,
                          fontSize: f.fontSize,
                          fontWeight: f.fontWeight || 'normal',
                          color: f.color || '#111111',
                          align: f.align || 'left',
                          prefix: f.prefix,
                          suffix: f.suffix,
                          strike: f.strike,
                          multiLine: f.multiLine,
                        })
                      }
                      className="w-full text-left px-2.5 py-1.5 rounded-md border border-slate-200 hover:border-[#007a87] hover:bg-teal-50/40 transition-colors flex items-center justify-between gap-2 group"
                    >
                      <div>
                        <div className="text-[11px] font-medium text-slate-700">{f.label}</div>
                        <div className="text-[10px] font-mono text-slate-400">{f.field}</div>
                      </div>
                      <Plus className="h-3 w-3 text-slate-400 group-hover:text-[#007a87] shrink-0" />
                    </button>
                  ))}
                </div>
              </details>
              )}
            </TabsContent>

            <TabsContent value="shapes" className="flex-1 min-h-0 overflow-y-auto p-3 space-y-3 mt-0">
              {doc.designMode === 'preprint' && (
                <div className="rounded-md bg-amber-50 border border-amber-200 px-2.5 py-2 text-[11px] text-amber-900 leading-snug">
                  โหมด Pre-Print นิยมเพิ่มเฉพาะฟิลด์ในแท็บ <strong>&ldquo;ข้อมูล&rdquo;</strong> เพื่อไม่ให้ทับลวดลายบนกระดาษจริงของลูกค้า
                </div>
              )}
              <div className="space-y-1.5 pb-2 border-b border-slate-100">
                <Label className="text-[11px] font-semibold text-slate-600">รูปภาพสินค้า & โลโก้</Label>
                <div className="grid grid-cols-2 gap-2">
                  <button
                    type="button"
                    onClick={() => {
                      const sz = roundMm(Math.max(10, Math.min(doc.width * 0.32, doc.height * 0.45)), 0.5);
                      addElement({
                        type: 'image',
                        field: 'product.image',
                        label: 'รูปสินค้า (อัตโนมัติ)',
                        objectFit: 'contain',
                        x: roundMm(Math.max(1, doc.width - sz - 3), 0.5),
                        y: 3,
                        width: sz,
                        height: sz,
                        borderRadius: 1,
                      });
                    }}
                    className="p-2.5 rounded-md border border-slate-200 hover:border-[#007a87] hover:bg-teal-50/40 flex flex-col items-center gap-1 text-xs"
                  >
                    <ImageIcon className="h-5 w-5 text-[#007a87]" />
                    <span>รูปสินค้า (DB)</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => uploadImgInputRef.current?.click()}
                    className="p-2.5 rounded-md border border-slate-200 hover:border-[#007a87] hover:bg-teal-50/40 flex flex-col items-center gap-1 text-xs"
                  >
                    <Upload className="h-5 w-5 text-sky-600" />
                    <span>อัปโหลดรูป/โลโก้</span>
                  </button>
                </div>
              </div>

              <div className="space-y-1.5">
                <Label className="text-[11px] font-semibold text-slate-600">ข้อความ & รูปทรงพื้นฐาน</Label>
                <div className="grid grid-cols-2 gap-2">
                  <button
                    type="button"
                    onClick={() =>
                      addElement({
                        type: 'static',
                        label: 'ข้อความอิสระ',
                        text: 'ข้อความใหม่',
                        x: 4,
                        y: 4,
                        width: 34,
                        height: 7,
                        fontSize: 12,
                        fontWeight: 'bold',
                        color: '#111111',
                        align: 'center',
                      })
                    }
                    className="p-2.5 rounded-md border border-slate-200 hover:border-[#007a87] hover:bg-teal-50/40 flex flex-col items-center gap-1 text-xs"
                  >
                    <Type className="h-5 w-5 text-slate-700" />
                    <span>ข้อความอิสระ</span>
                  </button>
                  <button
                    type="button"
                    onClick={() =>
                      addElement({
                        type: 'rect',
                        label: 'กล่องสี / พื้นหลัง',
                        x: 2,
                        y: 2,
                        width: 30,
                        height: 12,
                        fill: '#FEF08A',
                        borderColor: '#CA8A04',
                        borderWidth: 0.3,
                        borderRadius: 1,
                      })
                    }
                    className="p-2.5 rounded-md border border-slate-200 hover:border-[#007a87] hover:bg-teal-50/40 flex flex-col items-center gap-1 text-xs"
                  >
                    <div className="w-6 h-4 rounded-xs bg-yellow-300 border border-yellow-600" />
                    <span>กล่องสี</span>
                  </button>
                  <button
                    type="button"
                    onClick={() =>
                      addElement({
                        type: 'circle',
                        label: 'วงกลมตกแต่ง',
                        x: 4,
                        y: 4,
                        width: 14,
                        height: 14,
                        fill: '#DC2626',
                        borderWidth: 0,
                      })
                    }
                    className="p-2.5 rounded-md border border-slate-200 hover:border-[#007a87] hover:bg-teal-50/40 flex flex-col items-center gap-1 text-xs"
                  >
                    <div className="w-5 h-5 rounded-full bg-red-600" />
                    <span>วงกลม</span>
                  </button>
                  <button
                    type="button"
                    onClick={() =>
                      addElement({
                        type: 'line',
                        label: 'เส้นคั่นแนวนอน',
                        x: 4,
                        y: roundMm(doc.height / 2),
                        width: Math.max(10, doc.width - 8),
                        height: 0.5,
                        fill: '#94A3B8',
                        color: '#94A3B8',
                      })
                    }
                    className="p-2.5 rounded-md border border-slate-200 hover:border-[#007a87] hover:bg-teal-50/40 flex flex-col items-center gap-1 text-xs"
                  >
                    <div className="w-7 h-0.5 bg-slate-500 my-2" />
                    <span>เส้นคั่น</span>
                  </button>
                </div>
              </div>

              <div className="space-y-1.5 pt-2 border-t border-slate-100">
                <Label className="text-[11px] font-semibold text-slate-600">
                  แถบป้ายโปรโมชันสำเร็จรูป (คลิกเพื่อวาง)
                </Label>
                <div className="grid grid-cols-2 gap-2">
                  {DESIGN_BADGE_PRESETS.map((bp) => (
                    <button
                      key={bp.text}
                      type="button"
                      onClick={() =>
                        addElement({
                          type: 'badge',
                          label: bp.label,
                          text: bp.text,
                          fill: bp.fill,
                          color: bp.color,
                          borderRadius: bp.borderRadius,
                          x: 3,
                          y: 3,
                          width: bp.width,
                          height: bp.height,
                          fontSize: bp.fontSize,
                          fontWeight: 'bold',
                          align: 'center',
                        })
                      }
                      className="p-1.5 rounded-md border border-slate-200 hover:border-[#007a87] flex items-center justify-center"
                    >
                      <span
                        className="px-2 py-0.5 rounded text-[11px] font-bold w-full text-center"
                        style={{ backgroundColor: bp.fill, color: bp.color }}
                      >
                        {bp.text}
                      </span>
                    </button>
                  ))}
                </div>
              </div>
            </TabsContent>

            <TabsContent value="icons" className="flex-1 min-h-0 overflow-y-auto p-3 space-y-2 mt-0">
              <p className="text-[11px] text-slate-500">
                คลิกไอคอนเพื่อวางลงบนป้าย (ปรับสีและขนาดได้ในแถบด้านขวา):
              </p>
              <div className="grid grid-cols-3 gap-2">
                {DESIGN_ICON_LIST.map((ic) => (
                  <button
                    key={ic.name}
                    type="button"
                    onClick={() =>
                      addElement({
                        type: 'icon',
                        label: `ไอคอน ${ic.label}`,
                        iconName: ic.name,
                        x: 4,
                        y: 4,
                        width: 9,
                        height: 9,
                        color: ic.defaultColor,
                      })
                    }
                    className="p-2 rounded-md border border-slate-200 hover:border-[#007a87] hover:bg-teal-50/40 flex flex-col items-center gap-1"
                  >
                    <svg
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke={ic.defaultColor}
                      strokeWidth={2}
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      className="w-6 h-6"
                    >
                      {ic.paths.map((d, i) => (
                        <path key={i} d={d} />
                      ))}
                    </svg>
                    <span className="text-[10px] text-slate-600 truncate w-full text-center">
                      {ic.label}
                    </span>
                  </button>
                ))}
              </div>
            </TabsContent>

            <TabsContent value="layers" className="flex-1 min-h-0 overflow-y-auto p-2 space-y-1 mt-0">
              {doc.elements.length === 0 ? (
                <div className="text-center py-8 text-xs text-slate-400">
                  ยังไม่มีองค์ประกอบบนป้าย
                </div>
              ) : (
                [...doc.elements].reverse().map((el) => {
                  const isSel = el.id === selectedId;
                  return (
                    <div
                      key={el.id}
                      onClick={() => setSelectedId(el.id)}
                      className={`px-2 py-1.5 rounded-md border text-xs flex items-center justify-between gap-1 cursor-pointer ${
                        isSel
                          ? 'bg-teal-50 border-[#007a87] text-[#007a87]'
                          : 'bg-white border-slate-200 hover:bg-slate-50 text-slate-700'
                      }`}
                    >
                      <div className="min-w-0 flex-1">
                        <div className="font-medium truncate">{el.label || el.type}</div>
                        <div className="text-[10px] text-slate-400 font-mono">
                          {Number(el.x.toFixed(1))},{Number(el.y.toFixed(1))} · {Number(el.width.toFixed(1))}×{Number(el.height.toFixed(1))}mm
                        </div>
                      </div>
                      <div className="flex items-center gap-0.5 shrink-0" onClick={(e) => e.stopPropagation()}>
                        <button
                          type="button"
                          onClick={() => moveLayer(el.id, 'up')}
                          className="p-1 hover:bg-slate-200 rounded"
                          title="เลื่อนขึ้นบน"
                        >
                          <ArrowUp className="h-3 w-3" />
                        </button>
                        <button
                          type="button"
                          onClick={() => moveLayer(el.id, 'down')}
                          className="p-1 hover:bg-slate-200 rounded"
                          title="เลื่อนลงล่าง"
                        >
                          <ArrowDown className="h-3 w-3" />
                        </button>
                        <button
                          type="button"
                          onClick={() => updateElement(el.id, { visible: el.visible === false })}
                          className="p-1 hover:bg-slate-200 rounded"
                          title={el.visible === false ? 'แสดง' : 'ซ่อน'}
                        >
                          {el.visible === false ? (
                            <EyeOff className="h-3 w-3 text-slate-400" />
                          ) : (
                            <Eye className="h-3 w-3" />
                          )}
                        </button>
                        <button
                          type="button"
                          onClick={() => updateElement(el.id, { locked: !el.locked })}
                          className="p-1 hover:bg-slate-200 rounded"
                          title={el.locked ? 'ปลดล็อก' : 'ล็อกตำแหน่ง'}
                        >
                          {el.locked ? (
                            <Lock className="h-3 w-3 text-amber-600" />
                          ) : (
                            <Unlock className="h-3 w-3 text-slate-400" />
                          )}
                        </button>
                        <button
                          type="button"
                          onClick={() => deleteElement(el.id)}
                          className="p-1 hover:bg-red-100 text-red-500 rounded"
                          title="ลบ"
                        >
                          <Trash2 className="h-3 w-3" />
                        </button>
                      </div>
                    </div>
                  );
                })
              )}
            </TabsContent>
          </Tabs>
        </div>

        {/* Center Interactive Figma-Style Canvas */}
        <div
          className="flex-1 min-w-0 min-h-0 overflow-auto relative flex p-8"
          style={{
            backgroundColor: '#e2e8f0',
            backgroundImage: 'radial-gradient(#cbd5e1 1px, transparent 1px)',
            backgroundSize: '16px 16px',
          }}
          onPointerDown={() => setSelectedId(null)}
        >
          <div className="m-auto flex flex-col items-center gap-2">
          {doc.designMode === 'preprint' && !doc.preprint?.cellBgDataUrl && !doc.preprint?.sheetBgDataUrl && (
            <div className="flex flex-wrap items-center justify-center gap-2 rounded-lg border border-amber-300 bg-amber-50/95 px-3 py-2 text-xs text-amber-900 shadow-xs">
              <span>ยังไม่ได้นำเข้ากระดาษ Pre-Print:</span>
              <Button
                type="button"
                size="sm"
                className="h-7 text-xs bg-amber-600 hover:bg-amber-700 text-white"
                onClick={openImportPreprintModal}
              >
                <FileUp className="h-3.5 w-3.5 mr-1" />
                นำเข้าฟอร์ม Pre-Print (PDF/รูป)
              </Button>
              {doc.elements.length > 0 && (
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  className="h-7 text-xs bg-white border-amber-300 text-amber-900 hover:bg-amber-100"
                  onClick={handleStartBlankPreprint}
                >
                  เริ่มหน้าเปล่าทั้งหมด
                </Button>
              )}
            </div>
          )}

          {/* Scaled Artboard Wrapper */}
          <div
            className="relative shadow-xl bg-white"
            style={{
              width: `${doc.width * zoom}mm`,
              height: `${doc.height * zoom}mm`,
              outline: boardResizing
                ? '2px solid #007a87'
                : doc.designMode === 'preprint'
                  ? '1.5px dashed #d97706'
                  : undefined,
            }}
            onPointerDown={(e) => e.stopPropagation()}
          >
            {/* Board Resize Handles (e, s, se) — ลากปรับขนาดป้ายได้อิสระ */}
            <div
              onPointerDown={(e) => startBoardResize(e, 'e')}
              title="ลากขอบขวาเพื่อปรับความกว้างป้าย (กด Alt เพื่อขยับละเอียด 0.1 mm)"
              className="group absolute top-0 bottom-0 -right-2 w-3 cursor-ew-resize z-40 flex items-center justify-center"
            >
              <div className="w-1 h-8 rounded-full bg-slate-400/70 group-hover:bg-[#007a87] transition-colors" />
            </div>
            <div
              onPointerDown={(e) => startBoardResize(e, 's')}
              title="ลากขอบล่างเพื่อปรับความสูงป้าย (กด Alt เพื่อขยับละเอียด 0.1 mm)"
              className="group absolute left-0 right-0 -bottom-2 h-3 cursor-ns-resize z-40 flex items-center justify-center"
            >
              <div className="h-1 w-8 rounded-full bg-slate-400/70 group-hover:bg-[#007a87] transition-colors" />
            </div>
            <div
              onPointerDown={(e) => startBoardResize(e, 'se')}
              title="ลากมุมขวาล่างเพื่อปรับขนาดป้าย กว้าง × สูง"
              className="absolute -right-2 -bottom-2 w-4 h-4 rounded-sm bg-white border-2 border-[#007a87] shadow-xs cursor-nwse-resize z-50 hover:scale-110 transition-transform"
            />
            {boardResizing && (
              <div className="absolute -bottom-8 right-0 bg-[#007a87] text-white text-[11px] px-2 py-0.5 rounded font-mono whitespace-nowrap pointer-events-none shadow-md z-50">
                {doc.width.toFixed(1)} × {doc.height.toFixed(1)} mm
              </div>
            )}
            {/* Actual Live Render underneath */}
            <CustomDesignRenderer
              layout={doc}
              line={sampleLine}
              scale={zoom}
              preprintView={doc.designMode === 'preprint' ? 'cell-guide' : 'ink'}
            />

            {doc.elements.length === 0 && (
              <div className="absolute inset-0 flex flex-col items-center justify-center p-4 text-center pointer-events-none select-none">
                <div
                  className={`rounded-md bg-white/90 border border-dashed px-4 py-3 text-xs text-slate-600 max-w-[280px] ${
                    doc.designMode === 'preprint' ? 'border-amber-400' : 'border-slate-300'
                  }`}
                >
                  <div className="font-medium text-slate-800">
                    {doc.designMode === 'preprint' ? 'หน้ากระดาษเปล่า (โหมด Pre-Print)' : 'หน้ากระดาษเปล่าสำหรับออกแบบป้าย'}
                  </div>
                  <div className="text-[11px] text-slate-400 mt-1 leading-relaxed">
                    {doc.designMode === 'preprint'
                      ? 'กด "นำเข้าฟอร์ม Pre-Print" ด้านบน หรือคลิกข้อมูลจากแถบซ้ายเพื่อวางจุดพิมพ์'
                      : 'คลิกเพิ่มข้อมูล รูปทรง หรือไอคอนจากแถบซ้าย หรือเลือกรูปแบบที่มีอยู่ด้านบนเพื่อแก้ไข'}
                  </div>
                </div>
              </div>
            )}

            {/* Grid Overlay (5mm major / 1mm minor) */}
            {showGrid && (
              <div
                className="absolute inset-0 pointer-events-none"
                style={{
                  backgroundImage: `linear-gradient(to right, rgba(0,122,135,0.12) 1px, transparent 1px), linear-gradient(to bottom, rgba(0,122,135,0.12) 1px, transparent 1px)`,
                  backgroundSize: `${5 * zoom}mm ${5 * zoom}mm`,
                }}
              />
            )}

            {/* Center Alignment Snap Guides */}
            {snapGuideX !== null && (
              <div
                className="absolute top-0 bottom-0 w-px bg-pink-500 pointer-events-none z-30"
                style={{ left: `${snapGuideX * zoom}mm` }}
              />
            )}
            {snapGuideY !== null && (
              <div
                className="absolute left-0 right-0 h-px bg-pink-500 pointer-events-none z-30"
                style={{ top: `${snapGuideY * zoom}mm` }}
              />
            )}

            {/* Interactive Hit/Selection Overlay Layer */}
            {doc.elements
              .filter((el) => el.visible !== false)
              .map((el) => {
                const isSel = el.id === selectedId;
                return (
                  <div
                    key={`overlay-${el.id}`}
                    onPointerDown={(e) => startMove(e, el)}
                    style={{
                      position: 'absolute',
                      left: `${el.x * zoom}mm`,
                      top: `${el.y * zoom}mm`,
                      width: `${Math.max(0.4, el.width) * zoom}mm`,
                      height: `${Math.max(0.4, el.height) * zoom}mm`,
                      cursor: el.locked ? 'not-allowed' : 'move',
                      outline: isSel
                        ? '2px solid #0284c7'
                        : '1px dashed rgba(100,116,139,0.22)',
                      outlineOffset: '0px',
                      zIndex: isSel ? 25 : 10,
                      boxSizing: 'border-box',
                      transform: el.rotation ? `rotate(${el.rotation}deg)` : undefined,
                      transformOrigin: 'center center',
                    }}
                    title={`${el.label} (${el.x}, ${el.y} · ${el.width}×${el.height} mm)`}
                  >
                    {isSel && !el.locked && (
                      <>
                        {(
                          [
                            ['nw', '-top-1.5 -left-1.5 cursor-nwse-resize'],
                            ['n', '-top-1.5 left-1/2 -translate-x-1/2 cursor-ns-resize'],
                            ['ne', '-top-1.5 -right-1.5 cursor-nesw-resize'],
                            ['e', 'top-1/2 -translate-y-1/2 -right-1.5 cursor-ew-resize'],
                            ['se', '-bottom-1.5 -right-1.5 cursor-nwse-resize'],
                            ['s', '-bottom-1.5 left-1/2 -translate-x-1/2 cursor-ns-resize'],
                            ['sw', '-bottom-1.5 -left-1.5 cursor-nesw-resize'],
                            ['w', 'top-1/2 -translate-y-1/2 -left-1.5 cursor-ew-resize'],
                          ] as Array<[HandleDir, string]>
                        ).map(([dir, cls]) => (
                          <div
                            key={dir}
                            onPointerDown={(e) => startResize(e, el, dir)}
                            className={`absolute w-2.5 h-2.5 bg-white border-2 border-sky-600 rounded-[1px] z-30 ${cls}`}
                          />
                        ))}
                        <div className="absolute -top-5 left-0 bg-sky-600 text-white text-[10px] px-1.5 py-0.5 rounded-xs font-mono whitespace-nowrap pointer-events-none shadow-xs">
                          {el.width.toFixed(1)} × {el.height.toFixed(1)} mm
                        </div>
                      </>
                    )}
                  </div>
                );
              })}
          </div>
          </div>
        </div>

        {/* Right Panel: Figma Inspector */}
        <div className="w-80 bg-white border-l border-slate-200 flex flex-col shrink-0 min-h-0 overflow-y-auto">
          {/* Card/Page Properties */}
          <div className="p-3 border-b border-slate-200 space-y-3">
            <div className="flex items-center justify-between gap-1.5 text-xs font-semibold text-slate-800">
              <div className="flex items-center gap-1.5">
                <Palette className="h-3.5 w-3.5 text-[#007a87]" />
                {doc.designMode === 'preprint'
                  ? `ฉากหลัง Pre-Print (${doc.width}×${doc.height} mm)`
                  : `พื้นหลังและกรอบป้าย (${doc.width}×${doc.height} mm)`}
              </div>
              {doc.designMode === 'preprint' && (
                <Badge className="bg-amber-100 text-amber-800 hover:bg-amber-100 border border-amber-300 text-[10px]">
                  Pre-Print
                </Badge>
              )}
            </div>

            {/* Label Artboard Dimensions Editor (always editable + draggable on canvas) */}
            <div className="rounded-md bg-slate-50 border border-slate-200 p-2 space-y-2">
              <div className="grid grid-cols-12 gap-1.5 items-end">
                <div className="col-span-4 space-y-0.5">
                  <Label className="text-[10px] text-slate-600">กว้าง (mm)</Label>
                  <Input
                    type="number"
                    step={0.5}
                    min={10}
                    max={420}
                    value={boardWInput}
                    onChange={(e) => {
                      const raw = e.target.value;
                      setBoardWInput(raw);
                      const val = parseFloat(raw);
                      if (Number.isFinite(val) && val >= 10 && val <= 420) {
                        resizeBoardTo(val, doc.height);
                      }
                    }}
                    onBlur={() => {
                      const val = parseFloat(boardWInput);
                      if (Number.isFinite(val)) {
                        resizeBoardTo(val, doc.height);
                        setBoardWInput(String(roundMm(Math.max(10, Math.min(420, val)), 0.1)));
                      } else {
                        setBoardWInput(String(doc.width));
                      }
                    }}
                    className="h-7 text-xs font-mono bg-white px-1.5"
                  />
                </div>
                <div className="col-span-4 space-y-0.5">
                  <Label className="text-[10px] text-slate-600">สูง (mm)</Label>
                  <Input
                    type="number"
                    step={0.5}
                    min={10}
                    max={420}
                    value={boardHInput}
                    onChange={(e) => {
                      const raw = e.target.value;
                      setBoardHInput(raw);
                      const val = parseFloat(raw);
                      if (Number.isFinite(val) && val >= 10 && val <= 420) {
                        resizeBoardTo(doc.width, val);
                      }
                    }}
                    onBlur={() => {
                      const val = parseFloat(boardHInput);
                      if (Number.isFinite(val)) {
                        resizeBoardTo(doc.width, val);
                        setBoardHInput(String(roundMm(Math.max(10, Math.min(420, val)), 0.1)));
                      } else {
                        setBoardHInput(String(doc.height));
                      }
                    }}
                    className="h-7 text-xs font-mono bg-white px-1.5"
                  />
                </div>
                <div className="col-span-4">
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    className="h-7 w-full text-[11px] px-1.5 bg-white"
                    onClick={() => resizeBoardTo(doc.height, doc.width, false)}
                    title="สลับกว้าง × สูง (แนวตั้ง ↔ แนวนอน)"
                  >
                    สลับแนว
                  </Button>
                </div>
              </div>
              <div className="flex items-center justify-between gap-1.5">
                <span className="text-[10px] text-slate-500">ลากขอบขวา/ล่างของป้ายเพื่อยืด-หดได้อิสระ</span>
                {doc.elements.length > 0 && (
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    className="h-6 text-[10px] px-2 bg-white border-[#007a87]/40 text-[#007a87] hover:bg-teal-50 shrink-0"
                    onClick={fitElementsToBoard}
                    title="ปรับตำแหน่งและขนาดข้อความที่ล้นให้พอดีกรอบป้าย"
                  >
                    จัดพอดีกรอบ
                  </Button>
                )}
              </div>
            </div>

            {doc.designMode === 'preprint' ? (
              selectedElement ? (
                <div className="flex items-center justify-between gap-2 text-[11px] text-slate-600">
                  <span className="truncate">
                    {doc.preprint?.sourceName ? `ฟอร์ม: ${doc.preprint.sourceName}` : 'ยังไม่มีไฟล์ฟอร์ม Pre-Print'}
                  </span>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    className="h-6 text-[10px] px-2 shrink-0 border-amber-300 text-amber-800 bg-amber-50 hover:bg-amber-100"
                    onClick={openImportPreprintModal}
                  >
                    <FileUp className="h-3 w-3 mr-1" />
                    เปลี่ยนไฟล์
                  </Button>
                </div>
              ) : (
              <div className="space-y-2.5">
                <div className="rounded-md bg-amber-50 border border-amber-200 p-2 text-[11px] text-amber-900 leading-relaxed">
                  ภาพฟอร์ม Pre-Print ใช้ช่วยเล็งตำแหน่งและดู Preview บนจอเท่านั้น <strong>ไม่ถูกพิมพ์ลงกระดาษจริง</strong>
                </div>

                {doc.preprint?.sourceName && (
                  <div className="text-[11px] text-slate-600 truncate">
                    ไฟล์: <strong className="text-slate-800">{doc.preprint.sourceName}</strong>
                  </div>
                )}

                <div className="space-y-1">
                  <div className="flex items-center justify-between text-[11px] text-slate-600">
                    <span>ความชัดภาพอ้างอิง</span>
                    <span className="font-mono font-semibold">
                      {Math.round((doc.preprint?.opacity ?? 0.72) * 100)}%
                    </span>
                  </div>
                  <input
                    type="range"
                    min={15}
                    max={100}
                    step={5}
                    value={Math.round((doc.preprint?.opacity ?? 0.72) * 100)}
                    onChange={(e) => {
                      const op = Math.max(0.15, Math.min(1, Number(e.target.value) / 100));
                      updateDocField(
                        {
                          preprint: {
                            ...(doc.preprint || {
                              scope: 'sheet',
                              paperSize: a4Metrics.paperSize,
                              paperWidthMm: a4Metrics.paperWidthMm,
                              paperHeightMm: a4Metrics.paperHeightMm,
                              opacity: 0.72,
                            }),
                            opacity: op,
                          },
                        },
                        false
                      );
                    }}
                    className="w-full accent-amber-600 cursor-pointer"
                  />
                </div>

                <div className="grid grid-cols-1 gap-1.5">
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    className="h-8 text-xs border-amber-400 text-amber-800 bg-amber-50 hover:bg-amber-100"
                    onClick={openImportPreprintModal}
                  >
                    <FileUp className="h-3.5 w-3.5 mr-1.5" />
                    นำเข้า / เปลี่ยนไฟล์ Pre-Print (PDF/รูป)
                  </Button>
                  {doc.elements.length > 0 && (
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      className="h-7 text-[11px] text-slate-700"
                      onClick={handleStartBlankPreprint}
                    >
                      ล้างเป็นหน้าเปล่าทั้งหมด
                    </Button>
                  )}
                </div>
              </div>
              )
            ) : (
              <>
                <ColorSwatchPicker
                  label="สีพื้นหลังป้าย"
                  value={doc.backgroundColor}
                  onChange={(c) => updateDocField({ backgroundColor: c })}
                />

                <div className="grid grid-cols-3 gap-2">
                  <div className="space-y-1">
                    <Label className="text-[11px] text-slate-600">ขอบ (mm)</Label>
                    <Input
                      type="number"
                      step={0.1}
                      min={0}
                      max={5}
                      value={doc.borderWidth}
                      onChange={(e) =>
                        updateDocField({ borderWidth: Math.max(0, parseFloat(e.target.value) || 0) })
                      }
                      className="h-7 text-xs"
                    />
                  </div>
                  <div className="space-y-1">
                    <Label className="text-[11px] text-slate-600">มุมโค้ง (mm)</Label>
                    <Input
                      type="number"
                      step={0.5}
                      min={0}
                      max={20}
                      value={doc.borderRadius}
                      onChange={(e) =>
                        updateDocField({ borderRadius: Math.max(0, parseFloat(e.target.value) || 0) })
                      }
                      className="h-7 text-xs"
                    />
                  </div>
                  <div className="space-y-1">
                    <Label className="text-[11px] text-slate-600">สีขอบ</Label>
                    <input
                      type="color"
                      value={doc.borderColor.startsWith('#') ? doc.borderColor.slice(0, 7) : '#d1d5db'}
                      onChange={(e) => updateDocField({ borderColor: e.target.value })}
                      className="h-7 w-full rounded border border-slate-300 cursor-pointer bg-white p-0.5"
                    />
                  </div>
                </div>
              </>
            )}
          </div>

          {/* Selected Element Inspector */}
          {selectedElement ? (
            <div className="p-3 space-y-3">
              <div className="flex items-center justify-between gap-2 border-b border-slate-100 pb-2">
                <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-800 truncate">
                  <Move className="h-3.5 w-3.5 text-sky-600 shrink-0" />
                  <span className="truncate">{selectedElement.label}</span>
                </div>
                <div className="flex items-center gap-1 shrink-0">
                  <Button
                    size="icon"
                    variant="outline"
                    className="h-6 w-6"
                    onClick={() => duplicateElement(selectedElement.id)}
                    title="ทำสำเนา (Ctrl+D)"
                  >
                    <Copy className="h-3 w-3" />
                  </Button>
                  <Button
                    size="icon"
                    variant="outline"
                    className="h-6 w-6 text-red-600 border-red-200 hover:bg-red-50"
                    onClick={() => deleteElement(selectedElement.id)}
                    title="ลบองค์ประกอบ (Delete)"
                  >
                    <Trash2 className="h-3 w-3" />
                  </Button>
                </div>
              </div>

              <div className="space-y-1">
                <Label className="text-[11px] text-slate-600">ชื่อเลเยอร์</Label>
                <Input
                  value={selectedElement.label}
                  onChange={(e) => updateElement(selectedElement.id, { label: e.target.value })}
                  className="h-7 text-xs"
                />
              </div>

              {/* Static / Badge Text */}
              {(selectedElement.type === 'static' || selectedElement.type === 'badge') && (
                <div className="space-y-1">
                  <Label className="text-[11px] text-slate-600">ข้อความที่แสดง</Label>
                  <Input
                    value={selectedElement.text ?? ''}
                    onChange={(e) => updateElement(selectedElement.id, { text: e.target.value })}
                    className="h-7 text-xs"
                  />
                </div>
              )}

              {/* Data Field Selector */}
              {(selectedElement.type === 'text' ||
                selectedElement.type === 'price' ||
                selectedElement.type === 'date') && (
                <div className="space-y-1">
                  <Label className="text-[11px] text-slate-600">ผูกข้อมูลจากฟิลด์</Label>
                  <Select
                    value={selectedElement.field || 'product.name'}
                    onValueChange={(v) => updateElement(selectedElement.id, { field: v })}
                  >
                    <SelectTrigger className="h-7 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {dbDataFields.main.concat(dbDataFields.adv).filter((f) => f.type !== 'barcode').map((f) => (
                        <SelectItem key={f.field} value={f.field}>
                          {f.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}

              {/* Image Element Controls */}
              {selectedElement.type === 'image' && (
                <div className="space-y-2 pt-2 border-t border-slate-100">
                  <Label className="text-[11px] text-slate-600">แหล่งที่มา & การจัดวางรูปภาพ</Label>
                  <div className="grid grid-cols-2 gap-1.5">
                    <Button
                      type="button"
                      size="sm"
                      variant={selectedElement.field === 'product.image' && !selectedElement.src ? 'default' : 'outline'}
                      className={`h-7 text-[11px] ${
                        selectedElement.field === 'product.image' && !selectedElement.src
                          ? 'bg-[#007a87] hover:bg-[#00606b]'
                          : ''
                      }`}
                      onClick={() =>
                        updateElement(selectedElement.id, {
                          field: 'product.image',
                          src: undefined,
                          label: 'รูปสินค้า (อัตโนมัติ)',
                        })
                      }
                    >
                      ใช้รูปสินค้า (DB)
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      variant="outline"
                      className="h-7 text-[11px]"
                      onClick={() => replaceImgInputRef.current?.click()}
                    >
                      <Upload className="h-3 w-3 mr-1" />
                      อัปโหลดรูปใหม่
                    </Button>
                  </div>
                  <div className="space-y-1">
                    <Label className="text-[10px] text-slate-500">การปรับสัดส่วนรูป (Object Fit)</Label>
                    <Select
                      value={selectedElement.objectFit || 'contain'}
                      onValueChange={(v) =>
                        updateElement(selectedElement.id, {
                          objectFit: v as DesignElement['objectFit'],
                        })
                      }
                    >
                      <SelectTrigger className="h-7 text-xs">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="contain">พอดีกรอบ (Contain — ไม่ตัดภาพ)</SelectItem>
                        <SelectItem value="cover">เต็มกรอบ (Cover — ครอบตัดพอดี)</SelectItem>
                        <SelectItem value="fill">ยืดเต็มกรอบ (Fill)</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>
              )}

              {/* Icon Selector */}
              {selectedElement.type === 'icon' && (
                <div className="space-y-1 pt-2 border-t border-slate-100">
                  <Label className="text-[11px] text-slate-600">เปลี่ยนไอคอน</Label>
                  <Select
                    value={selectedElement.iconName || 'tag'}
                    onValueChange={(v) =>
                      updateElement(selectedElement.id, { iconName: v as DesignIconName })
                    }
                  >
                    <SelectTrigger className="h-7 text-xs">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {DESIGN_ICON_LIST.map((ic) => (
                        <SelectItem key={ic.name} value={ic.name}>
                          {ic.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}

              {/* Typography Controls (Upfront) */}
              {selectedElement.type !== 'rect' &&
                selectedElement.type !== 'circle' &&
                selectedElement.type !== 'line' &&
                selectedElement.type !== 'icon' &&
                selectedElement.type !== 'image' && (
                  <div className="space-y-2 pt-2 border-t border-slate-100">
                    <div className="grid grid-cols-2 gap-2">
                      <div className="space-y-1">
                        <Label className="text-[11px] text-slate-600">ขนาดตัวอักษร (pt)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={4}
                          max={120}
                          value={selectedElement.fontSize ?? 10}
                          onChange={(e) =>
                            updateElement(selectedElement.id, {
                              fontSize: Math.max(4, parseFloat(e.target.value) || 10),
                            })
                          }
                          className="h-7 text-xs font-mono"
                        />
                      </div>
                      {selectedElement.type !== 'barcode' && (
                        <div className="space-y-1">
                          <Label className="text-[11px] text-slate-600">ความหนา</Label>
                          <Select
                            value={selectedElement.fontWeight || 'bold'}
                            onValueChange={(v) =>
                              updateElement(selectedElement.id, {
                                fontWeight: v as DesignElement['fontWeight'],
                              })
                            }
                          >
                            <SelectTrigger className="h-7 text-xs">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="normal">ปกติ (400)</SelectItem>
                              <SelectItem value="medium">กลาง (500)</SelectItem>
                              <SelectItem value="semibold">กึ่งหนา (600)</SelectItem>
                              <SelectItem value="bold">หนา (700)</SelectItem>
                              <SelectItem value="extrabold">หนามาก (800)</SelectItem>
                              <SelectItem value="black">หนาพิเศษ (900)</SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                      )}
                    </div>

                    {selectedElement.type !== 'barcode' && (
                      <div className="flex items-center justify-between gap-2">
                        <div className="inline-flex border border-slate-200 rounded-md overflow-hidden">
                          {(
                            [
                              ['left', AlignLeft],
                              ['center', AlignCenter],
                              ['right', AlignRight],
                            ] as const
                          ).map(([al, IconCmp]) => (
                            <button
                              key={al}
                              type="button"
                              onClick={() => updateElement(selectedElement.id, { align: al })}
                              className={`p-1.5 ${
                                (selectedElement.align || 'left') === al
                                  ? 'bg-[#007a87] text-white'
                                  : 'bg-white text-slate-600 hover:bg-slate-50'
                              }`}
                            >
                              <IconCmp className="h-3.5 w-3.5" />
                            </button>
                          ))}
                        </div>

                        <div className="flex items-center gap-3">
                          {(selectedElement.type === 'text' || selectedElement.type === 'static') && (
                            <label className="inline-flex items-center gap-1 text-xs text-slate-700 cursor-pointer">
                              <Checkbox
                                checked={Boolean(selectedElement.multiLine)}
                                onCheckedChange={(v) =>
                                  updateElement(selectedElement.id, { multiLine: !!v })
                                }
                              />
                              ตัดบรรทัด
                            </label>
                          )}
                          <label className="inline-flex items-center gap-1 text-xs text-slate-700 cursor-pointer">
                            <Checkbox
                              checked={Boolean(selectedElement.strike)}
                              onCheckedChange={(v) =>
                                updateElement(selectedElement.id, { strike: !!v })
                              }
                            />
                            ขีดฆ่า
                          </label>
                        </div>
                      </div>
                    )}

                    {selectedElement.type === 'barcode' && (
                      <label className="inline-flex items-center gap-2 text-xs text-slate-700 cursor-pointer pt-1">
                        <Checkbox
                          checked={selectedElement.showText !== false}
                          onCheckedChange={(v) =>
                            updateElement(selectedElement.id, { showText: !!v })
                          }
                        />
                        แสดงตัวเลขใต้บาร์โค้ด
                      </label>
                    )}
                  </div>
                )}

              {/* Color Controls (Upfront) */}
              <div className="space-y-3 pt-2 border-t border-slate-100">
                {selectedElement.type !== 'rect' && selectedElement.type !== 'circle' && (
                  <ColorSwatchPicker
                    label={selectedElement.type === 'icon' ? 'สีไอคอน' : 'สีตัวอักษร / เส้น'}
                    value={selectedElement.color || '#111111'}
                    onChange={(c) => updateElement(selectedElement.id, { color: c })}
                  />
                )}

                {selectedElement.type !== 'barcode' && selectedElement.type !== 'line' && (
                  <ColorSwatchPicker
                    label="สีพื้นหลังองค์ประกอบ"
                    value={selectedElement.fill || 'transparent'}
                    onChange={(c) => updateElement(selectedElement.id, { fill: c })}
                  />
                )}
              </div>

              {/* Collapsible Precision Geometry, Prefix/Suffix, Border/Radius/Opacity */}
              <div className="border border-slate-200 rounded-md overflow-hidden">
                <button
                  type="button"
                  onClick={() => setAdvancedPropsOpen((o) => !o)}
                  className="w-full flex items-center justify-between px-2.5 py-2 bg-slate-50 hover:bg-slate-100 text-[11px] font-medium text-slate-700 transition-colors"
                >
                  <span>ปรับละเอียด (พิกัด X/Y, หมุน, คำต่อท้าย, ขอบมุม)</span>
                  {advancedPropsOpen ? (
                    <ChevronUp className="h-3.5 w-3.5 text-slate-500" />
                  ) : (
                    <ChevronDown className="h-3.5 w-3.5 text-slate-500" />
                  )}
                </button>

                {advancedPropsOpen && (
                  <div className="p-2.5 space-y-3 bg-white border-t border-slate-200">
                    {/* Position & Size (mm) */}
                    <div className="grid grid-cols-4 gap-1.5">
                      <div className="space-y-1">
                        <Label className="text-[10px] text-slate-500">X (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          value={selectedElement.x}
                          onChange={(e) =>
                            updateElement(selectedElement.id, { x: parseFloat(e.target.value) || 0 })
                          }
                          className="h-7 text-xs px-1.5 font-mono"
                        />
                      </div>
                      <div className="space-y-1">
                        <Label className="text-[10px] text-slate-500">Y (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          value={selectedElement.y}
                          onChange={(e) =>
                            updateElement(selectedElement.id, { y: parseFloat(e.target.value) || 0 })
                          }
                          className="h-7 text-xs px-1.5 font-mono"
                        />
                      </div>
                      <div className="space-y-1">
                        <Label className="text-[10px] text-slate-500">W (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={0.5}
                          value={selectedElement.width}
                          onChange={(e) =>
                            updateElement(selectedElement.id, {
                              width: Math.max(0.5, parseFloat(e.target.value) || 1),
                            })
                          }
                          className="h-7 text-xs px-1.5 font-mono"
                        />
                      </div>
                      <div className="space-y-1">
                        <Label className="text-[10px] text-slate-500">H (mm)</Label>
                        <Input
                          type="number"
                          step={0.5}
                          min={0.2}
                          value={selectedElement.height}
                          onChange={(e) =>
                            updateElement(selectedElement.id, {
                              height: Math.max(0.2, parseFloat(e.target.value) || 1),
                            })
                          }
                          className="h-7 text-xs px-1.5 font-mono"
                        />
                      </div>
                    </div>

                    <div className="grid grid-cols-3 gap-1.5">
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        className="h-7 text-[11px] px-1"
                        onClick={() =>
                          updateElement(selectedElement.id, {
                            x: roundMm((doc.width - selectedElement.width) / 2, 0.5),
                          })
                        }
                      >
                        กึ่งกลางแนวนอน
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        className="h-7 text-[11px] px-1"
                        onClick={() =>
                          updateElement(selectedElement.id, {
                            y: roundMm((doc.height - selectedElement.height) / 2, 0.5),
                          })
                        }
                      >
                        กึ่งกลางแนวตั้ง
                      </Button>
                      <div className="flex items-center gap-1">
                        <Input
                          type="number"
                          step={5}
                          value={selectedElement.rotation ?? 0}
                          onChange={(e) =>
                            updateElement(selectedElement.id, {
                              rotation: parseFloat(e.target.value) || 0,
                            })
                          }
                          className="h-7 text-xs px-1.5 font-mono"
                          title="มุมหมุน (องศา)"
                        />
                        <span className="text-[10px] text-slate-500">°</span>
                      </div>
                    </div>

                    {(selectedElement.type === 'price' || selectedElement.type === 'text') && (
                      <div className="grid grid-cols-2 gap-2">
                        <div className="space-y-1">
                          <Label className="text-[10px] text-slate-500">คำนำหน้า (Prefix)</Label>
                          <Input
                            value={selectedElement.prefix ?? ''}
                            placeholder="เช่น ปกติ "
                            onChange={(e) =>
                              updateElement(selectedElement.id, { prefix: e.target.value })
                            }
                            className="h-7 text-xs"
                          />
                        </div>
                        <div className="space-y-1">
                          <Label className="text-[10px] text-slate-500">คำต่อท้าย (Suffix)</Label>
                          <Input
                            value={selectedElement.suffix ?? ''}
                            placeholder="เช่น  บาท"
                            onChange={(e) =>
                              updateElement(selectedElement.id, { suffix: e.target.value })
                            }
                            className="h-7 text-xs"
                          />
                        </div>
                      </div>
                    )}

                    <div className="grid grid-cols-3 gap-2">
                      {selectedElement.type !== 'line' && (
                        <>
                          <div className="space-y-1">
                            <Label className="text-[10px] text-slate-500">ขอบ (mm)</Label>
                            <Input
                              type="number"
                              step={0.1}
                              min={0}
                              max={5}
                              value={selectedElement.borderWidth ?? 0}
                              onChange={(e) =>
                                updateElement(selectedElement.id, {
                                  borderWidth: Math.max(0, parseFloat(e.target.value) || 0),
                                })
                              }
                              className="h-7 text-xs"
                            />
                          </div>
                          <div className="space-y-1">
                            <Label className="text-[10px] text-slate-500">มุมโค้ง (mm)</Label>
                            <Input
                              type="number"
                              step={0.5}
                              min={0}
                              max={20}
                              value={selectedElement.borderRadius ?? 0}
                              onChange={(e) =>
                                updateElement(selectedElement.id, {
                                  borderRadius: Math.max(0, parseFloat(e.target.value) || 0),
                                })
                              }
                              className="h-7 text-xs"
                            />
                          </div>
                        </>
                      )}
                      <div className="space-y-1">
                        <Label className="text-[10px] text-slate-500">โปร่งใส (%)</Label>
                        <Input
                          type="number"
                          step={10}
                          min={10}
                          max={100}
                          value={Math.round((selectedElement.opacity ?? 1) * 100)}
                          onChange={(e) =>
                            updateElement(selectedElement.id, {
                              opacity: Math.max(0.1, Math.min(1, (parseFloat(e.target.value) || 100) / 100)),
                            })
                          }
                          className="h-7 text-xs"
                        />
                      </div>
                    </div>
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div className="p-6 text-center text-xs text-slate-400 space-y-2">
              <Move className="h-6 w-6 mx-auto text-slate-300" />
              <p>คลิกเลือกองค์ประกอบบนป้ายเพื่อปรับตำแหน่ง สี และขนาด หรือเพิ่มจากเมนูด้านซ้าย</p>
            </div>
          )}
        </div>
      </div>

      {/* Modal: Preview รูปแบบจัดวาง (เลือกขนาดกระดาษ / ม้วน Thermal ได้อิสระ) */}
      <Dialog open={a4PreviewOpen} onOpenChange={setA4PreviewOpen}>
        <DialogContent className="w-[96vw] max-w-[96vw] sm:max-w-[96vw] h-[94vh] flex flex-col">
          <DialogHeader>
            <DialogTitle className="text-base flex items-center gap-2">
              <FileSpreadsheet className="h-4 w-4 text-[#007a87]" />
              Preview รูปแบบจัดวาง — {a4Metrics.paperLabel}
            </DialogTitle>
            <DialogDescription className="text-xs">
              รูปแบบ <strong>{doc.lbsCode}</strong> ({priType === '2' ? 'ราคาโปรโมชัน' : 'ราคาปกติ'}) — ขนาดป้าย{' '}
              <strong>
                {doc.width} × {doc.height} mm
              </strong>{' '}
              {a4Metrics.isRoll ? (
                <>
                  จัดวางบนม้วนกว้าง <strong>{a4Metrics.paperWidthMm} mm</strong> (แถวละ{' '}
                  <strong>{a4Metrics.cols} ดวง</strong> · Gap {a4Metrics.gapVMm} mm)
                </>
              ) : a4Metrics.isSingle ? (
                <>แสดงป้ายเดี่ยว 1:1 ขนาดจริง</>
              ) : (
                <>
                  วางได้ <strong>{a4Metrics.cols} คอลัมน์ × {a4Metrics.rows} แถว = {a4Metrics.perPage} ดวง/แผ่น</strong> (ใช้พื้นที่{' '}
                  {a4Metrics.areaPct.toFixed(1)}% ของแผ่น)
                </>
              )}
            </DialogDescription>
          </DialogHeader>

          <PreviewMediaToolbar
            config={previewMedia}
            onChange={setPreviewMedia}
            savedPaperSize={savedMetrics.paperSize}
            lbsGroup={currentSize.FTLbsGroup}
          />

          {!a4Metrics.isRoll && !a4Metrics.isSingle && (
            <div className="flex flex-wrap items-center justify-between gap-2 bg-slate-50/90 border border-slate-200 rounded-md px-2.5 py-1.5 text-[11px]">
              <div className="flex flex-wrap items-center gap-2">
                <span className="font-medium text-slate-700">ขอบและช่องว่าง (mm):</span>
                <label className="inline-flex items-center gap-1 text-slate-600">
                  <span>ขอบบน:</span>
                  <Input
                    type="number"
                    step={0.5}
                    min={0}
                    max={80}
                    value={a4Metrics.magTopMm}
                    onChange={(e) => {
                      const v = Math.max(0, parseFloat(e.target.value) || 0);
                      updateSheetSpacingDraft({ magTop: v, magBottom: v });
                    }}
                    className="h-6 w-14 text-[11px] font-mono bg-white px-1.5"
                  />
                </label>
                <label className="inline-flex items-center gap-1 text-slate-600">
                  <span>ขอบซ้าย:</span>
                  <Input
                    type="number"
                    step={0.5}
                    min={0}
                    max={80}
                    value={a4Metrics.magLeftMm}
                    onChange={(e) => {
                      const v = Math.max(0, parseFloat(e.target.value) || 0);
                      updateSheetSpacingDraft({ magLeft: v, magRight: v });
                    }}
                    className="h-6 w-14 text-[11px] font-mono bg-white px-1.5"
                  />
                </label>
                <label className="inline-flex items-center gap-1 text-slate-600">
                  <span>ช่องว่าง H:</span>
                  <Input
                    type="number"
                    step={0.5}
                    min={0}
                    max={30}
                    value={a4Metrics.gapHMm}
                    onChange={(e) =>
                      updateSheetSpacingDraft({ gapH: Math.max(0, parseFloat(e.target.value) || 0) })
                    }
                    className="h-6 w-14 text-[11px] font-mono bg-white px-1.5"
                  />
                </label>
                <label className="inline-flex items-center gap-1 text-slate-600">
                  <span>ช่องว่าง V:</span>
                  <Input
                    type="number"
                    step={0.5}
                    min={0}
                    max={30}
                    value={a4Metrics.gapVMm}
                    onChange={(e) =>
                      updateSheetSpacingDraft({ gapV: Math.max(0, parseFloat(e.target.value) || 0) })
                    }
                    className="h-6 w-14 text-[11px] font-mono bg-white px-1.5"
                  />
                </label>
              </div>
              <Button
                type="button"
                size="sm"
                variant="outline"
                className="h-6 text-[11px] px-2.5 bg-white border-[#007a87] text-[#007a87] hover:bg-teal-50"
                onClick={centerGridOnSheet}
                title="คำนวณขอบซ้าย/ขวา และ บน/ล่าง ให้อยู่กึ่งกลางกระดาษพอดี"
              >
                จัดกึ่งกลางแผ่น
              </Button>
            </div>
          )}

          <div className="flex flex-wrap items-center justify-between gap-2 pb-2 border-b border-slate-100">
            <div className="flex flex-wrap items-center gap-2 text-xs">
              <Badge variant="outline">
                {a4Metrics.isRoll
                  ? `หน้ากว้างม้วน: ${a4Metrics.paperWidthMm} mm`
                  : `กระดาษ: ${a4Metrics.paperWidthMm} × ${a4Metrics.paperHeightMm} mm`}
              </Badge>
              <Badge variant="outline">1 ดวง: {doc.width} × {doc.height} mm</Badge>
              <Badge className="bg-[#007a87] text-white">
                {a4Metrics.isRoll ? `${a4Metrics.cols} ดวง / แถวม้วน` : `${a4Metrics.perPage} ดวง / แผ่น`}
              </Badge>
              {doc.designMode === 'preprint' && !a4Metrics.isRoll && !a4Metrics.isSingle && (
                <div className="inline-flex rounded-md border border-slate-200 bg-slate-100 p-0.5 ml-1">
                  <button
                    type="button"
                    onClick={() => setSheetPreviewMode('guide')}
                    className={`px-2.5 py-0.5 text-[11px] font-medium rounded-sm transition-colors ${
                      sheetPreviewMode === 'guide'
                        ? 'bg-amber-600 text-white shadow-2xs'
                        : 'text-slate-600 hover:text-slate-900'
                    }`}
                  >
                    จำลองทาบลง Pre-Print
                  </button>
                  <button
                    type="button"
                    onClick={() => setSheetPreviewMode('ink')}
                    className={`px-2.5 py-0.5 text-[11px] font-medium rounded-sm transition-colors ${
                      sheetPreviewMode === 'ink'
                        ? 'bg-white text-slate-900 shadow-2xs'
                        : 'text-slate-600 hover:text-slate-900'
                    }`}
                  >
                    เฉพาะหมึกที่จะพิมพ์จริง
                  </button>
                </div>
              )}
            </div>
            <div className="flex items-center gap-1.5">
              <span className="text-xs text-slate-500">มาตราส่วน:</span>
              {[0.5, 0.68, 0.85, 1, 1.5].map((s) => (
                <Button
                  key={s}
                  size="sm"
                  variant={a4Scale === s ? 'default' : 'outline'}
                  className={`h-7 text-xs px-2 ${a4Scale === s ? 'bg-[#007a87] hover:bg-[#00606b]' : ''}`}
                  onClick={() => setA4Scale(s)}
                >
                  {Math.round(s * 100)}%
                </Button>
              ))}
            </div>
          </div>

          <div className="flex-1 min-h-0 overflow-auto bg-slate-200 rounded-lg p-6 flex justify-center items-start">
            {(() => {
              const sampleRollRows = 4;
              const rollTotalH = Number(
                (4 + sampleRollRows * doc.height + (sampleRollRows - 1) * a4Metrics.gapVMm).toFixed(1)
              );
              const renderBoxH = a4Metrics.isRoll ? rollTotalH : a4Metrics.paperHeightMm;

              const actualSheetBox = a4Metrics.isRoll ? (
                <div
                  className="bg-slate-100 shadow-xl border-x-2 border-slate-500 border-y border-dashed border-slate-400 relative"
                  style={{
                    width: `${a4Metrics.paperWidthMm * a4Scale}mm`,
                    paddingTop: `${2 * a4Scale}mm`,
                    paddingBottom: `${2 * a4Scale}mm`,
                    paddingLeft: `${a4Metrics.magLeftMm * a4Scale}mm`,
                    paddingRight: `${a4Metrics.magRightMm * a4Scale}mm`,
                    display: 'grid',
                    gridTemplateColumns: `repeat(${a4Metrics.cols}, ${doc.width * a4Scale}mm)`,
                    columnGap: `${a4Metrics.gapHMm * a4Scale}mm`,
                    rowGap: `${a4Metrics.gapVMm * a4Scale}mm`,
                    justifyContent: 'start',
                    boxSizing: 'border-box',
                  }}
                >
                  {Array.from({ length: a4Metrics.cols * sampleRollRows }).map((_, idx) => (
                    <div
                      key={`roll-cmp-${idx}`}
                      className="bg-white ring-1 ring-slate-300/90 rounded-[2px] overflow-hidden"
                      style={{
                        width: `${doc.width * a4Scale}mm`,
                        height: `${doc.height * a4Scale}mm`,
                      }}
                    >
                      <CustomDesignRenderer
                        layout={doc}
                        line={sampleLine}
                        scale={a4Scale}
                        preprintView="ink"
                      />
                    </div>
                  ))}
                </div>
              ) : (
                <div
                  className="bg-white shadow-xl border-2 border-[#007a87] relative overflow-hidden"
                  style={{
                    width: `${a4Metrics.paperWidthMm * a4Scale}mm`,
                    height: `${a4Metrics.paperHeightMm * a4Scale}mm`,
                    paddingTop: `${a4Metrics.magTopMm * a4Scale}mm`,
                    paddingBottom: `${a4Metrics.magBottomMm * a4Scale}mm`,
                    paddingLeft: `${a4Metrics.magLeftMm * a4Scale}mm`,
                    paddingRight: `${a4Metrics.magRightMm * a4Scale}mm`,
                    display: 'grid',
                    gridTemplateColumns: `repeat(${a4Metrics.cols}, ${doc.width * a4Scale}mm)`,
                    gridAutoRows: `${doc.height * a4Scale}mm`,
                    columnGap: `${a4Metrics.gapHMm * a4Scale}mm`,
                    rowGap: `${a4Metrics.gapVMm * a4Scale}mm`,
                    alignContent: 'start',
                    justifyContent: 'start',
                    boxSizing: 'border-box',
                  }}
                >
                  {doc.designMode === 'preprint' &&
                    sheetPreviewMode === 'guide' &&
                    !a4Metrics.isSingle &&
                    doc.preprint?.sheetBgDataUrl && (
                      <img
                        src={doc.preprint.sheetBgDataUrl}
                        alt="Full Pre-print Sheet"
                        style={{
                          position: 'absolute',
                          inset: 0,
                          width: '100%',
                          height: '100%',
                          objectFit: 'fill',
                          opacity: Math.max(0.2, Math.min(1, doc.preprint.opacity ?? 0.8)),
                          filter:
                            doc.preprint.contrast && doc.preprint.contrast !== 100
                              ? `contrast(${doc.preprint.contrast}%)`
                              : undefined,
                          pointerEvents: 'none',
                        }}
                      />
                    )}
                  {Array.from({ length: a4Metrics.perPage }).map((_, idx) => (
                    <CustomDesignRenderer
                      key={`a4-cmp-${idx}`}
                      layout={doc}
                      line={sampleLine}
                      scale={a4Scale}
                      preprintView={
                        doc.designMode === 'preprint'
                          ? sheetPreviewMode === 'ink'
                            ? 'ink'
                            : !a4Metrics.isSingle && doc.preprint?.sheetBgDataUrl
                              ? 'sheet-overlay'
                              : 'cell-guide'
                          : 'ink'
                      }
                    />
                  ))}
                  {previewMedia.refFrame !== 'NONE' && (
                    <div className="absolute bottom-1 right-1 bg-[#007a87]/90 text-white text-[10px] font-mono px-1.5 py-0.5 rounded pointer-events-none">
                      {a4Metrics.paperSize}: {a4Metrics.paperWidthMm}×{a4Metrics.paperHeightMm} mm
                    </div>
                  )}
                </div>
              );

              return (
                <div className="space-y-1.5">
                  <div className="text-[11px] font-mono text-slate-600 flex flex-wrap justify-between gap-2 px-1">
                    <span>
                      {a4Metrics.isRoll
                        ? `← ม้วนสติ๊กเกอร์กว้าง = ${a4Metrics.paperWidthMm} mm (แสดงตัวอย่าง ${sampleRollRows} แถว · Gap = ${a4Metrics.gapVMm} mm) →`
                        : `← กว้าง = ${a4Metrics.paperWidthMm} mm · สูง = ${a4Metrics.paperHeightMm} mm →`}
                    </span>
                  </div>

                  <PreviewReferenceFrameWrapper
                    refFrame={previewMedia.refFrame}
                    sheetWidthMm={a4Metrics.paperWidthMm}
                    sheetHeightMm={renderBoxH}
                    scale={a4Scale}
                  >
                    {actualSheetBox}
                  </PreviewReferenceFrameWrapper>
                </div>
              );
            })()}
          </div>

          <DialogFooter className="gap-2 pt-2 border-t border-slate-100">
            <Button variant="outline" onClick={() => setA4PreviewOpen(false)}>
              ปิดหน้าต่าง Preview
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Modal: บันทึกเป็นแบบใหม่ (ตั้งชื่อใหม่ และปรับขนาด W×H mm ได้) */}
      <Dialog open={saveAsOpen} onOpenChange={setSaveAsOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="text-base flex items-center gap-2">
              <FilePlus2 className="h-4 w-4 text-[#007a87]" />
              {saveAsBlank
                ? 'สร้างรูปแบบป้ายใหม่ (หน้ากระดาษเปล่า)'
                : 'บันทึกเป็นรูปแบบป้ายใหม่ (ไม่ทับของเก่า)'}
            </DialogTitle>
            <DialogDescription className="text-xs">
              {saveAsBlank
                ? 'สร้างรูปแบบป้ายใหม่โดยเริ่มจากหน้ากระดาษเปล่า เพื่อวางองค์ประกอบเองตั้งแต่ต้น'
                : 'สร้างรูปแบบป้ายใหม่แยกต่างหากเพื่อนำไปเลือกพิมพ์ในหน้าพิมพ์ป้ายราคา โดยไม่ทับแบบเดิม'}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-3 py-2">
            <div className="space-y-1">
              <Label className="text-xs font-medium">
                <span className="text-red-500">*</span> ชื่อแบบป้ายใหม่
              </Label>
              <Input
                value={saveAsName}
                onChange={(e) => setSaveAsName(e.target.value)}
                placeholder="เช่น ป้ายราคา 10x3.8 แบบมีรูปสินค้า"
                className="h-8 text-xs"
              />
            </div>

            <label className="block text-xs">ขนาดและกระดาษ
              <select className="mt-1 block w-full rounded border p-2" value={saveSizeCode} onChange={e => setSaveSizeCode(e.target.value)}>
                {activeSizes.map(size => <option key={size.FTLbsCode} value={size.FTLbsCode}>{size.FTLbsCode} · {size.FCLbsWidth} × {size.FCLbsHeight} มม.</option>)}
                <option value="">เพิ่มขนาดและกระดาษใหม่</option>
              </select>
            </label>
            {!existingSaveSize && <>
            <div className="grid grid-cols-2 gap-2">
              <div className="space-y-1">
                <Label className="text-xs font-medium">กว้าง (mm)</Label>
                <Input
                  type="number"
                  min={15}
                  max={210}
                  step={0.5}
                  value={saveAsWidth}
                  onChange={(e) => setSaveAsWidth(parseFloat(e.target.value) || doc.width)}
                  className="h-8 text-xs font-mono"
                />
              </div>
              <div className="space-y-1">
                <Label className="text-xs font-medium">สูง (mm)</Label>
                <Input
                  type="number"
                  min={10}
                  max={297}
                  step={0.5}
                  value={saveAsHeight}
                  onChange={(e) => setSaveAsHeight(parseFloat(e.target.value) || doc.height)}
                  className="h-8 text-xs font-mono"
                />
              </div>
            </div>

            <div className="space-y-1">
              <Label className="text-xs font-medium">รหัสขนาดใหม่ (ไม่บังคับ — เว้นว่างให้ระบบตั้งอัตโนมัติ)</Label>
              <Input
                value={saveAsCode}
                onChange={(e) => setSaveAsCode(e.target.value)}
                placeholder={`เช่น CUS-${Math.round(saveAsWidth)}x${Math.round(saveAsHeight)}-01`}
                className="h-8 text-xs font-mono"
              />
            </div>

            </>}
            <div className="rounded-md bg-slate-50 border border-slate-200 p-2.5 text-[11px] text-slate-600 space-y-0.5">
              <div>
                {existingSaveSize ? <>{existingSaveSize.FCLbsWidth} × {existingSaveSize.FCLbsHeight} มม. · {existingSaveSize.FTLbsPaperSize || 'A4'} · {existingSaveSize.FNLbsCol} × {existingSaveSize.FNLbsRow} ดวง/แผ่น</> : <>{saveAsWidth} × {saveAsHeight} มม. · {a4Metrics.paperSize} · {a4Metrics.cols} × {a4Metrics.rows} ดวง/แผ่น</>}

              </div>
              <div className="grid grid-cols-2 gap-2 py-1">
                <label>ประเภทป้าย<select className="block w-full rounded border p-1" value={newPrice} onChange={e=>setNewPrice(e.target.value==='2'?'2':'1')}><option value="1">ราคาปกติ</option><option value="2">ราคาโปรโมชัน</option></select></label>
                <label>วิธีพิมพ์<select className="block w-full rounded border p-1" value={newMode} onChange={e=>setNewMode(e.target.value==='preprint'?'preprint':'full')}><option value="full">พิมพ์ครบจบ</option><option value="preprint">Pre-Print</option></select></label>
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => setSaveAsOpen(false)}>
              ยกเลิก
            </Button>
            <Button
              type="button"
              className="bg-[#007a87] hover:bg-[#00606b] text-white"
              disabled={saving}
              onClick={handleConfirmSaveAs}
            >
              {saving && <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />}
              สร้างและบันทึกแบบใหม่
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Modal: ยืนยันบันทึกทับรูปแบบมาตรฐานเดิม */}
      <Dialog open={overwriteConfirmOpen} onOpenChange={setOverwriteConfirmOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="text-base flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 text-amber-600" />
              ยืนยันการบันทึกทับแบบเดิม ({selectedTemplate?.FTLtmCode || designCode})
            </DialogTitle>
            <DialogDescription className="text-xs">
              ระบบจะบันทึก Layout ที่ออกแบบนี้ทับการแสดงผลของรูปแบบ <strong>{selectedTemplate?.FTLtmCode || designCode}</strong> ในโหมด{' '}
              <strong>{priType === '2' ? 'ราคาโปรโมชัน' : 'ราคาปกติ'}</strong>
            </DialogDescription>
          </DialogHeader>

          <div className="rounded-md bg-amber-50 border border-amber-200 p-3 text-xs text-amber-900 space-y-1.5">
            <div>
              • เมื่อพิมพ์ป้ายด้วยรหัส <strong>{selectedTemplate?.FTLtmCode || designCode}</strong> จะแสดงผลตามแบบที่ออกแบบไว้นี้แทนแบบ STD
            </div>
            <div>
              • สามารถกดปุ่ม <strong>คืนค่า STD เดิม</strong> ในหน้า Design ได้ทุกเมื่อเพื่อย้อนกลับเป็นแบบมาตรฐานเดิม (หรือเลือก{' '}
              <strong>บันทึกเป็นแบบใหม่</strong> เพื่อไม่ทับของเดิม)
            </div>
          </div>

          <DialogFooter className="gap-2 sm:gap-0">
            <Button
              type="button"
              variant="outline"
              disabled={saving}
              onClick={() => setOverwriteConfirmOpen(false)}
            >
              ยกเลิก
            </Button>
            <Button
              type="button"
              className="bg-amber-600 hover:bg-amber-700 text-white"
              disabled={saving}
              onClick={handleConfirmOverwriteStd}
            >
              {saving && <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />}
              ยืนยันบันทึกทับเดิม
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Modal: ยืนยันคืนค่า STD เดิม / รีเซ็ตแบบ */}
      <Dialog open={resetConfirmOpen} onOpenChange={setResetConfirmOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="text-base flex items-center gap-2">
              <RotateCcw className="h-4 w-4 text-[#007a87]" />
              {!isCustomCreatedFormat
                ? `คืนค่าแบบมาตรฐาน (STD) เดิม — ${selectedTemplate?.FTLtmCode || designCode}`
                : `รีเซ็ตแบบป้ายเป็นค่าเริ่มต้น — ${selectedTemplate?.FTLtmCode || designCode}`}
            </DialogTitle>
            <DialogDescription className="text-xs">
              {!isCustomCreatedFormat
                ? `ลบ Layout ที่บันทึกทับไว้สำหรับ ${selectedTemplate?.FTLtmCode || designCode} (${priType === '2' ? 'ราคาโปรโมชัน' : 'ราคาปกติ'}) เพื่อกลับไปใช้แบบมาตรฐาน (STD) เดิมของระบบ`
                : `ล้าง Layout ที่บันทึกไว้สำหรับ ${selectedTemplate?.FTLtmCode || designCode} (${priType === '2' ? 'ราคาโปรโมชัน' : 'ราคาปกติ'}) และสร้างโครงตั้งต้นใหม่`}
            </DialogDescription>
          </DialogHeader>

          <div className="rounded-md bg-slate-50 border border-slate-200 p-3 text-xs text-slate-700">
            {!isCustomCreatedFormat
              ? 'หลังยืนยัน ทั้งหน้าพิมพ์ป้ายและหน้า Preview จะกลับมาแสดงผลตามแบบมาตรฐาน (STD) เดิมทันที'
              : 'หลังยืนยัน ข้อมูลตำแหน่ง/รูปแบบที่เคยบันทึกไว้ในโหมดราคานี้จะถูกล้างออก'}
          </div>

          <DialogFooter className="gap-2 sm:gap-0">
            <Button
              type="button"
              variant="outline"
              disabled={saving}
              onClick={() => setResetConfirmOpen(false)}
            >
              ยกเลิก
            </Button>
            <Button
              type="button"
              className="bg-[#007a87] hover:bg-[#00606b] text-white"
              disabled={saving}
              onClick={() => void executeReset(true)}
            >
              {saving && <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />}
              {!isCustomCreatedFormat ? 'ยืนยันคืนค่า STD เดิม' : 'ยืนยันรีเซ็ต'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Modal: นำเข้าฟอร์ม Pre-Print (PDF / รูปภาพ) */}
      <Dialog open={importPreprintOpen} onOpenChange={setImportPreprintOpen}>
        <DialogContent className="w-[96vw] max-w-[96vw] sm:max-w-[95vw] lg:max-w-6xl h-[92vh] flex flex-col">
          <DialogHeader>
            <DialogTitle className="text-base flex items-center gap-2">
              <FileUp className="h-4 w-4 text-amber-600" />
              นำเข้าฟอร์ม Pre-Print (PDF / รูปภาพ) มาวางเป็นฉากหลังอ้างอิง
            </DialogTitle>
            <DialogDescription className="text-xs">
              ภาพ Pre-Print ที่นำเข้าจะแสดงเฉพาะบนหน้า Design และหน้า Preview เพื่อช่วยเล็งตำแหน่ง — ระบบจะไม่พิมพ์ภาพนี้ลงกระดาษจริง
            </DialogDescription>
          </DialogHeader>

          <div className="grid grid-cols-1 md:grid-cols-12 gap-4 flex-1 min-h-0 overflow-y-auto py-1">
            {/* Left: File picker + Live Overlay Preview */}
            <div className="md:col-span-6 flex flex-col gap-2.5">
              <div className="flex items-center gap-2">
                <Button
                  type="button"
                  size="sm"
                  className="bg-amber-600 hover:bg-amber-700 text-white text-xs"
                  disabled={impLoading}
                  onClick={() => preprintFileInputRef.current?.click()}
                >
                  {impLoading ? (
                    <Loader2 className="h-3.5 w-3.5 mr-1.5 animate-spin" />
                  ) : (
                    <Upload className="h-3.5 w-3.5 mr-1.5" />
                  )}
                  เลือกไฟล์ PDF หรือ รูปภาพ...
                </Button>
                <span className="text-[11px] text-slate-600 truncate">
                  {impParsed?.fileName || doc.preprint?.sourceName || 'ยังไม่ได้เลือกไฟล์ใหม่'}
                </span>
              </div>

              {impParsed?.isPdf && impParsed.pages.length > 1 && (
                <div className="flex items-center gap-1.5 flex-wrap">
                  <span className="text-[11px] text-slate-500">เลือกหน้า PDF:</span>
                  {impParsed.pages.map((p) => (
                    <Button
                      key={p.pageNumber}
                      type="button"
                      size="sm"
                      variant={impPageNum === p.pageNumber ? 'default' : 'outline'}
                      className={`h-6 px-2 text-[11px] ${impPageNum === p.pageNumber ? 'bg-[#007a87] hover:bg-[#00606b]' : ''}`}
                      onClick={() => setImpPageNum(p.pageNumber)}
                    >
                      หน้า {p.pageNumber}
                    </Button>
                  ))}
                </div>
              )}

              {(() => {
                const previewImgUrl =
                  impParsed?.pages.find((p) => p.pageNumber === impPageNum)?.dataUrl ||
                  doc.preprint?.sheetBgDataUrl ||
                  doc.preprint?.cellBgDataUrl;
                const aspect = impPaperW / Math.max(1, impPaperH);
                return (
                  <div className="flex-1 min-h-[380px] bg-slate-200 rounded-md border border-slate-300 p-4 flex items-center justify-center overflow-auto">
                    {previewImgUrl ? (
                      <div
                        className="relative bg-white shadow-lg border border-slate-400 overflow-hidden"
                        style={
                          impScope === 'sheet'
                            ? { width: '100%', maxWidth: '460px', aspectRatio: `${aspect}` }
                            : { width: '100%', maxWidth: '460px', aspectRatio: `${impLabelW / Math.max(1, impLabelH)}` }
                        }
                      >
                        <img
                          src={previewImgUrl}
                          alt="Pre-print Preview"
                          className="w-full h-full object-fill"
                          style={{
                            opacity: impOpacity,
                            filter: impContrast !== 100 ? `contrast(${impContrast}%)` : undefined,
                          }}
                        />
                        {impScope === 'sheet' && (() => {
                          const totalGridW = impMagLeft + Math.max(1, impCols) * impLabelW + Math.max(0, impCols - 1) * impGapH;
                          const totalGridH = impMagTop + Math.max(1, impRows) * impLabelH + Math.max(0, impRows - 1) * impGapV;
                          const maxAllowX = impPaperW - impMagRight;
                          const maxAllowY = impPaperH - impMagBottom;
                          const overRight = totalGridW - maxAllowX > 0.15;
                          const overBottom = totalGridH - maxAllowY > 0.15;
                          return (
                            <div className="absolute inset-0 pointer-events-none">
                              {/* กรอบพื้นที่ 4 ขอบกระดาษ (Safe Area: บน/ล่าง/ซ้าย/ขวา) */}
                              <div
                                style={{
                                  position: 'absolute',
                                  left: `${(impMagLeft / Math.max(1, impPaperW)) * 100}%`,
                                  top: `${(impMagTop / Math.max(1, impPaperH)) * 100}%`,
                                  right: `${(impMagRight / Math.max(1, impPaperW)) * 100}%`,
                                  bottom: `${(impMagBottom / Math.max(1, impPaperH)) * 100}%`,
                                  border: overRight || overBottom ? '1.5px dashed #dc2626' : '1.5px dashed #ea580c',
                                  boxSizing: 'border-box',
                                }}
                              />
                              {Array.from({ length: Math.min(48, Math.max(1, impCols) * Math.max(1, impRows)) }).map((_, idx) => {
                                const r = Math.floor(idx / Math.max(1, impCols));
                                const c = idx % Math.max(1, impCols);
                                const xMm = impMagLeft + c * (impLabelW + impGapH);
                                const yMm = impMagTop + r * (impLabelH + impGapV);
                                const cellOverflow = xMm + impLabelW - maxAllowX > 0.15 || yMm + impLabelH - maxAllowY > 0.15;
                                return (
                                  <div
                                    key={`grid-cell-${idx}`}
                                    style={{
                                      position: 'absolute',
                                      left: `${(xMm / Math.max(1, impPaperW)) * 100}%`,
                                      top: `${(yMm / Math.max(1, impPaperH)) * 100}%`,
                                      width: `${(impLabelW / Math.max(1, impPaperW)) * 100}%`,
                                      height: `${(impLabelH / Math.max(1, impPaperH)) * 100}%`,
                                      border: idx === 0
                                        ? '2px solid #0284c7'
                                        : cellOverflow
                                          ? '1px dashed #dc2626'
                                          : '1px dashed rgba(0, 122, 135, 0.85)',
                                      backgroundColor: idx === 0
                                        ? 'rgba(2, 132, 199, 0.08)'
                                        : cellOverflow
                                          ? 'rgba(220, 38, 38, 0.08)'
                                          : 'transparent',
                                      boxSizing: 'border-box',
                                    }}
                                  />
                                );
                              })}
                            </div>
                          );
                        })()}
                      </div>
                    ) : (
                      <div className="text-center text-xs text-slate-500 space-y-1 px-4">
                        <FileUp className="h-8 w-8 mx-auto text-slate-400" />
                        <p>เลือกไฟล์ PDF ฟอร์ม Pre-Print (เช่น Pre-print form SizeA4 / SizeB5) หรือรูปสแกน</p>
                      </div>
                    )}
                  </div>
                );
              })()}
            </div>

            {/* Right: Sheet & Grid Settings */}
            <div className="md:col-span-6 space-y-3">
              {preprints.length > 0 && (
                <div className="space-y-1.5 pb-2 border-b border-slate-200">
                  <Label className="text-xs font-medium text-[#007a87]">
                    เลือกจากคลัง Pre-Print ในฐานข้อมูล ({preprints.length})
                  </Label>
                  <div className="flex flex-wrap gap-1.5 max-h-24 overflow-y-auto">
                    {preprints.map((pp) => (
                      <Button
                        key={pp.FTPprCode}
                        type="button"
                        size="sm"
                        variant={doc.preprint?.preprintCode === pp.FTPprCode ? 'default' : 'outline'}
                        className={`h-7 text-[11px] px-2 ${
                          doc.preprint?.preprintCode === pp.FTPprCode
                            ? 'bg-[#007a87] hover:bg-[#00606b]'
                            : 'bg-white'
                        }`}
                        onClick={() => handleSelectSavedPreprint(pp)}
                        title={`${pp.FTPprFileName || pp.FTPprName} · ${pp.FTPpsCode} (${pp.FNPprCol}×${pp.FNPprRow})`}
                      >
                        {pp.FTPprName} ({pp.FTPpsCode} {pp.FNPprCol}×{pp.FNPprRow})
                      </Button>
                    ))}
                  </div>
                </div>
              )}

              <div className="space-y-1">
                <Label className="text-xs font-medium">รูปแบบภาพที่นำเข้า</Label>
                <div className="grid grid-cols-2 gap-2">
                  <button
                    type="button"
                    onClick={() => setImpScope('sheet')}
                    className={`px-2.5 py-1.5 rounded border text-xs text-left ${
                      impScope === 'sheet'
                        ? 'border-[#007a87] bg-teal-50 text-[#007a87] font-semibold'
                        : 'border-slate-200 bg-white text-slate-600'
                    }`}
                  >
                    เต็มแผ่น (A4 / B5)
                    <div className="text-[10px] font-normal opacity-80">ทาบทั้งแผ่น + ตัดช่องแรกเข้า Canvas</div>
                  </button>
                  <button
                    type="button"
                    onClick={() => setImpScope('label')}
                    className={`px-2.5 py-1.5 rounded border text-xs text-left ${
                      impScope === 'label'
                        ? 'border-[#007a87] bg-teal-50 text-[#007a87] font-semibold'
                        : 'border-slate-200 bg-white text-slate-600'
                    }`}
                  >
                    ป้ายเดี่ยว 1 ดวง
                    <div className="text-[10px] font-normal opacity-80">ใช้รูปนี้ซ้ำทุกช่องบนกระดาษ</div>
                  </button>
                </div>
              </div>

              <div className="grid grid-cols-12 gap-2.5">
                <div className="space-y-1 col-span-12 sm:col-span-6">
                  <Label className="text-[11px] font-medium">ขนาดกระดาษ</Label>
                  <Select
                    value={impPaperSize}
                    onValueChange={(v) => {
                      const code = v as PaperSizeCode;
                      const psInfo = findPaperPreset(code);
                      setImpPaperSize(code);
                      if (!psInfo.isCustom) {
                        setImpPaperW(psInfo.widthMm);
                        setImpPaperH(psInfo.heightMm);
                      }
                    }}
                  >
                    <SelectTrigger className="h-8 text-xs w-full">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {allPaperSizes.map((p) => (
                        <SelectItem key={p.FTPpsCode} value={p.FTPpsCode}>
                          {p.FTPpsName}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
                <div className="space-y-1 col-span-6 sm:col-span-3">
                  <Label className="text-[11px] font-medium">กว้างแผ่น (mm)</Label>
                  <Input
                    type="number"
                    step={0.5}
                    min={30}
                    disabled={!findPaperPreset(impPaperSize).isCustom}
                    value={impPaperW}
                    onChange={(e) => setImpPaperW(parseFloat(e.target.value) || 210)}
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="space-y-1 col-span-6 sm:col-span-3">
                  <Label className="text-[11px] font-medium">สูงแผ่น (mm)</Label>
                  <Input
                    type="number"
                    step={0.5}
                    min={30}
                    disabled={!findPaperPreset(impPaperSize).isCustom}
                    value={impPaperH}
                    onChange={(e) => setImpPaperH(parseFloat(e.target.value) || 297)}
                    className="h-8 text-xs font-mono"
                  />
                </div>
                <div className="col-span-12 flex flex-wrap items-center justify-between gap-2 text-[11px]">
                  <span className="text-slate-500">
                    ทิศทางแผ่น: <strong>{impPaperW > impPaperH ? 'แนวนอน (Landscape)' : 'แนวตั้ง (Portrait)'}</strong> · {impPaperW}×{impPaperH} mm
                  </span>
                  <Button
                    type="button"
                    size="sm"
                    variant="outline"
                    className="h-6 text-[11px] px-2 bg-white"
                    onClick={() => {
                      const nextW = impPaperH;
                      const nextH = impPaperW;
                      const matched = detectPaperPresetFromDimensions(nextW, nextH, allPaperSizes);
                      setImpPaperSize(matched);
                      setImpPaperW(nextW);
                      setImpPaperH(nextH);
                    }}
                  >
                    สลับแนวตั้ง ↔ แนวนอน
                  </Button>
                </div>
                {(Math.abs(impLabelW - doc.width) > 0.5 || Math.abs(impLabelH - doc.height) > 0.5) && (
                  <div className="col-span-12 rounded border border-amber-200 bg-amber-50 px-2.5 py-1.5 text-[11px] text-amber-900">
                    ขนาดดวงที่ตั้งไว้ (<strong>{impLabelW}×{impLabelH} mm</strong>) ต่างจากแบบป้ายที่เปิดอยู่ ({doc.width}×{doc.height} mm) — หลังกดนำเข้าแนะนำให้กด <strong>บันทึกเป็นแบบใหม่</strong> เพื่อไม่ให้ทับขนาดเดิม
                  </div>
                )}
                {(() => {
                  const chosenPage = impParsed?.pages.find((p) => p.pageNumber === impPageNum) || impParsed?.pages[0];
                  if (!chosenPage || impScope !== 'sheet') return null;
                  const chosenRatio = impPaperW / Math.max(1, impPaperH);
                  const diffPct = Math.abs(chosenRatio - chosenPage.aspectRatio) / Math.max(0.01, chosenPage.aspectRatio);
                  const sizeMismatch =
                    Boolean(chosenPage.mmWidth && chosenPage.mmHeight) &&
                    (Math.abs(impPaperW - (chosenPage.mmWidth || 0)) > 3 ||
                      Math.abs(impPaperH - (chosenPage.mmHeight || 0)) > 3);
                  if (diffPct <= 0.05 && !sizeMismatch) return null;
                  return (
                    <div className="col-span-12 rounded border border-red-200 bg-red-50 px-2.5 py-1.5 text-[11px] text-red-800 flex flex-wrap items-center justify-between gap-2">
                      <span>
                        ขนาดกระดาษที่เลือก (<strong>{impPaperW}×{impPaperH} mm</strong>) ไม่ตรงกับหน้า PDF จริง
                        {chosenPage.mmWidth && chosenPage.mmHeight ? ` (${chosenPage.mmWidth}×${chosenPage.mmHeight} mm)` : ''} — อาจทำให้กรอบทาบไม่ตรงจริง
                      </span>
                      {chosenPage.mmWidth && chosenPage.mmHeight && (
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          className="h-6 text-[10px] px-2 bg-white border-red-300 text-red-800 hover:bg-red-100 shrink-0"
                          onClick={() => {
                            const w = chosenPage.mmWidth || 210;
                            const h = chosenPage.mmHeight || 297;
                            const matched = detectPaperPresetFromDimensions(w, h, allPaperSizes);
                            setImpPaperSize(matched);
                            setImpPaperW(w);
                            setImpPaperH(h);
                          }}
                        >
                          ใช้ขนาดจริง ({chosenPage.mmWidth}×{chosenPage.mmHeight})
                        </Button>
                      )}
                    </div>
                  );
                })()}
              </div>

              <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 bg-amber-50/70 p-2.5 rounded border border-amber-200">
                <div className="space-y-1">
                  <div className="flex items-center justify-between text-[11px] text-amber-900">
                    <span className="font-medium">ความชัดพื้นทาบ</span>
                    <span className="font-mono font-semibold">{Math.round(impOpacity * 100)}%</span>
                  </div>
                  <input
                    type="range"
                    min={15}
                    max={100}
                    step={5}
                    value={Math.round(impOpacity * 100)}
                    onChange={(e) => setImpOpacity(Math.max(0.15, Math.min(1, Number(e.target.value) / 100)))}
                    className="w-full accent-amber-600 cursor-pointer"
                  />
                </div>
                <div className="space-y-1">
                  <div className="flex items-center justify-between text-[11px] text-amber-900">
                    <span className="font-medium">ความเข้มเส้น/ภาพ</span>
                    <span className="font-mono font-semibold">{impContrast}%</span>
                  </div>
                  <input
                    type="range"
                    min={60}
                    max={200}
                    step={5}
                    value={impContrast}
                    onChange={(e) => setImpContrast(Math.max(60, Math.min(200, Number(e.target.value) || 100)))}
                    className="w-full accent-amber-600 cursor-pointer"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 bg-slate-50 p-2.5 rounded border border-slate-200">
                <div className="space-y-1">
                  <Label className="text-[10px] font-medium">คอลัมน์</Label>
                  <Input
                    type="number"
                    min={1}
                    max={12}
                    value={impCols}
                    onChange={(e) => setImpCols(Math.max(1, parseInt(e.target.value) || 1))}
                    className="h-7 text-xs font-mono bg-white"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[10px] font-medium">แถว</Label>
                  <Input
                    type="number"
                    min={1}
                    max={20}
                    value={impRows}
                    onChange={(e) => setImpRows(Math.max(1, parseInt(e.target.value) || 1))}
                    className="h-7 text-xs font-mono bg-white"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[10px] font-medium">กว้างดวง (mm)</Label>
                  <Input
                    type="number"
                    step={0.1}
                    min={10}
                    value={impLabelW}
                    onChange={(e) => {
                      setImpAutoFit(false);
                      setImpLabelW(parseFloat(e.target.value) || 49);
                    }}
                    className="h-7 text-xs font-mono bg-white"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[10px] font-medium">สูงดวง (mm)</Label>
                  <Input
                    type="number"
                    step={0.1}
                    min={10}
                    value={impLabelH}
                    onChange={(e) => {
                      setImpAutoFit(false);
                      setImpLabelH(parseFloat(e.target.value) || 70);
                    }}
                    className="h-7 text-xs font-mono bg-white"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[10px] font-medium">ขอบบน (mm)</Label>
                  <Input
                    type="number"
                    step={0.1}
                    min={0}
                    value={impMagTop}
                    onChange={(e) => setImpMagTop(Math.max(0, parseFloat(e.target.value) || 0))}
                    className="h-7 text-xs font-mono bg-white"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[10px] font-medium">ขอบล่าง (mm)</Label>
                  <Input
                    type="number"
                    step={0.1}
                    min={0}
                    value={impMagBottom}
                    onChange={(e) => setImpMagBottom(Math.max(0, parseFloat(e.target.value) || 0))}
                    className="h-7 text-xs font-mono bg-white"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[10px] font-medium">ขอบซ้าย (mm)</Label>
                  <Input
                    type="number"
                    step={0.1}
                    min={0}
                    value={impMagLeft}
                    onChange={(e) => setImpMagLeft(Math.max(0, parseFloat(e.target.value) || 0))}
                    className="h-7 text-xs font-mono bg-white"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[10px] font-medium">ขอบขวา (mm)</Label>
                  <Input
                    type="number"
                    step={0.1}
                    min={0}
                    value={impMagRight}
                    onChange={(e) => setImpMagRight(Math.max(0, parseFloat(e.target.value) || 0))}
                    className="h-7 text-xs font-mono bg-white"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[10px] font-medium">ช่องว่าง H (mm)</Label>
                  <Input
                    type="number"
                    step={0.1}
                    min={0}
                    value={impGapH}
                    onChange={(e) => setImpGapH(Math.max(0, parseFloat(e.target.value) || 0))}
                    className="h-7 text-xs font-mono bg-white"
                  />
                </div>
                <div className="space-y-1">
                  <Label className="text-[10px] font-medium">ช่องว่าง V (mm)</Label>
                  <Input
                    type="number"
                    step={0.1}
                    min={0}
                    value={impGapV}
                    onChange={(e) => setImpGapV(Math.max(0, parseFloat(e.target.value) || 0))}
                    className="h-7 text-xs font-mono bg-white"
                  />
                </div>
                <div className="col-span-2 flex items-end justify-end pb-0.5">
                  <span className="text-[10px] text-slate-500">
                    กรอบสีส้ม = 4 ขอบกระดาษ · กรอบสีฟ้า = ช่องที่ 1
                  </span>
                </div>
                {(() => {
                  const usedW = impMagLeft + Math.max(1, impCols) * impLabelW + Math.max(0, impCols - 1) * impGapH;
                  const usedH = impMagTop + Math.max(1, impRows) * impLabelH + Math.max(0, impRows - 1) * impGapV;
                  const remRight = Number((impPaperW - usedW).toFixed(1));
                  const remBottom = Number((impPaperH - usedH).toFixed(1));
                  const overRight = remRight < impMagRight - 0.15;
                  const overBottom = remBottom < impMagBottom - 0.15;
                  return (
                    <div className="col-span-2 sm:col-span-4 space-y-1.5 pt-1">
                      <div className="flex flex-wrap items-center justify-between gap-2">
                        <label className="inline-flex items-center gap-1.5 text-[11px] font-medium text-[#007a87] cursor-pointer">
                          <Checkbox
                            checked={impAutoFit}
                            onCheckedChange={(v) => setImpAutoFit(!!v)}
                          />
                          ปรับขนาดดวงอัตโนมัติเมื่อแก้ 4 ขอบ/ตาราง
                        </label>
                        <Button
                          type="button"
                          size="sm"
                          variant="outline"
                          className="h-7 text-[11px] border-[#007a87] text-[#007a87] bg-white"
                          onClick={handleAutoFitPreprintCell}
                        >
                          <Calculator className="h-3 w-3 mr-1" />
                          คำนวณขนาดดวงพอดีกระดาษ
                        </Button>
                      </div>
                      <div className={`text-[10px] font-mono px-2 py-1 rounded border ${overRight || overBottom ? 'bg-red-50 border-red-200 text-red-700' : 'bg-white border-slate-200 text-slate-600'}`}>
                        พื้นที่เหลือจริง: ขวา {remRight} mm (ตั้งไว้ {impMagRight}) · ล่าง {remBottom} mm (ตั้งไว้ {impMagBottom})
                        {(overRight || overBottom) && ' — ขนาดดวงเกินกรอบที่ตั้งไว้!'}
                      </div>
                    </div>
                  );
                })()}
              </div>

              <label className="flex items-start gap-2 text-xs text-slate-700 cursor-pointer pt-1">
                <Checkbox
                  checked={impStripDecor}
                  onCheckedChange={(v) => setImpStripDecor(!!v)}
                  className="mt-0.5"
                />
                <span>
                  ล้างแถบสี/กรอบตกแต่งเดิมออกอัตโนมัติ (เหลือเฉพาะข้อมูลสินค้า ราคา บาร์โค้ด สำหรับพิมพ์ลง Pre-Print)
                </span>
              </label>
            </div>
          </div>

          <DialogFooter className="pt-2 border-t border-slate-100">
            <Button type="button" variant="outline" onClick={() => setImportPreprintOpen(false)}>
              ยกเลิก
            </Button>
            <Button
              type="button"
              className="bg-amber-600 hover:bg-amber-700 text-white"
              disabled={saving || impLoading}
              onClick={() => void handleApplyImportedPreprint()}
            >
              {saving && <Loader2 className="h-3.5 w-3.5 mr-1 animate-spin" />}
              นำมาใช้เป็นพื้นหลัง Pre-Print
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
