'use client';

import { useState } from 'react';

interface QRProps {
  _pkg: string;
  amt: string;
  _pay: string;
  back: () => void;
  done: () => void;
}

export function QR({ amt, back, done }: QRProps) {
  const [copied, setCopied] = useState(false);
  const address = 'TXyDnQqMVRXvXxXXXXXXXXXXXXXXXXXXXX'; // Example address

  const copy = () => {
    navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="max-w-md mx-auto p-6 bg-white rounded-lg shadow-sm">
      <h2 className="text-xl font-bold text-center mb-6">Send Payment</h2>

      <div className="bg-gray-100 p-8 rounded-lg mb-6 text-center">
        <div className="w-48 h-48 bg-gray-300 mx-auto mb-4 flex items-center justify-center">
          <span className="text-gray-600">QR Code</span>
        </div>
        <p className="text-2xl font-bold text-blue-600">${amt}</p>
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
