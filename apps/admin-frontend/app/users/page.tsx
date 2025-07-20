import { AdminGuard } from '@/components/auth/AdminGuard';
import { AdminLayout } from '@/components/layout/AdminLayout';
import { EnhancedUserList } from '@/components/admin/EnhancedUserList';

export default function UsersPage() {
  return (
    <AdminGuard>
      <AdminLayout>
        <div className="space-y-6">
          <EnhancedUserList />
        </div>
      </AdminLayout>
    </AdminGuard>
  );
}
