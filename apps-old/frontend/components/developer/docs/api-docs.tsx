'use client';

import { useQuery } from '@tanstack/react-query';
import { ENDPOINT_CATEGORIES } from './data/endpoints';
import { DocsSidebar } from './docs-sidebar';
import { EndpointSection } from './endpoint-section';
import { useDocsState } from './hooks/use-docs-state';
import { getApiKeysAction } from '@/app/actions/developer';

export function ApiDocs() {
  const { activeSection, sidebarOpen, toggleSidebar, scrollToSection } = useDocsState();

  const { data: keysData } = useQuery({
    queryKey: ['api-keys-for-docs'],
    queryFn: async () => {
      const res = await getApiKeysAction({ status: 'active' });
       
      return (res as any)?.data?.items ?? [];
    },
    staleTime: 60_000,
  });
   
  const apiKeys = (keysData as any[]) ?? [];

  return (
    <div className="flex gap-6">
      <DocsSidebar
        categories={ENDPOINT_CATEGORIES}
        activeSection={activeSection}
        onSelect={scrollToSection}
        open={sidebarOpen}
        onToggle={toggleSidebar}
      />

      <div className="min-w-0 flex-1">
        {/* Hero */}
        <div className="mb-8">
          <div className="h-[3px] w-16 rounded-full bg-gradient-to-r from-[#7645d9] to-[#1fc7d4]" />
          <h1 className="mt-3 text-3xl font-bold text-foreground">API Reference</h1>
          <p className="mt-2 text-muted-foreground">
            Integrate EPSX analytics into your applications. Use your API key as a Bearer token — same endpoints, same data.
          </p>
        </div>

        {/* Auth guide card */}
        <div className="mb-8 rounded-2xl border border-border/20 bg-card p-5 shadow-xl">
          <h3 className="text-sm font-semibold text-foreground">Authentication</h3>
          <p className="mt-1 text-sm text-muted-foreground">
            All requests use the <code className="rounded bg-background px-1.5 py-0.5 text-xs">Authorization: Bearer &lt;token&gt;</code> header.
            Your API key works like a JWT — the middleware auto-detects the type.
          </p>
          <pre className="mt-3 rounded-xl bg-slate-900 p-3 font-mono text-xs text-gray-300">
            curl -H &quot;Authorization: Bearer YOUR_API_KEY&quot; https://api.epsx.io/api/analytics/rankings
          </pre>
        </div>

        {/* Endpoint sections */}
        <div className="space-y-10">
          {ENDPOINT_CATEGORIES.map((cat) => (
            <EndpointSection key={cat.id} category={cat} apiKeys={apiKeys} />
          ))}
        </div>
      </div>
    </div>
  );
}
