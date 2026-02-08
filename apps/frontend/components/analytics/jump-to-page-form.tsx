'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';

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
      <span className="text-sm text-slate-700 dark:text-slate-200 font-medium">Go to page:</span>
      <input
        type="number"
        min={1}
        max={totalPages}
        value={page}
        onChange={(e) => setPage(e.target.value)}
        placeholder={String(currentPage)}
        className="w-16 px-2 py-1 text-sm border border-orange-200 dark:border-orange-400/30 rounded-lg bg-white/60 dark:bg-gray-800/60 backdrop-blur-sm text-slate-700 dark:text-slate-200 text-center hover:bg-orange-50 dark:hover:bg-orange-900/30 hover:border-orange-300 dark:hover:border-orange-400 focus:outline-none focus:ring-2 focus:ring-orange-500 dark:focus:ring-orange-400 focus:border-orange-500 dark:focus:border-orange-400 transition-all duration-200"
      />
      <Button 
        type="submit" 
        size="sm"
        className="bg-gradient-to-r from-orange-500 to-pink-500 hover:from-orange-600 hover:to-pink-600 text-white shadow-lg transition-all duration-200"
      >
        Go
      </Button>
    </form>
  );
}