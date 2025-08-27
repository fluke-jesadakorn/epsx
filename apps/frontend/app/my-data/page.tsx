import MyDataClientWrapper from '@/components/my-data/MyDataClientWrapper';
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'My Data - EPSX Analytics',
  description: 'Track and analyze your portfolio with professional-grade analytics',
};

// Server-side rendering for better performance
export const dynamic = 'force-dynamic';

export default async function MyDataPage() {
  // For now, we'll pass an empty array since we don't have user data yet
  // In a real implementation, we would fetch the user's assets from a database
  const initialAssets: { symbol: string; name: string }[] = [];
  
  return <MyDataClientWrapper initialAssets={initialAssets} />;
}