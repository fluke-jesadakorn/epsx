'use client';

import { memo } from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { CheckCircle, XCircle, AlertTriangle } from 'lucide-react';

interface PermissionStats {
  activePermissions: Array<{id: string; status: string; is_expired: boolean}>;
  expiredPermissions: Array<{id: string; status: string; is_expired: boolean}>;
  revokedPermissions: Array<{id: string; status: string; is_expired: boolean}>;
}

interface PermissionStatsCardsProps {
  stats: PermissionStats;
}

function PermissionStatsCards({ stats }: PermissionStatsCardsProps) {
  const { activePermissions, expiredPermissions, revokedPermissions } = stats;

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
      {/* Active Permissions Card */}
      <Card className="bg-gradient-to-br from-green-50 to-emerald-50 border-green-200 dark:from-green-950/20 dark:to-emerald-950/20 dark:border-green-800">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-green-100 rounded-lg dark:bg-green-900/30">
                <CheckCircle className="h-6 w-6 text-green-600 dark:text-green-400" />
              </div>
              <div>
                <p className="text-2xl font-bold text-green-700 dark:text-green-300">
                  {activePermissions.length}
                </p>
                <p className="text-sm text-green-600 dark:text-green-400">Active</p>
              </div>
            </div>
            {activePermissions.length > 0 && (
              <div className="h-2 w-2 bg-green-500 rounded-full opacity-75"></div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Expired Permissions Card */}
      <Card className={`${expiredPermissions.length > 0 
        ? 'bg-gradient-to-br from-red-50 to-rose-50 border-red-200 dark:from-red-950/20 dark:to-rose-950/20 dark:border-red-800' 
        : 'bg-gradient-to-br from-gray-50 to-slate-50 border-gray-200 dark:from-gray-950/20 dark:to-slate-950/20 dark:border-gray-700'
      }`}>
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className={`p-2 rounded-lg ${expiredPermissions.length > 0 
                ? 'bg-red-100 dark:bg-red-900/30' 
                : 'bg-gray-100 dark:bg-gray-900/30'
              }`}>
                <XCircle className={`h-6 w-6 ${expiredPermissions.length > 0 
                  ? 'text-red-600 dark:text-red-400' 
                  : 'text-gray-500 dark:text-gray-400'
                }`} />
              </div>
              <div>
                <p className={`text-2xl font-bold ${expiredPermissions.length > 0 
                  ? 'text-red-700 dark:text-red-300' 
                  : 'text-gray-700 dark:text-gray-300'
                }`}>{expiredPermissions.length}</p>
                <p className={`text-sm ${expiredPermissions.length > 0 
                  ? 'text-red-600 dark:text-red-400' 
                  : 'text-gray-600 dark:text-gray-400'
                }`}>Expired</p>
              </div>
            </div>
            {expiredPermissions.length > 0 && (
              <div className="flex items-center gap-1">
                <AlertTriangle className="h-4 w-4 text-red-500 opacity-75" />
                <div className="h-2 w-2 bg-red-500 rounded-full opacity-75"></div>
              </div>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Revoked Permissions Card */}
      <Card className="bg-gradient-to-br from-orange-50 to-yellow-50 border-orange-200 dark:from-orange-950/20 dark:to-yellow-950/20 dark:border-orange-800">
        <CardContent className="p-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="p-2 bg-orange-100 rounded-lg dark:bg-orange-900/30">
                <XCircle className="h-6 w-6 text-orange-600 dark:text-orange-400" />
              </div>
              <div>
                <p className="text-2xl font-bold text-orange-700 dark:text-orange-300">
                  {revokedPermissions.length}
                </p>
                <p className="text-sm text-orange-600 dark:text-orange-400">Revoked</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

export default memo(PermissionStatsCards);