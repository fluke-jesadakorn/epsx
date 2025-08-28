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
      <label htmlFor="limit-selector" className="text-sm text-gray-600">
        Items per page:
      </label>
      <select
        id="limit-selector"
        value={currentLimit}
        onChange={(e) => handleLimitChange(e.target.value)}
        className="px-3 py-2 text-sm border border-gray-300 rounded-lg bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-orange-500 min-h-[44px]"
      >
        <option value="10">10</option>
        <option value="20">20</option>
        <option value="50">50</option>
        <option value="100">100</option>
      </select>
    </div>
  );
}