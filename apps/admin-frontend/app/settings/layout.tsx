import { checkPageAccess } from '@/lib/check-page-access';

export default async function SettingsLayout({ children }: { children: React.ReactNode }) {
  await checkPageAccess('admin:security:read', '/settings');
  return <>{children}</>;
}

export const dynamic = 'force-dynamic';
