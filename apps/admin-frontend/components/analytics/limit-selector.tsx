'use client';

import { memo } from 'react';

interface LimitSelectorProps {
  currentLimit: number;
  onLimitChange: (limit: number) => void;
  isLoading?: boolean;
}

const limitOptions = [10, 20, 50, 100];

const LimitSelector = memo<LimitSelectorProps>(({ currentLimit, onLimitChange, isLoading }) => {
  return (
    <div className="flex items-center gap-2">
      <label htmlFor="limit-selector" className="text-sm text-gray-600">
        Items per page:
      </label>
      <select
        id="limit-selector"
        value={currentLimit}
        onChange={(e) => onLimitChange(Number(e.target.value))}
        disabled={isLoading}
        className="px-3 py-2 text-sm border border-gray-300 rounded-lg bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-orange-500 disabled:opacity-50 disabled:cursor-not-allowed min-h-[44px]"
      >
        {limitOptions.map((limit) => (
          <option key={limit} value={limit}>
            {limit}
          </option>
        ))}
      </select>
    </div>
  );
});

LimitSelector.displayName = 'limit-selector';

export default LimitSelector;