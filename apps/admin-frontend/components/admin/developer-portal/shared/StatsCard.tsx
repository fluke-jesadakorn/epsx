'use client';

import { LucideIcon } from 'lucide-react';
import React from 'react';

interface StatsCardProps {
    title: string;
    value: number | string;
    icon: LucideIcon;
    iconBgColor: string;
    iconColor: string;
}

/**
 * Reusable stats card component for dashboard metrics
 * @param root0
 * @param root0.title
 * @param root0.value
 * @param root0.icon
 * @param root0.iconBgColor
 * @param root0.iconColor
 */
export const StatsCard: React.FC<StatsCardProps> = ({
    title,
    value,
    icon: Icon,
    iconBgColor,
    iconColor,
}) => {
    return (
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-6">
            <div className="flex items-center">
                <div className={`p-2 ${iconBgColor} rounded-lg`}>
                    <Icon className={`w-6 h-6 ${iconColor}`} />
                </div>
                <div className="ml-4">
                    <p className="text-sm font-medium text-gray-600 dark:text-gray-300">
                        {title}
                    </p>
                    <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                        {typeof value === 'number' ? value.toLocaleString() : value}
                    </p>
                </div>
            </div>
        </div>
    );
};
