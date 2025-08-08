import { AlertCircle, Shield, ArrowLeft } from 'lucide-react';
import { Button } from '@/components/ui/button';
import Link from 'next/link';

interface AccessDeniedProps {
  reason?: string;
  requiredPermissions?: string[];
  showBackButton?: boolean;
  className?: string;
}

/**
 * Server-rendered access denied page
 * Clean, simple design without client-side logic
 */
export function AccessDenied({
  reason = 'You do not have permission to access this page',
  requiredPermissions = [],
  showBackButton = true,
  className = '',
}: AccessDeniedProps) {
  return (
    <div className={`min-h-screen flex items-center justify-center bg-background ${className}`}>
      <div className="max-w-md w-full mx-4">
        <div className="text-center space-y-6">
          {/* Icon */}
          <div className="flex justify-center">
            <div className="rounded-full bg-red-100 dark:bg-red-900/20 p-6">
              <Shield className="h-12 w-12 text-red-600 dark:text-red-400" />
            </div>
          </div>
          
          {/* Title */}
          <div className="space-y-2">
            <h1 className="text-2xl font-bold text-foreground">Access Denied</h1>
            <p className="text-muted-foreground">{reason}</p>
          </div>
          
          {/* Required Permissions */}
          {requiredPermissions.length > 0 && (
            <div className="bg-muted rounded-lg p-4">
              <div className="flex items-center gap-2 mb-2">
                <AlertCircle className="h-4 w-4 text-muted-foreground" />
                <span className="text-sm font-medium text-muted-foreground">
                  Required Permissions:
                </span>
              </div>
              <div className="space-y-1">
                {requiredPermissions.map((permission) => (
                  <div key={permission} className="text-xs font-mono bg-background px-2 py-1 rounded">
                    {permission}
                  </div>
                ))}
              </div>
            </div>
          )}
          
          {/* Actions */}
          <div className="flex flex-col sm:flex-row gap-3 justify-center">
            {showBackButton && (
              <Button variant="outline" asChild>
                <Link href="/">
                  <ArrowLeft className="h-4 w-4 mr-2" />
                  Go Home
                </Link>
              </Button>
            )}
            <Button asChild>
              <Link href="/contact">
                Request Access
              </Link>
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}