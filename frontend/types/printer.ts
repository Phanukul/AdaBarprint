export type PrinterBrand = 'ZEBRA' | 'HONEYWELL' | 'TSC' | 'ARGOX' | 'GODEX' | 'BIXOLON' | 'SAMSUNG' | 'WINDOWS';

export type PrinterLanguage = 'ZPL' | 'DP' | 'TSPL' | 'PPLA' | 'EZPL' | 'BXL' | 'A4';

export interface PrinterConfig {
  id: string;
  name: string;
  brand: PrinterBrand;
  model: string;
  ip: string;
  port: number;
  dpi: number;
  labelWidth: number;
  labelHeight: number;
  speed: number;
  darkness: number;
  enabled: boolean;
  tray?: string;
  availableTrays?: string[];
}
