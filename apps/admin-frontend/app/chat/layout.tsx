import type { ReactNode } from 'react';

import { checkPageAccess } from '@/lib/check-page-access';

export default async function ChatLayout({ children }: { children: ReactNode }) {
  await checkPageAccess('admin:chat:manage', '/chat');
  return <div className="h-full">{children}</div>;
}

export const dynamic = 'force-dynamic';
