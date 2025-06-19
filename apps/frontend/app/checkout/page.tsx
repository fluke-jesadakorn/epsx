'use client';

import { useEffect, useState } from 'react';
import AssetSelection from './components/AssetSelection';
import CheckoutForm from './components/CheckoutForm';
import { useAuth } from '@/context/auth-context';
import { useRouter } from 'next/navigation';

export default function CheckoutPage() {
  const { user, loading } = useAuth();
  const router = useRouter();
  const [selectedAsset, setSelectedAsset] = useState('');

  const isLoggedIn = !!user;
  const userEmail = user?.email;

  useEffect(() => {
    if (!loading && !isLoggedIn) {
      router.push('/login');
    }
  }, [loading, isLoggedIn, router]);

  if (loading) {
    return <div>Loading...</div>;
  }

  if (!isLoggedIn || !userEmail) {
    return null;
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold mb-6">Checkout</h1>
      <div className="max-w-2xl mx-auto">
        <AssetSelection
          selectedAsset={selectedAsset}
          onSelectAction={setSelectedAsset}
        />
        {selectedAsset && <CheckoutForm currency={selectedAsset} />}
      </div>
    </div>
  );
}
