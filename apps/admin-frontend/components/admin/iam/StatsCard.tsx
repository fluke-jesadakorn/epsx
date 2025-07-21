import React from 'react';
import type { LucideIcon } from 'lucide-react';

interface StatsCardProps {
  title: string;
  value: number;
  icon: LucideIcon;
  color: 'blue' | 'green' | 'purple' | 'orange' | 'red';
  loading?: boolean;
  trend?: {
    value: number;
    type: 'increase' | 'decrease';
  };
  subtitle?: string;
}

export const StatsCard: React.FC<StatsCardProps> = ({
  title,
  value,
  icon: Icon,
  color,
  loading = false,
  trend,
  subtitle
}) => {
  const colorClasses = {
    blue: 'bg-blue-500 text-white',
    green: 'bg-green-500 text-white',
    purple: 'bg-purple-500 text-white',
    orange: 'bg-orange-500 text-white',
    red: 'bg-red-500 text-white',
  };

  const trendColorClasses = {
    increase: 'text-green-600',
    decrease: 'text-red-600',
  };

  return (
    <div className="bg-white dark:bg-gray-800 overflow-hidden rounded-lg border border-gray-200 dark:border-gray-700 shadow-sm hover:shadow-md transition-shadow">
      <div className="p-6">
        <div className="flex items-center">
          <div className="flex-shrink-0">
            <div className={`w-10 h-10 rounded-lg flex items-center justify-center ${colorClasses[color]}`}>
              <Icon className="h-6 w-6" />
            </div>
          </div>
          <div className="ml-4 w-0 flex-1">
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                  {title}
                </p>
                <div className="text-2xl font-semibold text-gray-900 dark:text-white">
                  {loading ? (
                    <div className="animate-pulse">
                      <div className="h-8 w-16 bg-gray-200 dark:bg-gray-600 rounded"></div>
                    </div>
                  ) : (
                    <span>{value.toLocaleString()}</span>
                  )}
                </div>
                {subtitle && (
                  <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">{subtitle}</p>
                )}
              </div>
              {trend && !loading && (
                <div className={`text-sm font-medium ${trendColorClasses[trend.type]}`}>
                  <span className="flex items-center">
                    {trend.type === 'increase' ? '↗' : '↘'} {trend.value}%
                  </span>
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
