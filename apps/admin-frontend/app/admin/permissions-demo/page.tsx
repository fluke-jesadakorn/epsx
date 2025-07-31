import { Metadata } from 'next';
import { ModulePermissionDemo } from '@/components/admin/ModulePermissionDemo';

export const metadata: Metadata = {
  title: 'Module Permissions Demo | Admin',
  description: 'Demonstration of the module-based permission system',
};

export default function PermissionsDemoPage() {
  return (
    <div className="min-h-screen bg-gray-50">
      <ModulePermissionDemo />
    </div>
  );
}
