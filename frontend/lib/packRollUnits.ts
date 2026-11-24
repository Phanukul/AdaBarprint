export interface RollGrid {
  cols: number;
  cellWidth: number;
  cellHeight: number;
  left: number;
  gap: number;
  count: number;
}

export interface PrintUnit {
  kind: string;
  payload: Record<string, unknown>;
  roll?: RollGrid;
}

interface NamedUnit { name: string; unit: PrintUnit }

function stockKey({ unit }: NamedUnit) {
  const g = unit.roll;
  const p = unit.payload;
  if (unit.kind !== 'WINDOWS' || !g || g.cols < 2 || !g.count || p.paperSize !== 'ROLL') return;
  return JSON.stringify([p.printerName, p.trayName || '', p.paperWidthMm, p.paperHeightMm,
    g.cols, g.cellWidth, g.cellHeight, g.left, g.gap]);
}

async function joinRoll(group: NamedUnit[]): Promise<NamedUnit> {
  const first = group[0].unit;
  const grid = first.roll!;
  const images: string[] = [];
  let canvas: HTMLCanvasElement | undefined;
  let column = 0;
  let count = 0;
  for (const { unit } of group) {
    let remaining = unit.roll!.count;
    const pages = unit.payload.pageImages as string[];
    if (pages.length !== Math.ceil(remaining / grid.cols)) throw new Error('จำนวนภาพม้วนไม่ตรงกับจำนวนป้าย');
    for (const src of pages) {
      const cells = Math.min(grid.cols, remaining);
      remaining -= cells;
      count += cells;
      if (!column && cells === grid.cols) {
        images.push(src);
        continue;
      }
      const image = new Image();
      image.src = src;
      await image.decode();
      const px = image.width / Number(unit.payload.paperWidthMm);
      const left = (col: number) => Math.round((grid.left + col * (grid.cellWidth + grid.gap)) * px);
      for (let i = 0; i < cells; i++) {
        if (!canvas) {
          canvas = document.createElement('canvas');
          canvas.width = image.width;
          canvas.height = image.height;
          const ctx = canvas.getContext('2d')!;
          ctx.fillStyle = 'white';
          ctx.fillRect(0, 0, canvas.width, canvas.height);
        }
        const width = Math.round(grid.cellWidth * px);
        canvas.getContext('2d')!.drawImage(image, left(i), 0, width, image.height,
          left(column), 0, width, image.height);
        if (++column === grid.cols) {
          images.push(canvas.toDataURL('image/png'));
          column = 0;
          canvas = undefined;
        }
      }
    }
  }
  if (canvas) images.push(canvas.toDataURL('image/png'));
  const name = [...new Set(group.map(entry => entry.name))].join(' + ');
  return { name, unit: { ...first, roll: { ...grid, count },
    payload: { ...first.payload, priTypeLabel: name, pageImages: images } } };
}

export async function packRollUnits(sets: { name: string; units: PrintUnit[] }[]): Promise<NamedUnit[]> {
  const entries = sets.flatMap(set => set.units.map(unit => ({ name: set.name, unit })));
  const result: NamedUnit[] = [];
  for (let start = 0; start < entries.length;) {
    const key = stockKey(entries[start]);
    let end = start + 1;
    while (key && end < entries.length && stockKey(entries[end]) === key) end++;
    result.push(end === start + 1 ? entries[start] : await joinRoll(entries.slice(start, end)));
    start = end;
  }
  return result;
}
