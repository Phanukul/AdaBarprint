import type { DesignElement, LabelDesignLayout } from '@/types/design';

export function previewLayout(raw?: string | null, size?: { FCLbsWidth: number; FCLbsHeight: number }): LabelDesignLayout | null {
  try {
    const layout = JSON.parse(raw || 'null');
    if (!Array.isArray(layout?.elements)) return null;
    return {
      ...layout, customDesign: true,
      width: layout.width || size?.FCLbsWidth,
      height: layout.height || size?.FCLbsHeight,
      elements: layout.elements.map((el: DesignElement, i: number) => ({
        ...el, id: el.id || `element-${i}`, x: el.x ?? 0, y: el.y ?? 0,
        width: el.width ?? 1, height: el.height ?? 1,
      })),
    };
  } catch { return null; }
}
