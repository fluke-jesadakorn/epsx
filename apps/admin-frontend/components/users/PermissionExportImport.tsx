'use client';

import { useState, useRef } from 'react';
import { 
  Download, 
  Upload, 
  FileText, 
  Copy, 
  Check, 
  AlertTriangle, 
  Users,
  Settings,
  Shield,
  Database,
  RefreshCw,
  CheckCircle,
  XCircle,
  History,
  FileSpreadsheet,
  FileJson,
  Archive,
  Eye,
  Play,
  Pause
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Checkbox } from '@/components/ui/form-components';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogFooter,
} from '@/components/ui/dialog';
import { useToast } from '@/components/ui/use-toast';
import { format } from 'date-fns';

import type { UnifiedUserData } from '@/lib/types/unified-user';
import {
  exportUserPermissions,
  bulkExportUserPermissions,
  validatePermissionImport,
  importUserPermissions,
  generatePermissionAuditReport,
  createSystemPermissionBackup,
  exportPermissionTemplates,
  importPermissionTemplates,
  PermissionExportData,
  PermissionImportData,
  ImportValidationResult,
} from '@/lib/actions/permission-export-import-actions';

interface PermissionExportImportProps {
  user: UnifiedUserData;
  onPermissionsUpdated?: () => void;
  className?: string;
}

export function PermissionExportImport({ 
  user, 
  onPermissionsUpdated, 
  className = '' 
}: PermissionExportImportProps) {
  const { toast } = useToast();
  const fileInputRef = useRef<HTMLInputElement>(null);
  
  // State
  const [activeTab, setActiveTab] = useState<'export' | 'import' | 'bulk' | 'templates' | 'backup'>('export');
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [copied, setCopied] = useState(false);
  
  // Export State
  const [exportFormat, setExportFormat] = useState<'json' | 'csv' | 'xlsx'>('json');
  const [exportOptions, setExportOptions] = useState({
    includeHistory: false,
    includeTemporary: true,
  });
  const [exportData, setExportData] = useState<PermissionExportData | null>(null);
  
  // Import State
  const [importData, setImportData] = useState('');
  const [importFile, setImportFile] = useState<File | null>(null);
  const [validationResult, setValidationResult] = useState<ImportValidationResult | null>(null);
  const [importOptions, setImportOptions] = useState({
    includeRoles: true,
    includeCustomPermissions: true,
    includeProfiles: true,
    includeTemporary: false,
    replaceExisting: false,
    dryRun: false,
  });
  
  // Bulk Export State
  const [bulkUserIds, setBulkUserIds] = useState<string[]>([]);
  const [bulkUserInput, setBulkUserInput] = useState('');
  const [bulkFormat, setBulkFormat] = useState<'json' | 'csv' | 'xlsx'>('json');
  const [bulkOptions, setBulkOptions] = useState({
    includeHistory: false,
    includeTemporary: true,
    groupBy: 'user' as 'user' | 'role' | 'profile',
  });
  
  // Template State
  const [templateNames, setTemplateNames] = useState<string[]>([]);
  const [templateNameInput, setTemplateNameInput] = useState('');
  
  // Backup State
  const [backupOptions, setBackupOptions] = useState({
    format: 'json' as 'json' | 'sql',
    includeHistory: true,
    includeTemporary: true,
    compression: 'gzip' as 'none' | 'gzip' | 'zip',
  });

  // Export Functions
  const handleSingleExport = async () => {
    try {
      setLoading(true);
      setProgress(0);

      const progressInterval = setInterval(() => {
        setProgress(prev => Math.min(prev + 10, 90));
      }, 200);

      const result = await exportUserPermissions(user.id, exportFormat, exportOptions);
      
      clearInterval(progressInterval);
      setProgress(100);

      if (result.success && result.data) {
        setExportData(result.data);
        
        if (exportFormat === 'json') {
          // For JSON, show in UI for copy/download
          setExportData(result.data);
        } else {
          // For CSV/Excel, trigger download
          const filename = `permissions-${user.email}-${format(new Date(), 'yyyy-MM-dd')}.${exportFormat}`;
          // The API would return a download URL or blob
          toast({
            title: 'Export Complete',
            description: `Permissions exported to ${filename}`,
          });
        }
      } else {
        throw new Error(result.error?.message || 'Export failed');
      }
    } catch (error) {
      console.error('Export failed:', error);
      toast({
        title: 'Export Failed',
        description: error instanceof Error ? error.message : 'An unexpected error occurred',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
      setProgress(0);
    }
  };

  const handleBulkExport = async () => {
    if (bulkUserIds.length === 0) {
      toast({
        title: 'Validation Error',
        description: 'Please add at least one user ID',
        variant: 'destructive',
      });
      return;
    }

    try {
      setLoading(true);
      setProgress(0);

      const progressInterval = setInterval(() => {
        setProgress(prev => Math.min(prev + 5, 90));
      }, 500);

      const result = await bulkExportUserPermissions({
        userIds: bulkUserIds,
        format: bulkFormat,
        includeHistory: bulkOptions.includeHistory,
        includeTemporary: bulkOptions.includeTemporary,
        groupBy: bulkOptions.groupBy,
      });

      clearInterval(progressInterval);
      setProgress(100);

      if (result.success && result.data) {
        toast({
          title: 'Bulk Export Complete',
          description: `Exported permissions for ${result.data.summary.totalUsers} users`,
        });

        // Open download URL
        window.open(result.data.downloadUrl, '_blank');
      } else {
        throw new Error(result.error?.message || 'Bulk export failed');
      }
    } catch (error) {
      console.error('Bulk export failed:', error);
      toast({
        title: 'Bulk Export Failed',
        description: error instanceof Error ? error.message : 'An unexpected error occurred',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
      setProgress(0);
    }
  };

  // Import Functions
  const handleFileUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setImportFile(file);
      const reader = new FileReader();
      reader.onload = (e) => {
        const content = e.target?.result as string;
        setImportData(content);
        handleValidateImport(content);
      };
      reader.readAsText(file);
    }
  };

  const handleValidateImport = async (data?: string) => {
    const importContent = data || importData;
    if (!importContent.trim()) return;

    try {
      const parsedData = JSON.parse(importContent) as PermissionExportData;
      
      const result = await validatePermissionImport(user.id, parsedData, {
        includeRoles: importOptions.includeRoles,
        includeCustomPermissions: importOptions.includeCustomPermissions,
        includeProfiles: importOptions.includeProfiles,
        includeTemporary: importOptions.includeTemporary,
      });

      if (result.success && result.data) {
        setValidationResult(result.data);
      } else {
        setValidationResult({
          isValid: false,
          errors: [result.error?.message || 'Validation failed'],
          warnings: [],
          preview: {
            rolesToAdd: 0,
            rolesToRemove: 0,
            permissionsToAdd: 0,
            permissionsToRemove: 0,
            profilesToAdd: 0,
            profilesToRemove: 0,
          },
        });
      }
    } catch (error) {
      setValidationResult({
        isValid: false,
        errors: [`Parse error: ${error instanceof Error ? error.message : 'Invalid JSON'}`],
        warnings: [],
        preview: {
          rolesToAdd: 0,
          rolesToRemove: 0,
          permissionsToAdd: 0,
          permissionsToRemove: 0,
          profilesToAdd: 0,
          profilesToRemove: 0,
        },
      });
    }
  };

  const handleImport = async () => {
    if (!importData.trim() || !validationResult?.isValid) return;

    try {
      setLoading(true);
      setProgress(0);

      const progressInterval = setInterval(() => {
        setProgress(prev => Math.min(prev + 15, 90));
      }, 300);

      const parsedData = JSON.parse(importData) as PermissionExportData;
      
      const result = await importUserPermissions({
        userId: user.id,
        importData: parsedData,
        replaceExisting: importOptions.replaceExisting,
        importOptions,
      });

      clearInterval(progressInterval);
      setProgress(100);

      if (result.success && result.data) {
        toast({
          title: 'Import Complete',
          description: `Successfully imported permissions (${result.data.summary.rolesAdded} roles, ${result.data.summary.permissionsAdded} permissions, ${result.data.summary.profilesAdded} profiles)`,
        });

        setImportData('');
        setImportFile(null);
        setValidationResult(null);
        onPermissionsUpdated?.();
      } else {
        throw new Error(result.error?.message || 'Import failed');
      }
    } catch (error) {
      console.error('Import failed:', error);
      toast({
        title: 'Import Failed',
        description: error instanceof Error ? error.message : 'An unexpected error occurred',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
      setProgress(0);
    }
  };

  // Utility Functions
  const handleCopyExport = async () => {
    if (!exportData) return;

    try {
      await navigator.clipboard.writeText(JSON.stringify(exportData, null, 2));
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
      toast({
        title: 'Copied to Clipboard',
        description: 'Permission data copied to clipboard',
      });
    } catch (error) {
      toast({
        title: 'Copy Failed',
        description: 'Failed to copy to clipboard',
        variant: 'destructive',
      });
    }
  };

  const handleDownloadExport = () => {
    if (!exportData) return;

    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `permissions-${user.email}-${format(new Date(), 'yyyy-MM-dd')}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    toast({
      title: 'Export Downloaded',
      description: 'Permission data downloaded as JSON file',
    });
  };

  const addBulkUserId = () => {
    if (bulkUserInput.trim() && !bulkUserIds.includes(bulkUserInput.trim())) {
      setBulkUserIds([...bulkUserIds, bulkUserInput.trim()]);
      setBulkUserInput('');
    }
  };

  const removeBulkUserId = (userId: string) => {
    setBulkUserIds(bulkUserIds.filter(id => id !== userId));
  };

  const getFormatIcon = (format: string) => {
    switch (format) {
      case 'json': return <FileJson className="h-4 w-4" />;
      case 'csv': return <FileSpreadsheet className="h-4 w-4" />;
      case 'xlsx': return <FileSpreadsheet className="h-4 w-4" />;
      default: return <FileText className="h-4 w-4" />;
    }
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Progress Bar */}
      {loading && (
        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span>Processing...</span>
            <span>{progress}%</span>
          </div>
          <Progress value={progress} className="w-full" />
        </div>
      )}

      <Tabs value={activeTab} onValueChange={(value: any) => setActiveTab(value)}>
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="export" className="flex items-center gap-1">
            <Download className="h-3 w-3" />
            Export
          </TabsTrigger>
          <TabsTrigger value="import" className="flex items-center gap-1">
            <Upload className="h-3 w-3" />
            Import
          </TabsTrigger>
          <TabsTrigger value="bulk" className="flex items-center gap-1">
            <Users className="h-3 w-3" />
            Bulk
          </TabsTrigger>
          <TabsTrigger value="templates" className="flex items-center gap-1">
            <Settings className="h-3 w-3" />
            Templates
          </TabsTrigger>
          <TabsTrigger value="backup" className="flex items-center gap-1">
            <Archive className="h-3 w-3" />
            Backup
          </TabsTrigger>
        </TabsList>

        {/* Single User Export */}
        <TabsContent value="export" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Download className="h-5 w-5" />
                Export User Permissions
              </CardTitle>
              <CardDescription>
                Export {user.email}'s permissions for backup or migration
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Export Options */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>Export Format</Label>
                  <Select value={exportFormat} onValueChange={(value: any) => setExportFormat(value)}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="json">
                        <div className="flex items-center gap-2">
                          <FileJson className="h-4 w-4" />
                          JSON (Structured Data)
                        </div>
                      </SelectItem>
                      <SelectItem value="csv">
                        <div className="flex items-center gap-2">
                          <FileSpreadsheet className="h-4 w-4" />
                          CSV (Spreadsheet)
                        </div>
                      </SelectItem>
                      <SelectItem value="xlsx">
                        <div className="flex items-center gap-2">
                          <FileSpreadsheet className="h-4 w-4" />
                          Excel (Advanced Spreadsheet)
                        </div>
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-3">
                  <Label>Include Additional Data</Label>
                  <div className="space-y-2">
                    <div className="flex items-center gap-2">
                      <Checkbox
                        id="include-history"
                        checked={exportOptions.includeHistory}
                        onChange={(e) => setExportOptions(prev => ({ ...prev, includeHistory: e.target.checked }))}
                      />
                      <Label htmlFor="include-history" className="text-sm">Permission History</Label>
                    </div>
                    <div className="flex items-center gap-2">
                      <Checkbox
                        id="include-temporary"
                        checked={exportOptions.includeTemporary}
                        onChange={(e) => setExportOptions(prev => ({ ...prev, includeTemporary: e.target.checked }))}
                      />
                      <Label htmlFor="include-temporary" className="text-sm">Temporary Permissions</Label>
                    </div>
                  </div>
                </div>
              </div>

              {/* Export Summary */}
              <div className="grid grid-cols-3 gap-4 p-4 bg-accent/20 rounded-lg">
                <div className="text-center">
                  <div className="text-2xl font-bold text-blue-600">{user.roles.length}</div>
                  <div className="text-sm text-muted-foreground">Roles</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-green-600">{user.customPermissions.length}</div>
                  <div className="text-sm text-muted-foreground">Permissions</div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-purple-600">{user.permissionProfiles.length}</div>
                  <div className="text-sm text-muted-foreground">Profiles</div>
                </div>
              </div>

              {/* Export Actions */}
              <div className="flex gap-2">
                <Button onClick={handleSingleExport} disabled={loading} className="flex-1">
                  {getFormatIcon(exportFormat)}
                  <span className="ml-2">Export as {exportFormat.toUpperCase()}</span>
                </Button>
              </div>

              {/* Export Results */}
              {exportData && exportFormat === 'json' && (
                <div className="space-y-4">
                  <div className="space-y-2">
                    <Label>Export Preview</Label>
                    <Textarea
                      value={JSON.stringify(exportData, null, 2)}
                      readOnly
                      className="font-mono text-xs bg-accent/20 min-h-[200px]"
                    />
                  </div>

                  <div className="flex gap-2">
                    <Button onClick={handleCopyExport} variant="outline" className="flex-1">
                      {copied ? <Check className="h-4 w-4 mr-2" /> : <Copy className="h-4 w-4 mr-2" />}
                      {copied ? 'Copied!' : 'Copy to Clipboard'}
                    </Button>
                    <Button onClick={handleDownloadExport} variant="outline" className="flex-1">
                      <Download className="h-4 w-4 mr-2" />
                      Download JSON
                    </Button>
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Import */}
        <TabsContent value="import" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Upload className="h-5 w-5" />
                Import User Permissions
              </CardTitle>
              <CardDescription>
                Import permissions from a backup file or JSON data
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* Import Warning */}
              <Alert>
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription>
                  <strong>Warning:</strong> Importing permissions will modify this user's current permissions. 
                  Use "Replace Existing" to completely overwrite or uncheck to merge permissions.
                </AlertDescription>
              </Alert>

              {/* Import Options */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-3">
                  <Label>Import Options</Label>
                  <div className="space-y-2">
                    <div className="flex items-center gap-2">
                      <Checkbox
                        id="import-roles"
                        checked={importOptions.includeRoles}
                        onChange={(e) => setImportOptions(prev => ({ ...prev, includeRoles: e.target.checked }))}
                      />
                      <Label htmlFor="import-roles" className="text-sm">Roles</Label>
                    </div>
                    <div className="flex items-center gap-2">
                      <Checkbox
                        id="import-permissions"
                        checked={importOptions.includeCustomPermissions}
                        onChange={(e) => setImportOptions(prev => ({ ...prev, includeCustomPermissions: e.target.checked }))}
                      />
                      <Label htmlFor="import-permissions" className="text-sm">Custom Permissions</Label>
                    </div>
                    <div className="flex items-center gap-2">
                      <Checkbox
                        id="import-profiles"
                        checked={importOptions.includeProfiles}
                        onChange={(e) => setImportOptions(prev => ({ ...prev, includeProfiles: e.target.checked }))}
                      />
                      <Label htmlFor="import-profiles" className="text-sm">Permission Profiles</Label>
                    </div>
                    <div className="flex items-center gap-2">
                      <Checkbox
                        id="import-temporary"
                        checked={importOptions.includeTemporary}
                        onChange={(e) => setImportOptions(prev => ({ ...prev, includeTemporary: e.target.checked }))}
                      />
                      <Label htmlFor="import-temporary" className="text-sm">Temporary Permissions</Label>
                    </div>
                  </div>
                </div>

                <div className="space-y-3">
                  <Label>Import Mode</Label>
                  <div className="space-y-2">
                    <div className="flex items-center gap-2">
                      <Checkbox
                        id="replace-existing"
                        checked={importOptions.replaceExisting}
                        onChange={(e) => setImportOptions(prev => ({ ...prev, replaceExisting: e.target.checked }))}
                      />
                      <Label htmlFor="replace-existing" className="text-sm">Replace Existing Permissions</Label>
                    </div>
                    <div className="flex items-center gap-2">
                      <Checkbox
                        id="dry-run"
                        checked={importOptions.dryRun}
                        onChange={(e) => setImportOptions(prev => ({ ...prev, dryRun: e.target.checked }))}
                      />
                      <Label htmlFor="dry-run" className="text-sm">Dry Run (Preview Only)</Label>
                    </div>
                  </div>
                </div>
              </div>

              {/* File Upload */}
              <div className="space-y-2">
                <Label>Upload Permission File</Label>
                <div className="flex gap-2">
                  <Input
                    type="file"
                    accept=".json,.txt"
                    onChange={handleFileUpload}
                    ref={fileInputRef}
                    className="flex-1"
                  />
                  <Button 
                    onClick={() => fileInputRef.current?.click()} 
                    variant="outline"
                  >
                    <Upload className="h-4 w-4 mr-2" />
                    Browse
                  </Button>
                </div>
                {importFile && (
                  <div className="text-sm text-muted-foreground">
                    Selected: {importFile.name} ({(importFile.size / 1024).toFixed(1)} KB)
                  </div>
                )}
              </div>

              {/* Manual Input */}
              <div className="space-y-2">
                <Label>Or Paste Permission Data (JSON)</Label>
                <Textarea
                  value={importData}
                  onChange={(e) => {
                    setImportData(e.target.value);
                    setValidationResult(null);
                  }}
                  onBlur={() => handleValidateImport()}
                  placeholder="Paste exported permission JSON data here..."
                  className="font-mono text-xs min-h-[150px]"
                />
              </div>

              {/* Validation Button */}
              <Button onClick={() => handleValidateImport()} disabled={!importData.trim()}>
                <Eye className="h-4 w-4 mr-2" />
                Validate Import Data
              </Button>

              {/* Validation Results */}
              {validationResult && (
                <div className={`space-y-4 p-4 rounded-lg border ${validationResult.isValid ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'}`}>
                  <div className="flex items-center gap-2">
                    {validationResult.isValid ? (
                      <CheckCircle className="h-5 w-5 text-green-600" />
                    ) : (
                      <XCircle className="h-5 w-5 text-red-600" />
                    )}
                    <span className="font-medium">
                      {validationResult.isValid ? 'Validation Passed' : 'Validation Failed'}
                    </span>
                  </div>

                  {/* Validation Errors */}
                  {validationResult.errors.length > 0 && (
                    <div className="space-y-2">
                      <Label className="text-sm font-medium text-red-800">Errors:</Label>
                      {validationResult.errors.map((error, index) => (
                        <div key={index} className="text-sm text-red-700 bg-red-100 p-2 rounded">
                          {error}
                        </div>
                      ))}
                    </div>
                  )}

                  {/* Validation Warnings */}
                  {validationResult.warnings.length > 0 && (
                    <div className="space-y-2">
                      <Label className="text-sm font-medium text-yellow-800">Warnings:</Label>
                      {validationResult.warnings.map((warning, index) => (
                        <div key={index} className="text-sm text-yellow-700 bg-yellow-100 p-2 rounded">
                          {warning}
                        </div>
                      ))}
                    </div>
                  )}

                  {/* Import Preview */}
                  {validationResult.isValid && (
                    <div className="space-y-3">
                      <Label className="font-medium">Import Preview:</Label>
                      <div className="grid grid-cols-2 md:grid-cols-3 gap-4 text-sm">
                        <div className="space-y-1">
                          <div className="text-muted-foreground">Roles</div>
                          <div className="text-green-600">+{validationResult.preview.rolesToAdd}</div>
                          <div className="text-red-600">-{validationResult.preview.rolesToRemove}</div>
                        </div>
                        <div className="space-y-1">
                          <div className="text-muted-foreground">Permissions</div>
                          <div className="text-green-600">+{validationResult.preview.permissionsToAdd}</div>
                          <div className="text-red-600">-{validationResult.preview.permissionsToRemove}</div>
                        </div>
                        <div className="space-y-1">
                          <div className="text-muted-foreground">Profiles</div>
                          <div className="text-green-600">+{validationResult.preview.profilesToAdd}</div>
                          <div className="text-red-600">-{validationResult.preview.profilesToRemove}</div>
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Import Action */}
                  {validationResult.isValid && (
                    <Button 
                      onClick={handleImport} 
                      disabled={loading}
                      variant={importOptions.replaceExisting ? "destructive" : "default"}
                      className="w-full"
                    >
                      {loading ? (
                        <>
                          <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                          {importOptions.dryRun ? 'Testing Import...' : 'Importing...'}
                        </>
                      ) : (
                        <>
                          {importOptions.dryRun ? <Eye className="h-4 w-4 mr-2" /> : <Play className="h-4 w-4 mr-2" />}
                          {importOptions.dryRun ? 'Test Import (Dry Run)' : importOptions.replaceExisting ? 'Replace All Permissions' : 'Merge Permissions'}
                        </>
                      )}
                    </Button>
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Bulk Export */}
        <TabsContent value="bulk" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Users className="h-5 w-5" />
                Bulk Permission Export
              </CardTitle>
              <CardDescription>
                Export permissions for multiple users at once
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-6">
              {/* User Selection */}
              <div className="space-y-4">
                <Label>Target Users ({bulkUserIds.length} selected)</Label>
                <div className="flex gap-2">
                  <Input
                    placeholder="Enter user ID or email"
                    value={bulkUserInput}
                    onChange={(e) => setBulkUserInput(e.target.value)}
                    onKeyPress={(e) => e.key === 'Enter' && addBulkUserId()}
                  />
                  <Button onClick={addBulkUserId} size="sm">
                    Add
                  </Button>
                </div>
                {bulkUserIds.length > 0 && (
                  <div className="flex flex-wrap gap-2">
                    {bulkUserIds.map((userId) => (
                      <Badge key={userId} variant="secondary" className="flex items-center gap-1">
                        {userId}
                        <button onClick={() => removeBulkUserId(userId)}>
                          <XCircle className="h-3 w-3" />
                        </button>
                      </Badge>
                    ))}
                  </div>
                )}
              </div>

              {/* Bulk Export Options */}
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div className="space-y-2">
                  <Label>Export Format</Label>
                  <Select value={bulkFormat} onValueChange={(value: any) => setBulkFormat(value)}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="json">JSON</SelectItem>
                      <SelectItem value="csv">CSV</SelectItem>
                      <SelectItem value="xlsx">Excel</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                <div className="space-y-2">
                  <Label>Group By</Label>
                  <Select value={bulkOptions.groupBy} onValueChange={(value: any) => setBulkOptions(prev => ({ ...prev, groupBy: value }))}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="user">User</SelectItem>
                      <SelectItem value="permissions">Permissions</SelectItem>
                      <SelectItem value="profile">Profile</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="space-y-2">
                <Label>Additional Data</Label>
                <div className="flex gap-4">
                  <div className="flex items-center gap-2">
                    <Checkbox
                      id="bulk-history"
                      checked={bulkOptions.includeHistory}
                      onChange={(e) => setBulkOptions(prev => ({ ...prev, includeHistory: e.target.checked }))}
                    />
                    <Label htmlFor="bulk-history" className="text-sm">Include History</Label>
                  </div>
                  <div className="flex items-center gap-2">
                    <Checkbox
                      id="bulk-temporary"
                      checked={bulkOptions.includeTemporary}
                      onChange={(e) => setBulkOptions(prev => ({ ...prev, includeTemporary: e.target.checked }))}
                    />
                    <Label htmlFor="bulk-temporary" className="text-sm">Include Temporary</Label>
                  </div>
                </div>
              </div>

              <Button 
                onClick={handleBulkExport} 
                disabled={loading || bulkUserIds.length === 0}
                className="w-full"
              >
                {loading ? (
                  <>
                    <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                    Exporting...
                  </>
                ) : (
                  <>
                    <Download className="h-4 w-4 mr-2" />
                    Export {bulkUserIds.length} User{bulkUserIds.length !== 1 ? 's' : ''} as {bulkFormat.toUpperCase()}
                  </>
                )}
              </Button>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Permission Templates */}
        <TabsContent value="templates" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Settings className="h-5 w-5" />
                Permission Templates
              </CardTitle>
              <CardDescription>
                Export and import reusable permission templates
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-center py-8 text-muted-foreground">
                <Settings className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>Permission templates feature coming soon</p>
                <p className="text-xs">Create reusable permission sets for common roles</p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* System Backup */}
        <TabsContent value="backup" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Archive className="h-5 w-5" />
                System Backup & Restore
              </CardTitle>
              <CardDescription>
                Create full system backups and restore from previous backups
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="text-center py-8 text-muted-foreground">
                <Archive className="h-12 w-12 mx-auto mb-4 opacity-50" />
                <p>System backup feature coming soon</p>
                <p className="text-xs">Full system permission backup and restore capabilities</p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}