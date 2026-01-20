'use client';

/**
 * ADMIN AUTH PAGE
 * Uses shared AuthModal for consistent premium authentication experience
 * Modal auto-opens on page load for admin authentication
 */

import { PageLayout, PageSkeleton } from '@/components/shared';
import { AuthModal } from '@/shared/components/auth';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { useRouter, useSearchParams } from 'next/navigation';
import { Suspense, useEffect, useState } from 'react';
import { useAccount } from 'wagmi';

function AuthPageContent() {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [showModal, setShowModal] = useState(true);
  const [mounted, setMounted] = useState(false);

  const { isConnected } = useAccount();
  const { isAuthenticated, user, hasPermissionForDisplay } = useSharedAuth();

  const returnUrl = searchParams.get('return_url') || '/';
  const decodedReturnUrl = decodeURIComponent(returnUrl);
  const finalReturnUrl =
    decodedReturnUrl === '/' || decodedReturnUrl === '/auth' || decodedReturnUrl === '/login'
      ? '/'
      : decodedReturnUrl;

  const reason = searchParams.get('reason');

  useEffect(() => {
    setMounted(true);
  }, []);

  // Redirect if already authenticated with admin permissions
  useEffect(() => {
    if (!mounted) return;

    const hasAdminPermission =
      hasPermissionForDisplay('admin:*:*') ||
      user?.permissions?.some((p) => p.startsWith('admin:'));

    if (isAuthenticated && user && hasAdminPermission) {
      router.push(finalReturnUrl);
      router.refresh();
    }
  }, [mounted, isAuthenticated, user, hasPermissionForDisplay, router, finalReturnUrl]);

  const handleAuthSuccess = () => {
    router.push(finalReturnUrl);
    router.refresh();
  };

  // Don't allow closing modal on auth page - user must authenticate
  const handleClose = () => {
    // Only allow close if going back is possible
    if (typeof window !== 'undefined' && window.history.length > 1) {
      router.back();
    }
  };

  if (!mounted) {
    return <PageSkeleton showHeader={false} stats={0} rows={0} />;
  }

  // Check if already authenticated
  const hasAdminPermission =
    hasPermissionForDisplay('admin:*:*') ||
    user?.permissions?.some((p) => p.startsWith('admin:'));

  if (isAuthenticated && user && hasAdminPermission) {
    return (
      <PageLayout>
        <div className="flex flex-col items-center justify-center py-16">
          <div className="w-16 h-16 bg-success/10 rounded-2xl flex items-center justify-center mb-4">
            <span className="text-3xl">✅</span>
          </div>
          <p className="text-lg font-medium text-foreground">Admin Access Granted!</p>
          <p className="text-muted-foreground">Redirecting...</p>
        </div>
      </PageLayout>
    );
  }

  return (
    <PageLayout>
      {/* Reason message display */}
      {reason && (
        <div className="fixed top-0 left-0 right-0 p-3 bg-destructive text-destructive-foreground text-center text-sm z-50">
          {reason === 'no-session' && 'Your session has expired. Please sign in again.'}
          {reason === 'no-admin-permissions' && 'Admin permissions required.'}
        </div>
      )}

      <AuthModal
        isOpen={showModal}
        onClose={handleClose}
        variant="admin"
        onSuccess={handleAuthSuccess}
      />
    </PageLayout>
  );
}

export default function AuthPage() {
  return (
    <Suspense fallback={<PageSkeleton showHeader={false} stats={0} rows={0} />}>
      <AuthPageContent />
    </Suspense>
  );
}
