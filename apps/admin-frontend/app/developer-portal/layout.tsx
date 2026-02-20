import { checkPageAccess } from '@/lib/check-page-access';

export default async function DeveloperPortalLayout({ children }: { children: React.ReactNode }) {
  await checkPageAccess('admin:users:manage', '/developer-portal');
  return <>{children}</>;
}

export const dynamic = 'force-dynamic';
