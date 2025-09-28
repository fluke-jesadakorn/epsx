'use client';

import { useRouter } from 'next/navigation';

interface PaginationButtonProps {
  page: number;
  currentParams: string;
  disabled?: boolean;
  className?: string;
  children: React.ReactNode;
}

export default function PaginationButton({
  page,
  currentParams,
  disabled = false,
  className = '',
  children
}: PaginationButtonProps) {
  const router = useRouter();

  const handleClick = () => {
    if (disabled) return;
    
    const params = new URLSearchParams(currentParams);
    params.set('page', String(page));
    router.push(`/analytics?${params.toString()}`);
  };

  return (
    <button
      onClick={handleClick}
      disabled={disabled}
      className={className}
    >
      {children}
    </button>
  );
}