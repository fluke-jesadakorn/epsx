'use client';

import { useState } from 'react';
import { Plans } from './Plans';
import { Methods } from './Methods';
import { Total } from './Total';
import { QR } from './QR';
import { Done } from './Done';

interface PayProps {
  pkg?: string;
  amt?: string;
}

export function Pay({ pkg = '', amt = '' }: PayProps) {
  const [step, setStep] = useState<'pick' | 'qr' | 'done'>('pick');
  const [data, setData] = useState({
    pkg,
    amt,
    pay: 'USDT_TRC20',
  });
  const [deposit, setDeposit] = useState<{
    address: string;
    currency: string;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleGo = async () => {
    if (!data.pkg || !data.amt || !data.pay) return;
    setLoading(true);
    setError(null);
    try {
      // Replace with actual userId from auth context if available
      const userId =
        (typeof window !== 'undefined' &&
          window.localStorage.getItem('userId')) ||
        'demo-user';
      const res = await fetch('/api/v1/payments/crypto/deposit-address', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          currency: data.pay,
          userId,
          packageId: data.pkg,
        }),
      });
      const result = await res.json();
      if (!res.ok || !result.deposit)
        throw new Error(result.error || 'Failed to get deposit address');
      setDeposit({
        address: result.deposit.address,
        currency: result.deposit.currency,
      });
      setStep('qr');
    } catch (e: any) {
      setError(e.message || 'Failed to get deposit address');
    } finally {
      setLoading(false);
    }
  };

  if (step === 'done') return <Done />;
  if (step === 'qr' && deposit) {
    return (
      <QR
        amt={data.amt}
        address={deposit.address}
        currency={deposit.currency}
        back={() => setStep('pick')}
        done={() => setStep('done')}
      />
    );
  }

  return (
    <div className="max-w-2xl mx-auto space-y-6">
      <h1 className="text-2xl font-bold text-center mb-8">Quick Payment</h1>

      <Plans
        val={data.pkg}
        set={(pkg, amt) => setData({ ...data, pkg, amt: amt.toString() })}
      />

      <Methods val={data.pay} set={(pay) => setData({ ...data, pay })} />

      <Total {...data} go={handleGo} />

      {loading && (
        <div className="text-center text-gray-500">
          Loading deposit address...
        </div>
      )}
      {error && <div className="text-center text-red-500">{error}</div>}
    </div>
  );
}
