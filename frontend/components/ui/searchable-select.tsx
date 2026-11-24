import * as React from 'react';
import { createPortal } from 'react-dom';
import { Check, ChevronDown, Search, X } from 'lucide-react';
import { cn } from '@/lib/utils';

export interface SearchableOption {
  value: string;
  label: string;
  /** ข้อความรองสำหรับแสดงหรือช่วยค้นหา (เช่น รหัส, ขนาด, IP) */
  keywords?: string;
  /** แท็กสั้นๆ แสดงด้านหน้า เช่น [Pre-Print], [STD] */
  badge?: string;
  badgeClassName?: string;
}

interface SearchableSelectProps {
  value: string;
  onValueChange: (value: string) => void;
  options: SearchableOption[];
  placeholder?: string;
  searchPlaceholder?: string;
  emptyText?: string;
  disabled?: boolean;
  className?: string;
  dropdownMinWidth?: number;
  title?: string;
  ariaLabel?: string;
}

export function SearchableSelect({
  value,
  onValueChange,
  options,
  placeholder = 'เลือกรายการ...',
  searchPlaceholder = 'พิมพ์คำค้นหา (รหัส / ชื่อ)...',
  emptyText = 'ไม่พบรายการที่ค้นหา',
  disabled = false,
  className,
  dropdownMinWidth = 260,
  title,
  ariaLabel,
}: SearchableSelectProps) {
  const [open, setOpen] = React.useState(false);
  const [query, setQuery] = React.useState('');
  const [activeIndex, setActiveIndex] = React.useState(0);
  const [portalStyle, setPortalStyle] = React.useState<React.CSSProperties>({});
  const [portalHost, setPortalHost] = React.useState<HTMLElement | null>(null);

  const triggerRef = React.useRef<HTMLButtonElement>(null);
  const menuRef = React.useRef<HTMLDivElement>(null);
  const inputRef = React.useRef<HTMLInputElement>(null);

  const selectedOption = React.useMemo(
    () => options.find((opt) => opt.value === value),
    [options, value]
  );

  const filteredOptions = React.useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return options;
    return options.filter((opt) => {
      const hay = `${opt.label} ${opt.value} ${opt.badge ?? ''} ${opt.keywords ?? ''}`.toLowerCase();
      return hay.includes(q);
    });
  }, [options, query]);

  const updatePosition = React.useCallback(() => {
    const trigger = triggerRef.current;
    if (!trigger) return;
    const dialogEl = trigger.closest('[role="dialog"]') as HTMLElement | null;
    const host = dialogEl ?? document.body;
    setPortalHost(host);

    const rect = trigger.getBoundingClientRect();
    const width = Math.min(Math.max(rect.width, dropdownMinWidth), Math.max(0, window.innerWidth - 16));
    const estHeight = 320;
    const spaceBelow = window.innerHeight - rect.bottom;
    const openUpward = spaceBelow < estHeight && rect.top > spaceBelow;

    if (!dialogEl) {
      const maxLeft = Math.max(8, window.innerWidth - width - 8);
      setPortalStyle({
        position: 'fixed',
        left: Math.min(rect.left, maxLeft),
        width,
        ...(openUpward
          ? { bottom: window.innerHeight - rect.top + 4 }
          : { top: rect.bottom + 4 }),
        zIndex: 9999,
      });
    } else {
      const hostRect = dialogEl.getBoundingClientRect();
      const relLeft = Math.max(8, Math.min(rect.left - hostRect.left, hostRect.width - width - 8));
      const relTop = openUpward
        ? Math.max(8, rect.top - hostRect.top + dialogEl.scrollTop - estHeight - 4)
        : rect.bottom - hostRect.top + dialogEl.scrollTop + 4;
      setPortalStyle({
        position: 'absolute',
        left: relLeft,
        top: relTop,
        width: Math.min(width, Math.max(220, hostRect.width - 16)),
        zIndex: 9999,
      });
    }
  }, [dropdownMinWidth]);

  React.useEffect(() => {
    if (!open) return;
    setQuery('');
    setActiveIndex(0);
    updatePosition();
    const timer = window.setTimeout(() => {
      inputRef.current?.focus();
    }, 10);

    const onOutside = (e: MouseEvent) => {
      const target = e.target as Node;
      if (
        triggerRef.current?.contains(target) ||
        menuRef.current?.contains(target)
      ) {
        return;
      }
      setOpen(false);
    };
    const onScrollOrResize = () => updatePosition();

    document.addEventListener('mousedown', onOutside);
    window.addEventListener('resize', onScrollOrResize);
    window.addEventListener('scroll', onScrollOrResize, true);
    return () => {
      window.clearTimeout(timer);
      document.removeEventListener('mousedown', onOutside);
      window.removeEventListener('resize', onScrollOrResize);
      window.removeEventListener('scroll', onScrollOrResize, true);
    };
  }, [open, updatePosition]);

  React.useEffect(() => {
    setActiveIndex(0);
  }, [query]);

  const handleSelect = (val: string) => {
    onValueChange(val);
    setOpen(false);
    triggerRef.current?.focus();
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Escape') {
      e.preventDefault();
      setOpen(false);
      triggerRef.current?.focus();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      setActiveIndex((prev) =>
        filteredOptions.length === 0 ? 0 : (prev + 1) % filteredOptions.length
      );
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setActiveIndex((prev) =>
        filteredOptions.length === 0
          ? 0
          : (prev - 1 + filteredOptions.length) % filteredOptions.length
      );
    } else if (e.key === 'Enter' && filteredOptions[activeIndex]) {
      e.preventDefault();
      handleSelect(filteredOptions[activeIndex].value);
    }
  };

  const hostNode = portalHost ?? (typeof document !== 'undefined' ? document.body : null);

  return (
    <>
      <button
        ref={triggerRef}
        aria-label={ariaLabel}
        aria-expanded={open}
        aria-haspopup="listbox"
        type="button"
        disabled={disabled}
        title={title ?? (selectedOption ? `${selectedOption.badge ? `${selectedOption.badge} ` : ''}${selectedOption.label}` : placeholder)}
        onClick={() => !disabled && setOpen((o) => !o)}
        className={cn(
          'border-input flex h-8 w-full items-center justify-between gap-2 rounded-md border bg-white px-2.5 py-1.5 text-xs shadow-xs transition-[color,box-shadow] outline-none hover:bg-slate-50/70 focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50',
          className
        )}
      >
        <span className="flex items-center gap-1.5 min-w-0 flex-1 text-left">
          {selectedOption ? (
            <>
              {selectedOption.badge && (
                <span
                  className={cn(
                    'shrink-0 rounded px-1 py-0.5 text-[10px] font-medium leading-none',
                    selectedOption.badgeClassName || 'bg-slate-100 text-slate-700'
                  )}
                >
                  {selectedOption.badge}
                </span>
              )}
              <span title={selectedOption.label} className="truncate text-slate-900">{selectedOption.label}</span>
            </>
          ) : (
            <span className="truncate text-muted-foreground">{placeholder}</span>
          )}
        </span>
        <ChevronDown className="h-3.5 w-3.5 shrink-0 opacity-50" />
      </button>

      {open &&
        hostNode &&
        createPortal(
          <div
            ref={menuRef}
            style={portalStyle}
            className="pointer-events-auto rounded-md border border-slate-200 bg-white text-slate-900 shadow-lg overflow-hidden"
          >
            <div className="flex items-center gap-1.5 border-b border-slate-200 bg-slate-50 px-2.5 py-2">
              <Search className="h-3.5 w-3.5 shrink-0 text-slate-400" />
              <input
                ref={inputRef}
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder={searchPlaceholder}
                className="w-full bg-transparent text-xs outline-none placeholder:text-slate-400"
              />
              {query && (
                <button
                  type="button"
                  onClick={() => {
                    setQuery('');
                    inputRef.current?.focus();
                  }}
                  className="rounded p-0.5 text-slate-400 hover:bg-slate-200 hover:text-slate-700"
                  title="ล้างคำค้นหา"
                >
                  <X className="h-3 w-3" />
                </button>
              )}
            </div>

            <div className="max-h-64 overflow-y-auto p-1">
              {filteredOptions.length === 0 ? (
                <div className="py-5 text-center text-xs text-slate-400">{emptyText}</div>
              ) : (
                filteredOptions.map((opt, idx) => {
                  const isSelected = opt.value === value;
                  const isHighlighted = idx === activeIndex;
                  return (
                    <button
                      key={`${opt.value}-${idx}`}
                      type="button"
                      onMouseEnter={() => setActiveIndex(idx)}
                      onClick={() => handleSelect(opt.value)}
                      className={cn(
                        'flex w-full items-center justify-between gap-2 rounded-sm px-2 py-1.5 text-left text-xs transition-colors',
                        isHighlighted ? 'bg-teal-50 text-[#00606b]' : 'hover:bg-slate-50',
                        isSelected && 'font-medium bg-teal-50/70'
                      )}
                    >
                      <span className="flex items-center gap-1.5 min-w-0 flex-1">
                        {opt.badge && (
                          <span
                            className={cn(
                              'shrink-0 rounded px-1.5 py-0.5 text-[10px] font-medium leading-none',
                              opt.badgeClassName || 'bg-slate-100 text-slate-700'
                            )}
                          >
                            {opt.badge}
                          </span>
                        )}
                        <span className="min-w-0 whitespace-normal break-words">{opt.label}</span>
                      </span>
                      {isSelected && <Check className="h-3.5 w-3.5 shrink-0 text-[#007a87]" />}
                    </button>
                  );
                })
              )}
            </div>

            <div className="border-t border-slate-100 bg-slate-50/70 px-2.5 py-1 text-[10px] text-slate-400 flex items-center justify-between">
              <span>พบ {filteredOptions.length} รายการ</span>
              <span>↑↓ เลือก · Enter ยืนยัน · Esc ปิด</span>
            </div>
          </div>,
          hostNode
        )}
    </>
  );
}
