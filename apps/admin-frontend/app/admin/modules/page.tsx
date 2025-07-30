import { Metadata } from 'next';
import { ModuleManagementClient } from '@/components/admin/ModuleManagementClient';

export const metadata: Metadata = {
  title: 'Module Management | Admin',
  description: 'Manage modules and user access assignments',
};

export default function ModuleManagementPage() {
  return (
    <div className="min-h-screen bg-gray-50">
      <ModuleManagementClient />
    </div>
  );
}
