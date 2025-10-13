'use client';

import { useState, useEffect, useCallback } from 'react';
import { useSearchParams, useRouter } from 'next/navigation';
import { Star, Gift, Users, TrendingUp } from 'lucide-react';
import { env } from '../../../../shared/env/schema';

interface AffiliateInfo {
  code: string;
  name?: string;
  commissionRate: number;
  tier?: string;
  isValid: boolean;
}

interface AffiliateStats {
  totalReferrals: number;
  conversionRate: number;
  avgCommission: number;
  tier: string;
}

interface AffiliateTrackerProps {
  children?: React.ReactNode;
  onAffiliateDetected?: (_affiliateInfo: AffiliateInfo) => void;
}

export function AffiliateTracker({ children, onAffiliateDetected }: AffiliateTrackerProps) {
  const [affiliateInfo, setAffiliateInfo] = useState<AffiliateInfo | null>(null);
  const [affiliateStats, setAffiliateStats] = useState<AffiliateStats | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  
  const searchParams = useSearchParams();
  const _router = useRouter();

  // Track affiliate attribution
  const trackAffiliate = useCallback(async (code: string) => {
    if (!code || isLoading) return;

    setIsLoading(true);
    try {
      const baseUrl = env.BACKEND_URL;
      const response = await fetch(`${baseUrl}/api/public/plans?affiliate_code=${code}`);
      
      if (response.ok) {
        const result = await response.json();
        if (result.success) {
          // Simulate affiliate validation (normally would come from backend)
          const mockAffiliateInfo: AffiliateInfo = {
            code,
            name: getAffiliateDisplayName(code),
            commissionRate: 15, // This would come from the backend
            tier: getAffiliateTier(code),
            isValid: true
          };
          
          setAffiliateInfo(mockAffiliateInfo);
          
          // Store in cookies for persistence
          document.cookie = `affiliate_attribution=${encodeURIComponent(JSON.stringify({
            code,
            timestamp: Date.now(),
            info: mockAffiliateInfo
          }))}; path=/; max-age=2592000; SameSite=lax`; // 30 days

          // Load affiliate stats
          loadAffiliateStats(code);
          
          // Trigger callback
          onAffiliateDetected?.(mockAffiliateInfo);
          
          // Track the referral click (fire and forget)
          trackReferralClick(code);
        }
      }
    } catch (error) {
      console.error('Error tracking affiliate:', error);
    } finally {
      setIsLoading(false);
    }
  }, [isLoading, onAffiliateDetected]);

  // Initialize affiliate tracking
  useEffect(() => {
    const initializeTracking = () => {
      // Check URL parameters for affiliate codes
      const urlAffiliateCode = searchParams.get('ref') || 
                              searchParams.get('affiliate') || 
                              searchParams.get('aff') ||
                              searchParams.get('partner');

      if (urlAffiliateCode) {
        trackAffiliate(urlAffiliateCode);
        return;
      }

      // Check cookies for existing attribution, fallback to localStorage for migration
      const cookies = document.cookie.split(';').reduce((acc, cookie) => {
        const [key, value] = cookie.trim().split('=');
        if (key && value) acc[key] = value;
        return acc;
      }, {} as Record<string, string>);
      
      const storedAttribution = cookies.affiliate_attribution || localStorage.getItem('affiliateAttribution');
      if (storedAttribution) {
        try {
          const attribution = JSON.parse(storedAttribution);
          const ageHours = (Date.now() - attribution.timestamp) / (1000 * 60 * 60);
          
          // Attribution valid for 30 days
          if (ageHours < 24 * 30 && attribution.info) {
            setAffiliateInfo(attribution.info);
            loadAffiliateStats(attribution.code);
            onAffiliateDetected?.(attribution.info);
          }
        } catch (error) {
          console.error('Error parsing stored affiliate attribution:', error);
          // Remove from both cookie and localStorage
          document.cookie = 'affiliate_attribution=; max-age=0; path=/; SameSite=lax';
          localStorage.removeItem('affiliateAttribution');
        }
      }
    };

    initializeTracking();
  }, [searchParams, trackAffiliate, onAffiliateDetected]);

  // Track referral click
  const trackReferralClick = async (code: string) => {
    try {
      const baseUrl = env.BACKEND_URL;
      
      // Get user's IP and other tracking info
      const trackingData = {
        affiliateCode: code,
        url: window.location.href,
        userAgent: navigator.userAgent,
        timestamp: new Date().toISOString(),
        referrer: document.referrer
      };

      // Fire and forget - don't wait for response
      fetch(`${baseUrl}/api/public/plans/details/1?affiliate_code=${code}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json'
        }
      }).catch(() => {
        // Ignore errors for tracking
      });

      console.log('🎯 Affiliate referral tracked:', trackingData);
    } catch (error) {
      console.error('Error tracking referral click:', error);
    }
  };

  // Load affiliate performance stats
  const loadAffiliateStats = async (code: string) => {
    try {
      // Mock affiliate stats - in real implementation, fetch from backend
      const mockStats: AffiliateStats = {
        totalReferrals: Math.floor(Math.random() * 500) + 50,
        conversionRate: Math.random() * 20 + 10,
        avgCommission: Math.random() * 50 + 25,
        tier: getAffiliateTier(code)
      };
      
      setAffiliateStats(mockStats);
    } catch (error) {
      console.error('Error loading affiliate stats:', error);
    }
  };

  // Get affiliate display name from code
  const getAffiliateDisplayName = (code: string): string => {
    const nameMap: Record<string, string> = {
      'TESTCODE': 'Test Partner',
      'TECHPRO': 'Tech Influencer Pro',
      'CRYPTOHUB': 'Crypto Trader Hub',
      'APIDEVS': 'API Developer Community',
      'TRADEPRO': 'Trading Academy Pro'
    };
    
    return nameMap[code] || 'Partner Network';
  };

  // Get affiliate tier
  const getAffiliateTier = (code: string): string => {
    if (['CRYPTOHUB', 'TRADEPRO'].includes(code)) return 'Elite';
    if (['TECHPRO', 'APIDEVS'].includes(code)) return 'Premium';
    return 'Standard';
  };

  // Get tier color and icon
  const getTierDisplay = (tier: string) => {
    switch (tier) {
      case 'Elite':
        return {
          color: 'from-purple-500 to-pink-500',
          icon: <Star className="h-4 w-4" />,
          text: 'Elite Partner'
        };
      case 'Premium':
        return {
          color: 'from-blue-500 to-indigo-500',
          icon: <TrendingUp className="h-4 w-4" />,
          text: 'Premium Partner'
        };
      default:
        return {
          color: 'from-green-500 to-emerald-500',
          icon: <Users className="h-4 w-4" />,
          text: 'Standard Partner'
        };
    }
  };

  // Clear affiliate attribution
  const clearAttribution = () => {
    setAffiliateInfo(null);
    setAffiliateStats(null);
    localStorage.removeItem('affiliateAttribution');
  };

  return (
    <>
      {/* Affiliate Attribution Display */}
      {affiliateInfo && (
        <div className="sticky top-0 z-50">
          <div className="bg-gradient-to-r from-purple-600 via-blue-600 to-indigo-600 text-white shadow-lg">
            <div className="container mx-auto px-4 py-3">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <div className="flex items-center gap-2">
                    <Gift className="h-5 w-5 animate-bounce" />
                    <span className="font-semibold">
                      You're earning {affiliateInfo.commissionRate}% rewards!
                    </span>
                  </div>
                  
                  <div className="hidden sm:flex items-center gap-2 text-sm">
                    {getTierDisplay(affiliateInfo.tier || 'Standard').icon}
                    <span>Referred by {affiliateInfo.name}</span>
                  </div>

                  {affiliateStats && (
                    <div className="hidden md:flex items-center gap-4 text-xs opacity-90">
                      <span>{affiliateStats.totalReferrals} referrals</span>
                      <span>•</span>
                      <span>{affiliateStats.conversionRate.toFixed(1)}% conversion</span>
                      <span>•</span>
                      <span className={`px-2 py-1 rounded-full text-white bg-gradient-to-r ${getTierDisplay(affiliateStats.tier).color}`}>
                        {getTierDisplay(affiliateStats.tier).text}
                      </span>
                    </div>
                  )}
                </div>

                <button
                  onClick={clearAttribution}
                  className="text-xs hover:bg-white/20 px-3 py-1 rounded transition-colors"
                >
                  Clear
                </button>
              </div>

              {/* Mobile layout */}
              <div className="sm:hidden mt-2 text-sm">
                <div className="flex items-center gap-2">
                  {getTierDisplay(affiliateInfo.tier || 'Standard').icon}
                  <span>Referred by {affiliateInfo.name}</span>
                </div>
              </div>
            </div>

            {/* Animated gradient overlay */}
            <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white/10 to-transparent animate-shimmer" />
          </div>
        </div>
      )}

      {/* Render children */}
      {children}

      {/* Tracking pixel (hidden) */}
      {affiliateInfo && (
        <div 
          className="hidden"
          data-affiliate-code={affiliateInfo.code}
          data-affiliate-name={affiliateInfo.name}
          data-commission-rate={affiliateInfo.commissionRate}
        />
      )}
    </>
  );
}

// Custom hook for affiliate data
export function useAffiliate() {
  const [affiliateInfo, setAffiliateInfo] = useState<AffiliateInfo | null>(null);

  useEffect(() => {
    const storedAttribution = localStorage.getItem('affiliateAttribution');
    if (storedAttribution) {
      try {
        const attribution = JSON.parse(storedAttribution);
        const ageHours = (Date.now() - attribution.timestamp) / (1000 * 60 * 60);
        
        if (ageHours < 24 * 30 && attribution.info) {
          setAffiliateInfo(attribution.info);
        }
      } catch (error) {
        console.error('Error parsing affiliate attribution:', error);
      }
    }
  }, []);

  return affiliateInfo;
}