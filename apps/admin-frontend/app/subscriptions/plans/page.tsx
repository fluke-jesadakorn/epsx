import { redirect } from 'next/navigation';

export const dynamic = 'force-dynamic';

/**
 * Legacy /subscriptions/plans route - redirects to unified Access Management
 */
export default function LegacyPlansPage() {
  redirect('/subscriptions?type=subscription');
}
