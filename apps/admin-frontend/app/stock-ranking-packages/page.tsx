import React from 'react';
import { Metadata } from 'next';
import { AdminLayout } from '@/components/layout/AdminLayout';
import StockRankingPackageDashboard from '@/components/admin/StockRankingPackageDashboard';

export const metadata: Metadata = {
  title: 'Stock Ranking Packages | EPSX Admin',
  description: 'Assign and manage stock ranking access packages for users',
};

export default async function StockRankingPackagesPage() {
  return (
    <AdminLayout>
      <StockRankingPackageDashboard />
    </AdminLayout>
  );
}