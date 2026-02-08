'use client';

import { useRouter } from 'next/navigation';

interface LimitSelectorFormProps {
  currentParams: string;
  currentLimit: number;
}

export default function LimitSelectorForm({ currentParams, currentLimit }: LimitSelectorFormProps) {
  const router = useRouter();

  const handleLimitChange = (newLimit: string) => {
    const params = new URLSearchParams(currentParams);
    params.set('limit', newLimit);
    params.set('page', '1'); // Reset to page 1
    router.push(`/analytics?${params.toString()}`);
  };

  return (
    <div className="flex items-center gap-2">
      <label htmlFor="limit-selector" className="text-sm text-slate-700 dark:text-slate-200 font-medium">
        Items per page:
      </label>
      <select
        id="limit-selector"
        value={currentLimit}
        onChange={(e) => handleLimitChange(e.target.value)}
        className="px-3 py-2 text-sm border border-orange-200 dark:border-orange-400/30 rounded-lg bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm text-slate-700 dark:text-slate-200 hover:bg-orange-50 dark:hover:bg-orange-900/30 hover:border-orange-300 dark:hover:border-orange-400 focus:outline-none focus:ring-2 focus:ring-orange-500 dark:focus:ring-orange-400 focus:border-orange-500 dark:focus:border-orange-400 min-h-[44px] transition-all duration-200"
      >
        <option value="10">10</option>
        <option value="20">20</option>
        <option value="50">50</option>
        <option value="100">100</option>
      </select>
    </div>
  );
}