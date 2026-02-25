'use client';

import { useRouter } from 'next/navigation';
import { useAnalyticsTransition } from './analytics-transition-provider';

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
  const { pending, start } = useAnalyticsTransition();

  const handleClick = () => {
    if (disabled || pending) {return;}

    const params = new URLSearchParams(currentParams);
    params.set('page', String(page));
    start(() => router.push(`/analytics?${params.toString()}`));
  };

  return (
    <button
      onClick={handleClick}
      disabled={disabled || pending}
      className={className}
    >
      {children}
    </button>
  );
}