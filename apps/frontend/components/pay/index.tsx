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

  if (step === 'done') return <Done />;
  if (step === 'qr') {
    return (
      <QR {...data} back={() => setStep('pick')} done={() => setStep('done')} />
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

      <Total
        {...data}
        go={() => setStep(data.pay.includes('USDT') ? 'qr' : 'done')}
      />
    </div>
  );
}
