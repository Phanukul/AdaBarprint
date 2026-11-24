import React from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

export function TablePagination({
  total,
  page,
  pageSize,
  onPageChange,
  onPageSizeChange,
}: {
  total: number;
  page: number;
  pageSize: number;
  onPageChange: (p: number) => void;
  onPageSizeChange?: (s: number) => void;
}) {
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const safePage = Math.min(Math.max(1, page), totalPages);
  const start = total === 0 ? 0 : (safePage - 1) * pageSize + 1;
  const end = Math.min(total, safePage * pageSize);

  return (
    <div className="flex flex-wrap items-center justify-between gap-2 pt-2 text-xs text-slate-600">
      <div className="flex items-center gap-2">
        <span>
          แสดง {start}–{end} จากทั้งหมด {total} รายการ
        </span>
        {onPageSizeChange && (
          <Select value={String(pageSize)} onValueChange={(v) => onPageSizeChange(Number(v))}>
            <SelectTrigger className="h-7 w-24 text-xs bg-white">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="10">10 / หน้า</SelectItem>
              <SelectItem value="20">20 / หน้า</SelectItem>
              <SelectItem value="50">50 / หน้า</SelectItem>
            </SelectContent>
          </Select>
        )}
      </div>
      <div className="flex items-center gap-1">
        <Button
          type="button"
          size="sm"
          variant="outline"
          className="h-7 px-2 text-xs bg-white"
          disabled={safePage <= 1}
          onClick={() => onPageChange(safePage - 1)}
        >
          <ChevronLeft className="h-3.5 w-3.5 mr-0.5" />
          ก่อนหน้า
        </Button>
        <span className="px-2 font-mono text-[11px] text-slate-700">
          หน้า {safePage} / {totalPages}
        </span>
        <Button
          type="button"
          size="sm"
          variant="outline"
          className="h-7 px-2 text-xs bg-white"
          disabled={safePage >= totalPages}
          onClick={() => onPageChange(safePage + 1)}
        >
          ถัดไป
          <ChevronRight className="h-3.5 w-3.5 ml-0.5" />
        </Button>
      </div>
    </div>
  );
}
