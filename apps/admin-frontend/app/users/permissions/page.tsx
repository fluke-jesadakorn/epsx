import { PermissionManagement } from '@/components/permissions/PermissionManagement';
import { UnifiedAuth } from '@/lib/auth/unified-auth';
import { UnifiedAdminClient } from '@/lib/api/unified-admin-client';
import { notFound } from 'next/navigation';

export default async function UserPermissionsPage() {
  const session = await UnifiedAuth.getSession();
  
  if (!session?.user) {
    notFound();
  }
  
  if (!UnifiedAuth.hasPermission(session.user, 'admin:permissions:view')) {
    notFound();
  }
  
  const client = new UnifiedAdminClient();
  let users: any[] = [];
  
  try {
    const response = await client.getUsers({ limit: 100 });
    users = response.success ? (response.data as any) || [] : [];
  } catch (error) {
    console.error('Failed to load users:', error);
  }
  
  return (
    <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-6">
      <PermissionManagement 
        users={users}
        currentUser={session.user as any}
      />
    </div>
  );
}