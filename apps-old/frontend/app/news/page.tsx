import type { Metadata } from 'next';
import { Newspaper } from 'lucide-react';
import { getPublicNews } from '@/app/actions/news';
import { NewsList } from '@/components/news/news-list';

export const revalidate = 60;

export const metadata: Metadata = {
  title: 'News — EPSX',
  description: 'Latest news and updates from EPSX analytics platform',
};

export default async function NewsPage({ searchParams }: { searchParams: Promise<{ page?: string }> }) {
  const { page: pageParam } = await searchParams;
  const page = Math.max(1, Number(pageParam ?? '1'));

  const res = await getPublicNews(page, 12);
  const data = res.success === true && res.data !== null
    ? res.data
    : { articles: [], total: 0, page, limit: 12 };

  return (
    <div className="relative min-h-screen bg-gray-50 dark:bg-slate-950">
      {/* Background orbs */}
      <div className="fixed inset-0 z-0 pointer-events-none overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-b from-white dark:from-slate-950 via-gray-50 dark:via-slate-900 to-white dark:to-slate-950" />
        <div className="absolute -top-40 -right-32 h-[500px] w-[500px] rounded-full bg-purple-600/8 dark:bg-purple-600/15 blur-3xl" />
        <div className="absolute top-1/3 -left-32 h-[400px] w-[400px] rounded-full bg-cyan-500/5 dark:bg-cyan-500/10 blur-3xl" />
        <div className="absolute bottom-20 right-1/4 h-[300px] w-[300px] rounded-full bg-[#7645d9]/5 dark:bg-[#7645d9]/10 blur-3xl" />
      </div>

      {/* Content */}
      <div className="relative z-10 mx-auto max-w-7xl px-4 py-12 sm:py-16">
        {/* Header */}
        <div className="mb-12 text-center">
          <div className="inline-flex items-center gap-2 px-3 py-1.5 rounded-full border border-[#1fc7d4]/20 bg-[#1fc7d4]/5 text-[#1fc7d4] text-xs font-semibold mb-5">
            <Newspaper className="w-3.5 h-3.5" />
            EPSX Platform
          </div>
          <h1 className="text-4xl sm:text-5xl font-extrabold text-foreground mb-4">
            {'News & '}
            <span className="bg-gradient-to-r from-[#7645d9] to-[#1fc7d4] bg-clip-text text-transparent">
              Updates
            </span>
          </h1>
          <p className="text-muted-foreground max-w-xl mx-auto leading-relaxed">
            Stay informed with the latest platform updates, feature releases, and market insights from the EPSX team.
          </p>
          {data.total > 0 && (
            <p className="mt-3 text-sm text-muted-foreground/60">
              {data.total} {data.total === 1 ? 'article' : 'articles'}
            </p>
          )}
        </div>

        <NewsList data={data} />
      </div>
    </div>
  );
}
