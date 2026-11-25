import React from 'react';
import { flushSync } from 'react-dom';
import { createRoot } from 'react-dom/client';
import { LabelPreview } from './components/LabelPreview';
import { capturePagesToPngBase64 } from './lib/captureLabelPages';
import type { LabelPreviewLine } from './components/LabelPreview';
import type { ConfigLabelSizeRow } from './components/SettingsPanel';

async function render() {
  const { size, contexts, thermal, design }: { design?: import('./types/design').LabelDesignLayout; size: ConfigLabelSizeRow; contexts: LabelPreviewLine['ctx'][]; thermal: boolean } =
    JSON.parse(document.getElementById('job')!.textContent!);
  const w = size.FCLbsWidth, h = size.FCLbsHeight;
  const cols = Math.max(1, size.FNLbsCol || 1);
  const pw = size.FCLbsPaperWidth || 210, ph = size.FCLbsPaperHeight || 297;
  const margin = design ? 3 : 0;
  const gap = design ? 2 : 0;
  const offsetX = size.FCLbsOffsetX || 0, offsetY = size.FCLbsOffsetY || 0;
  const root = createRoot(document.getElementById('page')!);
  flushSync(() => root.render(
    <div data-capture-sheet="1" style={{ position: 'relative', overflow: 'hidden', width: pw+'mm', height: ph+'mm', background: 'white', fontFamily: 'Tahoma, sans-serif' }}>
      {contexts.map((ctx, slot) => <div key={slot} style={{
        position: 'absolute', width: w+'mm', height: h+'mm', overflow: 'hidden',
        left: ((size.FCLbsMagLeft ?? margin) + (design ? offsetX : 0) + slot % cols * (w + (size.FCLbsGapH ?? gap)))+'mm',
        top: ((size.FCLbsMagTop ?? margin) + (design ? offsetY : 0) + Math.floor(slot / cols) * (h + (size.FCLbsGapV ?? gap)))+'mm',
      }}><div style={{ transform: `translate(${design ? 0 : offsetX}mm,${design ? 0 : offsetY}mm)` }}>
        <LabelPreview customLayout={design} preprintView="ink" line={{ ctx, barcode: ctx.product.barcode, priType: ctx.legacyPriType }} lbsCode={size.FTLbsCode} labelW={w} labelH={h} />
      </div></div>)}
    </div>
  ));
  const [image] = await capturePagesToPngBase64('page', pw, ph, thermal);
  if (image) document.body.innerHTML = '<output>'+image+'</output>';
}
void render().catch(error => {
  const message = document.createElement('output');
  message.id = 'render-error';
  message.textContent = error instanceof Error ? error.message : String(error);
  document.body.replaceChildren(message);
});
