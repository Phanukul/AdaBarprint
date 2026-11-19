import React from 'react';
import { Input } from '@/components/ui/input';
import {
  getPaperSizeDisplayName,
  getReferenceFrameDims,
  type PreviewMediaConfig,
  type PreviewMediaMode,
  type PreviewRefFrame,
} from '@/lib/previewMedia';

interface PreviewMediaToolbarProps {
  config: PreviewMediaConfig;
  onChange: (next: PreviewMediaConfig) => void;
  savedPaperSize: string;
  lbsGroup?: string;
}

const MODE_OPTIONS: Array<{ value: PreviewMediaMode; label: string }> = [
  { value: 'AUTO', label: 'ตามค่าแบบป้าย' },
  { value: 'ROLL', label: 'ม้วน Thermal (Roll)' },
  { value: 'A4', label: 'A4' },
  { value: 'A4_L', label: 'A4 แนวนอน' },
  { value: 'A5', label: 'A5' },
  { value: 'A5_L', label: 'A5 แนวนอน' },
  { value: 'A6', label: 'A6' },
  { value: 'A3', label: 'A3' },
  { value: 'B5_JIS', label: 'B5 JIS' },
  { value: 'B5_ISO', label: 'B5 ISO' },
  { value: 'LETTER', label: 'Letter' },
  { value: 'LEGAL', label: 'Legal/F4' },
  { value: 'SINGLE', label: 'ป้ายเดี่ยว 1:1' },
  { value: 'CUSTOM', label: 'กำหนดเอง' },
];

export const PreviewMediaToolbar: React.FC<PreviewMediaToolbarProps> = ({
  config,
  onChange,
  savedPaperSize,
  lbsGroup,
}) => {
  const savedLabel = getPaperSizeDisplayName(savedPaperSize);

  const handleModeSelect = (mode: PreviewMediaMode) => {
    const isSmallerSheet =
      mode === 'B5_JIS' || mode === 'B5_ISO' || mode === 'A5' || mode === 'A5_L' || mode === 'A6';
    const autoSmaller =
      mode === 'AUTO' &&
      (savedPaperSize === 'B5_JIS' ||
        savedPaperSize === 'B5_ISO' ||
        savedPaperSize === 'A5' ||
        savedPaperSize === 'A5_L' ||
        savedPaperSize === 'A6');
    const nextRef: PreviewRefFrame =
      isSmallerSheet || autoSmaller
        ? 'A4'
        : mode === 'ROLL' || mode === 'SINGLE' || mode === 'A3'
          ? 'NONE'
          : config.refFrame;
    onChange({ ...config, mode, refFrame: nextRef });
  };

  return (
    <div className="flex flex-wrap items-center justify-between gap-2 bg-slate-50 border border-slate-200 rounded-md px-2.5 py-1.5 text-[11px]">
      <div className="flex flex-wrap items-center gap-1.5">
        <span className="font-medium text-slate-700 mr-0.5">ขนาด Preview:</span>
        <div className="inline-flex flex-wrap rounded border border-slate-200 bg-white p-0.5 gap-0.5">
          {MODE_OPTIONS.map((opt) => {
            const active = config.mode === opt.value;
            const text =
              opt.value === 'AUTO' ? `${opt.label} (${savedLabel})` : opt.label;
            return (
              <button
                key={opt.value}
                type="button"
                onClick={() => handleModeSelect(opt.value)}
                className={`px-2 py-0.5 rounded-xs font-medium transition-colors ${
                  active
                    ? 'bg-[#007a87] text-white'
                    : opt.value === 'ROLL' && lbsGroup === '2'
                      ? 'text-amber-800 bg-amber-50 hover:bg-amber-100'
                      : 'text-slate-600 hover:bg-slate-100'
                }`}
              >
                {text}
              </button>
            );
          })}
        </div>

        {config.mode === 'ROLL' && (
          <div className="flex items-center gap-2 ml-1 bg-amber-50 border border-amber-200 rounded px-2 py-0.5">
            <label className="inline-flex items-center gap-1 text-amber-900">
              <span>ดวง/แถว:</span>
              <Input
                type="number"
                min={1}
                max={6}
                value={config.rollCols}
                onChange={(e) =>
                  onChange({
                    ...config,
                    rollCols: Math.max(1, Math.min(6, parseInt(e.target.value) || 1)),
                  })
                }
                className="h-5 w-11 text-[11px] font-mono bg-white px-1"
              />
            </label>
            <label className="inline-flex items-center gap-1 text-amber-900">
              <span>Gap (mm):</span>
              <Input
                type="number"
                step={0.5}
                min={0}
                max={20}
                value={config.rollGapV}
                onChange={(e) =>
                  onChange({
                    ...config,
                    rollGapV: Math.max(0, parseFloat(e.target.value) || 0),
                  })
                }
                className="h-5 w-12 text-[11px] font-mono bg-white px-1"
              />
            </label>
          </div>
        )}

        {config.mode === 'CUSTOM' && (
          <div className="flex items-center gap-1.5 ml-1 bg-teal-50 border border-teal-200 rounded px-2 py-0.5">
            <label className="inline-flex items-center gap-1 text-teal-900">
              <span>กว้าง:</span>
              <Input
                type="number"
                min={10}
                max={500}
                value={config.customWidthMm}
                onChange={(e) =>
                  onChange({
                    ...config,
                    customWidthMm: Math.max(10, parseFloat(e.target.value) || 10),
                  })
                }
                className="h-5 w-14 text-[11px] font-mono bg-white px-1"
              />
            </label>
            <span className="text-teal-700">×</span>
            <label className="inline-flex items-center gap-1 text-teal-900">
              <span>สูง:</span>
              <Input
                type="number"
                min={10}
                max={500}
                value={config.customHeightMm}
                onChange={(e) =>
                  onChange({
                    ...config,
                    customHeightMm: Math.max(10, parseFloat(e.target.value) || 10),
                  })
                }
                className="h-5 w-14 text-[11px] font-mono bg-white px-1"
              />
            </label>
            <span className="text-teal-700">mm</span>
          </div>
        )}
      </div>

      <div className="flex items-center gap-1.5">
        <span className="text-slate-600">กรอบเทียบด้านหลัง:</span>
        <div className="inline-flex rounded border border-slate-200 bg-white p-0.5 gap-0.5 text-[10px]">
          {(
            [
              { value: 'NONE', label: 'ไม่แสดง' },
              { value: 'A4', label: 'เทียบ A4' },
              { value: 'B5_JIS', label: 'เทียบ B5' },
            ] as const
          ).map((rf) => (
            <button
              key={rf.value}
              type="button"
              onClick={() => onChange({ ...config, refFrame: rf.value })}
              className={`px-1.5 py-0.5 rounded-xs font-medium ${
                config.refFrame === rf.value
                  ? 'bg-slate-700 text-white'
                  : 'text-slate-600 hover:bg-slate-100'
              }`}
            >
              {rf.label}
            </button>
          ))}
        </div>
      </div>
    </div>
  );
};

interface PreviewReferenceFrameWrapperProps {
  refFrame: PreviewRefFrame;
  sheetWidthMm: number;
  sheetHeightMm: number;
  scale: number;
  children: React.ReactNode;
}

export const PreviewReferenceFrameWrapper: React.FC<PreviewReferenceFrameWrapperProps> = ({
  refFrame,
  sheetWidthMm,
  sheetHeightMm,
  scale,
  children,
}) => {
  const frame = getReferenceFrameDims(refFrame);
  if (!frame) {
    return <>{children}</>;
  }
  const outerW = Math.max(frame.widthMm, sheetWidthMm);
  const outerH = Math.max(frame.heightMm, sheetHeightMm);

  return (
    <div
      className="mx-auto relative"
      style={{
        width: `${outerW * scale}mm`,
        height: `${outerH * scale}mm`,
        boxSizing: 'border-box',
      }}
    >
      <div
        className="absolute top-0 left-0 border-2 border-dashed border-slate-400 bg-slate-100/75 shadow-inner pointer-events-none"
        style={{
          width: `${frame.widthMm * scale}mm`,
          height: `${frame.heightMm * scale}mm`,
          boxSizing: 'border-box',
        }}
      >
        <div className="absolute bottom-1 right-1.5 text-[10px] font-mono text-slate-500 bg-white/85 px-1 rounded border border-slate-200">
          อ้างอิง: {frame.label}
        </div>
      </div>
      <div className="relative z-10 inline-block">{children}</div>
    </div>
  );
};
