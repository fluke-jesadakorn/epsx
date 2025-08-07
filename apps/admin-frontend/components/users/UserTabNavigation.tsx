/**
 * User Tab Navigation Component
 * Server Component providing tab navigation for user profile pages
 */

import Link from 'next/link'
import { User, Shield, Package, Activity, CreditCard } from 'lucide-react'

interface UserTabNavigationProps {
  userId: string
}

export function UserTabNavigation({ userId }: UserTabNavigationProps) {
  const tabs = [
    {
      href: `/users/${userId}/overview`,
      label: 'Overview',
      icon: User,
      description: 'Basic info and quick actions',
      testId: 'overview-tab'
    },
    {
      href: `/users/${userId}/permissions`,
      label: 'Permissions',
      icon: Shield,
      description: 'Roles, policies, and custom permissions',
      testId: 'permissions-tab'
    },
    {
      href: `/users/${userId}/modules`,
      label: 'Modules',
      icon: Package,
      description: 'Module access and quotas',
      testId: 'modules-tab'
    },
    {
      href: `/users/${userId}/packages`,
      label: 'Packages',
      icon: CreditCard,
      description: 'Billing and subscription packages',
      testId: 'packages-tab'
    },
    {
      href: `/users/${userId}/activity`,
      label: 'Activity',
      icon: Activity,
      description: 'Login history and audit logs',
      testId: 'activity-tab'
    }
  ]

  return (
    <div className="border-b border-muted">
      <nav className="flex space-x-8" aria-label="User profile tabs">
        {tabs.map((tab) => {
          const Icon = tab.icon
          return (
            <Link
              key={tab.href}
              href={tab.href}
              className="flex items-center gap-2 py-4 px-1 border-b-2 border-transparent hover:border-muted-foreground/50 transition-colors group"
              data-testid={tab.testId}
            >
              <Icon className="h-4 w-4 text-muted-foreground group-hover:text-foreground" />
              <div className="flex flex-col">
                <span className="text-sm font-medium text-muted-foreground group-hover:text-foreground">
                  {tab.label}
                </span>
                <span className="text-xs text-muted-foreground hidden md:block">
                  {tab.description}
                </span>
              </div>
            </Link>
          )
        })}
      </nav>
    </div>
  )
}