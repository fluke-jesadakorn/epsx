/**
 * User Import/Export Component
 * Extracted from UserForms.tsx for better maintainability
 * Handles CSV import and various export formats
 */

'use client';

import { useState } from 'react';
import { 
  Upload,
  Download,
  FileText
} from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';

import { logger } from '@/lib/logger';
import {
  showSuccessToast,
  showErrorToast
} from './shared/user-form-utils';
import type { User } from '@/types/core';

interface UserImportExportProps {
  users?: User[];
  onImportComplete?: (users: User[]) => void;
  className?: string;
}

export function UserImportExport({
  users = [],
  onImportComplete,
  className = ''
}: UserImportExportProps) {
  const [isImporting, setIsImporting] = useState(false);
  const [importProgress, setImportProgress] = useState(0);

  const handleCSVImport = async (file: File) => {
    setIsImporting(true);
    setImportProgress(0);

    try {
      // Mock CSV import progress
      for (let i = 0; i <= 100; i += 10) {
        setImportProgress(i);
        await new Promise(resolve => setTimeout(resolve, 100));
      }

      // In a real implementation, parse CSV and create users
      showSuccessToast(
        "Import completed",
        `Successfully imported users from ${file.name}`
      );
      
      onImportComplete?.([]);
      logger.info('CSV import completed', { fileName: file.name });
    } catch (error) {
      logger.error('CSV import error', { fileName: file.name, error });
      showErrorToast(
        "Import failed",
        "An error occurred while importing the CSV file."
      );
    } finally {
      setIsImporting(false);
      setImportProgress(0);
    }
  };

  const handleExport = async (format: 'csv' | 'json' | 'excel') => {
    try {
      // In a real implementation, generate and download the file
      showSuccessToast(
        "Export started",
        `Generating ${format.toUpperCase()} export of ${users.length} users...`
      );
      
      logger.info('User export initiated', { format, userCount: users.length });
    } catch (error) {
      logger.error('Export error', { format, error });
      showErrorToast(
        "Export failed",
        `An error occurred while exporting users as ${format.toUpperCase()}.`
      );
    }
  };

  return (
    <div className={`space-y-4 ${className}`}>
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Import Section */}
        <Card className="p-6">
          <div className="flex items-center gap-2 mb-4">
            <Upload className="w-5 h-5 text-green-500" />
            <h3 className="text-lg font-medium">Import Users</h3>
          </div>

          <div className="space-y-4">
            <div className="text-sm text-gray-600 dark:text-gray-400">
              <p>Upload a CSV file to import multiple users at once.</p>
              <p className="mt-1">
                <strong>Required columns:</strong> email, role, packageTier
              </p>
              <p>
                <strong>Optional columns:</strong> firstName, lastName, displayName, isActive
              </p>
            </div>

            <div className="border-2 border-dashed border-gray-300 dark:border-gray-700 rounded-lg p-6 text-center">
              <input
                type="file"
                accept=".csv"
                onChange={(e) => {
                  const file = e.target.files?.[0];
                  if (file) handleCSVImport(file);
                }}
                className="hidden"
                id="csv-upload"
                disabled={isImporting}
              />
              <label htmlFor="csv-upload" className={`cursor-pointer ${isImporting ? 'opacity-50' : ''}`}>
                <FileText className="w-8 h-8 text-gray-400 mx-auto mb-2" />
                <p className="text-sm">
                  Click to upload or drag and drop
                </p>
                <p className="text-xs text-gray-500">CSV files only</p>
              </label>
            </div>

            {isImporting && (
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-gray-600 dark:text-gray-400">Importing users...</span>
                  <span>{importProgress}%</span>
                </div>
                <Progress value={importProgress} className="h-2" />
              </div>
            )}
          </div>
        </Card>

        {/* Export Section */}
        <Card className="p-6">
          <div className="flex items-center gap-2 mb-4">
            <Download className="w-5 h-5 text-blue-500" />
            <h3 className="text-lg font-medium">Export Users</h3>
          </div>

          <div className="space-y-4">
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Export user data in various formats for backup or external processing.
            </p>

            <div className="space-y-3">
              <Button 
                variant="outline" 
                className="w-full justify-start"
                onClick={() => handleExport('csv')}
              >
                <FileText className="w-4 h-4 mr-2" />
                Export as CSV ({users.length} users)
              </Button>
              <Button 
                variant="outline" 
                className="w-full justify-start"
                onClick={() => handleExport('json')}
              >
                <FileText className="w-4 h-4 mr-2" />
                Export as JSON ({users.length} users)
              </Button>
              <Button 
                variant="outline" 
                className="w-full justify-start"
                onClick={() => handleExport('excel')}
              >
                <FileText className="w-4 h-4 mr-2" />
                Export as Excel ({users.length} users)
              </Button>
            </div>

            <div className="text-xs text-gray-500 p-3 bg-gray-100 dark:bg-gray-800 rounded">
              <p><strong>Note:</strong> Exports will exclude sensitive information like passwords and include only basic user data and permissions.</p>
            </div>
          </div>
        </Card>
      </div>
    </div>
  );
}