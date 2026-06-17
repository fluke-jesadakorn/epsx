'use client';

import React from 'react';
import { WifiOff, RefreshCw, Home, Bell } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import Link from 'next/link';

export default function OfflinePage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 dark:from-slate-900 dark:to-slate-800 flex items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardContent className="pt-8 pb-8 text-center space-y-6">
          {/* Offline Icon */}
          <div className="flex justify-center">
            <div className="w-20 h-20 bg-orange-100 dark:bg-orange-900/20 rounded-full flex items-center justify-center">
              <WifiOff className="w-10 h-10 text-orange-600 dark:text-orange-400" />
            </div>
          </div>

          {/* Title and Message */}
          <div className="space-y-2">
            <h1 className="text-2xl font-bold text-slate-900 dark:text-slate-100">
              You're Offline
            </h1>
            <p className="text-slate-600 dark:text-slate-400">
              Please check your internet connection and try again.
            </p>
          </div>

          {/* Offline Features */}
          <div className="bg-slate-50 dark:bg-slate-800/50 rounded-lg p-4 text-left">
            <h3 className="text-sm font-medium text-slate-900 dark:text-slate-100 mb-3">
              Available Offline:
            </h3>
            <ul className="text-sm text-slate-600 dark:text-slate-400 space-y-2">
              <li className="flex items-center gap-2">
                <div className="w-2 h-2 bg-green-500 rounded-full" />
                View cached notifications
              </li>
              <li className="flex items-center gap-2">
                <div className="w-2 h-2 bg-green-500 rounded-full" />
                Browse previously loaded analytics
              </li>
              <li className="flex items-center gap-2">
                <div className="w-2 h-2 bg-green-500 rounded-full" />
                Access user settings
              </li>
              <li className="flex items-center gap-2">
                <div className="w-2 h-2 bg-orange-500 rounded-full" />
                Limited: Real-time data and trading
              </li>
            </ul>
          </div>

          {/* Action Buttons */}
          <div className="space-y-3">
            <Button
              onClick={() => window.location.reload()}
              className="w-full flex items-center gap-2"
            >
              <RefreshCw className="w-4 h-4" />
              Try Again
            </Button>
            
            <div className="flex gap-2">
              <Button variant="outline" asChild className="flex-1">
                <Link href="/" className="flex items-center gap-2">
                  <Home className="w-4 h-4" />
                  Home
                </Link>
              </Button>
              
              <Button variant="outline" asChild className="flex-1">
                <Link href="/notifications" className="flex items-center gap-2">
                  <Bell className="w-4 h-4" />
                  Notifications
                </Link>
              </Button>
            </div>
          </div>

          {/* Tips */}
          <div className="text-xs text-slate-500 dark:text-slate-500 border-t pt-4">
            <p className="font-medium mb-1">Tip:</p>
            <p>
              This app works offline with limited functionality. 
              Your data will sync when you're back online.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}