/**
 * Export utilities for analytics data
 * Provides functions to export data in various formats
 */

export type ExportFormat = 'csv' | 'json' | 'xlsx';

export interface ExportOptions {
  format: ExportFormat;
  filename?: string;
  includeHeaders?: boolean;
}

// Basic export implementations
export async function exportUnifiedAnalyticsData(data: any[], options: ExportOptions): Promise<void> {
  const filename = options.filename || `analytics-data.${options.format}`;
  downloadData(data, filename, options.format);
}

export async function exportCurrentViewData(data: any[], options: ExportOptions): Promise<void> {
  const filename = options.filename || `current-view.${options.format}`;
  downloadData(data, filename, options.format);
}

export async function exportGrowthLeadersData(data: any[], options: ExportOptions): Promise<void> {
  const filename = options.filename || `growth-leaders.${options.format}`;
  downloadData(data, filename, options.format);
}

export async function exportFilteredData(data: any[], filters: any, options: ExportOptions): Promise<void> {
  const filename = options.filename || `filtered-data.${options.format}`;
  downloadData(data, filename, options.format);
}

function downloadData(data: any[], filename: string, format: ExportFormat): void {
  let content: string;
  let mimeType: string;
  
  switch (format) {
    case 'json':
      content = JSON.stringify(data, null, 2);
      mimeType = 'application/json';
      break;
    case 'csv':
      content = convertToCSV(data);
      mimeType = 'text/csv';
      break;
    default:
      content = JSON.stringify(data, null, 2);
      mimeType = 'application/json';
  }
  
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}

function convertToCSV(data: any[]): string {
  if (!data.length) {return '';}
  
  const headers = Object.keys(data[0]);
  const csvRows = [headers.join(',')];
  
  for (const row of data) {
    const values = headers.map(header => {
      const escaped = (`${  row[header]}`).replace(/"/g, '\\"');
      return `"${escaped}"`;
    });
    csvRows.push(values.join(','));
  }
  
  return csvRows.join('\n');
}