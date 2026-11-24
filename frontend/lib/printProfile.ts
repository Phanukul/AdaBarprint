export interface PrinterRoute {printer:string;tray:string;protocol:string}
export interface ProfileMapping {
  normalPrinter:string; normalTray:string; normalProtocol?:string;
  promoPrinter:string; promoTray:string; promoProtocol?:string;
  templateRoutes?:Record<string,PrinterRoute>;
}
// Manual printing resolves this route; Auto only copies it on explicit request. Price status
// describes business data; it does not override an explicitly selected template.
export function resolveTemplateRoute(mapping:ProfileMapping|undefined|null, code:string, legacyKind:string):PrinterRoute {
  const saved=mapping?.templateRoutes?.[code];
  if(saved)return saved;
  const promo=legacyKind==='2';
  return {printer:(promo?mapping?.promoPrinter:mapping?.normalPrinter)||'',tray:(promo?mapping?.promoTray:mapping?.normalTray)||'',protocol:(promo?mapping?.promoProtocol:mapping?.normalProtocol)||'WINDOWS'};
}

export function manualTemplateProblem(layoutJson:string|null|undefined):string {
 try {const layout=JSON.parse(layoutJson||'null');if(!Array.isArray(layout?.elements)||!layout.elements.length)return 'แบบป้ายยังไม่มีองค์ประกอบ';
 const unsupported=layout.elements.filter((e:{type:string})=>!['price','text','static','barcode','qrcode','image','rect','circle','badge','line','icon','date'].includes(e.type));
 return unsupported.length?'แบบป้ายนี้มีองค์ประกอบที่ยังไม่รองรับ: '+[...new Set(unsupported.map((e:{type:string})=>e.type))].join(', '):'';
 }catch{return 'ข้อมูลแบบป้ายไม่ถูกต้อง';}
}

// Store one manual route in both legacy columns so every price type uses it.
export function withManualRoute<T extends ProfileMapping>(mapping:T, route:PrinterRoute):T {
 return {...mapping,normalPrinter:route.printer,normalTray:route.tray,normalProtocol:route.protocol,
   promoPrinter:route.printer,promoTray:route.tray,promoProtocol:route.protocol};
}
