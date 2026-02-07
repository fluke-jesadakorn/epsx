'use client';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { type User } from '@/shared/types/auth';
import { AlertTriangle, Clock, Database, Download, FileText, RefreshCw, Shield, Trash2 } from 'lucide-react';
import { useState } from 'react';
import { toast } from 'sonner';

interface DataManagementProps {
  user: User;
}

interface DataExportState {
  isExporting: boolean;
  exportType?: 'minimal' | 'full' | 'analytics';
}

interface DataDeletionState {
  isDeleting: boolean;
  confirmationText: string;
  showConfirmation: boolean;
}

export function DataManagement({ user }: DataManagementProps) {
  const [exportState, setExportState] = useState<DataExportState>({
    isExporting: false,
  });

  const [deletionState, setDeletionState] = useState<DataDeletionState>({
    isDeleting: false,
    confirmationText: '',
    showConfirmation: false,
  });

  const handleDataExport = async (type: 'minimal' | 'full' | 'analytics') => {
    setExportState({ isExporting: true, exportType: type });

    try {
      const response = await fetch('/api/user/export-data', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ export_type: type }),
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to export data');
      }

      // Get the blob and create download
      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `epsx-data-export-${type}-${new Date().toISOString().split('T')[0]}.json`;
      document.body.appendChild(a);
      a.click();
      window.URL.revokeObjectURL(url);
      document.body.removeChild(a);

      toast.success('Data exported successfully');

    } catch (error: any) {
      console.error('Data export error:', error);
      toast.error(error.message || 'Failed to export data');
    } finally {
      setExportState({ isExporting: false });
    }
  };

  const handleAccountDeletion = async () => {
    if (deletionState.confirmationText !== 'DELETE MY ACCOUNT') {
      toast.error('Please type "DELETE MY ACCOUNT" to confirm');
      return;
    }

    setDeletionState(prev => ({ ...prev, isDeleting: true }));

    try {
      const response = await fetch('/api/user/delete-account', {
        method: 'DELETE',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ confirmation: deletionState.confirmationText }),
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to delete account');
      }

      toast.success('Account deletion initiated. You will be logged out shortly.');

      // Redirect to logout after short delay
      setTimeout(() => {
        window.location.href = '/api/auth/logout';
      }, 2000);

    } catch (error: any) {
      console.error('Account deletion error:', error);
      toast.error(error.message || 'Failed to delete account');
      setDeletionState(prev => ({ ...prev, isDeleting: false }));
    }
  };

  const formatDate = (timestamp?: number) => {
    if (!timestamp) {return 'Not available';}
    return new Date(timestamp * 1000).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="space-y-6">
      {/* Data Overview */}
      <Card className="border-orange-100 dark:border-slate-700">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Database className="h-5 w-5 text-orange-500" />
            Data Overview
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
              <div className="text-2xl font-bold text-slate-900 dark:text-slate-100">
                {user.permissions?.length || 0}
              </div>
              <div className="text-sm text-slate-600 dark:text-slate-400">
                Active Permissions
              </div>
            </div>

            <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
              <div className="text-2xl font-bold text-slate-900 dark:text-slate-100">
                {user.tier?.toUpperCase() || 'FREE'}
              </div>
              <div className="text-sm text-slate-600 dark:text-slate-400">
                Account Tier
              </div>
            </div>

            <div className="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
              <div className="text-2xl font-bold text-slate-900 dark:text-slate-100">
                <Badge variant={user.verified ? "default" : "secondary"}>
                  {user.verified ? 'Verified' : 'Unverified'}
                </Badge>
              </div>
              <div className="text-sm text-slate-600 dark:text-slate-400">
                Account Status
              </div>
            </div>
          </div>

          <div className="pt-4 border-t border-slate-200 dark:border-slate-700">
            <div className="flex items-center gap-2 text-sm text-slate-600 dark:text-slate-400">
              <Clock className="h-4 w-4" />
              Last permission update: {formatDate(user.permission_last_updated)}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Data Export */}
      <Card className="border-orange-100 dark:border-slate-700">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Download className="h-5 w-5 text-orange-500" />
            Export Your Data
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm text-slate-600 dark:text-slate-400">
            Download your personal data in JSON format. Choose the export type based on your needs.
          </p>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div className="p-4 border border-slate-200 dark:border-slate-700 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <FileText className="h-4 w-4 text-blue-500" />
                <span className="font-medium text-slate-900 dark:text-slate-100">
                  Basic Info
                </span>
              </div>
              <p className="text-sm text-slate-600 dark:text-slate-400 mb-3">
                Account details, email, permissions, and settings.
              </p>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleDataExport('minimal')}
                disabled={exportState.isExporting}
                className="w-full"
              >
                {exportState.isExporting && exportState.exportType === 'minimal' && (
                  <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                )}
                Export Basic
              </Button>
            </div>

            <div className="p-4 border border-slate-200 dark:border-slate-700 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Database className="h-4 w-4 text-green-500" />
                <span className="font-medium text-slate-900 dark:text-slate-100">
                  Complete Data
                </span>
              </div>
              <p className="text-sm text-slate-600 dark:text-slate-400 mb-3">
                All personal data including activity logs and session history.
              </p>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleDataExport('full')}
                disabled={exportState.isExporting}
                className="w-full"
              >
                {exportState.isExporting && exportState.exportType === 'full' && (
                  <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                )}
                Export Full
              </Button>
            </div>

            <div className="p-4 border border-slate-200 dark:border-slate-700 rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Shield className="h-4 w-4 text-purple-500" />
                <span className="font-medium text-slate-900 dark:text-slate-100">
                  Analytics Only
                </span>
              </div>
              <p className="text-sm text-slate-600 dark:text-slate-400 mb-3">
                Usage statistics, preferences, and analytics data.
              </p>
              <Button
                variant="outline"
                size="sm"
                onClick={() => handleDataExport('analytics')}
                disabled={exportState.isExporting}
                className="w-full"
              >
                {exportState.isExporting && exportState.exportType === 'analytics' && (
                  <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                )}
                Export Analytics
              </Button>
            </div>
          </div>

          <Alert>
            <Shield className="h-4 w-4" />
            <AlertDescription>
              Exported data includes only your personal information. Shared or public data is not included.
            </AlertDescription>
          </Alert>
        </CardContent>
      </Card>

      {/* Account Deletion */}
      <Card className="border-red-200 dark:border-red-800 bg-red-50 dark:bg-red-950/20">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-red-700 dark:text-red-300">
            <AlertTriangle className="h-5 w-5" />
            Danger Zone
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              <strong>Warning:</strong> Account deletion is permanent and cannot be undone.
              All your data, permissions, and access will be permanently removed.
            </AlertDescription>
          </Alert>

          <div className="space-y-2">
            <h4 className="font-medium text-red-700 dark:text-red-300">
              What happens when you delete your account:
            </h4>
            <ul className="text-sm text-red-600 dark:text-red-400 space-y-1 ml-4">
              <li>• All personal data and account information will be permanently deleted</li>
              <li>• Access to all EPSX services will be immediately revoked</li>
              <li>• Any active subscriptions will be cancelled</li>
              <li>• Web3 wallet connections will be disconnected</li>
              <li>• This action cannot be undone</li>
            </ul>
          </div>

          <Dialog
            open={deletionState.showConfirmation}
            onOpenChange={(open) => setDeletionState(prev => ({
              ...prev,
              showConfirmation: open,
              confirmationText: open ? prev.confirmationText : ''
            }))}
          >
            <DialogTrigger asChild>
              <Button variant="destructive" className="bg-red-600 hover:bg-red-700">
                <Trash2 className="h-4 w-4 mr-2" />
                Delete Account
              </Button>
            </DialogTrigger>
            <DialogContent className="sm:max-w-md">
              <DialogHeader>
                <DialogTitle className="text-red-700 dark:text-red-300">
                  Delete Account Confirmation
                </DialogTitle>
              </DialogHeader>
              <div className="space-y-4">
                <Alert variant="destructive">
                  <AlertTriangle className="h-4 w-4" />
                  <AlertDescription>
                    This action is permanent and cannot be undone.
                  </AlertDescription>
                </Alert>

                <div>
                  <label className="text-sm font-medium text-slate-700 dark:text-slate-300">
                    Type "DELETE MY ACCOUNT" to confirm:
                  </label>
                  <input
                    type="text"
                    value={deletionState.confirmationText}
                    onChange={(e) => setDeletionState(prev => ({
                      ...prev,
                      confirmationText: e.target.value
                    }))}
                    className="w-full mt-1 px-3 py-2 border border-red-300 dark:border-red-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-red-500 dark:bg-slate-800"
                    placeholder="DELETE MY ACCOUNT"
                    disabled={deletionState.isDeleting}
                  />
                </div>

                <div className="flex gap-2">
                  <Button
                    variant="destructive"
                    onClick={handleAccountDeletion}
                    disabled={
                      deletionState.isDeleting ||
                      deletionState.confirmationText !== 'DELETE MY ACCOUNT'
                    }
                    className="flex-1 bg-red-600 hover:bg-red-700"
                  >
                    {deletionState.isDeleting ? (
                      <>
                        <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                        Deleting...
                      </>
                    ) : (
                      <>
                        <Trash2 className="h-4 w-4 mr-2" />
                        Delete Account
                      </>
                    )}
                  </Button>
                  <Button
                    variant="outline"
                    onClick={() => setDeletionState(prev => ({
                      ...prev,
                      showConfirmation: false,
                      confirmationText: ''
                    }))}
                    disabled={deletionState.isDeleting}
                  >
                    Cancel
                  </Button>
                </div>
              </div>
            </DialogContent>
          </Dialog>
        </CardContent>
      </Card>
    </div>
  );
}