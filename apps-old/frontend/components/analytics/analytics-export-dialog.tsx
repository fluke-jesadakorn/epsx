'use client';

import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import type { UnifiedAnalyticsRankingsResponse } from '@/lib/api-client';
import type { ExportFormat } from '@/lib/export-utils';
import {
  exportCurrentViewData,
  exportFilteredData,
  exportGrowthLeadersData,
  exportUnifiedAnalyticsData
} from '@/lib/export-utils';
import type { AnalyticsFilters } from '@/types/analytics';
import { Download, FileDown } from 'lucide-react';
import { useState } from 'react';

interface AnalyticsExportDialogProps {
  data: UnifiedAnalyticsRankingsResponse | null;
  isLoading: boolean;
  filters: AnalyticsFilters;
  growthLeaders: any[];
  priceLeaders: any[];
}

export function AnalyticsExportDialog({
  data,
  isLoading,
  filters,
  growthLeaders,
  priceLeaders
}: AnalyticsExportDialogProps) {
  const [showExportDialog, setShowExportDialog] = useState(false);
  const [exportFormat, setExportFormat] = useState<ExportFormat>('json');
  const [exportFilename, setExportFilename] = useState('');
  const [includeMetadata, setIncludeMetadata] = useState(true);
  const [includeQuarterlyData, setIncludeQuarterlyData] = useState(true);
  const [exportType, setExportType] = useState<'current' | 'filtered' | 'leaders' | 'full'>('current');

  const handleExport = () => {
    if (!data) {return;}

    const options = {
      format: exportFormat,
      filename: exportFilename || undefined,
      includeMetadata,
      includeQuarterlyData,
    };

    switch (exportType) {
      case 'current':
        exportCurrentViewData(data.rankings, options);
        break;
      case 'filtered':
        exportFilteredData(data.rankings, filters, options);
        break;
      case 'leaders':
        exportGrowthLeadersData([...growthLeaders, ...priceLeaders], options);
        break;
      case 'full':
        exportUnifiedAnalyticsData(data.rankings, options);
        break;
    }

    setShowExportDialog(false);
  };

  const getExportDescription = () => {
    switch (exportType) {
      case 'current':
        return `Export current page data (${data?.rankings.length ?? 0} records)`;
      case 'filtered':
        return `Export all filtered data (${data?.pagination.total_items ?? 0} total records)`;
      case 'leaders':
        return 'Export Growth performance leaders only';
      case 'full':
        return 'Export complete dataset with metadata';
      default:
        return '';
    }
  };

  return (
    <Dialog open={showExportDialog} onOpenChange={setShowExportDialog}>
      <DialogTrigger asChild>
        <button
          className="flex items-center gap-2 rounded-xl border border-purple-200 bg-white/80 px-4 py-2 text-sm font-semibold text-purple-700 shadow-lg backdrop-blur-sm transition-all duration-300 hover:bg-purple-50 hover:scale-105 dark:border-purple-400/20 dark:bg-slate-800/80 dark:text-purple-400 dark:hover:bg-slate-700/80"
          disabled={!data || isLoading}
        >
          <Download className="h-4 w-4" />
          <span className="hidden sm:inline">Export Data</span>
          <span className="sm:hidden">Export</span>
        </button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileDown className="h-5 w-5 text-purple-600" />
            Export Analytics Data
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-6 py-4">
          <div>
            <Label className="text-sm font-medium">Export Type</Label>
            <Select value={exportType} onValueChange={(value: any) => setExportType(value)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="current">Current Page</SelectItem>
                <SelectItem value="filtered">Filtered Data</SelectItem>
                <SelectItem value="leaders">Growth Leaders</SelectItem>
                <SelectItem value="full">Full Dataset</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-gray-500 mt-1">{getExportDescription()}</p>
          </div>

          <div>
            <Label className="text-sm font-medium">Format</Label>
            <Select value={exportFormat} onValueChange={(value: ExportFormat) => setExportFormat(value)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="json">JSON</SelectItem>
                <SelectItem value="csv">CSV</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <Label className="text-sm font-medium">Filename (optional)</Label>
            <Input
              value={exportFilename}
              onChange={(e) => setExportFilename(e.target.value)}
              placeholder="Leave empty for auto-generated name"
            />
          </div>

          <div className="space-y-3">
            <div className="flex items-center space-x-2">
              <Checkbox
                id="metadata"
                checked={includeMetadata}
                onCheckedChange={(checked) => setIncludeMetadata(Boolean(checked))}
              />
              <Label htmlFor="metadata" className="text-sm">Include metadata</Label>
            </div>
            <div className="flex items-center space-x-2">
              <Checkbox
                id="quarterly"
                checked={includeQuarterlyData}
                onCheckedChange={(checked) => setIncludeQuarterlyData(Boolean(checked))}
              />
              <Label htmlFor="quarterly" className="text-sm">Include quarterly data</Label>
            </div>
          </div>

          <div className="flex justify-end space-x-2">
            <Button variant="outline" onClick={() => setShowExportDialog(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleExport}
              className="bg-gradient-to-r from-purple-500 to-purple-600 text-white"
            >
              <Download className="h-4 w-4 mr-2" />
              Export
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
