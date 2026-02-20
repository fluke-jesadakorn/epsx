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
    params.set('page', '1');
    router.push(`/analytics?${params.toString()}`);
  };

  return (
    <div className="flex items-center gap-2">
      <label htmlFor="limit-selector" className="text-sm text-slate-400">
        Per page:
      </label>
      <select
        id="limit-selector"
        value={currentLimit}
        onChange={(e) => handleLimitChange(e.target.value)}
        className="h-8 rounded-lg border border-gray-200 dark:border-white/[0.08] bg-gray-100 dark:bg-slate-800/60 px-2 text-sm text-slate-200 hover:bg-slate-700/60 focus:outline-none focus:ring-1 focus:ring-purple-500 transition-colors"
      >
        <option value="10">10</option>
        <option value="20">20</option>
        <option value="50">50</option>
        <option value="100">100</option>
      </select>
    </div>
  );
}
