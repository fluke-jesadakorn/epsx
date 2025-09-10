'use client';

import { useState, useEffect } from 'react';
import { X, Sparkles, Clock, Users } from 'lucide-react';

interface Promotion {
  id: number;
  name: string;
  code: string;
  discountType: 'percentage' | 'fixed';
  discountValue: number;
  description: string;
  endDate: string;
  usageLimit: number | null;
  currentUsage: number;
}

interface PromotionalBannerProps {
  className?: string;
}

export function PromotionalBanner({ className = '' }: PromotionalBannerProps) {
  const [activePromotion, setActivePromotion] = useState<Promotion | null>(null);
  const [isVisible, setIsVisible] = useState(false);
  const [timeLeft, setTimeLeft] = useState<string>('');

  useEffect(() => {
    fetchActivePromotion();
  }, []);

  // Update countdown timer
  useEffect(() => {
    if (!activePromotion) return;

    const timer = setInterval(() => {
      const now = new Date().getTime();
      const endTime = new Date(activePromotion.endDate).getTime();
      const distance = endTime - now;

      if (distance > 0) {
        const days = Math.floor(distance / (1000 * 60 * 60 * 24));
        const hours = Math.floor((distance % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
        const minutes = Math.floor((distance % (1000 * 60 * 60)) / (1000 * 60));

        if (days > 0) {
          setTimeLeft(`${days}d ${hours}h ${minutes}m`);
        } else {
          setTimeLeft(`${hours}h ${minutes}m`);
        }
      } else {
        setTimeLeft('Expired');
        setIsVisible(false);
      }
    }, 1000);

    return () => clearInterval(timer);
  }, [activePromotion]);

  const fetchActivePromotion = async () => {
    try {
      // For now, simulate an active promotion
      // In a real implementation, this would fetch from the backend
      const mockPromotion: Promotion = {
        id: 1,
        name: 'Black Friday Sale',
        code: 'BLACKFRIDAY24',
        discountType: 'percentage',
        discountValue: 25,
        description: '25% off all plans - Limited time offer!',
        endDate: new Date(Date.now() + 86400000 * 7).toISOString(), // 7 days from now
        usageLimit: 1000,
        currentUsage: 347
      };

      setActivePromotion(mockPromotion);
      setIsVisible(true);
    } catch (error) {
      console.error('Error fetching promotional data:', error);
    }
  };

  const getDiscountText = (promotion: Promotion) => {
    if (promotion.discountType === 'percentage') {
      return `${promotion.discountValue}% OFF`;
    } else {
      return `$${promotion.discountValue} OFF`;
    }
  };

  const getUsagePercentage = (promotion: Promotion) => {
    if (!promotion.usageLimit) return 0;
    return Math.min((promotion.currentUsage / promotion.usageLimit) * 100, 100);
  };

  if (!isVisible || !activePromotion) return null;

  return (
    <div className={`relative overflow-hidden ${className}`}>
      <div className="bg-gradient-to-r from-orange-500 via-red-500 to-pink-500 text-white">
        <div className="container mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4 flex-1">
              <div className="flex items-center gap-2">
                <Sparkles className="h-5 w-5 animate-spin" />
                <span className="font-bold text-lg">{getDiscountText(activePromotion)}</span>
              </div>

              <div className="hidden sm:block text-sm opacity-90">
                {activePromotion.description}
              </div>

              <div className="flex items-center gap-2 text-sm">
                <Clock className="h-4 w-4" />
                <span className="font-semibold">{timeLeft}</span>
              </div>

              {activePromotion.usageLimit && (
                <div className="hidden md:flex items-center gap-2 text-sm">
                  <Users className="h-4 w-4" />
                  <span>
                    {activePromotion.currentUsage}/{activePromotion.usageLimit} used
                  </span>
                  <div className="w-16 h-2 bg-white/20 rounded-full overflow-hidden">
                    <div 
                      className="h-full bg-white/60 transition-all duration-500"
                      style={{ width: `${getUsagePercentage(activePromotion)}%` }}
                    />
                  </div>
                </div>
              )}
            </div>

            <div className="flex items-center gap-4">
              <div className="hidden sm:block">
                <code className="bg-white/20 text-white px-3 py-1 rounded font-mono text-sm">
                  {activePromotion.code}
                </code>
              </div>

              <button
                onClick={() => setIsVisible(false)}
                className="p-1 hover:bg-white/20 rounded transition-colors"
                aria-label="Close banner"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
          </div>

          {/* Mobile layout for description */}
          <div className="sm:hidden mt-2 text-sm opacity-90">
            {activePromotion.description}
          </div>

          {/* Mobile layout for code */}
          <div className="sm:hidden mt-2">
            <code className="bg-white/20 text-white px-3 py-1 rounded font-mono text-sm">
              Use code: {activePromotion.code}
            </code>
          </div>
        </div>

        {/* Animated gradient overlay */}
        <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent animate-shimmer" />
      </div>
    </div>
  );
}