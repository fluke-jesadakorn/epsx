'use client';

import { Loader2 } from 'lucide-react';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import type { PaymentLoadingState } from '@/app/constants/packages';

interface Props {
  state: PaymentLoadingState;
  children: React.ReactNode;
}

export function PaymentLoading({ state, children }: Props) {
  if (state.state === 'loading') {
    return (
      <Card className="w-full max-w-md mx-auto">
        <CardHeader>
          <div className="flex items-center justify-center">
            <Loader2 className="h-8 w-8 animate-spin text-primary" />
          </div>
        </CardHeader>
        <CardContent className="text-center">
          <p className="text-sm text-muted-foreground">Processing payment...</p>
        </CardContent>
      </Card>
    );
  }

  return <>{children}</>;
}
