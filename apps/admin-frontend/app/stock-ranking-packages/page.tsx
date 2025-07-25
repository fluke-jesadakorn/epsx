import React from 'react';
import { Metadata } from 'next';
import StockRankingPackageDashboard from '@/components/admin/StockRankingPackageDashboard';

export const metadata: Metadata = {
  title: 'Stock Ranking Packages | EPSX Admin',
  description: 'Assign and manage stock ranking access packages for users',
};

export default function StockRankingPackagesPage() {
  return <StockRankingPackageDashboard />;
}