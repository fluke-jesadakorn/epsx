'use client';

/**
 * ADMIN AUTH PAGE
 * Uses shared AuthModal for consistent premium authentication experience
 * Modal auto-opens on page load for admin authentication
 */

import { AuthModal } from '@/shared/components/auth';
import { useSharedAuth } from '@/shared/components/auth/Provider';
import { useRouter, useSearchParams } from 'next/navigation';
import { useEffect, useState } from 'react';
import { useAccount } from 'wagmi';

export default function AuthPage() {
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
    return (
      <div className="auth-page-loading">
        <div className="auth-page-spinner" />
        <p>Loading...</p>
      </div>
    );
  }

  // Check if already authenticated
  const hasAdminPermission =
    hasPermissionForDisplay('admin:*:*') ||
    user?.permissions?.some((p) => p.startsWith('admin:'));

  if (isAuthenticated && user && hasAdminPermission) {
    return (
      <div className="auth-page-loading">
        <p>✅ Admin Access Granted! Redirecting...</p>
      </div>
    );
  }

  return (
    <div className="auth-page-bg">
      {/* Reason message display */}
      {reason && (
        <div className="auth-reason-banner">
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

      {/* Background styling */}
      <style jsx>{`
        .auth-page-bg {
          min-height: 100vh;
          display: flex;
          align-items: center;
          justify-content: center;
          background: linear-gradient(135deg, #0f0f1a 0%, #1a1a2e 50%, #16162a 100%);
        }
        .auth-page-loading {
          min-height: 100vh;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          color: #fff;
          gap: 1rem;
        }
        .auth-page-spinner {
          width: 40px;
          height: 40px;
          border: 3px solid rgba(245, 158, 11, 0.3);
          border-top-color: #f59e0b;
          border-radius: 50%;
          animation: spin 0.8s linear infinite;
        }
        @keyframes spin {
          to { transform: rotate(360deg); }
        }
        .auth-reason-banner {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          padding: 0.75rem;
          background: rgba(239, 68, 68, 0.9);
          color: #fff;
          text-align: center;
          font-size: 0.875rem;
          z-index: 100;
        }
      `}</style>
    </div>
  );
}
