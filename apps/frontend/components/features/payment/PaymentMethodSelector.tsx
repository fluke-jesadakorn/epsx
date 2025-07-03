'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

interface PaymentMethodSelectorProps {
  onMethodSelect: (method: PaymentMethod) => void;
  selectedMethod?: PaymentMethod;
}

export interface PaymentMethod {
  id: string;
  name: string;
  type: 'card' | 'crypto' | 'bank' | 'wallet';
  icon: string;
  description: string;
  processing_time: string;
  fees: string;
  supported_currencies?: string[];
  is_popular?: boolean;
}

const PAYMENT_METHODS: PaymentMethod[] = [
  {
    id: 'card',
    name: 'Credit/Debit Card',
    type: 'card',
    icon: '💳',
    description: 'Visa, Mastercard, American Express',
    processing_time: 'Instant',
    fees: '2.9% + $0.30',
    is_popular: true,
  },
  {
    id: 'apple_pay',
    name: 'Apple Pay',
    type: 'wallet',
    icon: '🍎',
    description: 'Pay with Touch ID or Face ID',
    processing_time: 'Instant',
    fees: '2.9% + $0.30',
    is_popular: true,
  },
  {
    id: 'google_pay',
    name: 'Google Pay',
    type: 'wallet',
    icon: '🔍',
    description: 'Pay with your Google account',
    processing_time: 'Instant',
    fees: '2.9% + $0.30',
  },
  {
    id: 'usdt_trc20',
    name: 'USDT (TRC20)',
    type: 'crypto',
    icon: '₮',
    description: 'Tether on Tron network',
    processing_time: '5-10 minutes',
    fees: '~$1',
    supported_currencies: ['USDT'],
  },
  {
    id: 'usdt_erc20',
    name: 'USDT (ERC20)',
    type: 'crypto',
    icon: '₮',
    description: 'Tether on Ethereum network',
    processing_time: '10-20 minutes',
    fees: '$5-50 (gas)',
    supported_currencies: ['USDT'],
  },
  {
    id: 'btc',
    name: 'Bitcoin',
    type: 'crypto',
    icon: '₿',
    description: 'Bitcoin payments',
    processing_time: '30-60 minutes',
    fees: '$5-20',
    supported_currencies: ['BTC'],
  },
  {
    id: 'eth',
    name: 'Ethereum',
    type: 'crypto',
    icon: '⟠',
    description: 'Ethereum payments',
    processing_time: '10-20 minutes',
    fees: '$5-50 (gas)',
    supported_currencies: ['ETH'],
  },
  {
    id: 'bank_transfer',
    name: 'Bank Transfer',
    type: 'bank',
    icon: '🏦',
    description: 'Direct bank transfer',
    processing_time: '1-3 business days',
    fees: 'Free',
  },
];

export default function PaymentMethodSelector({ onMethodSelect, selectedMethod }: PaymentMethodSelectorProps) {
  const [selected, setSelected] = useState<string>(selectedMethod?.id || '');
  
  const handleMethodSelect = (methodId: string) => {
    setSelected(methodId);
    const method = PAYMENT_METHODS.find(m => m.id === methodId);
    if (method) {
      onMethodSelect(method);
    }
  };

  const getMethodTypeColor = (type: PaymentMethod['type']) => {
    switch (type) {
      case 'card':
        return 'bg-blue-100 text-blue-800';
      case 'crypto':
        return 'bg-orange-100 text-orange-800';
      case 'bank':
        return 'bg-green-100 text-green-800';
      case 'wallet':
        return 'bg-purple-100 text-purple-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <span className="text-2xl">💳</span>
          Choose Payment Method
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="grid gap-4">
          {PAYMENT_METHODS.map((method) => (
            <div key={method.id} className="relative">
              <input
                type="radio"
                id={method.id}
                name="payment-method"
                value={method.id}
                checked={selected === method.id}
                onChange={() => handleMethodSelect(method.id)}
                className="sr-only peer"
              />
              <label
                htmlFor={method.id}
                className="flex items-center gap-4 p-4 border-2 border-gray-200 rounded-lg cursor-pointer hover:border-primary/50 hover:bg-gray-50 peer-checked:border-primary peer-checked:bg-primary/5 transition-all"
              >
                <div className="flex items-center gap-3 flex-1">
                  <span className="text-2xl">{method.icon}</span>
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <span className="font-medium">{method.name}</span>
                      {method.is_popular && (
                        <Badge variant="secondary" className="text-xs">
                          Popular
                        </Badge>
                      )}
                      <Badge 
                        variant="outline" 
                        className={`text-xs ${getMethodTypeColor(method.type)}`}
                      >
                        {method.type}
                      </Badge>
                    </div>
                    <p className="text-sm text-gray-600 mb-1">{method.description}</p>
                    <div className="flex items-center gap-4 text-xs text-gray-500">
                      <span>⏱️ {method.processing_time}</span>
                      <span>💰 {method.fees}</span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center">
                  <div className="w-4 h-4 border-2 border-gray-300 rounded-full peer-checked:bg-primary peer-checked:border-primary flex items-center justify-center">
                    {selected === method.id && (
                      <div className="w-2 h-2 bg-white rounded-full"></div>
                    )}
                  </div>
                </div>
              </label>
            </div>
          ))}
        </div>
        
        {selected && (
          <div className="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
            <h4 className="font-medium text-blue-900 mb-2">Selected Method</h4>
            <p className="text-sm text-blue-800">
              {PAYMENT_METHODS.find(m => m.id === selected)?.name} - {' '}
              {PAYMENT_METHODS.find(m => m.id === selected)?.description}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
