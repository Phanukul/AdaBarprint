/** Accept a barcode/quantity list, or the named columns from our CSV export. */
export function parseBarcodeImport(source: string): Array<{ barcode: string; quantity: number }> {
  const text = source.replace(/^\uFEFF/, '');
  const firstLine = text.split(/\r?\n/, 1)[0];
  const delimiter = ['\t', ';', ','].sort((a,b) => firstLine.split(b).length-firstLine.split(a).length)[0];
  const rows: string[][] = [];
  let row: string[] = [], cell = '', quoted = false;
  for (let i=0;i<text.length;i++) {
    const char=text[i];
    if (char==='"') {
      if (quoted && text[i+1]==='"') {cell+='"';i++;}
      else quoted=!quoted;
    } else if (!quoted && (char===delimiter || char==='\n' || char==='\r')) {
      row.push(cell.trim());cell='';
      if(char!==delimiter) {
        if(row.some(Boolean))rows.push(row);
        row=[];
        if(char==='\r'&&text[i+1]==='\n')i++;
      }
    } else cell+=char;
  }
  if(quoted)throw new Error('เครื่องหมายคำพูดในไฟล์ CSV ไม่ครบคู่');
  row.push(cell.trim());if(row.some(Boolean))rows.push(row);
  if(!rows.length)throw new Error('ไม่พบข้อมูลในไฟล์');
  const headers=rows[0].map(x=>x.toLowerCase());
  const barcodeIndex=headers.findIndex(x=>['barcode','ftbarcode','บาร์โค้ด'].includes(x));
  const quantityIndex=headers.findIndex(x=>['qty','quantity','fnplbqty','จำนวน'].includes(x));
  const data=barcodeIndex>=0?rows.slice(1):rows;
  if(data.length>5000)throw new Error('นำเข้าได้ไม่เกิน 5,000 รายการต่อครั้ง');
  return data.map((cols,index)=>{
    const barcode=cols[barcodeIndex>=0?barcodeIndex:0]||'';
    const raw=cols[barcodeIndex>=0?quantityIndex:1];
    const quantity=raw===undefined||raw===''?1:Number(raw);
    if(!barcode||!Number.isInteger(quantity)||quantity<1||quantity>10000)throw new Error(`ข้อมูลแถวที่ ${index+(barcodeIndex>=0?2:1)} ไม่ถูกต้อง: ระบุบาร์โค้ดและจำนวนเต็ม 1–10,000`);
    return {barcode,quantity};
  });
}
