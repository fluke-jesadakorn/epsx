import { cn } from '@/lib/utils';
import { Check } from 'lucide-react';
import type { PaymentStep } from '../types';

interface StepIndicatorProps {
  currentStep: PaymentStep;
}

export function StepIndicator({ currentStep }: StepIndicatorProps) {
  return (
    <div className="mb-6 flex items-center justify-center sm:mb-8">
      <div className="flex items-center space-x-2 sm:space-x-4">
        {(['package', 'payment', 'confirmation'] as PaymentStep[]).map(
          (step, index) => (
            <div key={step} className="flex items-center">
              <div
                className={cn(
                  'flex h-8 w-8 items-center justify-center rounded-full text-xs font-bold transition-all sm:h-10 sm:w-10 sm:text-sm',
                  currentStep === step
                    ? 'bg-primary text-primary-foreground scale-110'
                    : index <
                      ['package', 'payment', 'confirmation'].indexOf(
                        currentStep
                      )
                      ? 'bg-green-500 text-white'
                      : 'bg-muted text-muted-foreground'
                )}
              >
                {index <
                  ['package', 'payment', 'confirmation'].indexOf(currentStep) ? (
                  <Check className="h-4 w-4" />
                ) : (
                  index + 1
                )}
              </div>
              {index < 2 && (
                <div
                  className={cn(
                    'h-0.5 w-6 transition-colors sm:w-12',
                    index <
                      ['package', 'payment', 'confirmation'].indexOf(
                        currentStep
                      )
                      ? 'bg-green-500'
                      : 'bg-muted'
                  )}
                />
              )}
            </div>
          )
        )}
      </div>
    </div>
  );
}
