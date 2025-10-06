import { Metadata } from 'next';
import React from 'react';

import StockRankingPackageDashboard from '@/components/admin/StockPackageDash';

export const metadata: Metadata = {
  title: 'Stock Ranking Packages | EPSX Admin',
  description: 'Assign and manage stock ranking access packages for users',
};

/**
 *
 */
export default async function StockRankingPackagesPage() {
  return (
    <StockRankingPackageDashboard />
  );
}