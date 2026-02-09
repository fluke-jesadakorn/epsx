import { useEffect, useState } from 'react';
import { fetchPlans } from '../constants';
import type { PaymentPackage, PaymentStep } from '../types';

export function usePaymentState(preselectedPackage?: string) {
  const [packages, setPackages] = useState<PaymentPackage[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedPackage, setSelectedPackage] = useState<number | string | null>(null);
  const [currentStep, setCurrentStep] = useState<PaymentStep>('package');
  const [selectedPaymentMethod, setSelectedPaymentMethod] = useState('metamask');

  useEffect(() => {
    const loadPlans = async () => {
      try {
        setLoading(true);
        setError(null);
        const plans = await fetchPlans();
        setPackages(plans);

        if (preselectedPackage) {
          const selectedPlan = plans.find(
            p =>
              p.id === preselectedPackage ||
              p.plan_type.toLowerCase() === preselectedPackage.toLowerCase() ||
              p.name.toLowerCase() === preselectedPackage.toLowerCase()
          );
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
          setSelectedPackage(selectedPlan?.id ?? plans[0]?.id ?? null);
        } else {
          const defaultPlan = plans[0];
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition
          setSelectedPackage(defaultPlan?.id || null);
        }
      } catch (_err) {
      // Error logged silently
        setError('Failed to load plans. Please try again.');
        setPackages([]);
        setSelectedPackage(null);
      } finally {
        setLoading(false);
      }
    };

    void loadPlans();
  }, [preselectedPackage]);

  const selectedPkg = packages.find(pkg => pkg.id === selectedPackage) ?? null;

  return {
    packages,
    loading,
    error,
    selectedPackage,
    setSelectedPackage,
    currentStep,
    setCurrentStep,
    selectedPaymentMethod,
    setSelectedPaymentMethod,
    selectedPkg,
  };
}
