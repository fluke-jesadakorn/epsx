'use client';

import { ArrowLeft, Loader2 } from 'lucide-react';
import Link from 'next/link';
import { useRouter, useParams } from 'next/navigation';
import { useEffect, useState } from 'react';

import { PolicyCard } from '@/components/access-control';
import { type AccessPolicy, getPolicyEditUrl, getPolicyMembersUrl } from '@/components/access-control/types';
import { PageLayout, PageSkeleton } from '@/components/shared';
import { Button } from '@/components/ui/button';
import { accessPolicyClient } from '@/lib/api/access-policy-client';
import { useSharedAuth } from '@/shared/components/auth/Provider';

/**
 * Policy Detail Page
 * Shows policy details and redirects to appropriate edit page
 */
export default function PolicyDetailPage() {
  const { isAuthenticated, isLoading: authLoading } = useSharedAuth();
  const router = useRouter();
  const params = useParams();
  const policyId = params.id as string;

  const [policy, setPolicy] = useState<AccessPolicy | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      router.push('/auth');
      return;
    }

    if (!isAuthenticated || !policyId) return;

    const loadPolicy = async () => {
      setIsLoading(true);
      setError(null);

      try {
        const data = await accessPolicyClient.getPolicy(policyId);
        if (data) {
          setPolicy(data);
        } else {
          setError('Policy not found');
        }
      } catch (err) {
        console.error('Failed to load policy:', err);
        setError(err instanceof Error ? err.message : 'Failed to load policy');
      } finally {
        setIsLoading(false);
      }
    };

    loadPolicy();
  }, [isAuthenticated, authLoading, policyId, router]);

  if (authLoading || !isAuthenticated) {
    return <PageSkeleton showHeader stats={0} rows={3} />;
  }

  if (isLoading) {
    return (
      <PageLayout>
        <div className="flex items-center justify-center py-20">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      </PageLayout>
    );
  }

  if (error || !policy) {
    return (
      <PageLayout>
        <div className="flex flex-col items-center justify-center py-20 text-center">
          <div className="text-4xl mb-4">⚠️</div>
          <h2 className="text-xl font-semibold text-foreground mb-2">
            {error || 'Policy not found'}
          </h2>
          <p className="text-muted-foreground mb-6">
            The policy you're looking for doesn't exist or you don't have access.
          </p>
          <Link href="/subscriptions">
            <Button>
              <ArrowLeft className="h-4 w-4 mr-2" />
              Back to Access Management
            </Button>
          </Link>
        </div>
      </PageLayout>
    );
  }

  const editUrl = getPolicyEditUrl(policy);
  const membersUrl = getPolicyMembersUrl(policy);

  return (
    <PageLayout>
      <div className="space-y-6">
        {/* Header */}
        <div className="flex items-center gap-4">
          <Link
            href="/subscriptions"
            className="p-2 rounded-xl bg-card hover:bg-muted transition-colors border border-border"
          >
            <ArrowLeft className="h-5 w-5" />
          </Link>
          <div>
            <h1 className="text-2xl font-bold text-foreground">{policy.name}</h1>
            <p className="text-muted-foreground">Policy Details</p>
          </div>
        </div>

        {/* Policy Card */}
        <PolicyCard policy={policy} />

        {/* Actions */}
        <div className="flex flex-wrap gap-4">
          <Link href={editUrl}>
            <Button className="gap-2">
              Edit Policy
            </Button>
          </Link>
          <Link href={membersUrl}>
            <Button variant="outline" className="gap-2">
              {policy.sourceType === 'plan' ? 'View Subscribers' : 'Manage Members'}
            </Button>
          </Link>
          <Link href="/subscriptions">
            <Button variant="ghost" className="gap-2">
              <ArrowLeft className="h-4 w-4" />
              Back to List
            </Button>
          </Link>
        </div>

        {/* Permissions List */}
        {policy.permissions.length > 0 && (
          <div className="rounded-2xl bg-card border border-border p-6">
            <h3 className="text-lg font-semibold text-foreground mb-4">
              Permissions ({policy.permissions.length})
            </h3>
            <div className="flex flex-wrap gap-2">
              {policy.permissions.map((perm) => (
                <span
                  key={perm}
                  className="text-sm px-3 py-1.5 bg-muted text-muted-foreground rounded-lg font-mono"
                >
                  {perm}
                </span>
              ))}
            </div>
          </div>
        )}

        {/* Metadata */}
        <div className="rounded-2xl bg-card border border-border p-6">
          <h3 className="text-lg font-semibold text-foreground mb-4">Details</h3>
          <dl className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <dt className="text-sm text-muted-foreground">Type</dt>
              <dd className="text-foreground font-medium capitalize">{policy.type.replace('_', ' ')}</dd>
            </div>
            <div>
              <dt className="text-sm text-muted-foreground">Status</dt>
              <dd className="text-foreground font-medium">{policy.isActive ? 'Active' : 'Inactive'}</dd>
            </div>
            <div>
              <dt className="text-sm text-muted-foreground">Members</dt>
              <dd className="text-foreground font-medium">{policy.memberCount}</dd>
            </div>
            {policy.pricing && (
              <div>
                <dt className="text-sm text-muted-foreground">Price</dt>
                <dd className="text-foreground font-medium">
                  {policy.pricing.amount === 0 ? 'Free' : `$${policy.pricing.amount} ${policy.pricing.currency}`}
                </dd>
              </div>
            )}
            {policy.revenue !== undefined && (
              <div>
                <dt className="text-sm text-muted-foreground">Revenue (30d)</dt>
                <dd className="text-foreground font-medium">${policy.revenue.toFixed(2)}</dd>
              </div>
            )}
            {policy.expiryDays && (
              <div>
                <dt className="text-sm text-muted-foreground">Default Expiry</dt>
                <dd className="text-foreground font-medium">{policy.expiryDays} days</dd>
              </div>
            )}
            <div>
              <dt className="text-sm text-muted-foreground">Created</dt>
              <dd className="text-foreground font-medium">
                {new Date(policy.createdAt).toLocaleDateString()}
              </dd>
            </div>
            <div>
              <dt className="text-sm text-muted-foreground">Last Updated</dt>
              <dd className="text-foreground font-medium">
                {new Date(policy.updatedAt).toLocaleDateString()}
              </dd>
            </div>
          </dl>
        </div>
      </div>
    </PageLayout>
  );
}
