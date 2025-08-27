import { UnifiedAnalyticsRankingsResponse, UnifiedRankingItem, CardDashboardResponse, SymbolCardData } from './api-client';

export type ExportFormat = 'json' | 'csv';

export interface ExportOptions {
  format: ExportFormat;
  filename?: string;
  includeMetadata?: boolean;
  includeQuarterlyData?: boolean;
}

// Convert analytics data to CSV format
function convertToCsv(data: any[]): string {
  if (data.length === 0) return '';

  const headers = Object.keys(data[0]);
  const csvContent = [
    headers.join(','),
    ...data.map(row => 
      headers.map(header => {
        const value = row[header];
        if (value === null || value === undefined) return '';
        if (typeof value === 'string' && (value.includes(',') || value.includes('"'))) {
          return `"${value.replace(/"/g, '""')}"`;
        }
        return String(value);
      }).join(',')
    )
  ].join('\n');

  return csvContent;
}

// Flatten nested objects for CSV export
function flattenObject(obj: any, prefix = ''): any {
  const flattened: any = {};
  
  for (const key in obj) {
    if (obj.hasOwnProperty(key)) {
      const newKey = prefix ? `${prefix}_${key}` : key;
      
      if (obj[key] !== null && typeof obj[key] === 'object' && !Array.isArray(obj[key])) {
        Object.assign(flattened, flattenObject(obj[key], newKey));
      } else if (Array.isArray(obj[key])) {
        // For arrays, just take the length or stringify
        flattened[`${newKey}_count`] = obj[key].length;
        if (obj[key].length > 0 && typeof obj[key][0] === 'object') {
          // For array of objects, include first item details
          Object.assign(flattened, flattenObject(obj[key][0], `${newKey}_first`));
        }
      } else {
        flattened[newKey] = obj[key];
      }
    }
  }
  
  return flattened;
}

// Export unified analytics rankings data
export function exportUnifiedAnalyticsData(
  data: UnifiedAnalyticsRankingsResponse,
  options: ExportOptions
): void {
  const timestamp = new Date().toISOString().split('T')[0];
  const defaultFilename = `eps-analytics-${timestamp}`;
  const filename = options.filename || defaultFilename;

  let exportData: any = {
    rankings: data.data,
    pagination: data.pagination,
  };

  if (options.includeMetadata) {
    exportData.metadata = data.metadata;
    exportData.processing_time_ms = data.processing_time_ms;
    exportData.success = data.success;
    exportData.exported_at = new Date().toISOString();
  }

  if (options.format === 'json') {
    const jsonContent = JSON.stringify(exportData, null, 2);
    downloadFile(jsonContent, `${filename}.json`, 'application/json');
  } else if (options.format === 'csv') {
    // Flatten rankings data for CSV
    const flattenedRankings = data.data.map(ranking => {
      const flattened = flattenObject(ranking);
      
      // Remove quarterly data if not requested
      if (!options.includeQuarterlyData) {
        Object.keys(flattened).forEach(key => {
          if (key.startsWith('quarterly_data_')) {
            delete flattened[key];
          }
        });
      }
      
      return flattened;
    });

    const csvContent = convertToCsv(flattenedRankings);
    downloadFile(csvContent, `${filename}.csv`, 'text/csv');
  }
}

// Export card dashboard data
export function exportCardDashboardData(
  data: CardDashboardResponse,
  options: ExportOptions
): void {
  const timestamp = new Date().toISOString().split('T')[0];
  const defaultFilename = `card-dashboard-${timestamp}`;
  const filename = options.filename || defaultFilename;

  let exportData: any = {
    cards: data.data,
    pagination: data.pagination,
  };

  if (options.includeMetadata) {
    exportData.metadata = data.metadata;
    exportData.processing_time_ms = data.processing_time_ms;
    exportData.success = data.success;
    exportData.exported_at = new Date().toISOString();
  }

  if (options.format === 'json') {
    const jsonContent = JSON.stringify(exportData, null, 2);
    downloadFile(jsonContent, `${filename}.json`, 'application/json');
  } else if (options.format === 'csv') {
    // Flatten card data for CSV
    const flattenedCards = data.data.map(card => {
      const flattened = flattenObject(card);
      
      // Remove quarterly performance if not requested
      if (!options.includeQuarterlyData) {
        Object.keys(flattened).forEach(key => {
          if (key.startsWith('quarterly_performance_')) {
            delete flattened[key];
          }
        });
      }
      
      return flattened;
    });

    const csvContent = convertToCsv(flattenedCards);
    downloadFile(csvContent, `${filename}.csv`, 'text/csv');
  }
}

// Export current view rankings (for real-time data)
export function exportCurrentViewData(
  rankings: UnifiedRankingItem[],
  options: ExportOptions & { viewType?: 'list' | 'card' }
): void {
  const timestamp = new Date().toISOString().split('T')[0];
  const viewType = options.viewType || 'list';
  const defaultFilename = `${viewType}-view-export-${timestamp}`;
  const filename = options.filename || defaultFilename;

  let exportData: any = {
    rankings,
    exported_at: new Date().toISOString(),
    view_type: viewType,
    total_records: rankings.length,
  };

  if (options.format === 'json') {
    const jsonContent = JSON.stringify(exportData, null, 2);
    downloadFile(jsonContent, `${filename}.json`, 'application/json');
  } else if (options.format === 'csv') {
    // Flatten rankings for CSV
    const flattenedRankings = rankings.map(ranking => {
      const flattened = flattenObject(ranking);
      
      // Remove quarterly data if not requested
      if (!options.includeQuarterlyData) {
        Object.keys(flattened).forEach(key => {
          if (key.startsWith('quarterly_data_')) {
            delete flattened[key];
          }
        });
      }
      
      return flattened;
    });

    const csvContent = convertToCsv(flattenedRankings);
    downloadFile(csvContent, `${filename}.csv`, 'text/csv');
  }
}

// Export filtered data with applied filters
export function exportFilteredData(
  data: UnifiedAnalyticsRankingsResponse,
  appliedFilters: any,
  options: ExportOptions
): void {
  const timestamp = new Date().toISOString().split('T')[0];
  const defaultFilename = `filtered-analytics-${timestamp}`;
  const filename = options.filename || defaultFilename;

  let exportData: any = {
    rankings: data.data,
    pagination: data.pagination,
    applied_filters: appliedFilters,
    exported_at: new Date().toISOString(),
  };

  if (options.includeMetadata) {
    exportData.metadata = data.metadata;
    exportData.processing_time_ms = data.processing_time_ms;
    exportData.success = data.success;
  }

  if (options.format === 'json') {
    const jsonContent = JSON.stringify(exportData, null, 2);
    downloadFile(jsonContent, `${filename}.json`, 'application/json');
  } else if (options.format === 'csv') {
    // Add filter information as comments in CSV
    let csvContent = `# Exported on: ${new Date().toISOString()}\n`;
    csvContent += `# Applied filters: ${JSON.stringify(appliedFilters)}\n`;
    csvContent += `# Total records: ${data.data.length}\n\n`;

    // Flatten rankings for CSV
    const flattenedRankings = data.data.map(ranking => {
      const flattened = flattenObject(ranking);
      
      if (!options.includeQuarterlyData) {
        Object.keys(flattened).forEach(key => {
          if (key.startsWith('quarterly_data_')) {
            delete flattened[key];
          }
        });
      }
      
      return flattened;
    });

    csvContent += convertToCsv(flattenedRankings);
    downloadFile(csvContent, `${filename}.csv`, 'text/csv');
  }
}

// Export Growth leaders data
export function exportGrowthLeadersData(
  epsLeaders: UnifiedRankingItem[],
  priceLeaders: UnifiedRankingItem[],
  options: ExportOptions
): void {
  const timestamp = new Date().toISOString().split('T')[0];
  const defaultFilename = `growth-leaders-${timestamp}`;
  const filename = options.filename || defaultFilename;

  const exportData = {
    eps_leaders: epsLeaders,
    price_leaders: priceLeaders,
    exported_at: new Date().toISOString(),
    summary: {
      total_eps_leaders: epsLeaders.length,
      total_price_leaders: priceLeaders.length,
      avg_eps_growth: epsLeaders.reduce((sum, item) => sum + item.analytics.growth_factor, 0) / epsLeaders.length,
      avg_price_growth: priceLeaders.reduce((sum, item) => {
        const latestGrowth = item.quarterly_data?.[0]?.price_growth || 0;
        const previousGrowth = item.quarterly_data?.[1]?.price_growth || 0;
        return sum + (latestGrowth === 0 ? previousGrowth : latestGrowth);
      }, 0) / priceLeaders.length,
    },
  };

  if (options.format === 'json') {
    const jsonContent = JSON.stringify(exportData, null, 2);
    downloadFile(jsonContent, `${filename}.json`, 'application/json');
  } else if (options.format === 'csv') {
    // Create separate sheets for leaders
    let csvContent = `# Growth Leaders Export - ${new Date().toISOString()}\n\n`;
    
    csvContent += '# EPS LEADERS\n';
    const flattenedEpsLeaders = epsLeaders.map(leader => flattenObject(leader));
    csvContent += convertToCsv(flattenedEpsLeaders);
    
    csvContent += '\n\n# PRICE LEADERS\n';
    const flattenedPriceLeaders = priceLeaders.map(leader => flattenObject(leader));
    csvContent += convertToCsv(flattenedPriceLeaders);

    downloadFile(csvContent, `${filename}.csv`, 'text/csv');
  }
}

// Utility function to trigger file download
function downloadFile(content: string, filename: string, mimeType: string): void {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  link.style.display = 'none';
  
  document.body.appendChild(link);
  link.click();
  
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

// Export templates and schemas
export function exportDataTemplate(format: ExportFormat): void {
  const template = {
    rankings: [
      {
        symbol: "EXAMPLE",
        company_name: "Example Company Inc",
        ranking_position: 1,
        current_price: 150.00,
        current_price_date: "2025-01-15T12:00:00Z",
        quarterly_data: [
          {
            quarter: "Q4 '24",
            date: "2024-12-31T00:00:00Z",
            price: 145.50,
            eps: 2.50,
            eps_growth: 15.5,
            price_growth: 8.2,
            volume: 1500000
          }
        ],
        market_data: {
          market_cap: 2500000000,
          volume_24h: 50000000,
          country: "america",
          sector: "Technology",
          exchange: "NASDAQ"
        },
        analytics: {
          growth_factor: 15.5,
          ranking_score: 95.8,
          trend: "bullish",
          volatility: 0.25
        }
      }
    ],
    metadata: {
      available_countries: ["america", "canada"],
      available_sectors: ["Technology", "Healthcare"],
      current_filters: {
        country: "america",
        sector: "Technology",
        sort_by: "growth_factor"
      },
      request_timestamp: "2025-01-15T12:00:00Z",
      data_source: "Diesel-EPS-Engine",
      enhanced_with_websocket: false
    }
  };

  const timestamp = new Date().toISOString().split('T')[0];
  const filename = `eps-analytics-template-${timestamp}`;

  if (format === 'json') {
    const jsonContent = JSON.stringify(template, null, 2);
    downloadFile(jsonContent, `${filename}.json`, 'application/json');
  } else if (format === 'csv') {
    const flattenedTemplate = template.rankings.map(ranking => flattenObject(ranking));
    const csvContent = convertToCsv(flattenedTemplate);
    downloadFile(csvContent, `${filename}.csv`, 'text/csv');
  }
}