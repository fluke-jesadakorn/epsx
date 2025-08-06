import type { Metadata } from 'next';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ShieldX, ArrowLeft, Home } from 'lucide-react';
import Link from 'next/link';

export const metadata: Metadata = {
  title: 'Unauthorized - EPSX',
  description: 'You do not have permission to access this page',
};

export default function UnauthorizedPage() {
  return (
    <div className="min-h-screen bg-gradient-to-br from-red-50 via-orange-50 to-yellow-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900 flex items-center justify-center p-4">
      <Card className="w-full max-w-md text-center">
        <CardHeader>
          <div className="mx-auto w-16 h-16 bg-red-100 dark:bg-red-900/20 rounded-full flex items-center justify-center mb-4">
            <ShieldX className="h-8 w-8 text-red-600 dark:text-red-400" />
          </div>
          <CardTitle className="text-2xl font-bold text-red-800 dark:text-red-300">
            Access Denied
          </CardTitle>
          <CardDescription>
            You don&apos;t have permission to access this page
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-6">
          <div className="text-sm text-muted-foreground">
            <p>
              This page requires special permissions that your account doesn&apos;t
              have.
            </p>
            <p className="mt-2">
              If you believe this is an error, please contact an administrator.
            </p>
          </div>

          <div className="space-y-3">
            <Link href="/dashboard">
              <Button className="w-full">
                <Home className="mr-2 h-4 w-4" />
                Go to Dashboard
              </Button>
            </Link>

            <Link href="/">
              <Button variant="outline" className="w-full">
                <ArrowLeft className="mr-2 h-4 w-4" />
                Back to Home
              </Button>
            </Link>
          </div>

          <div className="text-xs text-muted-foreground">
            <p>Need admin access? Contact your system administrator.</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
