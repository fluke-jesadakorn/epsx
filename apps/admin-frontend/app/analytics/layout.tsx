import { ReactNode } from 'react'

interface AnalyticsLayoutProps {
  children: ReactNode
}

export default function AnalyticsLayout({ children }: AnalyticsLayoutProps) {
  return (
    <div className="analytics-layout">
      {children}
    </div>
  )
}