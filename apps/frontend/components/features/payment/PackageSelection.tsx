'use client';

import { Card } from '@/components/ui/card';

interface Package {
  type: string;
  name: string;
  description: string;
  price: string;
  features: string[];
}

interface PackageSelectionProps {
  onSelect: (packageType: string, amount: string) => void;
  selectedPackage?: string;
}

const packages: Package[] = [
  {
    type: 'basic',
    name: 'Basic Plan',
    description: 'Perfect for getting started',
    price: '99',
    features: ['Basic market data', 'Standard analytics', '24/7 Support'],
  },
  {
    type: 'pro',
    name: 'Pro Plan',
    description: 'For serious traders',
    price: '199',
    features: [
      'Advanced market data',
      'Premium analytics',
      'Priority support',
      'Real-time alerts',
    ],
  },
  {
    type: 'enterprise',
    name: 'Enterprise Plan',
    description: 'Full feature access',
    price: '499',
    features: [
      'Full market data access',
      'Advanced analytics suite',
      'Dedicated support',
      'Custom solutions',
      'API access',
    ],
  },
];

export default function PackageSelection({
  onSelect,
  selectedPackage,
}: PackageSelectionProps) {
  return (
    <div className="grid gap-6 md:grid-cols-3">
      {packages.map((pkg) => (
        <Card
          key={pkg.type}
          className={`p-6 cursor-pointer hover:border-primary transition-colors ${
            selectedPackage === pkg.type ? 'border-primary' : ''
          }`}
          onClick={() => onSelect(pkg.type, pkg.price)}
        >
          <h3 className="text-xl font-bold mb-2">{pkg.name}</h3>
          <p className="text-muted-foreground mb-4">{pkg.description}</p>
          <div className="mb-6">
            <span className="text-3xl font-bold">${pkg.price}</span>
            <span className="text-muted-foreground">/year</span>
          </div>
          <ul className="space-y-2">
            {pkg.features.map((feature) => (
              <li key={feature} className="flex items-center gap-2">
                <svg
                  className="w-5 h-5 text-green-500"
                  fill="none"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth="2"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path d="M5 13l4 4L19 7" />
                </svg>
                <span>{feature}</span>
              </li>
            ))}
          </ul>
          <button
            onClick={() => onSelect(pkg.type, pkg.price)}
            className={`w-full mt-6 py-2 rounded-md ${
              selectedPackage === pkg.type
                ? 'bg-primary text-primary-foreground'
                : 'bg-secondary text-secondary-foreground hover:bg-primary hover:text-primary-foreground'
            }`}
          >
            {selectedPackage === pkg.type ? 'Selected' : 'Select Plan'}
          </button>
        </Card>
      ))}
    </div>
  );
}
