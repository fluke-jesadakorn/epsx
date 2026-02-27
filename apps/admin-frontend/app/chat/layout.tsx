import type { ReactNode } from 'react';

export default function ChatLayout({ children }: { children: ReactNode }) {
  return <div className="h-full">{children}</div>;
}

export const dynamic = 'force-dynamic';
