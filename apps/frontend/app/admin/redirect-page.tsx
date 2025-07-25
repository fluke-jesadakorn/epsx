'use client';

import { useEffect } from 'react';
import { getAdminConfig } from '@/lib/actions/admin.server';

/**
 * Legacy admin page that redirects to the new admin frontend
 * This ensures seamless migration of existing bookmarks and links
 */
export default function AdminRedirectPage() {
  useEffect(() => {
    // Fetch admin URL from server action
    getAdminConfig()
      .then(config => {
        // Redirect to admin frontend with current path preserved
        window.location.replace(`${config.adminUrl}/admin`);
      })
      .catch(() => {
        // Fallback URL if fetch fails
        window.location.replace('http://localhost:3001/admin');
      });
  }, []);

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="text-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
        <p className="text-gray-600">Redirecting to Admin Panel...</p>
      </div>
    </div>
  );
}
