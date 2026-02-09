/**
 * Data Processing and Export Utilities
 * Utilities for data export, analytics processing, and caching
 */

import { safeError } from '../logging';

// ============================================================================
// Export Types
// ============================================================================

export interface ExportOptions {
  format: 'csv' | 'json' | 'xlsx';
  filename?: string;
  includeHeaders?: boolean;
  dateFormat?: string;
  delimiter?: string;
}

export interface ExportResult {
  success: boolean;
  filename: string;
  size: number;
  downloadUrl?: string;
  error?: string;
}

// ============================================================================
// CSV Export Utilities
// ============================================================================

export function arrayToCSV(data: Record<string, unknown>[], options: Partial<ExportOptions> = {}): string {
  if (!Array.isArray(data) || data.length === 0) {
    return '';
  }

  const { includeHeaders = true, delimiter = ',' } = options;
  const headers = Object.keys(data[0]);
  const csvRows: string[] = [];

  // Add headers
  if (includeHeaders) {
    csvRows.push(headers.map(header => escapeCSVField(header)).join(delimiter));
  }

  // Add data rows
  for (const row of data) {
    const values = headers.map(header => {
      const value = row[header];
      return escapeCSVField(formatValue(value));
    });
    csvRows.push(values.join(delimiter));
  }

  return csvRows.join('\n');
}

function escapeCSVField(field: string): string {
  const stringField = String(field);

  // If field contains comma, newline, or quote, wrap in quotes and escape internal quotes
  if (stringField.includes(',') || stringField.includes('\n') || stringField.includes('"')) {
    return `"${stringField.replace(/"/g, '""')}"`;
  }

  return stringField;
}

function formatValue(value: unknown): string {
  if (value === null || value === undefined) {return '';}

  if (value instanceof Date) {
    return value.toISOString();
  }

  if (typeof value === 'object') {
    return JSON.stringify(value);
  }

  return String(value);
}

// ============================================================================
// JSON Export Utilities
// ============================================================================

export function exportToJSON(data: unknown, _options: Partial<ExportOptions> = {}): string {
  try {
    return JSON.stringify(data, null, 2);
  } catch (_error) {
    throw new Error(`JSON export failed: ${safeError(_error).message}`);
  }
}

// ============================================================================
// File Download Utilities
// ============================================================================

export function downloadBlob(blob: Blob, filename: string): void {
  if (typeof window === 'undefined') {
    throw new Error('File download is only available in browser environment');
  }

  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;

  // Trigger download
  document.body.appendChild(link);
  link.click();

  // Cleanup
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

export function downloadData(data: string, filename: string, mimeType = 'text/plain'): void {
  const blob = new Blob([data], { type: mimeType });
  downloadBlob(blob, filename);
}

export function downloadCSV(data: Record<string, unknown>[], filename = 'export.csv', options: Partial<ExportOptions> = {}): void {
  const csv = arrayToCSV(data, options);
  downloadData(csv, filename, 'text/csv');
}

export function downloadJSON(data: unknown, filename = 'export.json'): void {
  const json = exportToJSON(data);
  downloadData(json, filename, 'application/json');
}

// ============================================================================
// Analytics Data Processing
// ============================================================================

export interface AnalyticsDataPoint {
  timestamp: string;
  value: number;
  label?: string;
  metadata?: Record<string, unknown>;
}

export function processAnalyticsData(rawData: Record<string, unknown>[]): AnalyticsDataPoint[] {
  return rawData.map(item => {
    const timestamp = typeof item.timestamp === 'string' ? item.timestamp : new Date().toISOString();
    const value = typeof item.value === 'string' ? parseFloat(item.value) || 0 : 0;
    const label = (typeof item.label === 'string' ? item.label : '') || (typeof item.name === 'string' ? item.name : '');
    const metadata = typeof item.metadata === 'object' && item.metadata !== null ? item.metadata as Record<string, unknown> : {};

    return { timestamp, value, label, metadata };
  });
}

export function aggregateByPeriod(
  data: AnalyticsDataPoint[],
  period: 'hour' | 'day' | 'week' | 'month'
): AnalyticsDataPoint[] {
  const grouped = new Map<string, AnalyticsDataPoint[]>();

  for (const item of data) {
    const date = new Date(item.timestamp);
    let key: string;

    switch (period) {
      case 'hour':
        key = `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}-${date.getHours()}`;
        break;
      case 'day':
        key = `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}`;
        break;
      case 'week': {
        const weekStart = new Date(date);
        weekStart.setDate(date.getDate() - date.getDay());
        key = `${weekStart.getFullYear()}-${weekStart.getMonth()}-${weekStart.getDate()}`;
        break;
      }
      case 'month':
        key = `${date.getFullYear()}-${date.getMonth()}`;
        break;
    }

    if (!grouped.has(key)) {
      grouped.set(key, []);
    }
    const group = grouped.get(key);
    if (group !== undefined) {
      group.push(item);
    }
  }

  return Array.from(grouped.entries()).map(([key, items]) => ({
    timestamp: items[0].timestamp,
    value: items.reduce((sum, item) => sum + item.value, 0),
    label: `${period}_${key}`,
    metadata: {
      count: items.length,
      period,
      originalKey: key
    }
  }));
}

// ============================================================================
// Cache Utilities
// ============================================================================

export interface CacheOptions {
  ttl?: number; // Time to live in milliseconds
  maxSize?: number; // Maximum number of entries
  serialize?: boolean; // Whether to serialize complex objects
}

export class SimpleCache<T> {
  private cache = new Map<string, { value: T; expires: number; accessed: number }>();
  private options: Required<CacheOptions>;

  constructor(options: CacheOptions = {}) {
    this.options = {
      ttl: options.ttl ?? 5 * 60 * 1000, // 5 minutes default
      maxSize: options.maxSize ?? 1000,
      serialize: options.serialize ?? false
    };
  }

  set(key: string, value: T): void {
    // Remove expired entries if cache is full
    if (this.cache.size >= this.options.maxSize) {
      this.cleanup();
    }

    // If still full after cleanup, remove least recently accessed
    if (this.cache.size >= this.options.maxSize) {
      const entries = Array.from(this.cache.entries())
        .sort((a, b) => a[1].accessed - b[1].accessed);

      if (entries.length > 0) {
        this.cache.delete(entries[0][0]);
      }
    }

    const processedValue: T = this.options.serialize
      ? (JSON.parse(JSON.stringify(value)) as T)
      : value;

    this.cache.set(key, {
      value: processedValue,
      expires: Date.now() + this.options.ttl,
      accessed: Date.now()
    });
  }

  get(key: string): T | null {
    const entry = this.cache.get(key);

    if (!entry) {
      return null;
    }

    if (Date.now() > entry.expires) {
      this.cache.delete(key);
      return null;
    }

    // Update access time
    entry.accessed = Date.now();

    return entry.value;
  }

  has(key: string): boolean {
    return this.get(key) !== null;
  }

  delete(key: string): boolean {
    return this.cache.delete(key);
  }

  clear(): void {
    this.cache.clear();
  }

  size(): number {
    this.cleanup();
    return this.cache.size;
  }

  keys(): string[] {
    this.cleanup();
    return Array.from(this.cache.keys());
  }

  private cleanup(): void {
    const now = Date.now();

    for (const [key, entry] of this.cache.entries()) {
      if (now > entry.expires) {
        this.cache.delete(key);
      }
    }
  }

  getStats(): { size: number; hitRate: number; avgAge: number } {
    this.cleanup();

    const now = Date.now();
    let totalAge = 0;

    for (const entry of this.cache.values()) {
      totalAge += (now - (entry.expires - this.options.ttl));
    }

    return {
      size: this.cache.size,
      hitRate: 0, // Would need to track hits/misses
      avgAge: this.cache.size > 0 ? totalAge / this.cache.size : 0
    };
  }
}

// ============================================================================
// Data Validation
// ============================================================================

export function validateEmail(email: string): boolean {
  const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
  return emailRegex.test(email);
}

export function validateUrl(url: string): boolean {
  try {
    new URL(url);
    return true;
  } catch {
    return false;
  }
}

export function validateJSON(jsonString: string): boolean {
  try {
    JSON.parse(jsonString);
    return true;
  } catch {
    return false;
  }
}

export function sanitizeFilename(filename: string): string {
  return filename
    .replace(/[^\w\s\-.]/g, '') // Remove special characters except dash, dot, underscore
    .replace(/\s+/g, '_') // Replace spaces with underscores
    .toLowerCase();
}

// ============================================================================
// Number Formatting
// ============================================================================

export function formatCurrency(amount: number, currency = 'USD'): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency
  }).format(amount);
}

export function formatPercent(value: number, decimals = 2): string {
  return new Intl.NumberFormat('en-US', {
    style: 'percent',
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals
  }).format(value / 100);
}

export function formatNumber(value: number, decimals = 0): string {
  return new Intl.NumberFormat('en-US', {
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals
  }).format(value);
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) {return '0 Bytes';}

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))  } ${  sizes[i]}`;
}

// ============================================================================
// Exports
// ============================================================================

export const dataCache = new SimpleCache();

// Service Worker utilities
export function registerServiceWorker(swUrl = '/sw.js'): Promise<ServiceWorkerRegistration> {
  if (typeof window === 'undefined' || !('serviceWorker' in navigator)) {
    return Promise.reject(new Error('Service Worker not supported'));
  }

  return navigator.serviceWorker.register(swUrl);
}

export async function unregisterServiceWorker(): Promise<boolean> {
  if (typeof window === 'undefined' || !('serviceWorker' in navigator)) {
    return false;
  }

  try {
    const registration = await navigator.serviceWorker.ready;
    return await registration.unregister();
  } catch {
    return false;
  }
}