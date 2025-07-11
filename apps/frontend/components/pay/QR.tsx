'use client';

import { useState } from 'react';
import { QRCodeCanvas } from 'qrcode.react';

interface QRProps {
  amt: string;
  address: string;
  currency: string;
  back: () => void;
  done: () => void;
}

export function QR({ amt, address, currency, back, done }: QRProps) {
  const [copied, setCopied] = useState(false);

  const copy = () => {
    navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  // QR code value: address and amount (standard for wallets)
  const qrValue = JSON.stringify({
    address,
    amount: amt,
    currency,
  });

  return (
    <div className="max-w-md mx-auto p-6 bg-white rounded-lg shadow-sm">
      <h2 className="text-xl font-bold text-center mb-6">Send Payment</h2>

      <div className="bg-gray-100 p-8 rounded-lg mb-6 text-center">
        <div className="w-48 h-48 bg-white mx-auto mb-4 flex items-center justify-center rounded">
          <QRCodeCanvas value={qrValue} size={192} />
        </div>
        <p className="text-2xl font-bold text-blue-600">
          {amt} {currency}
        </p>
      </div>

      <div className="mb-6">
        <p className="text-sm text-gray-600 mb-2">Send to this address:</p>
        <div className="flex items-center gap-2">
          <input
            type="text"
            value={address}
            readOnly
            className="flex-1 p-2 border rounded bg-gray-50 text-xs"
          />
          <button
            onClick={copy}
            className="px-3 py-2 bg-gray-200 rounded hover:bg-gray-300 text-sm"
          >
            {copied ? '✓' : 'Copy'}
          </button>
        </div>
      </div>

      <div className="flex gap-3">
        <button
          onClick={back}
          className="flex-1 py-2 border border-gray-300 rounded hover:bg-gray-50"
        >
          Back
        </button>
        <button
          onClick={done}
          className="flex-1 py-2 bg-green-500 text-white rounded hover:bg-green-600"
        >
          Sent Payment
        </button>
      </div>
    </div>
  );
}
