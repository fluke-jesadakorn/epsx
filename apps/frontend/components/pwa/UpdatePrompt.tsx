'use client';

import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { 
  RefreshCw, 
  X, 
  AlertCircle,
  CheckCircle,
  Download
} from 'lucide-react';

interface UpdatePromptProps {
  onUpdate?: () => void;
  onDismiss?: () => void;
}

export function UpdatePrompt({ onUpdate, onDismiss }: UpdatePromptProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [isUpdating, setIsUpdating] = useState(false);
  const [registration, setRegistration] = useState<ServiceWorkerRegistration | null>(null);

  useEffect(() => {
    const handleUpdateAvailable = (event: CustomEvent) => {
      setRegistration(event.detail.registration);
      setIsVisible(true);
    };

    window.addEventListener('pwa:updateAvailable', handleUpdateAvailable as EventListener);

    return () => {
      window.removeEventListener('pwa:updateAvailable', handleUpdateAvailable as EventListener);
    };
  }, []);

  const handleUpdate = async () => {
    if (!registration || !registration.waiting) {
      return;
    }

    setIsUpdating(true);

    try {
      // Tell the waiting service worker to skip waiting
      registration.waiting.postMessage({ type: 'SKIP_WAITING' });

      // Listen for the controlling change
      navigator.serviceWorker.addEventListener('controllerchange', () => {
        // Reload the page to use the new service worker
        window.location.reload();
      });

      onUpdate?.();
    } catch (error) {
      console.error('Update failed:', error);
      setIsUpdating(false);
    }
  };

  const handleDismiss = () => {
    setIsVisible(false);
    onDismiss?.();
  };

  if (!isVisible) {
    return null;
  }

  return (
    <Card className="fixed top-4 left-4 right-4 z-50 mx-auto max-w-md shadow-lg border-blue-500/20 bg-blue-50 dark:bg-blue-950 md:left-auto md:right-4 md:max-w-sm">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-blue-500/10 flex items-center justify-center">
              <Download className="h-4 w-4 text-blue-600" />
            </div>
            <CardTitle className="text-lg text-blue-900 dark:text-blue-100">
              Update Available
            </CardTitle>
          </div>
          <Button 
            variant="ghost" 
            size="sm" 
            onClick={handleDismiss}
            className="p-1 h-6 w-6 hover:bg-blue-100 dark:hover:bg-blue-800"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      </CardHeader>
      
      <CardContent className="space-y-4">
        <div className="flex items-start gap-3">
          <AlertCircle className="h-5 w-5 text-blue-600 mt-0.5 flex-shrink-0" />
          <div className="space-y-1">
            <p className="text-sm font-medium text-blue-900 dark:text-blue-100">
              New version available
            </p>
            <p className="text-xs text-blue-700 dark:text-blue-300">
              A new version of EPSX is ready to install with improvements and bug fixes.
            </p>
          </div>
        </div>
        
        <div className="bg-blue-100 dark:bg-blue-900 rounded-lg p-3">
          <div className="flex items-center gap-2 text-xs text-blue-800 dark:text-blue-200">
            <CheckCircle className="h-3 w-3" />
            <span>Performance improvements</span>
          </div>
          <div className="flex items-center gap-2 text-xs text-blue-800 dark:text-blue-200 mt-1">
            <CheckCircle className="h-3 w-3" />
            <span>Bug fixes and stability</span>
          </div>
          <div className="flex items-center gap-2 text-xs text-blue-800 dark:text-blue-200 mt-1">
            <CheckCircle className="h-3 w-3" />
            <span>New features</span>
          </div>
        </div>
        
        <div className="flex gap-2">
          <Button 
            onClick={handleUpdate}
            disabled={isUpdating}
            className="flex-1 bg-blue-600 hover:bg-blue-700"
            size="sm"
          >
            {isUpdating ? (
              <>
                <RefreshCw className="h-3 w-3 mr-2 animate-spin" />
                Updating...
              </>
            ) : (
              <>
                <RefreshCw className="h-3 w-3 mr-2" />
                Update Now
              </>
            )}
          </Button>
          <Button 
            variant="outline" 
            onClick={handleDismiss}
            size="sm"
            className="border-blue-200 text-blue-700 hover:bg-blue-100 dark:border-blue-700 dark:text-blue-300 dark:hover:bg-blue-800"
          >
            Later
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}