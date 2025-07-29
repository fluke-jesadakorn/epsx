"use client";

import { useState } from "react";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { useAuth } from "@/context/auth-context";
import { useTheme, ThemeVariantSelector, DarkModeToggle } from "@epsx/ui";

export function AuthProviders() {
  const { user } = useAuth();
  const [isLoading, setIsLoading] = useState<boolean>(false);

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-semibold mb-4">Theme & Appearance</h3>
        <div className="space-y-4">
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 bg-purple-100 rounded-full flex items-center justify-center">
                🎨
              </div>
              <div>
                <p className="font-medium">Theme Variant</p>
                <p className="text-sm text-muted-foreground">Choose your preferred theme style</p>
              </div>
            </div>
            <ThemeVariantSelector />
          </div>
          
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
                🌙
              </div>
              <div>
                <p className="font-medium">Dark Mode</p>
                <p className="text-sm text-muted-foreground">Toggle between light and dark appearance</p>
              </div>
            </div>
            <DarkModeToggle showLabel />
          </div>
        </div>
      </div>

      <div>
        <h3 className="text-lg font-semibold mb-4">Connected Accounts</h3>
      
        <Alert>
        <AlertDescription>
          Third-party provider integration (Google, Apple, etc.) will be available in a future update. 
          Currently using secure backend authentication.
        </AlertDescription>
        </Alert>

        <div className="space-y-3">
        <div className="flex items-center justify-between p-4 border rounded-lg">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
              📧
            </div>
            <div>
              <p className="font-medium">Email Authentication</p>
              <p className="text-sm text-muted-foreground">{user?.email}</p>
            </div>
          </div>
          <span className="px-2 py-1 bg-green-100 text-green-800 text-xs rounded-full">
            Connected
          </span>
        </div>

        <div className="flex items-center justify-between p-4 border rounded-lg opacity-50">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-red-100 rounded-full flex items-center justify-center">
              🔴
            </div>
            <div>
              <p className="font-medium">Google</p>
              <p className="text-sm text-muted-foreground">Connect your Google account</p>
            </div>
          </div>
          <Button 
            variant="outline" 
            size="sm" 
            disabled
          >
            Coming Soon
          </Button>
        </div>

        <div className="flex items-center justify-between p-4 border rounded-lg opacity-50">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
              🍎
            </div>
            <div>
              <p className="font-medium">Apple</p>
              <p className="text-sm text-muted-foreground">Connect your Apple ID</p>
            </div>
          </div>
          <Button 
            variant="outline" 
            size="sm" 
            disabled
          >
            Coming Soon
          </Button>
        </div>
      </div>
    </div>
    </div>
  );
}