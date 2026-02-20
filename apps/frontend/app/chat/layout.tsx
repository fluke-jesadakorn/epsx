import type { ReactNode } from 'react';

export default function ChatLayout({ children }: { children: ReactNode }) {
  return <div className="min-h-screen bg-slate-50 dark:bg-background">{children}</div>;
}
