import { Metadata } from 'next';
import { ApiKeyRequestForm } from '@/components/public/ApiKeyRequestForm';

export const metadata: Metadata = {
  title: 'Request API Access | EPSX',
  description: 'Request access to the EPSX API platform for financial data integration',
};

export default function RequestAccessPage() {
  return (
    <div className="min-h-screen bg-gray-50 py-8">
      <ApiKeyRequestForm />
    </div>
  );
}
