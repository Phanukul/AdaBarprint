const ts=require('typescript');const fs=require('fs');const vm=require('vm');const assert=require('node:assert/strict');
const source=fs.readFileSync('frontend/lib/printProfile.ts','utf8');const context={exports:{}};vm.runInNewContext(ts.transpileModule(source,{compilerOptions:{module:ts.ModuleKind.CommonJS}}).outputText,context);
const {resolveTemplateRoute,manualTemplateProblem}=context.exports;
const mapping={normalPrinter:'Normal',normalTray:'1',promoPrinter:'Promo',promoTray:'2',templateRoutes:{BAR:{printer:'Sticker',tray:'Roll',protocol:'TSPL'}}};
assert.equal(resolveTemplateRoute(mapping,'BAR','1').printer,'Sticker');assert.equal(resolveTemplateRoute(mapping,'BAR','2').printer,'Sticker');
assert.equal(resolveTemplateRoute(mapping,'OLD','2').printer,'Promo');assert.equal(resolveTemplateRoute(mapping,'OLD','1').tray,'1');
assert.equal(resolveTemplateRoute(undefined,'BAR','1').printer,'');
assert.equal(manualTemplateProblem(JSON.stringify({elements:[{type:'barcode'}]})),'');
assert.equal(manualTemplateProblem(JSON.stringify({elements:[{type:'qrcode'}]})),'');
assert.notEqual(manualTemplateProblem(JSON.stringify({elements:[{type:'unknown'}]})),'');
console.log('Print profile checks passed: explicit template wins, legacy fallback preserved, unsupported layouts rejected.');

const unified=context.exports.withManualRoute(mapping,{printer:'One',tray:'3',protocol:'WINDOWS'});
assert.equal(resolveTemplateRoute(unified,'OLD','1').printer,'One');
assert.equal(resolveTemplateRoute(unified,'OLD','2').printer,'One');
assert.equal(resolveTemplateRoute(unified,'OLD','2').tray,'3');
assert.equal(mapping.promoPrinter,'Promo');
assert.equal(unified.templateRoutes,mapping.templateRoutes);
console.log('Single manual route applies to both legacy price types without mutating saved input.');
