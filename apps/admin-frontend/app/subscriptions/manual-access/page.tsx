import { redirect } from 'next/navigation';

export const dynamic = 'force-dynamic';

/**
 * Legacy /subscriptions/manual-access route - redirects to unified Access Management
 */
export default function LegacyManualAccessPage() {
    redirect('/subscriptions?type=manual');
}
