import React, { useEffect, useRef, useState } from 'react';
import JsBarcode from 'jsbarcode';
import {
  DESIGN_ICON_LIST,
  type DesignElement,
  type LabelDesignLayout,
} from '@/types/design';

export function detectBarcodeFormat(value: string): string {
  if (/^\d{13}$/.test(value)) return 'EAN13';
  if (/^\d{8}$/.test(value)) return 'EAN8';
  if (/^\d{12}$/.test(value)) return 'UPC';
  if (/^\d{14}$/.test(value)) return 'ITF14';
  if (/^[A-Z0-9\-. $/+%]+$/.test(value) && value.length <= 43) return 'CODE39';
  return 'CODE128';
}

export interface LabelPreviewLine {
  barcode: string;
  priType: string;
  lbsCode?: string;
  ctx: Record<string, any>;
}

function formatPriceParts(val: unknown): { intPart: string; decPart: string } {
  const num = typeof val === 'number' ? val : parseFloat(String(val ?? '0'));
  if (Number.isNaN(num)) return { intPart: String(val || '0'), decPart: '.00' };
  const [intPart, dec] = num.toFixed(2).split('.');
  return { intPart: Number(intPart).toLocaleString(), decPart: `.${dec}` };
}

function formatAdaDate(raw: unknown): string {
  if (!raw) return '';
  const s = String(raw);
  const dt = new Date(s);
  if (Number.isNaN(dt.getTime())) return s;
  return `${dt.getDate()}/${dt.getMonth() + 1}/${dt.getFullYear()}`;
}

function formatAdaPriceNum(val: unknown): string {
  const num = typeof val === 'number' ? val : parseFloat(String(val ?? '0'));
  if (Number.isNaN(num)) return String(val ?? '-');
  return num.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
}

function SafeProductImage({
  src,
  alt,
  style,
  fallback,
}: {
  src: string;
  alt: string;
  style?: React.CSSProperties;
  fallback?: React.ReactNode;
}) {
  const [failedSrc, setFailedSrc] = useState<string | null>(null);
  if (!src || failedSrc === src) {
    return <>{fallback ?? null}</>;
  }
  return (
    <img
      src={src}
      alt={alt}
      style={style}
      onError={() => setFailedSrc(src)}
    />
  );
}

function resolveDesignFieldValue(el: DesignElement, line: LabelPreviewLine): string {
  const ctx = line.ctx || {};
  const product = ctx.product || {};
  const price = ctx.price || {};
  const promo = ctx.promotion || {};
  const storeInfo = ctx.store || {};
  const tag = ctx.tag || {};

  if (el.type === 'static' || el.type === 'badge') {
    return el.text ?? '';
  }
  if (el.type === 'date' || el.field === 'tag.print_date') {
    return formatAdaDate(tag.print_date || new Date().toISOString()) || '';
  }

  let raw = '';
  switch (el.field) {
    case 'price.value': {
      const val = price.value ?? 0;
      raw = formatAdaPriceNum(val);
      break;
    }
    case 'price.old_value': {
      const oldVal = price.old_value ?? price.value ?? 0;
      raw = formatAdaPriceNum(oldVal).replace(/\.00$/, '');
      break;
    }
    case 'product.name':
      raw = String(product.name ?? '-');
      break;
    case 'product.name_oth':
      raw = String(product.name_oth ?? '');
      break;
    case 'product.barcode':
      raw = String(product.barcode || line.barcode || '');
      break;
    case 'product.code':
      raw = String(product.code ?? '');
      break;
    case 'product.dim':
      raw = String(product.dim ?? '');
      break;
    case 'tag.selling_unit':
      raw = String(tag.selling_unit || product.unit || 'ชิ้น / EACH');
      break;
    case 'tag.pri_per_unit':
      raw = String(tag.pri_per_unit ?? '');
      break;
    case 'tag.pri_status':
      raw = String(tag.pri_status || 'AC');
      break;
    case 'tag.sub_dept':
      raw = String(tag.sub_dept || product.sub_dept || '');
      break;
    case 'promotion.text':
      raw = String(promo.text || promo.name || 'โปรโมชันพิเศษ');
      break;
    case 'promotion.interval':
      raw = String(promo.interval || '');
      break;
    case 'store.name':
      raw = String(storeInfo.name || 'AdaPos Store');
      break;
    default:
      raw = el.text ?? '';
  }
  return `${el.prefix || ''}${raw}${el.suffix || ''}`;
}

function DesignBarcodeSvg({
  value,
  scale,
  heightMm,
  showText = true,
  fontSizePt = 7,
  color = '#111111',
}: {
  value: string;
  scale: number;
  heightMm: number;
  showText?: boolean;
  fontSizePt?: number;
  color?: string;
}) {
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!svgRef.current || !value) return;
    try {
      const fmt = detectBarcodeFormat(value);
      JsBarcode(svgRef.current, value, {
        format: fmt,
        width: 1.5,
        height: 28,
        displayValue: false,
        margin: 0,
        flat: true,
        lineColor: color || '#111111',
      });
    } catch {
      try {
        JsBarcode(svgRef.current, value, {
          format: 'CODE128',
          width: 1.5,
          height: 28,
          displayValue: false,
          margin: 0,
          flat: true,
          lineColor: color || '#111111',
        });
      } catch {
        // ignore barcode render failure
      }
    }
  }, [value, color]);

  if (!value) {
    return (
      <div
        style={{
          width: '100%',
          height: '100%',
          border: '1px dashed #94a3b8',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontSize: `${2 * scale}mm`,
          color: '#64748b',
        }}
      >
        BARCODE
      </div>
    );
  }

  const textHeightMm = showText ? Math.max(2.2, fontSizePt * 0.3528 * 1.15) : 0;
  const barHeightMm = Math.max(3, heightMm - textHeightMm);

  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        overflow: 'hidden',
      }}
    >
      <svg
        ref={svgRef}
        preserveAspectRatio="none"
        style={{
          width: '100%',
          height: `${barHeightMm * scale}mm`,
          display: 'block',
        }}
      />
      {showText && (
        <div
          style={{
            fontSize: `${Math.max(4.5, fontSizePt) * 0.3528 * scale}mm`,
            fontWeight: 600,
            color: color || '#111111',
            lineHeight: 1.05,
            letterSpacing: `${0.2 * scale}mm`,
            whiteSpace: 'nowrap',
          }}
        >
          {value}
        </div>
      )}
    </div>
  );
}

const FONT_WEIGHT_MAP: Record<NonNullable<DesignElement['fontWeight']>, number> = {
  normal: 400,
  medium: 500,
  semibold: 600,
  bold: 700,
  extrabold: 800,
  black: 900,
};

export type PreprintRenderView = 'cell-guide' | 'sheet-overlay' | 'ink';

export function CustomDesignRenderer({
  layout,
  line,
  scale = 1,
  preprintView,
}: {
  layout: LabelDesignLayout;
  line: LabelPreviewLine;
  scale?: number;
  preprintView?: PreprintRenderView;
}) {
  const barcodeVal = String(line.ctx?.product?.barcode || line.barcode || '');
  const wMm = layout.width * scale;
  const hMm = layout.height * scale;
  const isPreprint = layout.designMode === 'preprint';
  const viewMode: PreprintRenderView = preprintView ?? (isPreprint ? 'cell-guide' : 'ink');
  const showCellGuideImg =
    isPreprint && viewMode === 'cell-guide' && Boolean(layout.preprint?.cellBgDataUrl);

  return (
    <div
      style={{
        width: `${wMm}mm`,
        height: `${hMm}mm`,
        background: isPreprint ? 'transparent' : layout.backgroundColor || '#FFFFFF',
        border: isPreprint
          ? 'none'
          : (layout.borderWidth ?? 0.3) > 0
            ? `${Math.max(0.5, (layout.borderWidth ?? 0.3) * 3.78 * scale)}px solid ${layout.borderColor || '#d1d5db'}`
            : 'none',
        borderRadius: `${(layout.borderRadius ?? 0) * scale}mm`,
        boxSizing: 'border-box',
        position: 'relative',
        overflow: 'hidden',
        fontFamily: "'Noto Sans Thai', 'Tahoma', sans-serif",
      }}
      className="flex-shrink-0"
    >
      {showCellGuideImg && layout.preprint?.cellBgDataUrl && (
        <img
          src={layout.preprint.cellBgDataUrl}
          alt="Pre-print Guide"
          style={{
            position: 'absolute',
            inset: 0,
            width: '100%',
            height: '100%',
            objectFit: 'fill',
            opacity: Math.max(0.1, Math.min(1, layout.preprint.opacity ?? 0.7)),
            filter:
              layout.preprint.contrast && layout.preprint.contrast !== 100
                ? `contrast(${layout.preprint.contrast}%)`
                : undefined,
            pointerEvents: 'none',
            userSelect: 'none',
          }}
        />
      )}
      {layout.elements
        .filter(el=>!el.when||el.when.path.split('.').reduce<any>((v,key)=>v?.[key],line.ctx)===el.when.eq)
        .filter((el) => el.visible !== false)
        .map((el) => {
          const commonStyle: React.CSSProperties = {
            position: 'absolute',
            left: `${el.x * scale}mm`,
            top: `${el.y * scale}mm`,
            width: `${Math.max(0.2, el.width) * scale}mm`,
            height: `${Math.max(0.2, el.height) * scale}mm`,
            opacity: el.opacity ?? 1,
            transform: el.rotation ? `rotate(${el.rotation}deg)` : undefined,
            transformOrigin: 'center center',
            boxSizing: 'border-box',
            overflow: 'hidden',
          };

          if (el.type === 'rect' || el.type === 'circle' || el.type === 'line') {
            return (
              <div
                key={el.id}
                style={{
                  ...commonStyle,
                  background: el.fill || el.color || '#E2E8F0',
                  border:
                    (el.borderWidth ?? 0) > 0
                      ? `${Math.max(0.5, (el.borderWidth ?? 0) * 3.78 * scale)}px solid ${el.borderColor || '#94A3B8'}`
                      : 'none',
                  borderRadius:
                    el.type === 'circle'
                      ? '9999px'
                      : `${(el.borderRadius ?? 0) * scale}mm`,
                }}
              />
            );
          }

          if (el.type === 'icon') {
            const iconDef =
              DESIGN_ICON_LIST.find((i) => i.name === el.iconName) ||
              DESIGN_ICON_LIST[0];
            return (
              <div
                key={el.id}
                style={{
                  ...commonStyle,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  background: el.fill && el.fill !== 'transparent' ? el.fill : 'transparent',
                  borderRadius: `${(el.borderRadius ?? 0) * scale}mm`,
                }}
              >
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke={el.color || iconDef.defaultColor}
                  strokeWidth={2}
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  style={{ width: '100%', height: '100%' }}
                >
                  {iconDef.paths.map((d, idx) => (
                    <path key={idx} d={d} />
                  ))}
                </svg>
              </div>
            );
          }

          if (el.type === 'image') {
            const resolvedSrc =
              el.src ||
              (el.field === 'product.image' ? String(line.ctx?.product?.image || '') : '');
            const placeholderNode = (
              <div
                style={{
                  display: 'flex',
                  flexDirection: 'column',
                  alignItems: 'center',
                  justifyContent: 'center',
                  color: '#64748b',
                  gap: `${0.8 * scale}mm`,
                  padding: `${1 * scale}mm`,
                }}
              >
                <svg
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="#64748b"
                  strokeWidth={1.6}
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  style={{
                    width: `${Math.min(el.width, el.height) * 0.42 * scale}mm`,
                    height: `${Math.min(el.width, el.height) * 0.42 * scale}mm`,
                  }}
                >
                  <rect width="18" height="18" x="3" y="3" rx="2" ry="2" />
                  <circle cx="9" cy="9" r="2" />
                  <path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21" />
                </svg>
                <span style={{ fontSize: `${Math.max(1.8, Math.min(3.5, el.height * 0.14)) * scale}mm`, fontWeight: 600 }}>
                  รูปสินค้า
                </span>
              </div>
            );
            return (
              <div
                key={el.id}
                style={{
                  ...commonStyle,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  background:
                    el.fill && el.fill !== 'transparent'
                      ? el.fill
                      : resolvedSrc
                        ? 'transparent'
                        : '#f8fafc',
                  border:
                    (el.borderWidth ?? 0) > 0
                      ? `${Math.max(0.5, (el.borderWidth ?? 0) * 3.78 * scale)}px solid ${el.borderColor || '#cbd5e1'}`
                      : resolvedSrc
                        ? 'none'
                        : '1px dashed #94a3b8',
                  borderRadius: `${(el.borderRadius ?? 0) * scale}mm`,
                }}
              >
                <SafeProductImage
                  src={resolvedSrc}
                  alt={el.label || 'product'}
                  style={{
                    width: '100%',
                    height: '100%',
                    objectFit: el.objectFit || 'contain',
                    display: 'block',
                  }}
                  fallback={placeholderNode}
                />
              </div>
            );
          }

          if (el.type === 'barcode') {
            return (
              <div key={el.id} style={commonStyle}>
                <DesignBarcodeSvg
                  value={barcodeVal}
                  scale={scale}
                  heightMm={el.height}
                  showText={el.showText !== false}
                  fontSizePt={el.fontSize ?? 7}
                  color={el.color || '#111111'}
                />
              </div>
            );
          }

          const valText = resolveDesignFieldValue(el, line);
          const fontSizeMm = Math.max(4, el.fontSize ?? 10) * 0.3528 * scale;
          const align = el.align || 'left';
          const justify =
            align === 'center' ? 'center' : align === 'right' ? 'flex-end' : 'flex-start';
          const isPriceEl = el.type === 'price';
          const isMulti = Boolean(el.multiLine) && !isPriceEl && el.type !== 'badge';

          return (
            <div
              key={el.id}
              style={{
                ...commonStyle,
                display: 'flex',
                alignItems: isMulti ? 'flex-start' : 'center',
                justifyContent: justify,
                textAlign: align,
                fontSize: `${fontSizeMm}mm`,
                fontWeight: FONT_WEIGHT_MAP[el.fontWeight || 'normal'],
                color: el.color || '#111111',
                background: el.fill && el.fill !== 'transparent' ? el.fill : 'transparent',
                border:
                  (el.borderWidth ?? 0) > 0
                    ? `${Math.max(0.5, (el.borderWidth ?? 0) * 3.78 * scale)}px solid ${el.borderColor || '#94A3B8'}`
                    : 'none',
                borderRadius: `${(el.borderRadius ?? 0) * scale}mm`,
                textDecoration: el.strike ? 'line-through' : 'none',
                lineHeight: isMulti ? 1.18 : 1.05,
                padding: el.type === 'badge' || (el.fill && el.fill !== 'transparent') ? `0 ${1 * scale}mm` : 0,
                whiteSpace: isMulti ? 'normal' : 'nowrap',
                wordBreak: isMulti ? 'break-word' : undefined,
                textOverflow: isPriceEl || isMulti ? 'clip' : 'ellipsis',
              }}
            >
              <span
                style={
                  isPriceEl || isMulti
                    ? { maxWidth: '100%', width: isMulti ? '100%' : undefined }
                    : { overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '100%' }
                }
              >
                {valText}
              </span>
            </div>
          );
        })}
    </div>
  );
}

export function LabelPreview({
  line,
  scale = 1,
  labelW = 100,
  labelH = 38,
  lbsCode,
  customLayout,
  preprintView,
}: {
  line: LabelPreviewLine;
  scale?: number;
  labelW?: number;
  labelH?: number;
  lbsCode?: string;
  customLayout?: LabelDesignLayout | null;
  preprintView?: PreprintRenderView;
}) {
  const barcodeRef = useRef<SVGSVGElement>(null);
  const { ctx } = line;
  const product = ctx.product || {};
  const price = ctx.price || {};
  const promo = ctx.promotion || {};
  const storeInfo = ctx.store || {};
  const tag = ctx.tag || {};
  const effectiveLbs = lbsCode || line.lbsCode || '';
  const isPromo = line.priType === '2';
  const bgColor = isPromo ? '#FFF3C4' : '#FFFFFF';
  const w = labelW;
  const h = labelH;
  const barcodeValue = product.barcode || line.barcode || '';
  const isLandscape = w >= h;

  useEffect(() => {
    if (!barcodeRef.current || !barcodeValue) return;
    try {
      const fmt = detectBarcodeFormat(barcodeValue);
      JsBarcode(barcodeRef.current, barcodeValue, {
        format: fmt,
        width: 1.5,
        height: 28,
        displayValue: false,
        margin: 0,
        flat: true,
      });
    } catch {
      try {
        JsBarcode(barcodeRef.current, barcodeValue, {
          format: 'CODE128',
          width: 1.5,
          height: 28,
          displayValue: false,
          margin: 0,
          flat: true,
        });
      } catch {
        // barcode render failed
      }
    }
  }, [barcodeValue, effectiveLbs]);

  if (customLayout && customLayout.customDesign && Array.isArray(customLayout.elements)) {
    return (
      <CustomDesignRenderer
        layout={customLayout}
        line={line}
        scale={scale}
        preprintView={preprintView}
      />
    );
  }

  // ─── UFM Fuji Super Formats (from UFMFromat sheet) ─────────────────────────
  if (effectiveLbs === 'UFM-NOR-16') {
    const { intPart, decPart } = formatPriceParts(price.value);
    const norIntSize = (intPart.length >= 6 ? 6.2 : intPart.length >= 4 ? 7.4 : 8.5) * scale;
    const norDecSize = (intPart.length >= 6 ? 3.2 : 4.2) * scale;
    return (
      <div
        style={{
          width: `${w * scale}mm`,
          height: `${h * scale}mm`,
          background: '#FFFFFF',
          border: '1px solid #cbd5e1',
          boxSizing: 'border-box',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
          position: 'relative',
        }}
        className="flex-shrink-0"
      >
        {/* Top Green Header + Red Oval UFM Fuji Super */}
        <div style={{ background: '#3b6e38', height: `${9 * scale}mm`, display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>
          <div style={{ background: '#c9302c', color: '#fff', borderRadius: '999px', padding: `${0.5 * scale}mm ${3 * scale}mm`, fontSize: `${2.2 * scale}mm`, fontWeight: 700, fontStyle: 'italic', lineHeight: 1.1 }}>
            {storeInfo.name || 'UFM Fuji Super'}
          </div>
        </div>
        {/* Dept + SubCode */}
        <div style={{ display: 'flex', justifyContent: 'space-between', padding: `${0.5 * scale}mm ${2 * scale}mm 0`, fontSize: `${2.1 * scale}mm`, fontWeight: 600, color: '#111', minHeight: `${3 * scale}mm` }}>
          <span>{tag.sub_dept || ''}</span>
          <span>{storeInfo.code || ''}</span>
        </div>
        {/* Product Name */}
        <div style={{ padding: `${0.8 * scale}mm ${2 * scale}mm`, fontSize: `${2.5 * scale}mm`, fontWeight: 700, lineHeight: 1.2, color: '#111', overflow: 'hidden', maxHeight: `${12 * scale}mm` }}>
          {product.name || '-'}
        </div>
        {/* Big Italic Price */}
        <div style={{ marginTop: 'auto', padding: `0 ${2 * scale}mm`, textAlign: 'right', fontWeight: 800, fontStyle: 'italic', color: '#111', lineHeight: 1, whiteSpace: 'nowrap' }}>
          <span style={{ fontSize: `${norIntSize}mm` }}>{intPart}</span>
          <span style={{ fontSize: `${norDecSize}mm` }}>{decPart}</span>
        </div>
        {/* Bottom Barcode + Baht */}
        <div style={{ padding: `${0.8 * scale}mm ${2 * scale}mm`, display: 'flex', alignItems: 'flex-end', justifyContent: 'space-between', gap: `${1 * scale}mm` }}>
          <div style={{ flex: 1, minWidth: 0 }}>
            {barcodeValue && <svg ref={barcodeRef} style={{ width: '100%', height: `${7 * scale}mm`, maxWidth: '100%' }} />}
            <div style={{ fontSize: `${1.8 * scale}mm`, color: '#111', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              ({barcodeValue}) {product.code || ''}
            </div>
          </div>
          <div style={{ fontSize: `${2.6 * scale}mm`, fontWeight: 700, color: '#111', whiteSpace: 'nowrap', paddingBottom: `${2 * scale}mm` }}>
            Baht บาท
          </div>
        </div>
        <div style={{ background: '#3b6e38', height: `${1.5 * scale}mm`, flexShrink: 0 }} />
      </div>
    );
  }

  if (effectiveLbs === 'UFM-SALE-8') {
    const { intPart, decPart } = formatPriceParts(price.value);
    const curNum = Number(price.value ?? 0);
    const oldNum = price.old_value != null ? Number(price.old_value) : 0;
    const hasDiscount = isPromo && oldNum > curNum && curNum >= 0;
    const headerBg = isPromo ? '#f5c518' : '#3b6e38';
    const badgeBg = isPromo ? '#c9302c' : '#ffffff';
    const badgeColor = isPromo ? '#ffffff' : '#3b6e38';
    const badgeText = isPromo ? 'SALE' : 'ราคาปกติ';
    const priceColor = isPromo ? '#c9302c' : '#111111';
    const shortInterval = String(promo.interval || tag.print_date || '')
      .replace(/เริ่ม\s*/g, '')
      .replace(/\s*ถึง\s*/g, ' - ');
    const saleIntSize = (intPart.length >= 6 ? 8 : intPart.length >= 4 ? 9.5 : 13) * scale;
    const saleDecSize = (intPart.length >= 6 ? 4.8 : 6) * scale;
    return (
      <div
        style={{
          width: `${w * scale}mm`,
          height: `${h * scale}mm`,
          background: '#FFFFFF',
          border: '1px solid #cbd5e1',
          boxSizing: 'border-box',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
        }}
        className="flex-shrink-0"
      >
        {/* Top Header: Yellow SALE (Promo) or Green Regular (Normal) */}
        <div style={{ background: headerBg, height: `${11 * scale}mm`, display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: `0 ${2.5 * scale}mm`, flexShrink: 0 }}>
          <div style={{ background: badgeBg, color: badgeColor, padding: `${0.8 * scale}mm ${3 * scale}mm`, fontSize: `${(isPromo ? 5.2 : 3.6) * scale}mm`, fontWeight: 900, fontStyle: isPromo ? 'italic' : 'normal', lineHeight: 1, borderRadius: isPromo ? 0 : `${1 * scale}mm` }}>
            {badgeText}
          </div>
          <div style={{ fontSize: `${2.4 * scale}mm`, fontWeight: 700, color: isPromo ? '#111' : '#ffffff', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', marginLeft: `${2 * scale}mm` }}>
            {isPromo ? shortInterval : (tag.print_date || '')}
          </div>
        </div>
        {/* Barcode line */}
        <div style={{ padding: `${0.8 * scale}mm ${2.5 * scale}mm 0`, display: 'flex', justifyContent: 'space-between', fontSize: `${2.2 * scale}mm`, fontWeight: 600, color: '#111' }}>
          <span>Barcode : {product.code || ''} ({barcodeValue})</span>
          <span>{storeInfo.code || ''}</span>
        </div>
        {/* Product Name */}
        <div style={{ padding: `${1 * scale}mm ${2.5 * scale}mm`, fontSize: `${3.0 * scale}mm`, fontWeight: 700, textAlign: 'center', lineHeight: 1.2, color: '#111', overflow: 'hidden', maxHeight: `${14 * scale}mm` }}>
          {product.name || '-'}
        </div>
        {/* Old Price + Barcode Left / Big Price Right */}
        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: `0 ${3 * scale}mm`, minHeight: 0 }}>
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', width: '36%' }}>
            {hasDiscount && (
              <div style={{ fontSize: `${3.2 * scale}mm`, fontWeight: 700, color: '#111', textDecoration: 'line-through', textDecorationColor: '#c9302c' }}>
                {oldNum.toFixed(2)}
              </div>
            )}
            {barcodeValue && <svg ref={barcodeRef} style={{ width: '100%', height: `${7 * scale}mm`, maxWidth: '100%' }} />}
            <div style={{ fontSize: `${1.8 * scale}mm`, color: '#111' }}>{barcodeValue}</div>
          </div>
          <div style={{ textAlign: 'right', fontWeight: 900, color: priceColor, lineHeight: 1, whiteSpace: 'nowrap' }}>
            <span style={{ fontSize: `${saleIntSize}mm` }}>{intPart}</span>
            <span style={{ fontSize: `${saleDecSize}mm` }}>{decPart}</span>
          </div>
        </div>
        {/* Bottom Bar */}
        <div style={{ background: headerBg, height: `${4.5 * scale}mm`, display: 'flex', alignItems: 'center', justifyContent: 'center', flexShrink: 0 }}>
          <div style={{ background: '#c9302c', color: '#fff', borderRadius: '999px', padding: `${0.2 * scale}mm ${2.5 * scale}mm`, fontSize: `${1.9 * scale}mm`, fontWeight: 700, fontStyle: 'italic' }}>
            {storeInfo.name || 'UFM Fuji Super'}
          </div>
        </div>
      </div>
    );
  }

  if (effectiveLbs === 'UFM-PRO-A5') {
    const { intPart, decPart } = formatPriceParts(price.value);
    const showDec = decPart !== '.00';
    const curNum = Number(price.value ?? 0);
    const oldNum = price.old_value != null ? Number(price.old_value) : 0;
    const hasDiscount = isPromo && oldNum > curNum && curNum >= 0;
    const proIntSize = (intPart.length >= 5 || (intPart.length >= 4 && showDec) ? 20 : intPart.length === 4 ? 24 : intPart.length === 3 ? 30 : showDec ? 34 : 42) * scale;
    const proDecSize = Math.round(proIntSize * 0.45 * 10) / 10;
    const oldText = oldNum % 1 !== 0 ? `${oldNum.toFixed(2)}.-` : `${oldNum.toFixed(0)}.-`;
    return (
      <div
        style={{
          width: `${w * scale}mm`,
          height: `${h * scale}mm`,
          background: '#FFFFFF',
          border: `${2.5 * scale}mm solid ${isPromo ? '#991b1b' : '#3b6e38'}`,
          boxSizing: 'border-box',
          display: 'flex',
          flexDirection: 'column',
          overflow: 'hidden',
        }}
        className="flex-shrink-0"
      >
        <div style={{ background: isPromo ? 'linear-gradient(90deg, #eab308, #facc15)' : '#3b6e38', padding: `${2 * scale}mm ${4 * scale}mm`, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div style={{ background: '#c9302c', color: '#fff', borderRadius: '999px', padding: `${1 * scale}mm ${5 * scale}mm`, fontSize: `${4.5 * scale}mm`, fontWeight: 800, fontStyle: 'italic' }}>
            {storeInfo.name || 'UFM Fuji Super'}
          </div>
          <div style={{ background: isPromo ? '#ca8a04' : '#166534', color: '#fff', padding: `${1 * scale}mm ${4 * scale}mm`, fontSize: `${4 * scale}mm`, fontWeight: 700, borderRadius: `${1 * scale}mm` }}>
            {isPromo ? `${promo.interval || tag.print_date || ''} - มีจำนวนจำกัด` : (tag.print_date || '')}
          </div>
          <div style={{ background: isPromo ? '#991b1b' : '#ffffff', color: isPromo ? '#fff' : '#3b6e38', padding: `${1 * scale}mm ${3 * scale}mm`, fontSize: `${(isPromo ? 5 : 3.8) * scale}mm`, fontWeight: 900, borderRadius: `${1 * scale}mm` }}>
            {isPromo ? '安い!' : 'ราคาปกติ'}
          </div>
        </div>
        <div style={{ textAlign: 'center', padding: `${2 * scale}mm ${4 * scale}mm 0`, maxHeight: `${18 * scale}mm`, overflow: 'hidden', flexShrink: 0 }}>
          <div style={{ fontSize: `${5.5 * scale}mm`, fontWeight: 800, color: '#111', lineHeight: 1.15, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{product.name || '-'}</div>
          {product.name_oth && <div style={{ fontSize: `${4.2 * scale}mm`, color: '#333', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{product.name_oth}</div>}
        </div>
        <div style={{ flex: 1, display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: `${2 * scale}mm ${6 * scale}mm`, minHeight: 0 }}>
          <div style={{ width: '32%', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: `${2 * scale}mm` }}>
            <div style={{ width: '100%', height: `${45 * scale}mm`, border: product.image ? 'none' : '1px dashed #cbd5e1', display: 'flex', alignItems: 'center', justifyContent: 'center', color: '#94a3b8', fontSize: `${3 * scale}mm`, overflow: 'hidden' }}>
              {product.image ? (
                <SafeProductImage
                  src={product.image}
                  alt={product.name || ''}
                  style={{ maxWidth: '100%', maxHeight: '100%', objectFit: 'contain' }}
                  fallback={barcodeValue ? <svg ref={barcodeRef} style={{ width: '85%', height: 'auto' }} /> : null}
                />
              ) : barcodeValue ? (
                <svg ref={barcodeRef} style={{ width: '85%', height: 'auto' }} />
              ) : null}
            </div>
            <div style={{ background: isPromo ? '#b91c1c' : '#3b6e38', color: '#fff', borderRadius: '999px', padding: `${1 * scale}mm ${4 * scale}mm`, fontSize: `${3.2 * scale}mm`, fontWeight: 700 }}>
              BARCODE: {barcodeValue}
            </div>
          </div>
          <div style={{ flex: 1, textAlign: 'center', display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center' }}>
            {hasDiscount && (
              <div style={{ fontSize: `${4 * scale}mm`, color: '#111', alignSelf: 'flex-end', marginRight: `${6 * scale}mm` }}>
                Normal Price <span style={{ fontWeight: 700, margin: `0 ${1 * scale}mm` }}>通常</span> <span style={{ textDecoration: 'line-through', textDecorationColor: '#c9302c', fontWeight: 700 }}>{oldText}</span>
              </div>
            )}
            <div style={{ display: 'flex', alignItems: 'baseline', justifyContent: 'center', fontWeight: 900, color: isPromo ? '#dc2626' : '#111111', lineHeight: 0.95, whiteSpace: 'nowrap' }}>
              <span style={{ fontSize: `${proIntSize}mm`, WebkitTextStroke: isPromo ? `${0.4 * scale}mm #450a0a` : undefined }}>{intPart}</span>
              {showDec && (
                <span style={{ fontSize: `${proDecSize}mm`, WebkitTextStroke: isPromo ? `${0.25 * scale}mm #450a0a` : undefined, marginLeft: `${0.8 * scale}mm` }}>{decPart}</span>
              )}
            </div>
            <div style={{ fontSize: `${6 * scale}mm`, fontWeight: 700, fontStyle: 'italic', color: isPromo ? '#c9302c' : '#111111', alignSelf: 'flex-end', marginRight: `${6 * scale}mm` }}>
              ฿/Pcs.
            </div>
          </div>
        </div>
      </div>
    );
  }

  if (effectiveLbs === 'UFM-TIMESALE') {
    const { intPart, decPart } = formatPriceParts(price.value);
    const curNum = Number(price.value ?? 0);
    const oldNum = price.old_value != null ? Number(price.old_value) : 0;
    const hasDiscount = isPromo && oldNum > curNum && oldNum > 0 && curNum >= 0;
    const discountPct = hasDiscount ? Math.round(((oldNum - curNum) / oldNum) * 100) : 0;
    const tsIntSize = (intPart.length >= 5 ? 20 : intPart.length === 4 ? 22 : intPart.length === 3 ? 30 : 42) * scale;
    const tsDecSize = (intPart.length >= 5 ? 10 : intPart.length === 4 ? 11 : intPart.length === 3 ? 14 : 15) * scale;
    const tsOldText = oldNum % 1 !== 0 ? oldNum.toFixed(2) : oldNum.toFixed(0);
    return (
      <div
        style={{
          width: `${w * scale}mm`,
          height: `${h * scale}mm`,
          background: '#FFFFFF',
          border: `${1.5 * scale}mm solid ${isPromo ? '#facc15' : '#3b6e38'}`,
          boxSizing: 'border-box',
          display: 'flex',
          overflow: 'hidden',
          position: 'relative',
        }}
        className="flex-shrink-0"
      >
        {/* Left Block: Yellow Time Sale (Promo) or Green Regular (Normal) */}
        <div style={{ width: '28%', background: isPromo ? '#facc15' : '#3b6e38', padding: `${2 * scale}mm`, display: 'flex', flexDirection: 'column', justifyContent: 'space-between', alignItems: 'center', flexShrink: 0 }}>
          <div style={{ width: '100%', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <div style={{ background: '#c9302c', color: '#fff', borderRadius: '999px', padding: `${0.6 * scale}mm ${2.8 * scale}mm`, fontSize: `${2.3 * scale}mm`, fontWeight: 700, fontStyle: 'italic' }}>
              {storeInfo.name || 'UFM Fuji Super'}
            </div>
            {isPromo && (
              <span style={{ color: '#c9302c', fontSize: `${4.2 * scale}mm`, fontWeight: 900, fontStyle: 'italic', transform: 'rotate(-6deg)' }}>セール!</span>
            )}
          </div>
          <div style={{ textAlign: 'center', color: isPromo ? '#c9302c' : '#ffffff', fontWeight: 900, lineHeight: 0.92, width: '100%' }}>
            {isPromo && discountPct > 0 ? (
              <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: `${0.6 * scale}mm` }}>
                <span style={{ fontSize: `${(discountPct >= 100 ? 15 : 20) * scale}mm`, color: '#fde047', WebkitTextStroke: `${0.9 * scale}mm #b91c1c`, textShadow: `${0.5 * scale}mm ${0.5 * scale}mm 0 #991b1b` }}>{discountPct}</span>
                <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', lineHeight: 0.88 }}>
                  <span style={{ fontSize: `${8 * scale}mm`, color: '#fff', WebkitTextStroke: `${0.65 * scale}mm #b91c1c`, textShadow: `${0.4 * scale}mm ${0.4 * scale}mm 0 #991b1b` }}>%</span>
                  <span style={{ fontSize: `${6.8 * scale}mm`, color: '#fff', WebkitTextStroke: `${0.65 * scale}mm #b91c1c`, textShadow: `${0.4 * scale}mm ${0.4 * scale}mm 0 #991b1b` }}>OFF</span>
                </div>
              </div>
            ) : isPromo && promo.text ? (
              <div style={{ fontSize: `${5 * scale}mm` }}>{promo.text}</div>
            ) : !isPromo ? (
              <div style={{ fontSize: `${4.5 * scale}mm`, lineHeight: 1.2 }}>
                <div>ราคาปกติ</div>
                <div style={{ fontSize: `${3.2 * scale}mm`, fontWeight: 700, opacity: 0.9 }}>Normal Price</div>
              </div>
            ) : null}
            {isPromo && (
              <div style={{ fontSize: `${(discountPct > 0 ? 5.8 : 7) * scale}mm`, fontStyle: 'italic', marginTop: `${0.8 * scale}mm`, WebkitTextStroke: `${0.25 * scale}mm #7f1d1d` }}>Time Sale!</div>
            )}
          </div>
          <div style={{ background: isPromo ? '#b91c1c' : '#166534', color: '#fff', borderRadius: `${1.5 * scale}mm`, padding: `${1.2 * scale}mm ${1.8 * scale}mm`, fontSize: `${2.0 * scale}mm`, textAlign: 'center', width: '100%', lineHeight: 1.25 }}>
            <div>{isPromo ? `${promo.interval || tag.print_date || ''} - มีจำนวนจำกัด` : (tag.print_date || '')}</div>
            <div style={{ fontWeight: 700 }}>BARCODE : {barcodeValue}</div>
          </div>
        </div>
        {/* Right White Price & Info */}
        <div style={{ flex: 1, padding: `${2 * scale}mm ${3.5 * scale}mm`, display: 'flex', flexDirection: 'column', justifyContent: 'space-between', minWidth: 0, position: 'relative' }}>
          <div>
            <div style={{ fontSize: `${3.6 * scale}mm`, fontWeight: 800, color: '#111', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{product.name || '-'}</div>
            {product.name_oth && <div style={{ fontSize: `${2.8 * scale}mm`, color: '#333', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{product.name_oth}</div>}
          </div>
          <div style={{ flex: 1, display: 'flex', alignItems: 'stretch', justifyContent: 'space-between', minHeight: 0, gap: `${2 * scale}mm` }}>
            {/* Left column: Special Price (Promo) or Normal Price (Normal) */}
            <div style={{ display: 'flex', flexDirection: 'column', justifyContent: 'space-between', padding: `${1 * scale}mm 0`, flexShrink: 0 }}>
              {isPromo ? (
                <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: `${0.4 * scale}mm`, marginTop: `${3 * scale}mm` }}>
                  <div style={{ fontSize: `${2.8 * scale}mm`, fontWeight: 700, color: '#111', lineHeight: 1.1 }}>ราคาพิเศษ</div>
                  <div style={{ background: '#b91c1c', color: '#fff', padding: `${0.3 * scale}mm ${1.5 * scale}mm`, fontSize: `${2.1 * scale}mm`, fontWeight: 700, borderRadius: `${0.5 * scale}mm`, lineHeight: 1.1 }}>
                    Special Price
                  </div>
                  <div style={{ background: '#b91c1c', color: '#fff', padding: `${0.2 * scale}mm ${1.5 * scale}mm`, fontSize: `${3.2 * scale}mm`, fontWeight: 900, lineHeight: 1.1, borderRadius: `${0.5 * scale}mm` }}>
                    特価
                  </div>
                </div>
              ) : (
                <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', gap: `${0.4 * scale}mm`, marginTop: `${3 * scale}mm` }}>
                  <div style={{ fontSize: `${2.8 * scale}mm`, fontWeight: 700, color: '#111', lineHeight: 1.1 }}>ราคาปกติ</div>
                  <div style={{ background: '#3b6e38', color: '#fff', padding: `${0.3 * scale}mm ${1.5 * scale}mm`, fontSize: `${2.1 * scale}mm`, fontWeight: 700, borderRadius: `${0.5 * scale}mm`, lineHeight: 1.1 }}>
                    Normal Price
                  </div>
                </div>
              )}
              {hasDiscount && (
                <div style={{ fontSize: `${3.3 * scale}mm`, color: '#111', whiteSpace: 'nowrap', lineHeight: 1.15, marginBottom: `${0.5 * scale}mm` }}>
                  <div style={{ fontWeight: 600 }}>ราคาปกติ</div>
                  <div style={{ marginTop: `${0.3 * scale}mm` }}>
                    <span style={{ fontWeight: 900, fontSize: `${4.8 * scale}mm`, marginRight: `${1.5 * scale}mm` }}>定価</span>
                    <span style={{ textDecoration: 'line-through', textDecorationColor: '#dc2626', textDecorationThickness: `${0.6 * scale}mm`, fontWeight: 900, fontSize: `${4.8 * scale}mm` }}>{tsOldText}฿/Pcs.</span>
                  </div>
                </div>
              )}
            </div>

            {/* Center + Right: Integer Price + Decimal Flush on Same Baseline + Product Photo Upper-Right */}
            <div style={{ flex: 1, position: 'relative', display: 'flex', alignItems: 'flex-end', justifyContent: 'center', paddingBottom: `${2 * scale}mm`, minWidth: 0 }}>
              {product.image && (
                <div style={{ position: 'absolute', top: 0, right: 0, width: `${40 * scale}mm`, height: `${28 * scale}mm`, display: 'flex', alignItems: 'center', justifyContent: 'center', overflow: 'hidden', pointerEvents: 'none' }}>
                  <SafeProductImage src={product.image} alt={product.name || ''} style={{ maxWidth: '100%', maxHeight: '100%', objectFit: 'contain' }} />
                </div>
              )}
              <div style={{ display: 'flex', alignItems: 'baseline', justifyContent: 'center', fontWeight: 900, color: isPromo ? '#dc2626' : '#111111', lineHeight: 0.85, zIndex: 1, paddingRight: `${product.image ? 4 * scale : 0}mm`, whiteSpace: 'nowrap' }}>
                <span style={{ fontSize: `${tsIntSize}mm`, letterSpacing: '-0.04em', WebkitTextStroke: isPromo ? `${0.45 * scale}mm #450a0a` : undefined }}>{intPart}</span>
                <span style={{ fontSize: `${tsDecSize}mm`, letterSpacing: '-0.02em', WebkitTextStroke: isPromo ? `${0.25 * scale}mm #450a0a` : undefined, marginLeft: `${0.5 * scale}mm` }}>{decPart}฿</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // ─── ADA Standard Formats (from PRJ21006 Analyze Specification) ────────────
  // 1) ADA-100x38 (Frm_SQL_PriceTag1Normal10x3.8 / Frm_SQL_PriceTag1Pmt10x3.8)
  if (effectiveLbs === 'ADA-100x38') {
    const { intPart, decPart } = formatPriceParts(price.value);
    const hasOld = isPromo && price.old_value != null && Number(price.old_value) > Number(price.value ?? 0);
    const pIntSize = (intPart.length >= 6 ? 9.5 : intPart.length >= 4 ? 11.5 : 14.5) * scale;
    const pDecSize = (intPart.length >= 6 ? 5.2 : 6.8) * scale;
    const printDateStr = formatAdaDate(tag.print_date);
    const subDeptMeta = [product.sub_dept || product.code || '', product.phasing || '', product.reple_type || '']
      .filter(Boolean)
      .join(' ');
    const giftOrPromoHead = promo.isGift && promo.text ? promo.text : 'ลดราคา';

    return (
      <div
        style={{
          width: `${w * scale}mm`,
          height: `${h * scale}mm`,
          background: isPromo ? '#FFEA00' : '#FFFFFF',
          border: '1px solid #d1d5db',
          boxSizing: 'border-box',
          position: 'relative',
          overflow: 'hidden',
          padding: `${1.8 * scale}mm ${2.4 * scale}mm`,
          display: 'flex',
          flexDirection: 'column',
          fontFamily: "'Noto Sans Thai', 'Tahoma', sans-serif",
        }}
        className="flex-shrink-0"
      >
        {/* มุมสามเหลี่ยมแดงขวาล่าง */}
        <div
          style={{
            position: 'absolute',
            right: 0,
            bottom: 0,
            width: 0,
            height: 0,
            borderStyle: 'solid',
            borderWidth: `0 0 ${11 * scale}mm ${4.5 * scale}mm`,
            borderColor: 'transparent transparent #dc2626 transparent',
            pointerEvents: 'none',
          }}
        />

        <div style={{ display: 'flex', flex: 1, minHeight: 0, gap: `${2 * scale}mm` }}>
          {/* ซ้าย: ราคาปกติขีดฆ่า + โปรโมชัน + ราคาขายขนาดใหญ่ + ช่วงวันที่ */}
          <div style={{ width: '42%', display: 'flex', flexDirection: 'column', justifyContent: 'space-between', flexShrink: 0, minWidth: 0 }}>
            {isPromo ? (
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', fontSize: `${2.3 * scale}mm`, fontWeight: 700, color: '#111', lineHeight: 1.1 }}>
                {hasOld ? (
                  <span style={{ textDecoration: 'line-through', textDecorationColor: '#dc2626', textDecorationThickness: `${0.45 * scale}mm`, whiteSpace: 'nowrap' }}>
                    ปกติ {formatAdaPriceNum(price.old_value).replace(/\.00$/, '')}
                  </span>
                ) : (
                  <span />
                )}
                <span style={{ whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', marginLeft: `${1 * scale}mm` }}>
                  {giftOrPromoHead}
                </span>
              </div>
            ) : (
              <div style={{ height: `${2.2 * scale}mm` }} />
            )}

            <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'flex-start', color: '#000', fontWeight: 900, lineHeight: 0.9, whiteSpace: 'nowrap', margin: 'auto 0' }}>
              <span style={{ fontSize: `${pIntSize}mm`, letterSpacing: '-0.04em' }}>{intPart}</span>
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', marginLeft: `${0.6 * scale}mm`, paddingTop: `${0.4 * scale}mm` }}>
                <span style={{ fontSize: `${pDecSize}mm`, letterSpacing: '-0.02em', lineHeight: 0.95 }}>{decPart}</span>
                <span style={{ fontSize: `${2.8 * scale}mm`, fontWeight: 800, lineHeight: 1.05, marginTop: `${0.4 * scale}mm` }}>บาท</span>
              </div>
            </div>

            <div style={{ fontSize: `${1.85 * scale}mm`, color: '#111', fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', minHeight: `${2.4 * scale}mm` }}>
              {isPromo ? (promo.interval || '') : ''}
            </div>
          </div>

          {/* ขวา: ชื่อไทย/อังกฤษ ปริมาณ ราคาต่อหน่วย บาร์โค้ด สถานะ หน่วย รหัส วันที่ */}
          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', justifyContent: 'space-between', minWidth: 0, paddingRight: `${1.8 * scale}mm` }}>
            <div style={{ minWidth: 0 }}>
              <div style={{ fontSize: `${2.6 * scale}mm`, fontWeight: 700, color: '#111', lineHeight: 1.15, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                {product.name || '-'}
              </div>
              {product.name_oth && (
                <div style={{ fontSize: `${2.0 * scale}mm`, fontWeight: 600, color: '#222', lineHeight: 1.15, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                  {product.name_oth}
                </div>
              )}
              <div style={{ display: 'flex', gap: `${2 * scale}mm`, fontSize: `${1.9 * scale}mm`, color: '#222', lineHeight: 1.15, whiteSpace: 'nowrap', overflow: 'hidden' }}>
                {product.dim && <span>{product.dim}</span>}
                {tag.pri_per_unit && <span style={{ overflow: 'hidden', textOverflow: 'ellipsis' }}>{tag.pri_per_unit}</span>}
              </div>
            </div>

            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: `${1.5 * scale}mm`, minHeight: `${10.5 * scale}mm` }}>
              <div style={{ flex: 1, display: 'flex', flexDirection: 'column', alignItems: 'flex-start', minWidth: 0, overflow: 'hidden' }}>
                {barcodeValue && (
                  <>
                    <svg ref={barcodeRef} style={{ width: '100%', maxWidth: `${34 * scale}mm`, height: `${7.5 * scale}mm` }} />
                    <div style={{ fontSize: `${2.0 * scale}mm`, fontWeight: 600, letterSpacing: `${0.35 * scale}mm`, color: '#111', lineHeight: 1, whiteSpace: 'nowrap' }}>
                      {barcodeValue}
                    </div>
                  </>
                )}
              </div>
              <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', justifyContent: 'center', fontSize: `${1.9 * scale}mm`, color: '#111', fontWeight: 600, lineHeight: 1.3, flexShrink: 0 }}>
                <span>{tag.pri_status || 'AC'}</span>
                <span>{tag.selling_unit || 'ชิ้น / EACH'}</span>
              </div>
            </div>

            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', fontSize: `${1.85 * scale}mm`, color: '#111', fontWeight: 600, whiteSpace: 'nowrap', paddingRight: `${2 * scale}mm` }}>
              <span style={{ overflow: 'hidden', textOverflow: 'ellipsis' }}>{subDeptMeta}</span>
              <span>{printDateStr}</span>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // 2) ADA-50x38 (Frm_SQL_PriceTag2Normal5x3.8 / Frm_SQL_PriceTag2Pmt5x3.8)
  if (effectiveLbs === 'ADA-50x38') {
    const { intPart, decPart } = formatPriceParts(price.value);
    const hasOld = isPromo && price.old_value != null && Number(price.old_value) > Number(price.value ?? 0);
    const pIntSize = (intPart.length >= 6 ? 6.4 : intPart.length >= 4 ? 7.8 : 9.4) * scale;
    const pDecSize = (intPart.length >= 6 ? 3.8 : 4.8) * scale;
    const printDateStr = formatAdaDate(tag.print_date);
    const subDeptMeta = [product.sub_dept || product.code || '', product.phasing || '', product.reple_type || '']
      .filter(Boolean)
      .join(' ');
    const giftOrPromoHead = promo.isGift && promo.text ? promo.text : 'ลดราคา';

    return (
      <div
        style={{
          width: `${w * scale}mm`,
          height: `${h * scale}mm`,
          background: isPromo ? '#FFEA00' : '#FFFFFF',
          border: '1px solid #d1d5db',
          boxSizing: 'border-box',
          position: 'relative',
          overflow: 'hidden',
          padding: `${1.4 * scale}mm ${1.8 * scale}mm`,
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'space-between',
          fontFamily: "'Noto Sans Thai', 'Tahoma', sans-serif",
        }}
        className="flex-shrink-0"
      >
        <div
          style={{
            position: 'absolute',
            right: 0,
            bottom: 0,
            width: 0,
            height: 0,
            borderStyle: 'solid',
            borderWidth: `0 0 ${11 * scale}mm ${4.2 * scale}mm`,
            borderColor: 'transparent transparent #dc2626 transparent',
            pointerEvents: 'none',
          }}
        />

        {isPromo && (
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline', fontSize: `${1.9 * scale}mm`, fontWeight: 700, color: '#111', lineHeight: 1.05 }}>
            {hasOld ? (
              <span style={{ textDecoration: 'line-through', textDecorationColor: '#dc2626', textDecorationThickness: `${0.35 * scale}mm`, whiteSpace: 'nowrap' }}>
                ปกติ {formatAdaPriceNum(price.old_value).replace(/\.00$/, '')}
              </span>
            ) : (
              <span />
            )}
            <span style={{ whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', marginLeft: `${1 * scale}mm` }}>
              {giftOrPromoHead}
            </span>
          </div>
        )}

        <div style={{ display: 'flex', alignItems: 'baseline', justifyContent: 'center', fontWeight: 900, color: '#000', lineHeight: 0.92, whiteSpace: 'nowrap' }}>
          <span style={{ fontSize: `${pIntSize}mm`, letterSpacing: '-0.04em' }}>{intPart}</span>
          <span style={{ fontSize: `${pDecSize}mm`, letterSpacing: '-0.02em' }}>{decPart}</span>
          <span style={{ fontSize: `${2.3 * scale}mm`, fontWeight: 800, marginLeft: `${0.8 * scale}mm` }}>บาท</span>
        </div>

        <div style={{ minWidth: 0 }}>
          <div style={{ fontSize: `${2.25 * scale}mm`, fontWeight: 700, color: '#111', lineHeight: 1.12, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
            {product.name || '-'}
          </div>
          {product.name_oth && (
            <div style={{ fontSize: `${1.75 * scale}mm`, color: '#222', lineHeight: 1.12, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {product.name_oth}
            </div>
          )}
          {tag.pri_per_unit && (
            <div style={{ fontSize: `${1.65 * scale}mm`, color: '#222', lineHeight: 1.12, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {tag.pri_per_unit}
            </div>
          )}
        </div>

        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end', gap: `${1 * scale}mm`, paddingRight: `${2 * scale}mm` }}>
          <div style={{ flex: 1, minWidth: 0, overflow: 'hidden' }}>
            {barcodeValue && (
              <>
                <svg ref={barcodeRef} style={{ width: '100%', maxWidth: `${26 * scale}mm`, height: `${5.6 * scale}mm` }} />
                <div style={{ fontSize: `${1.65 * scale}mm`, fontWeight: 600, letterSpacing: `${0.2 * scale}mm`, color: '#111', lineHeight: 1, whiteSpace: 'nowrap' }}>
                  {barcodeValue}
                </div>
              </>
            )}
            <div style={{ fontSize: `${1.55 * scale}mm`, color: '#111', fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', marginTop: `${0.4 * scale}mm` }}>
              {isPromo && promo.interval ? `${promo.interval} ` : ''}{subDeptMeta}
            </div>
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', fontSize: `${1.6 * scale}mm`, color: '#111', fontWeight: 600, lineHeight: 1.22, flexShrink: 0 }}>
            <span>{tag.pri_status || 'AC'}</span>
            <span>{tag.selling_unit || 'ชิ้น / EACH'}</span>
            <span>{printDateStr}</span>
          </div>
        </div>
      </div>
    );
  }

  // 3) ADA Price Cards Landscape: ADA-200x142 (15), ADA-200x95 (14), ADA-200x95-SP (18)
  if (effectiveLbs === 'ADA-200x142' || effectiveLbs === 'ADA-200x95' || effectiveLbs === 'ADA-200x95-SP') {
    const isA5 = effectiveLbs === 'ADA-200x142';
    const isSpecial = effectiveLbs === 'ADA-200x95-SP';
    const { intPart, decPart } = formatPriceParts(price.value);
    const hasOld = isPromo && price.old_value != null && Number(price.old_value) > Number(price.value ?? 0);
    const oldParts = formatPriceParts(price.old_value);
    const cardIntSize = (isA5
      ? (intPart.length >= 6 ? 28 : intPart.length >= 4 ? 36 : 44)
      : (intPart.length >= 6 ? 21 : intPart.length >= 4 ? 26 : 32)) * scale;
    const cardDecSize = (isA5 ? 11.5 : 8.8) * scale;
    const badgeSize = (isA5 ? 42 : 31) * scale;
    const badgeBg = isPromo ? '#dc2626' : '#2e6f40';
    const topPromoLine1 = promo.isGift && promo.text ? promo.text : (promo.name || '');
    const topPromoLine2 = isA5 ? (promo.free_text || '') : '';

    return (
      <div
        style={{
          width: `${w * scale}mm`,
          height: `${h * scale}mm`,
          background: isPromo ? '#F2D263' : '#FFFFFF',
          border: '1px solid #d1d5db',
          boxSizing: 'border-box',
          position: 'relative',
          overflow: 'hidden',
          padding: `${(isA5 ? 4 : 2.8) * scale}mm ${(isA5 ? 5.5 : 4.2) * scale}mm`,
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'space-between',
          fontFamily: "'Noto Sans Thai', 'Tahoma', sans-serif",
        }}
        className="flex-shrink-0"
      >
        {/* แถบเฉียงมุมซ้าย + วงกลมตราโปรโมชัน / ปกติ */}
        <div
          style={{
            position: 'absolute',
            left: 0,
            top: 0,
            width: 0,
            height: 0,
            borderStyle: 'solid',
            borderWidth: `${(isA5 ? 64 : 48) * scale}mm ${(isA5 ? 24 : 18) * scale}mm 0 0`,
            borderColor: `${badgeBg} transparent transparent transparent`,
            pointerEvents: 'none',
          }}
        />
        <div
          style={{
            position: 'absolute',
            left: `${(isA5 ? 5 : 3.5) * scale}mm`,
            top: `${(isA5 ? -4 : -3) * scale}mm`,
            width: `${badgeSize}mm`,
            height: `${badgeSize}mm`,
            borderRadius: '9999px',
            background: badgeBg,
            border: `${0.9 * scale}mm solid ${isPromo ? '#F2D263' : '#e5e7eb'}`,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            color: '#fef08a',
            transform: 'rotate(-12deg)',
            textAlign: 'center',
            lineHeight: 1.05,
            boxSizing: 'border-box',
            zIndex: 1,
            pointerEvents: 'none',
          }}
        >
          {isPromo ? (
            isSpecial ? (
              <>
                <span style={{ fontSize: `${(isA5 ? 6.2 : 4.5) * scale}mm`, fontWeight: 900, color: '#fff' }}>สินค้าพิเศษ</span>
                <span style={{ fontSize: `${(isA5 ? 4.4 : 3.3) * scale}mm`, fontWeight: 700, fontStyle: 'italic', color: '#fde047', marginTop: `${0.5 * scale}mm` }}>Promotion</span>
              </>
            ) : (
              <>
                <span style={{ fontSize: `${(isA5 ? 7.2 : 5.2) * scale}mm`, fontWeight: 900, fontStyle: 'italic', color: '#fef08a' }}>ลด</span>
                <span style={{ fontSize: `${(isA5 ? 6.2 : 4.6) * scale}mm`, fontWeight: 900, fontStyle: 'italic', color: '#fef08a' }}>กระหน่ำ</span>
                <span style={{ fontSize: `${(isA5 ? 4.2 : 3.1) * scale}mm`, fontWeight: 800, color: '#fff', letterSpacing: '0.06em' }}>SALE</span>
              </>
            )
          ) : (
            <>
              <span style={{ fontSize: `${(isA5 ? 6.0 : 4.4) * scale}mm`, fontWeight: 900, color: '#fff' }}>ราคาปกติ</span>
              <span style={{ fontSize: `${(isA5 ? 3.5 : 2.7) * scale}mm`, fontWeight: 700, color: '#dcfce7', marginTop: `${0.4 * scale}mm` }}>Normal Price</span>
            </>
          )}
        </div>

        {/* แถวบน: ข้อความโปรโมชัน / Free text (รองรับ 2 บรรทัดสำหรับ A5) */}
        <div style={{ paddingLeft: `${(isA5 ? 48 : 36) * scale}mm`, minHeight: `${(isA5 ? 14 : 7) * scale}mm`, display: 'flex', flexDirection: 'column', justifyContent: 'center', zIndex: 2 }}>
          {isPromo && topPromoLine1 && (
            <div style={{ fontSize: `${(isA5 ? 5.6 : 4.2) * scale}mm`, fontWeight: 800, color: '#111', lineHeight: 1.15, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {topPromoLine1}
            </div>
          )}
          {isPromo && topPromoLine2 && (
            <div style={{ fontSize: `${4.3 * scale}mm`, fontWeight: 700, color: '#111', lineHeight: 1.15, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {topPromoLine2}
            </div>
          )}
        </div>

        {/* กลาง: ราคาขนาดใหญ่ */}
        <div style={{ paddingLeft: `${(isA5 ? 38 : 30) * scale}mm`, display: 'flex', alignItems: 'center', justifyContent: 'center', flex: 1, minHeight: 0, zIndex: 2 }}>
          <div style={{ display: 'flex', alignItems: 'flex-end', justifyContent: 'center', color: '#000', fontWeight: 900, lineHeight: 0.86, whiteSpace: 'nowrap' }}>
            <span style={{ fontSize: `${cardIntSize}mm`, letterSpacing: '-0.04em' }}>{intPart}</span>
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', marginLeft: `${1.5 * scale}mm`, paddingBottom: `${(isA5 ? 3 : 1.8) * scale}mm` }}>
              <span style={{ fontSize: `${cardDecSize}mm`, lineHeight: 0.9 }}>{decPart}</span>
              <span style={{ fontSize: `${(isA5 ? 6.5 : 5.0) * scale}mm`, fontWeight: 800, lineHeight: 1.05, marginTop: `${0.8 * scale}mm` }}>บาท</span>
            </div>
          </div>
        </div>

        {/* ล่าง: ซ้าย ราคาปกติ | กลาง รายละเอียดสินค้า | ขวา ช่วงวันที่ + บาร์โค้ด */}
        <div style={{ display: 'flex', alignItems: 'flex-end', justifyContent: 'space-between', gap: `${3 * scale}mm`, zIndex: 2, minHeight: `${(isA5 ? 27 : 20) * scale}mm` }}>
          <div style={{ width: '22%', flexShrink: 0 }}>
            {hasOld && (
              <div style={{ color: '#111', lineHeight: 1.1 }}>
                <div style={{ fontSize: `${(isA5 ? 4.2 : 3.2) * scale}mm`, fontWeight: 700 }}>ราคาปกติ</div>
                <div style={{ display: 'flex', alignItems: 'flex-start', fontWeight: 800, marginTop: `${0.5 * scale}mm`, whiteSpace: 'nowrap' }}>
                  <span style={{ fontSize: `${(isA5 ? 7.2 : 5.4) * scale}mm`, lineHeight: 0.95 }}>{oldParts.intPart}</span>
                  <div style={{ display: 'flex', flexDirection: 'column', marginLeft: `${0.6 * scale}mm`, lineHeight: 0.95 }}>
                    <span style={{ fontSize: `${(isA5 ? 3.5 : 2.7) * scale}mm` }}>{oldParts.decPart}</span>
                    <span style={{ fontSize: `${(isA5 ? 2.4 : 1.9) * scale}mm`, marginTop: `${0.3 * scale}mm` }}>บาท</span>
                  </div>
                </div>
              </div>
            )}
          </div>

          <div style={{ flex: 1, minWidth: 0, color: '#111', lineHeight: 1.2 }}>
            <div style={{ fontSize: `${(isA5 ? 4.5 : 3.4) * scale}mm`, fontWeight: 800, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {product.name || '-'}
            </div>
            {product.name_oth && (
              <div style={{ fontSize: `${(isA5 ? 3.4 : 2.6) * scale}mm`, fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
                {product.name_oth}
              </div>
            )}
            <div style={{ display: 'flex', gap: `${3 * scale}mm`, fontSize: `${(isA5 ? 3.1 : 2.4) * scale}mm`, fontWeight: 600, marginTop: `${0.5 * scale}mm`, whiteSpace: 'nowrap', overflow: 'hidden' }}>
              {product.dim && <span>{product.dim}</span>}
              <span>จำนวน 1 {tag.selling_unit || 'ชิ้น'}</span>
            </div>
          </div>

          <div style={{ width: '26%', display: 'flex', flexDirection: 'column', alignItems: 'center', flexShrink: 0, overflow: 'hidden' }}>
            {isPromo && promo.interval && (
              <div style={{ fontSize: `${(isA5 ? 2.6 : 2.1) * scale}mm`, fontWeight: 700, color: '#111', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '100%', marginBottom: `${0.6 * scale}mm` }}>
                {promo.interval}
              </div>
            )}
            {barcodeValue && (
              <>
                <svg ref={barcodeRef} style={{ width: '100%', maxWidth: `${36 * scale}mm`, height: `${(isA5 ? 10.5 : 8) * scale}mm` }} />
                <div style={{ fontSize: `${(isA5 ? 2.4 : 1.9) * scale}mm`, fontWeight: 700, letterSpacing: `${0.3 * scale}mm`, color: '#111', lineHeight: 1 }}>
                  {barcodeValue}
                </div>
              </>
            )}
          </div>
        </div>
      </div>
    );
  }

  // 4) ADA-100x142 (Frm_SQL_PriceTag16Pmt100x142 — A6 แนวตั้ง)
  if (effectiveLbs === 'ADA-100x142') {
    const { intPart, decPart } = formatPriceParts(price.value);
    const hasOld = isPromo && price.old_value != null && Number(price.old_value) > Number(price.value ?? 0);
    const oldParts = formatPriceParts(price.old_value);
    const cardIntSize = (intPart.length >= 6 ? 19 : intPart.length >= 4 ? 23 : 28) * scale;
    const cardDecSize = 8.5 * scale;
    const badgeBg = isPromo ? '#dc2626' : '#2e6f40';
    const topGiftText = promo.isGift && promo.text ? promo.text : '';

    return (
      <div
        style={{
          width: `${w * scale}mm`,
          height: `${h * scale}mm`,
          background: isPromo ? '#F2D263' : '#FFFFFF',
          border: '1px solid #d1d5db',
          boxSizing: 'border-box',
          position: 'relative',
          overflow: 'hidden',
          padding: `${3.5 * scale}mm ${4.5 * scale}mm`,
          display: 'flex',
          flexDirection: 'column',
          justifyContent: 'space-between',
          fontFamily: "'Noto Sans Thai', 'Tahoma', sans-serif",
        }}
        className="flex-shrink-0"
      >
        {/* แถบเฉียงมุมซ้าย + วงกลม สินค้าจัดรายการ */}
        <div
          style={{
            position: 'absolute',
            left: 0,
            top: 0,
            width: 0,
            height: 0,
            borderStyle: 'solid',
            borderWidth: `${56 * scale}mm ${18 * scale}mm 0 0`,
            borderColor: `${badgeBg} transparent transparent transparent`,
            pointerEvents: 'none',
          }}
        />
        <div
          style={{
            position: 'absolute',
            left: `${4 * scale}mm`,
            top: `${-3 * scale}mm`,
            width: `${34 * scale}mm`,
            height: `${34 * scale}mm`,
            borderRadius: '9999px',
            background: badgeBg,
            border: `${0.8 * scale}mm solid ${isPromo ? '#F2D263' : '#e5e7eb'}`,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            color: '#fef08a',
            transform: 'rotate(-12deg)',
            textAlign: 'center',
            lineHeight: 1.05,
            boxSizing: 'border-box',
            zIndex: 1,
            pointerEvents: 'none',
          }}
        >
          {isPromo ? (
            <>
              <span style={{ fontSize: `${5.0 * scale}mm`, fontWeight: 900, fontStyle: 'italic', color: '#fef08a' }}>สินค้า</span>
              <span style={{ fontSize: `${4.2 * scale}mm`, fontWeight: 900, fontStyle: 'italic', color: '#fef08a' }}>จัดรายการ</span>
              <span style={{ fontSize: `${3.1 * scale}mm`, fontWeight: 700, fontStyle: 'italic', color: '#fff', marginTop: `${0.4 * scale}mm` }}>Promotion</span>
            </>
          ) : (
            <>
              <span style={{ fontSize: `${5.2 * scale}mm`, fontWeight: 900, color: '#fff' }}>ราคาปกติ</span>
              <span style={{ fontSize: `${3.0 * scale}mm`, fontWeight: 700, color: '#dcfce7', marginTop: `${0.4 * scale}mm` }}>Normal</span>
            </>
          )}
        </div>

        {/* มุมขวาบน: ราคาปกติ + ข้อความของแถม */}
        <div style={{ display: 'flex', justifyContent: 'flex-end', alignItems: 'flex-start', minHeight: `${30 * scale}mm`, paddingLeft: `${38 * scale}mm`, zIndex: 2 }}>
          <div style={{ textAlign: 'right', color: '#111' }}>
            {hasOld && (
              <div>
                <div style={{ fontSize: `${3.6 * scale}mm`, fontWeight: 700 }}>ราคาปกติ</div>
                <div style={{ display: 'flex', alignItems: 'flex-start', justifyContent: 'flex-end', fontWeight: 800, lineHeight: 0.95 }}>
                  <span style={{ fontSize: `${6.8 * scale}mm` }}>{oldParts.intPart}</span>
                  <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', marginLeft: `${0.5 * scale}mm` }}>
                    <span style={{ fontSize: `${3.2 * scale}mm` }}>{oldParts.decPart}</span>
                    <span style={{ fontSize: `${2.2 * scale}mm`, marginTop: `${0.2 * scale}mm` }}>บาท</span>
                  </div>
                </div>
              </div>
            )}
            {isPromo && topGiftText && (
              <div style={{ fontSize: `${3.8 * scale}mm`, fontWeight: 800, color: '#111', marginTop: `${1 * scale}mm`, whiteSpace: 'nowrap' }}>
                {topGiftText}
              </div>
            )}
          </div>
        </div>

        {/* กลาง: ราคาขนาดใหญ่ */}
        <div style={{ display: 'flex', alignItems: 'flex-end', justifyContent: 'center', fontWeight: 900, color: '#000', lineHeight: 0.88, margin: 'auto 0', whiteSpace: 'nowrap', zIndex: 2 }}>
          <span style={{ fontSize: `${cardIntSize}mm`, letterSpacing: '-0.04em' }}>{intPart}</span>
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-start', marginLeft: `${1.2 * scale}mm`, paddingBottom: `${1.8 * scale}mm` }}>
            <span style={{ fontSize: `${cardDecSize}mm`, lineHeight: 0.9 }}>{decPart}</span>
            <span style={{ fontSize: `${4.8 * scale}mm`, fontWeight: 800, lineHeight: 1.05, marginTop: `${0.6 * scale}mm` }}>บาท</span>
          </div>
        </div>

        {/* รายละเอียดสินค้า */}
        <div style={{ color: '#111', lineHeight: 1.22, marginBottom: `${2 * scale}mm`, zIndex: 2 }}>
          <div style={{ fontSize: `${4.0 * scale}mm`, fontWeight: 800, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
            {product.name || '-'}
          </div>
          {product.name_oth && (
            <div style={{ fontSize: `${3.0 * scale}mm`, fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {product.name_oth}
            </div>
          )}
          {product.dim && (
            <div style={{ fontSize: `${2.8 * scale}mm`, fontWeight: 600 }}>{product.dim}</div>
          )}
          <div style={{ fontSize: `${2.8 * scale}mm`, fontWeight: 600 }}>
            จำนวน 1 {tag.selling_unit || 'ชิ้น'}
          </div>
        </div>

        {/* ล่างสุด: ช่วงวันที่ + บาร์โค้ด */}
        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', zIndex: 2 }}>
          {isPromo && promo.interval && (
            <div style={{ fontSize: `${2.4 * scale}mm`, fontWeight: 700, color: '#111', marginBottom: `${0.8 * scale}mm`, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis', maxWidth: '100%' }}>
              {promo.interval}
            </div>
          )}
          {barcodeValue && (
            <>
              <svg ref={barcodeRef} style={{ width: `${52 * scale}mm`, height: `${11 * scale}mm` }} />
              <div style={{ fontSize: `${2.4 * scale}mm`, fontWeight: 700, letterSpacing: `${0.4 * scale}mm`, color: '#111', lineHeight: 1 }}>
                {barcodeValue}
              </div>
            </>
          )}
        </div>
      </div>
    );
  }

  if (isLandscape) {
    return (
      <div
        style={{
          width: `${w * scale}mm`,
          height: `${h * scale}mm`,
          background: bgColor,
          border: '2px solid #000',
          boxSizing: 'border-box',
          borderRadius: '1px',
          padding: `${1.5 * scale}mm ${2 * scale}mm`,
          fontSize: `${scale * 2.8}mm`,
          position: 'relative',
          overflow: 'hidden',
          display: 'flex',
          flexDirection: 'column',
        }}
        className="flex-shrink-0"
      >
        {isPromo && promo.isDiscount && (
          <div style={{ color: '#CC0000', fontSize: `${scale * 2.8}mm`, fontWeight: 'bold', marginBottom: `${0.5 * scale}mm` }}>
            ลดราคา
          </div>
        )}
        {isPromo && promo.isGift && promo.text && (
          <div style={{ color: '#CC0000', fontSize: `${scale * 3}mm`, fontWeight: 'bold', marginBottom: `${0.5 * scale}mm` }}>
            {promo.text}
          </div>
        )}

        <div style={{ display: 'flex', flex: 1, gap: `${2 * scale}mm`, minHeight: 0 }}>
          <div style={{ width: '32%', display: 'flex', flexDirection: 'column', justifyContent: 'center', alignItems: 'center', flexShrink: 0 }}>
            {isPromo && promo.isDiscount && price.old_value != null && (
              <div style={{ fontSize: `${scale * 2.5}mm`, color: '#999', textDecoration: 'line-through', marginBottom: `${0.5 * scale}mm` }}>
                {typeof price.old_value === 'number' ? price.old_value.toLocaleString(undefined, { minimumFractionDigits: 2 }) : price.old_value}
              </div>
            )}
            <div style={{ fontSize: `${scale * 6.5}mm`, fontWeight: 'bold', color: isPromo ? '#CC0000' : '#000', lineHeight: 1, whiteSpace: 'nowrap' }}>
              {typeof price.value === 'number' ? price.value.toLocaleString(undefined, { minimumFractionDigits: 2 }) : (price.value || '-')}
            </div>
            <div style={{ fontSize: `${scale * 2.4}mm`, color: '#333' }}>บาท</div>
          </div>

          <div style={{ flex: 1, display: 'flex', flexDirection: 'column', justifyContent: 'flex-start', gap: `${0.5 * scale}mm`, overflow: 'hidden' }}>
            {storeInfo.name && (
              <div style={{ fontWeight: '700', fontSize: `${scale * 2.4}mm`, lineHeight: 1.2, color: '#111' }}>
                {storeInfo.name}
              </div>
            )}
            <div style={{ fontWeight: '600', fontSize: `${scale * 2.9}mm`, lineHeight: 1.2, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
              {product.name || '-'}
            </div>
            {product.name_oth && (
              <div style={{ color: '#666', fontSize: `${scale * 2.1}mm`, lineHeight: 1.2, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{product.name_oth}</div>
            )}
            {product.dim && (
              <div style={{ fontSize: `${scale * 2.1}mm`, color: '#555' }}>{product.dim}</div>
            )}
            {tag.pri_per_unit && (
              <div style={{ fontSize: `${scale * 1.9}mm`, color: '#777' }}>{tag.pri_per_unit}</div>
            )}
            <div style={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              marginTop: `${1 * scale}px`,
              overflow: 'hidden',
            }}>
              {barcodeValue ? (
                <div style={{
                  border: '1px dashed #999',
                  padding: `${1 * scale}px ${2 * scale}px`,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  background: '#fafafa',
                }}>
                  <svg ref={barcodeRef} style={{ height: '100%', width: 'auto', maxWidth: '100%' }} />
                </div>
              ) : (
                <div style={{
                  border: '1px dashed #bbb',
                  height: '100%',
                  width: '100%',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  fontSize: `${scale * 2.5}px`,
                  color: '#999',
                  background: '#fafafa',
                }}>
                  ไม่มีบาร์โค้ด
                </div>
              )}
            </div>
          </div>
        </div>

        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: `${scale * 1.9}mm`, color: '#666', marginTop: 'auto', paddingTop: `${0.5 * scale}mm`, borderTop: '1px solid #eee' }}>
          <span>{tag.sub_dept || ''}</span>
          <span>{tag.selling_unit || ''}</span>
          <span>{tag.print_date ? (isNaN(new Date(tag.print_date).getTime()) ? tag.print_date : new Date(tag.print_date).toLocaleDateString('th-TH', { day: '2-digit', month: '2-digit', year: '2-digit' })) : ''}</span>
        </div>
        {isPromo && promo.interval && (
          <div style={{ fontSize: `${scale * 1.8}mm`, color: '#CC0000', textAlign: 'center', marginTop: `${0.3 * scale}mm` }}>{promo.interval}</div>
        )}
      </div>
    );
  }

  return (
    <div
      style={{
        width: `${w * scale}mm`,
        height: `${h * scale}mm`,
        background: bgColor,
        border: '2px solid #000',
        boxSizing: 'border-box',
        borderRadius: '1px',
        padding: `${1.5 * scale}mm`,
        fontSize: `${scale * 2.2}mm`,
        position: 'relative',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
      }}
      className="flex-shrink-0"
    >
      {isPromo && promo.isDiscount && (
        <div style={{ color: '#CC0000', fontSize: `${scale * 2.2}mm`, fontWeight: 'bold', width: '100%', textAlign: 'center' }}>
          ลดราคา
        </div>
      )}
      {isPromo && promo.isGift && promo.text && (
        <div style={{ color: '#CC0000', fontSize: `${scale * 2.4}mm`, fontWeight: 'bold', width: '100%', textAlign: 'center' }}>
          {promo.text}
        </div>
      )}

      {/* Product name */}
      <div style={{ fontWeight: '600', fontSize: `${scale * 2.0}mm`, lineHeight: 1.15, textAlign: 'center', width: '100%', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', flexShrink: 0 }}>
        {product.name || '-'}
      </div>
      {product.name_oth && (
        <div style={{ color: '#666', fontSize: `${scale * 1.5}mm`, lineHeight: 1.15, textAlign: 'center', width: '100%', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', flexShrink: 0 }}>{product.name_oth}</div>
      )}

      {/* Old price (promo) */}
      {isPromo && promo.isDiscount && price.old_value != null && (
        <div style={{ fontSize: `${scale * 1.6}mm`, color: '#999', textDecoration: 'line-through', flexShrink: 0 }}>
          {typeof price.old_value === 'number' ? price.old_value.toLocaleString(undefined, { minimumFractionDigits: 2 }) : price.old_value}
        </div>
      )}

      <div style={{ fontSize: `${scale * 4.0}mm`, fontWeight: 'bold', color: isPromo ? '#CC0000' : '#000', lineHeight: 1, flexShrink: 0 }}>
        {typeof price.value === 'number' ? price.value.toLocaleString(undefined, { minimumFractionDigits: 2 }) : (price.value || '-')}
      </div>
      <div style={{ fontSize: `${scale * 1.6}mm`, color: '#333', marginBottom: `${0.4 * scale}mm`, flexShrink: 0 }}>บาท</div>

      <div style={{
        width: '92%',
        flex: 1,
        minHeight: 0,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        overflow: 'hidden',
      }}>
        {barcodeValue ? (
          <div style={{
            border: '1px dashed #999',
            padding: `${1 * scale}px ${2 * scale}px`,
            maxWidth: '100%',
            maxHeight: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            background: '#fafafa',
            overflow: 'hidden',
          }}>
            <svg ref={barcodeRef} style={{ width: '100%', height: 'auto', maxHeight: `${Math.max(6, h * 0.28) * scale}mm`, maxWidth: '100%' }} />
          </div>
        ) : (
          <div style={{
            border: '1px dashed #bbb',
            width: '100%',
            padding: `${2 * scale}px`,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: `${scale * 2.5}px`,
            color: '#999',
            background: '#fafafa',
          }}>
            ไม่มีบาร์โค้ด
          </div>
        )}
      </div>

      <div style={{ fontSize: `${scale * 2}px`, color: '#999', marginTop: 'auto', textAlign: 'center' }}>
        {product.code && <span>{product.code}</span>}
        {tag.print_date && <span> · {isNaN(new Date(tag.print_date).getTime()) ? tag.print_date : new Date(tag.print_date).toLocaleDateString('th-TH', { day: '2-digit', month: '2-digit', year: '2-digit' })}</span>}
      </div>
      {isPromo && promo.interval && (
        <div style={{ fontSize: `${scale * 2}px`, color: '#CC0000', textAlign: 'center' }}>{promo.interval}</div>
      )}
    </div>
  );
}
