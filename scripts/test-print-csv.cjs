const ts = require('typescript');
const fs = require('node:fs');
const vm = require('node:vm');
const assert = require('node:assert/strict');
const context = { exports: {} };
vm.runInNewContext(ts.transpileModule(fs.readFileSync('frontend/lib/printCsv.ts', 'utf8'), {
  compilerOptions: { module: ts.ModuleKind.CommonJS },
}).outputText, context);
const parse = (text) => JSON.parse(JSON.stringify(context.exports.parseBarcodeImport(text)));
assert.deepEqual(parse('\uFEFFpdtCode,pdtName,barcode,price,priType,qty\r\nP1,"กาแฟ, \"\"พิเศษ\"\"\nแพ็ก",0000123,39,1,2'), [{ barcode: '0000123', quantity: 2 }]);
assert.deepEqual(parse('บาร์โค้ด\tจำนวน\r\n00001\t3'), [{ barcode: '00001', quantity: 3 }]);
assert.deepEqual(parse('barcode\n00001'), [{ barcode: '00001', quantity: 1 }]);
for (const invalid of ['00001,0', '00001,1.5', '00001,10001', '"00001,2']) {
  assert.throws(() => parse(invalid));
}
console.log('CSV checks passed: Thai export columns, quoted multiline names, leading zeroes, default quantity and invalid-input rejection.');
