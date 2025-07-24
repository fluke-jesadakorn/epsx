'use client';

import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { pwaManager } from '@/lib/pwa';
import { 
  Download, 
  X, 
  Smartphone, 
  Monitor, 
  Zap,
  Bell,
  Wifi,
  Shield
} from 'lucide-react';

interface InstallPromptProps {
  onInstall?: () => void;
  onDismiss?: () => void;
}

export function InstallPrompt({ onInstall, onDismiss }: InstallPromptProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [isInstalling, setIsInstalling] = useState(false);
  const [isStandalone, setIsStandalone] = useState(false);

  useEffect(() => {
    const checkInstallability = () => {
      const canInstall = pwaManager.canInstall();
      const standalone = pwaManager.isRunningStandalone();
      
      setIsVisible(canInstall && !standalone);
      setIsStandalone(standalone);
    };

    checkInstallability();

    const handleInstallAvailable = () => {
      setIsVisible(true);
    };

    const handleInstalled = () => {
      setIsVisible(false);
      onInstall?.();
    };

    window.addEventListener('pwa:installAvailable', handleInstallAvailable);
    window.addEventListener('pwa:installed', handleInstalled);

    return () => {
      window.removeEventListener('pwa:installAvailable', handleInstallAvailable);
      window.removeEventListener('pwa:installed', handleInstalled);
    };
  }, [onInstall]);

  const handleInstall = async () => {
    setIsInstalling(true);
    
    try {
      const success = await pwaManager.install();
      if (success) {
        setIsVisible(false);
        onInstall?.();
      }
    } catch (error) {
      console.error('Installation failed:', error);
    } finally {
      setIsInstalling(false);
    }
  };

  const handleDismiss = () => {
    setIsVisible(false);
    onDismiss?.();
  };

  if (!isVisible || isStandalone) {
    return null;
  }

  return (
    <Card className="fixed bottom-4 left-4 right-4 z-50 mx-auto max-w-md shadow-lg border-primary/20 md:left-auto md:right-4 md:bottom-4 md:max-w-sm">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center">
              <Download className="h-4 w-4 text-primary" />
            </div>
            <CardTitle className="text-lg">Install EPSX</CardTitle>
          </div>
          <Button 
            variant="ghost" 
            size="sm" 
            onClick={handleDismiss}
            className="p-1 h-6 w-6"
          >
            <X className="h-4 w-4" />
          </Button>
        </div>
      </CardHeader>
      
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">
          Install EPSX for the best experience with faster loading, offline access, and native features.
        </p>
        
        <div className="grid grid-cols-2 gap-3">
          <div className="flex items-center gap-2 text-xs">
            <Zap className="h-3 w-3 text-green-600" />
            <span>Faster loading</span>
          </div>
          <div className="flex items-center gap-2 text-xs">
            <Wifi className="h-3 w-3 text-blue-600" />
            <span>Offline access</span>
          </div>
          <div className="flex items-center gap-2 text-xs">
            <Bell className="h-3 w-3 text-orange-600" />
            <span>Push notifications</span>
          </div>
          <div className="flex items-center gap-2 text-xs">
            <Shield className="h-3 w-3 text-purple-600" />
            <span>Secure & private</span>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          <Badge variant="outline" className="text-xs">
            <Smartphone className="h-3 w-3 mr-1" />
            Mobile
          </Badge>
          <Badge variant="outline" className="text-xs">
            <Monitor className="h-3 w-3 mr-1" />
            Desktop
          </Badge>
        </div>
        
        <div className="flex gap-2">
          <Button 
            onClick={handleInstall}
            disabled={isInstalling}
            className="flex-1"
            size="sm"
          >
            {isInstalling ? (
              <>
                <div className="animate-spin rounded-full h-3 w-3 border-2 border-white border-t-transparent mr-2" />
                Installing...
              </>
            ) : (
              <>
                <Download className="h-3 w-3 mr-2" />
                Install
              </>
            )}
          </Button>
          <Button 
            variant="outline" 
            onClick={handleDismiss}
            size="sm"
          >
            Later
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}