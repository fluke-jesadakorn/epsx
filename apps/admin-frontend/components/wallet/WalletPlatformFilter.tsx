/**
 * Wallet Platform Filter Component
 * Filter tabs for EPSX ecosystem platforms
 */
'use client';

import { BarChart3, Coins, CreditCard, TrendingUp } from 'lucide-react';
import React from 'react';

import { cn } from '@/lib/utils';

import type { Platform } from './types';

interface PlatformOption {
    value: Platform | 'all';
    label: string;
    icon: React.ReactNode;
    description: string;
    available: boolean;
}

const PLATFORM_OPTIONS: PlatformOption[] = [
    {
        value: 'all',
        label: 'All Platforms',
        icon: <span className="text-lg">🌐</span>,
        description: 'View all wallets',
        available: true,
    },
    {
        value: 'analytics',
        label: 'Analytics',
        icon: <BarChart3 className="h-4 w-4" />,
        description: 'EPSX Analytics',
        available: true,
    },
    {
        value: 'pay',
        label: 'Pay',
        icon: <CreditCard className="h-4 w-4" />,
        description: 'Coming Soon',
        available: false,
    },
    {
        value: 'token',
        label: 'Token',
        icon: <Coins className="h-4 w-4" />,
        description: 'Coming Soon',
        available: false,
    },
    {
        value: 'markets',
        label: 'Markets',
        icon: <TrendingUp className="h-4 w-4" />,
        description: 'Coming Soon',
        available: false,
    },
];

interface WalletPlatformFilterProps {
    value: Platform | 'all';
    onChange: (platform: Platform | 'all') => void;
    className?: string;
}

export function WalletPlatformFilter({
    value,
    onChange,
    className,
}: WalletPlatformFilterProps) {
    return (
        <div className={cn('w-full', className)}>
            <div className="relative overflow-hidden rounded-2xl bg-gradient-to-r from-blue-400/20 via-purple-400/20 to-pink-400/20 p-0.5">
                <div className="relative bg-white/95 dark:bg-gray-900/95 backdrop-blur-xl rounded-2xl p-2">
                    <div className="grid grid-cols-5 gap-2">
                        {PLATFORM_OPTIONS.map((option) => {
                            const isSelected = value === option.value;
                            const isDisabled = !option.available && option.value !== 'all';

                            return (
                                <button
                                    key={option.value}
                                    onClick={() => !isDisabled && onChange(option.value)}
                                    disabled={isDisabled}
                                    className={cn(
                                        'relative px-4 py-3 rounded-xl font-semibold text-sm min-h-[52px] flex items-center justify-center gap-2 transition-all',
                                        isSelected && !isDisabled
                                            ? 'bg-gradient-to-r from-blue-500 to-purple-500 text-white shadow-lg'
                                            : 'bg-white/80 dark:bg-gray-800/80 text-gray-700 dark:text-gray-300',
                                        !isDisabled && !isSelected && 'hover:bg-gray-100 dark:hover:bg-gray-700',
                                        isDisabled && 'opacity-50 cursor-not-allowed'
                                    )}
                                >
                                    {option.icon}
                                    <span className="hidden sm:inline">{option.label}</span>
                                    {isDisabled && (
                                        <span className="absolute -top-1 -right-1 px-1.5 py-0.5 text-[10px] font-medium bg-gray-200 dark:bg-gray-700 text-gray-500 dark:text-gray-400 rounded-full">
                                            Soon
                                        </span>
                                    )}
                                </button>
                            );
                        })}
                    </div>
                </div>
            </div>
        </div>
    );
}
