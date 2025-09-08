'use client'

import { useRouter } from 'next/navigation'

interface AdminPaginationButtonProps {
  page: number
  currentParams: string
  disabled?: boolean
  className?: string
  children: React.ReactNode
}

export default function AdminPaginationButton({
  page,
  currentParams,
  disabled = false,
  className = '',
  children
}: AdminPaginationButtonProps) {
  const router = useRouter()

  const handleClick = () => {
    if (disabled) return
    
    const params = new URLSearchParams(currentParams)
    params.set('page', String(page))
    router.push(`/analytics/eps?${params.toString()}`)
  }

  return (
    <button
      onClick={handleClick}
      disabled={disabled}
      className={className}
    >
      {children}
    </button>
  )
}