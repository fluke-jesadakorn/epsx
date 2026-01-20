'use client';

import { ShieldCheck } from 'lucide-react';

import { GroupHub } from '@/components/group/GroupHub';
import { PageHeader, PageLayout } from '@/components/shared';

export const dynamic = 'force-dynamic';

/**
 * Group and Permission Management Page
 * Uses unified page components for consistent design
 */
export default function GroupAndPermissionPage() {
  return (
    <PageLayout>
      <PageHeader
        title="Group & Permission Hub"
        subtitle="Manage permission groups, assignments, and wallet memberships"
        icon={ShieldCheck}
        gradient="warning"
      />

      <GroupHub />
    </PageLayout>
  );
}
