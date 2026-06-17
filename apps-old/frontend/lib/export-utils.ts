import type { AnalyticsFilters } from '@/types/analytics';

export type ExportFormat = 'json' | 'csv';

interface ExportOptions {
  format: ExportFormat;
  filename?: string;
  includeMetadata?: boolean;
  includeQuarterlyData?: boolean;
}

export function exportCurrentViewData(data: any[], options: ExportOptions): void {
  const filename = options.filename || `analytics-current-${Date.now()}.${options.format}`;
  downloadData(data, filename, options.format);
}

export function exportFilteredData(data: any[], _filters: AnalyticsFilters, options: ExportOptions): void {
  const filename = options.filename || `analytics-filtered-${Date.now()}.${options.format}`;
  downloadData(data, filename, options.format);
}

export function exportGrowthLeadersData(data: any[], options: ExportOptions): void {
  const filename = options.filename || `analytics-leaders-${Date.now()}.${options.format}`;
  downloadData(data, filename, options.format);
}

export function exportUnifiedAnalyticsData(data: any[], options: ExportOptions): void {
  const filename = options.filename || `analytics-full-${Date.now()}.${options.format}`;
  downloadData(data, filename, options.format);
}

function downloadData(data: any[], filename: string, format: ExportFormat): void {
  let content: string;
  let mimeType: string;

  if (format === 'json') {
    content = JSON.stringify(data, null, 2);
    mimeType = 'application/json';
  } else {
    content = convertToCSV(data);
    mimeType = 'text/csv';
  }

  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

function convertToCSV(data: any[]): string {
  if (data.length === 0) {return '';}

  const headers = Object.keys(data[0]);
  const csvRows = [
    headers.join(','),
    ...data.map(row =>
      headers.map(header => {
        const value = row[header];
        const stringValue = value === null || value === undefined ? '' : String(value);
        return stringValue.includes(',') ? `"${stringValue}"` : stringValue;
      }).join(',')
    )
  ];

  return csvRows.join('\n');
}
