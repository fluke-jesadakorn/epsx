'use client';

import { Check, Sparkles } from 'lucide-react';
import { Card, CardContent } from '@/components/ui/card';
import { PACKAGES, LEVEL_BENEFITS } from '@/app/constants/packages';
import type { Package } from '@/app/constants/packages';

interface PackageSelectionProps {
  selectedPackage: string;
  onSelect: (packageType: string, amount: string) => void;
}

const sortedPackages = PACKAGES.filter(pkg => 
  !pkg.id.startsWith('api_')
).sort((a, b) => a.price - b.price);

const isHighlighted = (pkg: Package) => 
  pkg.level === 'GOLD' || pkg.level === 'PLATINUM';

export default function PackageSelection({
  selectedPackage,
  onSelect
}: PackageSelectionProps) {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
      {sortedPackages.map((pkg) => (
        <Card
          key={pkg.id}
          className={`relative cursor-pointer transition-all duration-300 hover:scale-[1.02] hover:shadow-lg ${
            selectedPackage === pkg.id
              ? 'border-primary shadow-lg'
              : 'hover:border-primary/50'
          }`}
          onClick={() => onSelect(pkg.id, pkg.price.toString())}
        >
          <div
            className={`absolute inset-0 bg-gradient-to-br ${
              isHighlighted(pkg)
                ? 'from-blue-500/20 via-purple-500/20 to-pink-500/20'
                : `from-${pkg.color} to-${pkg.color}/50`
            } rounded-lg opacity-50`}
          />
          <CardContent className="relative p-6 space-y-4">
            <div className="flex items-center justify-between">
              <div className="space-y-1">
                <div className="flex items-center gap-2">
                  <h3 className="font-semibold">{pkg.name}</h3>
                  {isHighlighted(pkg) && (
                    <Sparkles className="h-4 w-4 text-yellow-500" />
                  )}
                </div>
                <p className="text-sm text-muted-foreground">
                  {pkg.price === 0 ? 'Free' : `${pkg.price} ${pkg.currency}`}
                </p>
              </div>
              {selectedPackage === pkg.id && (
                <div className="rounded-full bg-primary/10 p-1">
                  <Check className="h-4 w-4 text-primary" />
                </div>
              )}
            </div>

            <div className="space-y-2">
              <h4 className="text-sm font-medium">Features:</h4>
              <ul className="space-y-2 text-sm">
                {LEVEL_BENEFITS[pkg.level].slice(0, 3).map((benefit, index) => (
                  <li key={index} className="flex items-start gap-2">
                    <Check className="h-4 w-4 shrink-0 text-primary mt-0.5" />
                    <span className="text-muted-foreground">{benefit}</span>
                  </li>
                ))}
                {LEVEL_BENEFITS[pkg.level].length > 3 && (
                  <li className="text-muted-foreground text-sm">
                    +{LEVEL_BENEFITS[pkg.level].length - 3} more benefits
                  </li>
                )}
              </ul>
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
