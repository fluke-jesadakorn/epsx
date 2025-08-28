import MyDataClientWrapper from '@/components/my-data/MyDataClientWrapper';
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'My Data - EPSX Analytics',
  description: 'Track and analyze your portfolio with professional-grade analytics',
};

// Server-side rendering for better performance
export const dynamic = 'force-dynamic';

export default async function MyDataPage() {
  // In a real implementation, we would fetch the user's portfolio data from a database
  // For now, we're using mock data that's defined in the client component
  
  return <MyDataClientWrapper />;
}