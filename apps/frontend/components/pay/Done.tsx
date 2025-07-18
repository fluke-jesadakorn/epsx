'use client';

import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

export function Done() {
  const router = useRouter();
  
  useEffect(() => {
    setTimeout(() => {
      router.push('/dashboard?payment=success');
    }, 3000);
  }, [router]);
  
  return (
    <div className="max-w-md mx-auto p-8 bg-white rounded-lg shadow-sm text-center">
      <div className="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
        <svg className="w-8 h-8 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
        </svg>
      </div>
      
      <h2 className="text-2xl font-bold text-gray-900 mb-2">
        Payment Successful!
      </h2>
      
      <p className="text-gray-600 mb-6">
        Your plan has been activated.
      </p>
      
      <p className="text-sm text-gray-500">
        Redirecting to my data...
      </p>
    </div>
  );
}
