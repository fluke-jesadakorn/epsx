import { checkPageAccess } from '@/lib/check-page-access';

export default async function AuditLogLayout({ children }: { children: React.ReactNode }) {
  await checkPageAccess('admin:audit:read', '/audit-log');
  return <>{children}</>;
}

export const dynamic = 'force-dynamic';
