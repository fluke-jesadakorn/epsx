import type { Metadata } from 'next';
import { getPublicNews } from '@/app/actions/news';
import { NewsList } from '@/components/news/news-list';

export const revalidate = 300;

export const metadata: Metadata = {
  title: 'News — EPSX',
  description: 'Latest news and updates from EPSX analytics platform',
};

export default async function NewsPage() {
  const res = await getPublicNews(1, 12);
  const data = res.success === true && res.data !== undefined
    ? res.data
    : { articles: [], total: 0, page: 1, limit: 12 };

  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-12">
        <div className="mb-10">
          <h1 className="text-4xl font-bold text-foreground">News</h1>
          <p className="mt-2 text-muted-foreground">Latest updates from EPSX</p>
        </div>
        <NewsList initial={data} />
      </div>
    </div>
  );
}
