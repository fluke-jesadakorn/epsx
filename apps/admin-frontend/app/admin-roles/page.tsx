import { Metadata } from 'next';
import { AdminRoleManagement } from '@/components/admin/AdminRoleManagement';

export const metadata: Metadata = {
  title: 'Admin Role Management - EPSX Admin',
  description: 'Manage granular admin module assignments and permissions',
};

export default function AdminRolesPage() {
  return (
    <div className="p-6">
      <AdminRoleManagement />
    </div>
  );
}