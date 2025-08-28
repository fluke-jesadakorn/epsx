'use client';

import { useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';

interface PaginationButtonProps {
  page: number;
  currentParams: string;
  disabled?: boolean;
  variant?: 'default' | 'outline';
  className?: string;
  children: React.ReactNode;
}

export default function PaginationButton({
  page,
  currentParams,
  disabled = false,
  variant = 'outline',
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
    <Button
      onClick={handleClick}
      disabled={disabled}
      variant={variant}
      className={className}
    >
      {children}
    </Button>
  );
}