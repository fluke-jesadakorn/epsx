'use client';

import type { EndpointCategory } from './data/endpoints';
import { EndpointCard } from './endpoint-card';

interface EndpointSectionProps {
  category: EndpointCategory;
  apiKeys: { id: string; name: string; full_key?: string; key_prefix: string }[];
}

export function EndpointSection({ category, apiKeys }: EndpointSectionProps) {
  return (
    <section id={`section-${category.id}`} className="scroll-mt-20">
      <div className="mb-4">
        <h2 className="text-xl font-bold text-foreground">{category.title}</h2>
        <p className="mt-1 text-sm text-muted-foreground">{category.desc}</p>
      </div>
      <div className="space-y-3">
        {category.endpoints.map((ep) => (
          <EndpointCard key={`${ep.method}-${ep.path}`} endpoint={ep} apiKeys={apiKeys} />
        ))}
      </div>
    </section>
  );
}
