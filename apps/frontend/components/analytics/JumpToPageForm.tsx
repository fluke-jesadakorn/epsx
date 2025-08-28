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
      <span className="text-sm text-gray-600">Go to page:</span>
      <input
        type="number"
        min={1}
        max={totalPages}
        value={page}
        onChange={(e) => setPage(e.target.value)}
        placeholder={String(currentPage)}
        className="w-16 px-2 py-1 text-sm border border-gray-300 rounded text-center"
      />
      <Button type="submit" size="sm">Go</Button>
    </form>
  );
}