'use client';

interface TotalProps {
  pkg: string;
  amt: string;
  pay: string;
  go: () => void;
}

export function Total({ pkg, amt, pay, go }: TotalProps) {
  const planName = pkg === 'silver' ? 'Silver' : pkg === 'gold' ? 'Gold' : pkg === 'plat' ? 'Platinum' : '';
  const payName = pay.includes('USDT') ? pay.replace('_', ' ') : 'Credit Card';
  
  return (
    <div className="p-6 bg-white rounded-lg shadow-sm border-t-4 border-blue-500">
      <h2 className="text-lg font-semibold mb-4">Order Summary</h2>
      
      <div className="space-y-3 mb-6">
        <div className="flex justify-between">
          <span className="text-gray-600">Plan:</span>
          <span className="font-medium">{planName || 'None selected'}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-gray-600">Payment:</span>
          <span className="font-medium">{payName}</span>
        </div>
        <div className="pt-3 border-t flex justify-between items-center">
          <span className="text-lg font-semibold">Total:</span>
          <span className="text-2xl font-bold text-blue-600">
            {amt ? `$${amt}` : '$0'}
          </span>
        </div>
      </div>
      
      <button 
        onClick={go}
        disabled={!pkg || !amt}
        className="w-full py-3 bg-blue-500 text-white font-semibold rounded-lg disabled:opacity-50 disabled:cursor-not-allowed hover:bg-blue-600 transition-colors"
      >
        {pay.includes('USDT') ? 'Continue to Payment' : 'Pay Now'}
      </button>
    </div>
  );
}
