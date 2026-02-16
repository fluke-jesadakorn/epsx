import Link from 'next/link';
import { CATEGORIES, FEATURES } from './data';
import { ScreenshotImg } from './screenshot-img';

export const metadata = {
  title: 'EPSX Manual - Feature Guide',
  description: 'Complete feature guide with screenshots for the EPSX platform',
};

export default function ManualPage() {
  return (
    <div className="min-h-screen bg-gray-950 text-gray-100">
      <div className="flex">
        {/* Sidebar */}
        <aside className="sticky top-0 h-screen w-56 shrink-0 overflow-y-auto border-r border-gray-800 bg-gray-900/50 p-4">
          <h2 className="mb-4 text-lg font-semibold text-white">Categories</h2>
          <nav className="flex flex-col gap-1">
            {CATEGORIES.map(cat => (
              <a
                key={cat}
                href={`#${cat.toLowerCase().replace(/\s+/g, '-')}`}
                className="rounded px-3 py-1.5 text-sm text-gray-400 hover:bg-gray-800 hover:text-white transition-colors"
              >
                {cat}
              </a>
            ))}
          </nav>
        </aside>

        {/* Main content */}
        <main className="flex-1 p-8">
          <div className="mx-auto max-w-6xl">
            <h1 className="mb-2 text-3xl font-bold">EPSX Feature Manual</h1>
            <p className="mb-8 text-gray-400">
              Complete guide to all platform features. Screenshots auto-generated from E2E tests.
            </p>

            {CATEGORIES.map(cat => {
              const items = FEATURES.filter(f => f.category === cat);
              if (items.length === 0) return null;
              return (
                <section key={cat} id={cat.toLowerCase().replace(/\s+/g, '-')} className="mb-12">
                  <h2 className="mb-4 border-b border-gray-800 pb-2 text-xl font-semibold text-white">{cat}</h2>
                  <div className="grid gap-6 sm:grid-cols-2 lg:grid-cols-3">
                    {items.map(feature => (
                      <div key={feature.id} className="group overflow-hidden rounded-lg border border-gray-800 bg-gray-900/60 transition-colors hover:border-gray-600">
                        {/* Hero screenshot */}
                        <div className="relative aspect-video w-full overflow-hidden bg-gray-800">
                          <ScreenshotImg
                            src={`/screenshots/${feature.screenshots[0] ?? feature.id}.webp`}
                            alt={feature.name}
                          />
                        </div>

                        {/* Info */}
                        <div className="p-4">
                          <div className="mb-1 flex items-center gap-2">
                            <h3 className="font-medium text-white">{feature.name}</h3>
                            <span className="rounded bg-gray-800 px-1.5 py-0.5 text-xs text-gray-400 font-mono">
                              {feature.route}
                            </span>
                          </div>
                          <p className="mb-2 text-sm text-gray-400">{feature.desc}</p>
                          <Link
                            href={feature.route.includes('[') ? '#' : feature.route}
                            className="text-sm text-blue-400 hover:text-blue-300"
                          >
                            Open page &rarr;
                          </Link>
                        </div>
                      </div>
                    ))}
                  </div>
                </section>
              );
            })}
          </div>
        </main>
      </div>
    </div>
  );
}
