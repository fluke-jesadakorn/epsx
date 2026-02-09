'use client';

import { useState } from 'react';
import { 
  Zap, 
  ArrowRight, 
  Check, 
  Star, 
  Shield,
  Clock
} from 'lucide-react';
import { Card, CardContent, Button, Badge } from '@/components/ui';
import type { PermissionTemplateName } from '@/app/constants/packages';
import { PERMISSION_TEMPLATES } from '@/app/constants/packages';

interface PaymentWidgetProps {
  title?: string;
  subtitle?: string;
  templates?: Array<{
    id: PermissionTemplateName;
    name: string;
    price: number;
    popular?: boolean;
    features: string[];
  }>;
  className?: string;
}

const defaultTemplates = [
  {
    id: 'Silver Template' as PermissionTemplateName,
    name: 'Silver Access',
    price: 9.99,
    features: ['View 25 rankings', 'Advanced analytics', 'Priority support']
  },
  {
    id: 'Gold Template' as PermissionTemplateName,
    name: 'Gold Access',
    price: 19.99,
    popular: true,
    features: ['View 50 rankings', 'Premium tools', 'Professional analytics', 'Advanced features']
  },
  {
    id: 'Platinum Template' as PermissionTemplateName,
    name: 'Platinum Access',
    price: 29.99,
    features: ['View 100 rankings', 'VIP features', 'Custom reports', 'Priority support']
  }
];

export default function PaymentWidget({ 
  title = 'Upgrade Your Access',
  subtitle = 'Choose the perfect permission template for your needs',
  templates = defaultTemplates,
  className = ''
}: PaymentWidgetProps) {
  const [selectedTemplate, setSelectedTemplate] = useState(templates.find(t => t.popular)?.id ?? templates[0]?.id);

  const selectedTemplate_obj = templates.find(t => t.id === selectedTemplate);

  const handlePayNow = () => {
    if (selectedTemplate_obj) {
      // Redirect to main payment page with Web3-first flow
      window.location.href = `/payment?package=${selectedTemplate_obj.id}`;
    }
  };

  return (
    <Card className={`w-full max-w-md mx-auto shadow-lg border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 ${className}`}>
      <CardContent className="p-6">
        <div className="text-center mb-6">
          <h3 className="text-xl font-bold mb-2 text-gray-900 dark:text-white">{title}</h3>
          <p className="text-sm text-muted-foreground">{subtitle}</p>
        </div>

        {/* Template Selection */}
        <div className="space-y-2 mb-6">
          {templates.map((template) => (
            <div
              key={template.id}
              className={`p-3 rounded-lg border-2 cursor-pointer transition-all hover:shadow-md ${
                selectedTemplate === template.id
                  ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 dark:border-blue-400 shadow-md'
                  : 'border-gray-200 dark:border-gray-600 hover:border-blue-300 dark:hover:border-blue-400 bg-white dark:bg-gray-700'
              }`}
              onClick={() => setSelectedTemplate(template.id)}
            >
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-2 min-w-0 flex-1">
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1 flex-wrap">
                      <span className="font-semibold text-gray-900 dark:text-white text-sm sm:text-base">{template.name}</span>
                      {template.popular && (
                        <Badge className="text-xs bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200 flex items-center gap-1">
                          <Star className="h-3 w-3" />
                          Popular
                        </Badge>
                      )}
                    </div>
                    <div className="text-xs sm:text-sm text-muted-foreground truncate">
                      {template.features[0]}
                    </div>
                  </div>
                </div>
                <div className="text-right flex items-center gap-2">
                  <div className="text-lg font-bold text-blue-600 dark:text-blue-400">
                    ${template.price}
                  </div>
                  {selectedTemplate === template.id && (
                    <Check className="h-4 w-4 text-blue-500 flex-shrink-0" />
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Selected Template Details */}
        {selectedTemplate_obj && (
          <div className="bg-gray-50 dark:bg-gray-700 rounded-lg p-4 mb-6">
            <h4 className="font-semibold mb-2 text-gray-900 dark:text-white">What&apos;s included:</h4>
            <ul className="space-y-1">
              {selectedTemplate_obj.features.map((feature) => (
                <li key={feature} className="text-sm flex items-center gap-2">
                  <Check className="h-3 w-3 text-green-500 flex-shrink-0" />
                  <span className="text-gray-600 dark:text-gray-300">{feature}</span>
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Payment CTA */}
        <div className="space-y-3">
          <Button
            onClick={handlePayNow}
            className="w-full h-12 text-base sm:text-lg font-semibold bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-700 hover:to-purple-700 dark:from-blue-500 dark:to-purple-500 dark:hover:from-blue-600 dark:hover:to-purple-600 shadow-lg hover:shadow-xl transition-all duration-300 flex items-center gap-2"
          >
            <Zap className="h-5 w-5" />
            Pay ${selectedTemplate_obj?.price} Now
            <ArrowRight className="h-4 w-4" />
          </Button>

          <div className="flex items-center justify-center gap-4 text-xs text-muted-foreground">
            <div className="flex items-center gap-1">
              <Shield className="h-3 w-3" />
              <span>Secure</span>
            </div>
            <div className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              <span>Instant</span>
            </div>
            <div className="flex items-center gap-1">
              <Check className="h-3 w-3" />
              <span>Guaranteed</span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
