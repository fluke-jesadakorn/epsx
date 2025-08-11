'use client';

import React from 'react';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Grid3X3 } from 'lucide-react';
import { EnhancedAnalyticsRankingDashboard } from './EnhancedAnalyticsRankingDashboard';

interface AuthUser {
  user_id: string;
  email: string;
  role: string;
  permissions: string[];
  subscription_tier: string;
}

interface AnalyticsRankingDashboardProps {
  user?: AuthUser | null;
}

export function AnalyticsRankingDashboard({ user }: AnalyticsRankingDashboardProps) {
  return (
    <div className="space-y-4">
      {/* Analytics Header */}
      <Card className="bg-gradient-to-r from-purple-50 to-indigo-50 dark:from-purple-950/20 dark:to-indigo-950/20 border-purple-200">
        <CardContent className="p-4">
          <div className="flex items-center gap-2">
            <Grid3X3 className="h-5 w-5 text-purple-600" />
            <span className="font-semibold text-purple-800 dark:text-purple-200">
              Analytics Dashboard
            </span>
            <Badge variant="secondary" className="bg-purple-100 text-purple-800">
              Advanced Filters • Page Pagination • Full Featured
            </Badge>
          </div>
        </CardContent>
      </Card>
      
      <EnhancedAnalyticsRankingDashboard user={user} />
    </div>
  );
}
