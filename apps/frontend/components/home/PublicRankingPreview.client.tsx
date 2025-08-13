'use client';

import React from 'react';
import { Card, CardContent } from '@epsx/ui';
import { Button } from '@epsx/ui';
import { Badge } from '@epsx/ui';
import { Crown, Lock, ArrowRight } from 'lucide-react';
import { useRouter } from 'next/navigation';

export function PublicRankingPreviewClient() {
  const router = useRouter();

  const handleUpgrade = () => {
    router.push('/payment');
  };

  const handleViewMore = () => {
    router.push('/analytics');
  };

  return (
    <div className="relative">
      <Card className="border-2 border-dashed border-yellow-300 bg-gradient-to-br from-yellow-50 to-orange-50 dark:from-yellow-950/20 dark:to-orange-950/20">
        <CardContent className="p-8 text-center">
          <div className="space-y-6">
            <div className="flex justify-center">
              <div className="relative">
                <Crown className="h-16 w-16 text-yellow-500" />
                <Lock className="h-6 w-6 text-gray-600 absolute -top-1 -right-1 bg-white rounded-full p-1" />
              </div>
            </div>

            <div>
              <h3 className="text-2xl font-bold mb-3">
                🚀 Access Top 100 Rankings
              </h3>
              <p className="text-lg text-muted-foreground mb-2">
                You&apos;re seeing rankings #100-110. Unlock the top performers!
              </p>
              <div className="flex flex-wrap justify-center gap-2 text-sm">
                <Badge variant="secondary">✨ Top 100 Entities</Badge>
                <Badge variant="secondary">📊 Advanced Analytics</Badge>
                <Badge variant="secondary">📈 Growth Insights</Badge>
                <Badge variant="secondary">🎯 Performance Optimization</Badge>
              </div>
            </div>

            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <Button size="lg" onClick={handleUpgrade} className="gap-2">
                <Crown className="h-5 w-5" />
                Upgrade to Premium
                <ArrowRight className="h-4 w-4" />
              </Button>
              <Button
                size="lg"
                variant="outline"
                onClick={handleViewMore}
                className="gap-2"
              >
                View Analytics Demo
                <ArrowRight className="h-4 w-4" />
              </Button>
            </div>

            <div className="text-xs text-muted-foreground">
              Starting from $1/month • 30-day money-back guarantee
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}