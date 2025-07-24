import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Wallet, Shield, Clock, Star } from 'lucide-react';

interface PaymentMethod {
  id: string;
  name: string;
  icon: React.ReactNode;
  description: string;
  processingTime: string;
  fees: string;
  popular?: boolean;
  networks?: string[];
}

const PAYMENT_METHODS: PaymentMethod[] = [
  {
    id: 'USDT_TRC20',
    name: 'USDT (TRC20)',
    icon: <Wallet className="h-5 w-5" />,
    description: 'Fast and low-cost transactions',
    processingTime: '1-5 minutes',
    fees: '$0.50 - $1.00',
    popular: true,
    networks: ['TRC20'],
  },
  {
    id: 'USDC_ERC20',
    name: 'USDC (ERC20)',
    icon: <Shield className="h-5 w-5" />,
    description: 'Secure and regulated stablecoin',
    processingTime: '5-15 minutes',
    fees: '$2.00 - $5.00',
    networks: ['ERC20'],
  },
  {
    id: 'ETH',
    name: 'Ethereum (ETH)',
    icon: <Star className="h-5 w-5" />,
    description: 'Native Ethereum blockchain',
    processingTime: '5-15 minutes',
    fees: '$2.00 - $10.00',
    networks: ['ERC20'],
  },
  {
    id: 'BTC',
    name: 'Bitcoin (BTC)',
    icon: <Clock className="h-5 w-5" />,
    description: 'Original cryptocurrency',
    processingTime: '30-60 minutes',
    fees: '$1.00 - $5.00',
    networks: ['Bitcoin'],
  },
];

interface PaymentMethodSelectorServerProps {
  selectedMethod?: string;
  className?: string;
}

export function PaymentMethodSelectorServer({ selectedMethod, className }: PaymentMethodSelectorServerProps) {
  return (
    <div className={`space-y-4 ${className}`}>
      <div className="text-lg font-semibold mb-4">Select Payment Method</div>
      <div className="grid gap-3">
        {PAYMENT_METHODS.map((method) => (
          <Card
            key={method.id}
            className={`cursor-pointer transition-all duration-200 hover:shadow-md ${
              selectedMethod === method.id
                ? 'ring-2 ring-primary border-primary'
                : 'hover:border-primary/50'
            }`}
          >
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  {method.icon}
                  <span className="text-base">{method.name}</span>
                </div>
                <div className="flex items-center gap-2">
                  {method.popular && (
                    <Badge variant="secondary" className="text-xs">
                      Popular
                    </Badge>
                  )}
                  {selectedMethod === method.id && (
                    <div className="w-4 h-4 rounded-full bg-primary flex items-center justify-center">
                      <div className="w-2 h-2 rounded-full bg-white" />
                    </div>
                  )}
                </div>
              </CardTitle>
            </CardHeader>
            <CardContent className="pt-0">
              <p className="text-sm text-muted-foreground mb-2">
                {method.description}
              </p>
              <div className="flex justify-between items-center text-xs text-muted-foreground">
                <span>⏱️ {method.processingTime}</span>
                <span>💰 {method.fees}</span>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}