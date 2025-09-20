'use client';

import { PERMISSION_TEMPLATES, PermissionTemplateName } from '@/app/constants/packages';

const PERMISSION_TEMPLATE_PLANS = [
  { 
    template: 'Silver Template' as PermissionTemplateName, 
    displayTier: 'SILVER',
    price: 9.99,
    features: ['Premium access', 'View 25 rankings', 'Advanced analytics']
  },
  { 
    template: 'Gold Template' as PermissionTemplateName, 
    displayTier: 'GOLD', 
    price: 19.99,
    features: ['Professional access', 'View 50 rankings', 'Premium tools']
  },
  { 
    template: 'Platinum Template' as PermissionTemplateName, 
    displayTier: 'PLATINUM', 
    price: 29.99,
    features: ['VIP access', 'View 100 rankings', 'Advanced features']
  }
];

interface PlansProps {
  val: string;
  set: (template: PermissionTemplateName, amt: number) => void;
}

export function Plans({ val, set }: PlansProps) {
  return (
    <div className="p-6 bg-white rounded-lg shadow-sm">
      <h2 className="text-lg font-semibold mb-4">Choose Your Permission Template</h2>
      <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
        {PERMISSION_TEMPLATE_PLANS.map(plan => {
          const template = PERMISSION_TEMPLATES[plan.template];
          return (
            <div 
              key={plan.template}
              onClick={() => set(plan.template, plan.price)}
              className={`p-4 border-2 rounded-lg cursor-pointer transition-all ${
                val === plan.template 
                  ? 'border-blue-500 bg-blue-50' 
                  : 'border-gray-200 hover:border-gray-300'
              }`}
            >
              <h3 className="font-bold text-lg">{plan.displayTier}</h3>
              <p className="text-2xl font-bold text-blue-600 mt-2">${plan.price}</p>
              <p className="text-sm text-gray-500 mt-1">/month</p>
              <ul className="text-xs text-gray-600 mt-3 space-y-1">
                {template.features.slice(0, 2).map((feature, index) => (
                  <li key={index}>• {feature}</li>
                ))}
              </ul>
            </div>
          );
        })}
      </div>
    </div>
  );
}
