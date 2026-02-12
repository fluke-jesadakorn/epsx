import type { Metadata } from 'next';
import { CreditsPageClient } from './credits-page-client';

export const metadata: Metadata = {
  title: 'Credit Balance | EPSX',
  description: 'View your EPSX credit balance and transaction history',
};

export default function CreditsPage() {
  return <CreditsPageClient />;
}
