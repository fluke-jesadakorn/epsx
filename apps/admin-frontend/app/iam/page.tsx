'use client';

import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { Shield, Construction } from 'lucide-react';

export default function IAMPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <div className="p-6">
          <div className="max-w-2xl mx-auto text-center">
            <div className="pancake-card pancake-card-hover p-8">
              <div className="flex items-center justify-center mb-6">
                <div className="p-4 rounded-full bg-gradient-to-r from-orange-500 to-yellow-500">
                  <Construction className="h-10 w-10 text-white" />
                </div>
              </div>
              <h1 className="text-3xl font-bold mb-4 bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent">
                IAM Dashboard
              </h1>
              <p className="text-muted-foreground mb-6">
                Identity and Access Management features are currently under development.
              </p>
              <div className="flex items-center justify-center gap-2 text-sm text-muted-foreground">
                <Shield className="h-4 w-4" />
                <span>Enhanced IAM functionality coming soon</span>
              </div>
            </div>
          </div>
        </div>
      </AdminLayout>
    </AdminGuard>
  );
}
