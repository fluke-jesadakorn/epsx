import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { AdminUserManagement } from '@/components/admin/AdminUserManagement';
import { BulkUserLevelAssignment } from '@/components/admin/BulkUserLevelAssignment';

export default function UsersPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <div className="space-y-6">
          <AdminUserManagement />
          <BulkUserLevelAssignment />
        </div>
      </AdminLayout>
    </AdminGuard>
  );
}
