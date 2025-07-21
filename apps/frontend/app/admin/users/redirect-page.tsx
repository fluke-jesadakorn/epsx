'use client';

import { useEffect } from 'react';

/**
 * Legacy admin users page that redirects to the new admin frontend
 * This ensures seamless migration of existing bookmarks and links
 */
export default function AdminUsersRedirectPage() {
  useEffect(() => {
    // Get the admin frontend URL
    const adminUrl = process.env.NEXT_PUBLIC_ADMIN_FRONTEND_URL || 'http://localhost:3001';
    
    // Redirect to admin frontend users page
    window.location.replace(`${adminUrl}/users`);
  }, []);

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="text-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
        <p className="text-gray-600">Redirecting to User Management...</p>
      </div>
    </div>
  );
}
