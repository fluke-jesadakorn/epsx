/**
 * UNIFIED PERMISSION EXPIRY INDICATOR COMPONENT
 * 
 * Consolidates permission expiry display logic for both admin-frontend and frontend apps.
 * Replaces AdminPermissionExpiryIndicator and PermissionExpiryIndicator with a single,
 * platform-aware component that handles both single and multiple permission displays.
 * 
 * Features:
 * - Multiple display variants (badge, banner, card, dashboard, inline, detailed)
 * - Single or multiple permission handling
 * - Real-time countdown updates
 * - Platform-aware upgrade prompts
 * - Comprehensive health scoring
 * - Action buttons for extend/revoke operations
 * - Dialog-based detail views
 */
'use client';

import {
  AlertCircle,
  AlertTriangle,
  ArrowUp,
  CheckCircle,
  Clock,
  Eye,
  RefreshCw,
  Shield,
  XCircle,
  Zap
} from 'lucide-react';
import React, { useEffect, useState } from 'react';

import { format, formatDistanceToNow } from 'date-fns';
import { BaseButton as Button } from '../buttons/BaseButton';
import { BaseCard as Card } from '../cards/BaseCard';
import { BaseModal } from '../modals/BaseModal';

// Simple inline components to replace missing UI components
const Badge = ({ className, variant, children, ...props }: any) => (
  <span className={`inline-flex items-center px-2 py-1 rounded-full text-xs font-medium ${className}`} {...props}>
    {children}
  </span>
);

const Alert = ({ className, variant, children, ...props }: any) => (
  <div className={`rounded-lg border p-4 ${className}`} {...props}>
    {children}
  </div>
);

const AlertDescription = ({ children, ...props }: any) => (
  <div className="text-sm" {...props}>{children}</div>
);

const CardContent = ({ className, children, ...props }: any) => (
  <div className={`p-6 ${className}`} {...props}>{children}</div>
);

const Dialog = ({ children, ...props }: any) => (
  <BaseModal {...props}>{children}</BaseModal>
);

const DialogTrigger = ({ children, asChild, ...props }: any) => (
  <div {...props}>{children}</div>
);

const DialogContent = ({ children, ...props }: any) => (
  <div {...props}>{children}</div>
);

const DialogHeader = ({ children, ...props }: any) => (
  <div className="space-y-2" {...props}>{children}</div>
);

const DialogTitle = ({ children, className, ...props }: any) => (
  <h2 className={`text-lg font-semibold ${className}`} {...props}>{children}</h2>
);

const DialogDescription = ({ children, ...props }: any) => (
  <p className="text-sm text-gray-600" {...props}>{children}</p>
);

const Separator = ({ className, ...props }: any) => (
  <hr className={`border-t border-gray-200 ${className}`} {...props} />
);

// Platform types
export type Platform = 'admin' | 'frontend';

// Base permission info interface
export interface PermissionInfo {
  permission: string;
  basePermission: string;
  platform: string;
  resource: string;
  action: string;
  expiresAt?: number;
  isExpired: boolean;
  isExpiringSoon: boolean;
  timeRemaining: number;
  urgencyLevel: 'expired' | 'critical' | 'warning' | 'normal' | 'permanent';
  source?: string;
  grantedAt?: number;
}

// Permission data collection interface
export interface PermissionExpiryData {
  permissions: PermissionInfo[];
  totalCount: number;
  expiredCount: number;
  expiringSoonCount: number;
  criticalCount: number;
  permanentCount: number;
  healthScore: 'excellent' | 'good' | 'warning' | 'critical';
  nextExpiry?: PermissionInfo;
}

// Component props interface
export interface UnifiedPermissionExpiryIndicatorProps {
  /**
   * Platform context - determines styling and behavior
   */
  platform: Platform;

  /**
   * Single permission or array of permissions to display
   */
  permissions: string | string[];

  /**
   * Display variant
   */
  variant?: 'badge' | 'banner' | 'card' | 'compact' | 'detailed' | 'dashboard' | 'inline' | 'full';

  /**
   * Size for badge and compact variants
   */
  size?: 'xs' | 'sm' | 'md' | 'lg';

  /**
   * Whether to show live countdown
   */
  showCountdown?: boolean;

  /**
   * Whether to show detailed permission information
   */
  showDetails?: boolean;

  /**
   * Whether to show health scoring (multiple permissions)
   */
  showHealth?: boolean;

  /**
   * Whether to show action buttons
   */
  showActions?: boolean;

  /**
   * CSS class name
   */
  className?: string;

  /**
   * Callback when permission expires
   */
  onExpired?: (permission: PermissionInfo) => void;

  /**
   * Action callbacks (admin only)
   */
  onExtendPermission?: (permission: PermissionInfo) => void;
  onRevokePermission?: (permission: PermissionInfo) => void;
  onViewDetails?: (permission: PermissionInfo) => void;

  /**
   * Custom permission data provider
   */
  permissionDataProvider?: (permissions: string[]) => PermissionExpiryData;
}

// Hook interface for platform abstraction
interface PermissionHookInterface {
  hasPermission: (permission: string) => boolean;
  getPermissionExpiry?: (permission: string) => any;
  refreshPermissions?: () => Promise<void>;
  loading?: boolean;
}

// Platform-specific permission hook selector
function usePermissionHook(platform: Platform): PermissionHookInterface {
  // This will be registered by each platform
  if (typeof window !== 'undefined') {
    const hookRegistry = (window as any).__PERMISSION_HOOK_REGISTRY;
    if (hookRegistry && hookRegistry[platform]) {
      return hookRegistry[platform]();
    }
  }

  // Fallback for server-side rendering
  return {
    hasPermission: () => false,
    getPermissionExpiry: () => null,
    refreshPermissions: async () => { },
    loading: false
  };
}

export default function UnifiedPermissionExpiryIndicator({
  platform,
  permissions,
  variant = 'badge',
  size = 'sm',
  showCountdown = true,
  showDetails = false,
  showHealth = false,
  showActions = false,
  className = '',
  onExpired,
  onExtendPermission,
  onRevokePermission,
  onViewDetails,
  permissionDataProvider
}: UnifiedPermissionExpiryIndicatorProps) {
  const permissionHook = usePermissionHook(platform);
  const [currentTime, setCurrentTime] = useState(Date.now());
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);

  // Convert single permission to array for unified processing
  const permissionArray = Array.isArray(permissions) ? permissions : [permissions];

  // Update current time for live countdown
  useEffect(() => {
    if (!showCountdown) return;

    const interval = setInterval(() => {
      setCurrentTime(Date.now());
    }, 60000); // Update every minute

    return () => clearInterval(interval);
  }, [showCountdown]);

  // Process permissions into structured data
  const processedData: PermissionExpiryData = permissionDataProvider
    ? permissionDataProvider(permissionArray)
    : parsePermissions(permissionArray, permissionHook, currentTime);

  // Handle expiration callbacks
  useEffect(() => {
    if (onExpired) {
      processedData.permissions
        .filter(p => p.isExpired)
        .forEach(onExpired);
    }
  }, [processedData.permissions, onExpired]);

  // Handle refresh
  const handleRefresh = async () => {
    if (!permissionHook.refreshPermissions) return;

    setIsRefreshing(true);
    try {
      await permissionHook.refreshPermissions();
    } catch (error) {
      console.error('Failed to refresh permissions:', error);
    } finally {
      setIsRefreshing(false);
    }
  };

  // Don't render if no permissions or all permanent with no issues
  if (processedData.totalCount === 0 ||
    (processedData.expiredCount === 0 &&
      processedData.criticalCount === 0 &&
      processedData.expiringSoonCount === 0 &&
      variant === 'badge')) {
    return null;
  }

  // Route to appropriate variant component
  const VariantComponent = getVariantComponent(variant);

  return (
    <VariantComponent
      platform={platform}
      data={processedData}
      size={size}
      showCountdown={showCountdown}
      showDetails={showDetails}
      showHealth={showHealth}
      showActions={showActions}
      className={className}
      isDialogOpen={isDialogOpen}
      setIsDialogOpen={setIsDialogOpen}
      onRefresh={handleRefresh}
      isRefreshing={isRefreshing}
      onExtendPermission={onExtendPermission}
      onRevokePermission={onRevokePermission}
      onViewDetails={onViewDetails}
    />
  );
}

// ============================================================================
// VARIANT COMPONENTS
// ============================================================================

function BadgeVariant({
  data,
  size,
  className,
  isDialogOpen,
  setIsDialogOpen
}: VariantProps) {
  const overallUrgency = getOverallUrgency(data);
  const { colors, icon: Icon } = getUrgencyConfig(overallUrgency);
  const sizeClasses = getSizeClasses(size || 'sm');

  return (
    <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
      <DialogTrigger asChild>
        <Badge className={`inline-flex items-center cursor-pointer font-medium hover:opacity-80 ${colors} ${sizeClasses} ${className}`}>
          <Icon className={`${getIconSize(size)} mr-1`} />
          <span>
            {overallUrgency === 'expired' && `${data.expiredCount} Expired`}
            {overallUrgency === 'critical' && `${data.criticalCount} Critical`}
            {overallUrgency === 'warning' && `${data.expiringSoonCount} Expiring`}
            {overallUrgency === 'normal' && 'Valid'}
            {overallUrgency === 'permanent' && `${data.permanentCount} Permanent`}
          </span>
        </Badge>
      </DialogTrigger>

      <DialogContent>
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Icon className="h-5 w-5" />
            Permission Details
          </DialogTitle>
          <DialogDescription>
            Permission expiry information
          </DialogDescription>
        </DialogHeader>

        <PermissionDetailView data={data} />
      </DialogContent>
    </Dialog>
  );
}

function InlineVariant({
  data,
  platform,
  showDetails,
  className,
  isDialogOpen,
  setIsDialogOpen
}: VariantProps) {
  const firstPermission = data.permissions[0];
  if (!firstPermission) return null;

  const { icon: Icon } = getUrgencyConfig(firstPermission.urgencyLevel);

  return (
    <div className={`flex items-center gap-2 ${className}`}>
      <Icon className="h-4 w-4" />
      <span className={`text-sm ${getTextColor(firstPermission.urgencyLevel)}`}>
        {formatTimeRemaining(firstPermission.timeRemaining)}
      </span>

      {showDetails && (
        <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
          <DialogTrigger asChild>
            <Button variant="ghost" size="sm" className="h-6 px-2">
              <Eye className="h-3 w-3" />
            </Button>
          </DialogTrigger>

          <DialogContent>
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Icon className="h-5 w-5" />
                Permission Details
              </DialogTitle>
              <DialogDescription>
                Information about the permission
              </DialogDescription>
            </DialogHeader>

            <PermissionDetailView data={data} />
          </DialogContent>
        </Dialog>
      )}
    </div>
  );
}

function FullVariant({
  data,
  platform,
  showDetails,
  className,
  onRefresh,
  isRefreshing,
  isDialogOpen,
  setIsDialogOpen
}: VariantProps) {
  const firstPermission = data.permissions[0];
  if (!firstPermission) return null;

  const { icon: Icon } = getUrgencyConfig(firstPermission.urgencyLevel);
  const timeText = formatTimeRemaining(firstPermission.timeRemaining);
  const exactTime = firstPermission.expiresAt ? format(new Date(firstPermission.expiresAt * 1000), 'PPpp') : undefined;

  return (
    <Alert className={`${className} ${getAlertColors(firstPermission.urgencyLevel)}`}>
      <Icon className="h-5 w-5" />
      <AlertDescription>
        <div className="flex items-center justify-between">
          <div>
            <div className={`font-medium ${getTextColor(firstPermission.urgencyLevel)}`}>
              Permission: {firstPermission.basePermission}
            </div>
            <div className={`text-sm ${getTextColor(firstPermission.urgencyLevel)}`}>
              {timeText}
              {exactTime && (
                <span className="ml-2 text-muted-foreground">
                  ({exactTime})
                </span>
              )}
            </div>
          </div>

          <div className="flex gap-2">
            {showDetails && (
              <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
                <DialogTrigger asChild>
                  <Button variant="ghost" size="sm">
                    <Eye className="h-3 w-3" />
                    Details
                  </Button>
                </DialogTrigger>

                <DialogContent>
                  <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                      <Icon className="h-5 w-5" />
                      Permission Details
                    </DialogTitle>
                    <DialogDescription>
                      Information about the "{firstPermission.basePermission}" permission
                    </DialogDescription>
                  </DialogHeader>

                  <PermissionDetailView data={data} />
                </DialogContent>
              </Dialog>
            )}

            {onRefresh && (
              <Button
                variant="ghost"
                size="sm"
                onClick={onRefresh}
                disabled={isRefreshing}
              >
                <RefreshCw className={`h-3 w-3 ${isRefreshing ? 'animate-spin' : ''}`} />
                Refresh
              </Button>
            )}

            {(firstPermission.isExpired || firstPermission.urgencyLevel === 'critical') && platform === 'frontend' && (
              <Button
                size="sm"
                onClick={() => window.open('/upgrade', '_blank')}
              >
                <ArrowUp className="h-3 w-3" />
                Extend Access
              </Button>
            )}
          </div>
        </div>
      </AlertDescription>
    </Alert>
  );
}

// ============================================================================
// PERMISSION DATA PROCESSING
// ============================================================================

function parsePermissions(
  permissions: string[],
  permissionHook: PermissionHookInterface,
  currentTime: number
): PermissionExpiryData {
  const now = Math.floor(currentTime / 1000);
  const processedPermissions: PermissionInfo[] = [];

  for (const permission of permissions) {
    // Check if user has this permission
    if (!permissionHook.hasPermission(permission)) continue;

    let permissionInfo: PermissionInfo;

    // Try to get expiry information from hook first
    const expiryInfo = permissionHook.getPermissionExpiry?.(permission);

    if (expiryInfo && expiryInfo.claim?.expires_at) {
      const expiresAt = expiryInfo.claim.expires_at;
      const timeRemaining = expiresAt - now;
      const isExpired = timeRemaining <= 0;
      const isExpiringSoon = !isExpired && timeRemaining <= 86400; // 24 hours

      let urgencyLevel: PermissionInfo['urgencyLevel'] = 'normal';
      if (isExpired) urgencyLevel = 'expired';
      else if (timeRemaining <= 3600) urgencyLevel = 'critical'; // 1 hour
      else if (isExpiringSoon) urgencyLevel = 'warning';

      const [platform, resource, action] = permission.split(':');

      permissionInfo = {
        permission,
        basePermission: permission,
        platform: platform || '',
        resource: resource || '',
        action: action || '',
        expiresAt,
        isExpired,
        isExpiringSoon,
        timeRemaining,
        urgencyLevel,
        source: expiryInfo.claim?.source,
        grantedAt: expiryInfo.claim?.granted_at
      };
    } else {
      // Try parsing embedded timestamp format
      const parts = permission.split(':');
      const lastPart = parts[parts.length - 1];
      const timestamp = parseInt(lastPart || '', 10);

      if (!isNaN(timestamp) && timestamp > 1000000000) {
        const basePermission = parts.slice(0, -1).join(':');
        const [platform, resource, action] = basePermission.split(':');
        const timeRemaining = timestamp - now;
        const isExpired = timeRemaining <= 0;
        const isExpiringSoon = !isExpired && timeRemaining <= 86400;

        let urgencyLevel: PermissionInfo['urgencyLevel'] = 'normal';
        if (isExpired) urgencyLevel = 'expired';
        else if (timeRemaining <= 3600) urgencyLevel = 'critical';
        else if (isExpiringSoon) urgencyLevel = 'warning';

        permissionInfo = {
          permission,
          basePermission,
          platform: platform || '',
          resource: resource || '',
          action: action || '',
          expiresAt: timestamp,
          isExpired,
          isExpiringSoon,
          timeRemaining,
          urgencyLevel
        };
      } else {
        // Permanent permission
        const [platform, resource, action] = parts;
        permissionInfo = {
          permission,
          basePermission: permission,
          platform: platform || '',
          resource: resource || '',
          action: action || '',
          isExpired: false,
          isExpiringSoon: false,
          timeRemaining: Infinity,
          urgencyLevel: 'permanent'
        };
      }
    }

    processedPermissions.push(permissionInfo);
  }

  // Calculate summary statistics
  const expiredCount = processedPermissions.filter(p => p.isExpired).length;
  const expiringSoonCount = processedPermissions.filter(p => p.isExpiringSoon && !p.isExpired).length;
  const criticalCount = processedPermissions.filter(p => p.urgencyLevel === 'critical').length;
  const permanentCount = processedPermissions.filter(p => p.urgencyLevel === 'permanent').length;

  // Calculate health score
  const totalActive = processedPermissions.length - expiredCount;
  const healthRatio = processedPermissions.length > 0 ? totalActive / processedPermissions.length : 1;
  let healthScore: PermissionExpiryData['healthScore'] = 'excellent';

  if (expiredCount > 0 || criticalCount > 2) healthScore = 'critical';
  else if (expiringSoonCount > 3 || healthRatio < 0.8) healthScore = 'warning';
  else if (healthRatio < 0.95) healthScore = 'good';

  // Find next expiry
  const nextExpiry = processedPermissions
    .filter(p => !p.isExpired && p.expiresAt)
    .sort((a, b) => (a.expiresAt || 0) - (b.expiresAt || 0))[0];

  return {
    permissions: processedPermissions,
    totalCount: processedPermissions.length,
    expiredCount,
    expiringSoonCount,
    criticalCount,
    permanentCount,
    healthScore,
    nextExpiry
  };
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

function BannerVariant({
  data,
  showDetails,
  className
}: VariantProps) {
  const overallUrgency = getOverallUrgency(data);
  const { bgColors, icon: Icon } = getUrgencyConfig(overallUrgency);

  if (overallUrgency === 'permanent' && data.expiredCount === 0) return null;

  return (
    <Alert className={`${bgColors} ${className}`}>
      <Icon className="h-5 w-5" />
      <AlertDescription>
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-sm font-medium mb-1">
              {overallUrgency === 'expired' && 'Permissions Have Expired'}
              {overallUrgency === 'critical' && 'Critical Permission Issues'}
              {overallUrgency === 'warning' && 'Permissions Expiring Soon'}
              {overallUrgency === 'normal' && 'Permissions Valid'}
            </h3>
            {showDetails && <ExpiryDetails data={data} compact={true} />}
          </div>

          <div className="text-right text-sm">
            <div className="font-medium">{data.totalCount} Total</div>
            <div className="text-muted-foreground text-xs">
              Health: <span className={getHealthTextColor(data.healthScore)}>{data.healthScore}</span>
            </div>
          </div>
        </div>
      </AlertDescription>
    </Alert>
  );
}

function CardVariant({
  platform,
  data,
  showActions,
  showDetails,
  showHealth,
  className,
  onExtendPermission,
  onRevokePermission,
  onViewDetails
}: VariantProps) {
  const overallUrgency = getOverallUrgency(data);
  const { icon: Icon } = getUrgencyConfig(overallUrgency);

  return (
    <Card className={className}>
      <CardContent className="p-6">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center">
            <Icon className={`h-6 w-6 mr-3 ${getUrgencyIconColor(overallUrgency)}`} />
            <div>
              <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                Permission Status
              </h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {getStatusText(data)}
              </p>
            </div>
          </div>
          <BadgeVariant platform={platform} data={data} size="md" className="" isDialogOpen={false} setIsDialogOpen={() => { }} />
        </div>

        {showHealth && (
          <>
            <PermissionHealthDisplay data={data} />
            <Separator className="my-4" />
          </>
        )}

        {showDetails && (
          <>
            <ExpiryDetails
              data={data}
              showActions={showActions}
              onExtendPermission={onExtendPermission}
              onRevokePermission={onRevokePermission}
              onViewDetails={onViewDetails}
            />
            <Separator className="my-4" />
          </>
        )}

        {data.nextExpiry && (
          <div className="flex items-center text-sm text-gray-600 dark:text-gray-400">
            <Clock className="h-4 w-4 mr-2" />
            <span>
              Next expiry: <strong>{data.nextExpiry.basePermission}</strong> in{' '}
              <strong>{formatTimeRemaining(data.nextExpiry.timeRemaining)}</strong>
            </span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

function CompactVariant({
  data,
  size,
  className
}: VariantProps) {
  const overallUrgency = getOverallUrgency(data);
  const { icon: Icon } = getUrgencyConfig(overallUrgency);

  return (
    <div className={`inline-flex items-center text-sm ${className}`}>
      <Icon className={`${getIconSize(size)} mr-2 ${getUrgencyIconColor(overallUrgency)}`} />
      <span className="font-medium mr-2">{data.totalCount}</span>
      {data.expiredCount > 0 && (
        <Badge variant="destructive" className="text-xs px-1 py-0 h-5">
          {data.expiredCount}
        </Badge>
      )}
      {data.criticalCount > 0 && (
        <Badge variant="secondary" className="text-xs px-1 py-0 h-5 ml-1 bg-red-100 text-red-800">
          {data.criticalCount}
        </Badge>
      )}
      {data.expiringSoonCount > 0 && (
        <Badge variant="outline" className="text-xs px-1 py-0 h-5 ml-1 bg-yellow-100 text-yellow-800">
          {data.expiringSoonCount}
        </Badge>
      )}
    </div>
  );
}

function DetailedVariant({
  data,
  showActions,
  className,
  onExtendPermission,
  onRevokePermission,
  onViewDetails
}: VariantProps) {
  return (
    <div className={`space-y-4 ${className}`}>
      <PermissionHealthDisplay data={data} />
      <Separator />
      <ExpiryDetails
        data={data}
        showActions={showActions}
        onExtendPermission={onExtendPermission}
        onRevokePermission={onRevokePermission}
        onViewDetails={onViewDetails}
      />
    </div>
  );
}

function DashboardVariant({
  data,
  showHealth,
  className
}: VariantProps) {
  return (
    <div className={`grid grid-cols-2 md:grid-cols-4 gap-4 ${className}`}>
      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-2">
            <CheckCircle className="h-5 w-5 text-green-500" />
            <div>
              <p className="text-2xl font-bold">{data.totalCount - data.expiredCount}</p>
              <p className="text-sm text-muted-foreground">Active</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-yellow-500" />
            <div>
              <p className="text-2xl font-bold">{data.expiringSoonCount}</p>
              <p className="text-sm text-muted-foreground">Expiring</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-2">
            <XCircle className="h-5 w-5 text-red-500" />
            <div>
              <p className="text-2xl font-bold">{data.expiredCount}</p>
              <p className="text-sm text-muted-foreground">Expired</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardContent className="p-4">
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-blue-500" />
            <div>
              <p className="text-2xl font-bold">{data.permanentCount}</p>
              <p className="text-sm text-muted-foreground">Permanent</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {showHealth && (
        <Card className="col-span-2 md:col-span-4">
          <CardContent className="p-4">
            <PermissionHealthDisplay data={data} />
          </CardContent>
        </Card>
      )}
    </div>
  );
}

function getVariantComponent(variant: string) {
  const components = {
    badge: BadgeVariant,
    inline: InlineVariant,
    full: FullVariant,
    banner: BannerVariant,
    card: CardVariant,
    compact: CompactVariant,
    detailed: DetailedVariant,
    dashboard: DashboardVariant
  };
  return components[variant as keyof typeof components] || BadgeVariant;
}

function getOverallUrgency(data: PermissionExpiryData): 'expired' | 'critical' | 'warning' | 'normal' | 'permanent' {
  if (data.expiredCount > 0) return 'expired';
  if (data.criticalCount > 0) return 'critical';
  if (data.expiringSoonCount > 0) return 'warning';
  if (data.totalCount > data.permanentCount) return 'normal';
  return 'permanent';
}

function getUrgencyConfig(urgency: string) {
  const configs = {
    expired: {
      colors: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
      bgColors: 'bg-red-50 border-red-200 text-red-800 dark:bg-red-900/20 dark:border-red-800 dark:text-red-200',
      icon: XCircle
    },
    critical: {
      colors: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
      bgColors: 'bg-red-50 border-red-200 text-red-800 dark:bg-red-900/20 dark:border-red-800 dark:text-red-200',
      icon: AlertCircle
    },
    warning: {
      colors: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
      bgColors: 'bg-yellow-50 border-yellow-200 text-yellow-800 dark:bg-yellow-900/20 dark:border-yellow-800 dark:text-yellow-200',
      icon: AlertTriangle
    },
    normal: {
      colors: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
      bgColors: 'bg-green-50 border-green-200 text-green-800 dark:bg-green-900/20 dark:border-green-800 dark:text-green-200',
      icon: CheckCircle
    },
    permanent: {
      colors: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
      bgColors: 'bg-blue-50 border-blue-200 text-blue-800 dark:bg-blue-900/20 dark:border-blue-800 dark:text-blue-200',
      icon: Shield
    }
  };
  return configs[urgency as keyof typeof configs] || configs.normal;
}

function getSizeClasses(size: string): string {
  return {
    xs: 'px-2 py-1 text-xs',
    sm: 'px-2.5 py-1.5 text-xs',
    md: 'px-3 py-2 text-sm',
    lg: 'px-4 py-2 text-base'
  }[size] || 'px-2.5 py-1.5 text-xs';
}

function getIconSize(size?: string): string {
  return {
    xs: 'h-3 w-3',
    sm: 'h-4 w-4',
    md: 'h-5 w-5',
    lg: 'h-6 w-6'
  }[size || 'sm'] || 'h-4 w-4';
}

function getTextColor(urgency: string): string {
  return {
    expired: 'text-red-600',
    critical: 'text-red-500',
    warning: 'text-yellow-600',
    normal: 'text-green-600',
    permanent: 'text-blue-600'
  }[urgency] || 'text-gray-600';
}

function getAlertColors(urgency: string): string {
  return {
    expired: 'border-red-300 bg-red-50',
    critical: 'border-red-300 bg-red-50',
    warning: 'border-yellow-300 bg-yellow-50',
    normal: 'border-green-300 bg-green-50',
    permanent: 'border-blue-300 bg-blue-50'
  }[urgency] || 'border-gray-300 bg-gray-50';
}

function getUrgencyIconColor(urgency: string): string {
  return {
    expired: 'text-red-500',
    critical: 'text-red-400',
    warning: 'text-yellow-500',
    normal: 'text-green-500',
    permanent: 'text-blue-500'
  }[urgency] || 'text-gray-400';
}

function getHealthTextColor(health: string): string {
  return {
    excellent: 'text-green-600',
    good: 'text-blue-600',
    warning: 'text-yellow-600',
    critical: 'text-red-600'
  }[health] || 'text-gray-600';
}

function getStatusText(data: PermissionExpiryData): string {
  if (data.expiredCount > 0) {
    return `${data.expiredCount} permission${data.expiredCount !== 1 ? 's' : ''} expired`;
  }
  if (data.expiringSoonCount > 0) {
    return `${data.expiringSoonCount} permission${data.expiringSoonCount !== 1 ? 's' : ''} expiring soon`;
  }
  return 'All permissions valid';
}

function formatTimeRemaining(seconds: number): string {
  if (seconds === Infinity) return 'Permanent';
  if (seconds <= 0) return 'Expired';

  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 7) return `${Math.floor(days / 7)}w ${days % 7}d`;
  if (days > 0) return `${days}d ${hours % 24}h`;
  if (hours > 0) return `${hours}h ${minutes % 60}m`;
  if (minutes > 0) return `${minutes}m`;
  return `${seconds}s`;
}

// ============================================================================
// HELPER COMPONENTS
// ============================================================================

function ExpiryDetails({
  data,
  compact = false,
  showActions = false,
  onExtendPermission,
  onRevokePermission,
  onViewDetails
}: {
  data: PermissionExpiryData;
  compact?: boolean;
  showActions?: boolean;
  onExtendPermission?: (permission: PermissionInfo) => void;
  onRevokePermission?: (permission: PermissionInfo) => void;
  onViewDetails?: (permission: PermissionInfo) => void;
}) {
  const expiredPermissions = data.permissions.filter(p => p.isExpired);
  const criticalPermissions = data.permissions.filter(p => p.urgencyLevel === 'critical');
  const warningPermissions = data.permissions.filter(p => p.urgencyLevel === 'warning');

  return (
    <div className="space-y-3">
      {expiredPermissions.length > 0 && (
        <div>
          <h4 className="font-medium text-red-800 dark:text-red-200 mb-2">
            Expired ({expiredPermissions.length})
          </h4>
          <div className="space-y-1">
            {expiredPermissions.slice(0, compact ? 3 : 10).map((permission, i) => (
              <PermissionItem
                key={i}
                permission={permission}
                showActions={showActions}
                onExtendPermission={onExtendPermission}
                onRevokePermission={onRevokePermission}
                onViewDetails={onViewDetails}
              />
            ))}
            {expiredPermissions.length > (compact ? 3 : 10) && (
              <p className="text-xs text-muted-foreground">
                ... and {expiredPermissions.length - (compact ? 3 : 10)} more
              </p>
            )}
          </div>
        </div>
      )}

      {criticalPermissions.length > 0 && (
        <div>
          <h4 className="font-medium text-red-600 dark:text-red-300 mb-2">
            Critical - Expiring Soon ({criticalPermissions.length})
          </h4>
          <div className="space-y-1">
            {criticalPermissions.slice(0, compact ? 3 : 10).map((permission, i) => (
              <PermissionItem
                key={i}
                permission={permission}
                showActions={showActions}
                onExtendPermission={onExtendPermission}
                onRevokePermission={onRevokePermission}
                onViewDetails={onViewDetails}
              />
            ))}
          </div>
        </div>
      )}

      {warningPermissions.length > 0 && (
        <div>
          <h4 className="font-medium text-yellow-700 dark:text-yellow-300 mb-2">
            Warning - Expiring ({warningPermissions.length})
          </h4>
          <div className="space-y-1">
            {warningPermissions.slice(0, compact ? 3 : 5).map((permission, i) => (
              <PermissionItem
                key={i}
                permission={permission}
                showActions={showActions}
                onExtendPermission={onExtendPermission}
                onRevokePermission={onRevokePermission}
                onViewDetails={onViewDetails}
              />
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

function PermissionItem({
  permission,
  showActions = false,
  onExtendPermission,
  onRevokePermission,
  onViewDetails
}: {
  permission: PermissionInfo;
  showActions?: boolean;
  onExtendPermission?: (permission: PermissionInfo) => void;
  onRevokePermission?: (permission: PermissionInfo) => void;
  onViewDetails?: (permission: PermissionInfo) => void;
}) {
  return (
    <div className="flex items-center justify-between text-xs bg-gray-50 dark:bg-gray-800 p-2 rounded">
      <div className="flex-1">
        <div className="font-mono text-gray-900 dark:text-gray-100">
          {permission.basePermission}
        </div>
        {permission.expiresAt && (
          <div className="text-muted-foreground">
            {permission.isExpired
              ? `Expired ${formatDistanceToNow(new Date(permission.expiresAt * 1000), { addSuffix: true })}`
              : `Expires ${formatTimeRemaining(permission.timeRemaining)}`
            }
          </div>
        )}
      </div>

      {showActions && (
        <div className="flex gap-1 ml-2">
          {onViewDetails && (
            <button
              onClick={() => onViewDetails(permission)}
              className="p-1 hover:bg-gray-200 dark:hover:bg-gray-700 rounded"
              title="View Details"
            >
              <Eye className="h-3 w-3" />
            </button>
          )}
          {onExtendPermission && !permission.isExpired && (
            <button
              onClick={() => onExtendPermission(permission)}
              className="p-1 hover:bg-blue-100 dark:hover:bg-blue-900 rounded text-blue-600"
              title="Extend Permission"
            >
              <Zap className="h-3 w-3" />
            </button>
          )}
          {onRevokePermission && (
            <button
              onClick={() => onRevokePermission(permission)}
              className="p-1 hover:bg-red-100 dark:hover:bg-red-900 rounded text-red-600"
              title="Revoke Permission"
            >
              <XCircle className="h-3 w-3" />
            </button>
          )}
        </div>
      )}
    </div>
  );
}

function PermissionHealthDisplay({ data }: { data: PermissionExpiryData }) {
  const healthPercentage = Math.round(((data.totalCount - data.expiredCount) / data.totalCount) * 100);

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
      <div>
        <span className="text-muted-foreground">Total:</span>
        <span className="ml-2 font-medium">{data.totalCount}</span>
      </div>
      <div>
        <span className="text-muted-foreground">Active:</span>
        <span className="ml-2 font-medium text-green-600">{data.totalCount - data.expiredCount}</span>
      </div>
      <div>
        <span className="text-muted-foreground">Expired:</span>
        <span className="ml-2 font-medium text-red-600">{data.expiredCount}</span>
      </div>
      <div>
        <span className="text-muted-foreground">Expiring:</span>
        <span className="ml-2 font-medium text-yellow-600">{data.expiringSoonCount}</span>
      </div>
      <div className="col-span-2 md:col-span-4 pt-2 border-t">
        <div className="flex items-center justify-between">
          <span className="text-muted-foreground">Health Score:</span>
          <div className="flex items-center gap-2">
            <span className={`font-medium capitalize ${getHealthTextColor(data.healthScore)}`}>
              {data.healthScore}
            </span>
            <span className="text-xs text-muted-foreground">({healthPercentage}%)</span>
          </div>
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// SHARED INTERFACES
// ============================================================================

interface VariantProps {
  platform: Platform;
  data: PermissionExpiryData;
  size?: string;
  showCountdown?: boolean;
  showDetails?: boolean;
  showHealth?: boolean;
  showActions?: boolean;
  className: string;
  isDialogOpen: boolean;
  setIsDialogOpen: (open: boolean) => void;
  onRefresh?: () => void;
  isRefreshing?: boolean;
  onExtendPermission?: (permission: PermissionInfo) => void;
  onRevokePermission?: (permission: PermissionInfo) => void;
  onViewDetails?: (permission: PermissionInfo) => void;
}

function PermissionDetailView({ data }: { data: PermissionExpiryData }) {
  return (
    <div className="space-y-4">
      {/* Permission Info */}
      <div className="space-y-2">
        {data.permissions.length === 1 && data.permissions[0] && (
          <>
            <div>
              <span className="text-sm font-medium">Permission:</span>
              <code className="ml-2 text-sm bg-muted px-2 py-1 rounded">
                {data.permissions[0].basePermission}
              </code>
            </div>

            <div className="flex items-center gap-2">
              <span className="text-sm font-medium">Source:</span>
              {getSourceBadge(data.permissions[0].source || 'Unknown')}
            </div>

            {data.permissions[0].grantedAt && (
              <div>
                <span className="text-sm font-medium">Granted:</span>
                <span className="ml-2 text-sm text-muted-foreground">
                  {formatDistanceToNow(new Date(data.permissions[0].grantedAt * 1000), { addSuffix: true })}
                </span>
              </div>
            )}
          </>
        )}

        {data.permissions.length > 1 && (
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div><span className="font-medium">Total:</span> {data.totalCount}</div>
            <div><span className="font-medium text-green-600">Active:</span> {data.totalCount - data.expiredCount}</div>
            <div><span className="font-medium text-red-600">Expired:</span> {data.expiredCount}</div>
            <div><span className="font-medium text-yellow-600">Expiring:</span> {data.expiringSoonCount}</div>
          </div>
        )}
      </div>

      {/* Expiry Status */}
      <div className="p-3 border rounded-lg">
        <div className="flex items-center justify-between mb-2">
          <span className="font-medium">Status</span>
          <Badge
            variant={data.expiredCount > 0 ? 'destructive' : data.criticalCount > 0 ? 'destructive' : 'default'}
            className={
              data.expiredCount > 0 ? 'bg-red-100 text-red-800' :
                data.criticalCount > 0 ? 'bg-yellow-100 text-yellow-800' :
                  'bg-green-100 text-green-800'
            }
          >
            {data.expiredCount > 0 ? 'Issues Found' : data.criticalCount > 0 ? 'Needs Attention' : 'Healthy'}
          </Badge>
        </div>

        <div className="text-sm space-y-1">
          <div>Health Score: <span className={getHealthTextColor(data.healthScore)}>{data.healthScore}</span></div>
          {data.nextExpiry && (
            <div className="text-muted-foreground">
              Next expiry: {formatTimeRemaining(data.nextExpiry.timeRemaining)}
            </div>
          )}
        </div>
      </div>

      {/* Warning Messages */}
      {data.expiredCount > 0 && (
        <Alert variant="destructive">
          <AlertTriangle className="h-4 w-4" />
          <AlertDescription>
            {data.expiredCount} permission{data.expiredCount !== 1 ? 's have' : ' has'} expired. Some features may not be available until renewed.
          </AlertDescription>
        </Alert>
      )}

      {data.criticalCount > 0 && data.expiredCount === 0 && (
        <Alert>
          <Clock className="h-4 w-4" />
          <AlertDescription>
            {data.criticalCount} permission{data.criticalCount !== 1 ? 's' : ''} will expire soon. Consider renewing to maintain access.
          </AlertDescription>
        </Alert>
      )}
    </div>
  );
}

function getSourceBadge(source: string) {
  const colors: Record<string, string> = {
    Subscription: 'bg-green-100 text-green-800',
    Trial: 'bg-purple-100 text-purple-800',
    Admin: 'bg-blue-100 text-blue-800',
    System: 'bg-indigo-100 text-indigo-800',
    Legacy: 'bg-gray-100 text-gray-800'
  };

  return (
    <Badge variant="outline" className={colors[source] || 'bg-gray-100 text-gray-800'}>
      {source}
    </Badge>
  );
}

// ============================================================================
// PLATFORM INTEGRATION UTILITIES
// ============================================================================

export function registerPermissionHook(platform: Platform, hook: () => PermissionHookInterface): void {
  if (typeof window !== 'undefined') {
    (window as any).__PERMISSION_HOOK_REGISTRY = (window as any).__PERMISSION_HOOK_REGISTRY || {};
    (window as any).__PERMISSION_HOOK_REGISTRY[platform] = hook;
  }
}

export function registerUIComponents(components: Record<string, React.ComponentType<any>>): void {
  if (typeof window !== 'undefined') {
    (window as any).__UI_COMPONENT_REGISTRY = components;
  }
}