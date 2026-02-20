'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';

interface JumpToPageFormProps {
  currentParams: string;
  currentPage: number;
  totalPages: number;
}

export default function JumpToPageForm({
  currentParams,
  currentPage,
  totalPages
}: JumpToPageFormProps) {
  const [page, setPage] = useState('');
  const router = useRouter();

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const pageNum = parseInt(page, 10);
    if (pageNum >= 1 && pageNum <= totalPages) {
      const params = new URLSearchParams(currentParams);
      params.set('page', String(pageNum));
      router.push(`/analytics?${params.toString()}`);
      setPage('');
    }
  };

  return (
    <form onSubmit={handleSubmit} className="flex items-center gap-2">
      <span className="text-sm text-slate-400">Go to:</span>
      <input
        type="number"
        min={1}
        max={totalPages}
        value={page}
        onChange={(e) => setPage(e.target.value)}
        placeholder={String(currentPage)}
        className="w-14 h-8 rounded-lg border border-gray-200 dark:border-white/[0.08] bg-gray-100 dark:bg-slate-800/60 px-2 text-sm text-slate-200 text-center focus:outline-none focus:ring-1 focus:ring-purple-500 transition-colors"
      />
      <button
        type="submit"
        className="h-8 rounded-lg bg-purple-600 px-3 text-sm font-medium text-white hover:bg-purple-500 transition-colors"
      >
        Go
      </button>
    </form>
  );
}
