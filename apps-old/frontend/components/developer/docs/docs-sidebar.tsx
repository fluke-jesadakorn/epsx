'use client';

import type { EndpointCategory } from './data/endpoints';

interface DocsSidebarProps {
  categories: EndpointCategory[];
  activeSection: string | null;
  onSelect: (id: string) => void;
  open: boolean;
  onToggle: () => void;
}

export function DocsSidebar({ categories, activeSection, onSelect, open, onToggle }: DocsSidebarProps) {
  return (
    <>
      {/* Mobile toggle */}
      <button
        type="button"
        onClick={onToggle}
        className="fixed bottom-4 right-4 z-30 rounded-full bg-gradient-to-r from-[#7645d9] to-[#5a33b8] p-3 text-white shadow-lg lg:hidden"
      >
        <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={open ? 'M6 18L18 6M6 6l12 12' : 'M4 6h16M4 12h16M4 18h16'} />
        </svg>
      </button>

      {/* Sidebar */}
      <aside className={`${open ? 'translate-x-0' : '-translate-x-full'} fixed top-0 left-0 z-20 h-full w-56 border-r border-border/20 bg-card pt-20 transition-transform lg:relative lg:block lg:h-auto lg:w-56 lg:shrink-0 lg:translate-x-0 lg:border-r-0 lg:pt-0`}>
        <div className="px-4 py-4">
          <h3 className="mb-3 text-xs font-semibold uppercase tracking-wider text-muted-foreground">API Reference</h3>
          <nav className="space-y-1">
            {categories.map((cat) => (
              <button
                key={cat.id}
                type="button"
                onClick={() => onSelect(cat.id)}
                className={`flex w-full items-center gap-2 rounded-lg px-3 py-2 text-left text-sm transition-colors ${
                  activeSection === cat.id
                    ? 'bg-[#7645d9]/10 font-medium text-[#7645d9]'
                    : 'text-muted-foreground hover:bg-background hover:text-foreground'
                }`}
              >
                <span className="h-1.5 w-1.5 rounded-full bg-current opacity-50" />
                {cat.title}
                <span className="ml-auto text-xs text-muted-foreground/50">{cat.endpoints.length}</span>
              </button>
            ))}
          </nav>
        </div>

        {/* Quick start */}
        <div className="mx-4 mt-4 rounded-xl border border-border/10 bg-background p-3">
          <p className="text-xs font-medium text-foreground">Quick Start</p>
          <p className="mt-1 text-[11px] leading-relaxed text-muted-foreground">
            Pass your API key as a Bearer token in the Authorization header.
          </p>
          <code className="mt-2 block rounded-lg bg-slate-900 p-2 font-mono text-[10px] text-gray-300">
            Authorization: Bearer &lt;key&gt;
          </code>
        </div>
      </aside>

      {/* Mobile overlay */}
      {open && (
        <button
          type="button"
          onClick={onToggle}
          className="fixed inset-0 z-10 bg-black/40 lg:hidden"
          aria-label="Close sidebar"
        />
      )}
    </>
  );
}
