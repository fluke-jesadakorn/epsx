import { NewsManagement } from '@/components/news/news-management';
import { listNewsAction } from '@/app/news/actions';

export const revalidate = 30;

type StatusFilter = 'all' | 'draft' | 'published';

export default async function NewsPage({ searchParams }: { searchParams: Promise<{ page?: string; status?: string }> }) {
  const { page: pageParam, status: statusParam } = await searchParams;
  const page = Math.max(1, Number(pageParam ?? '1'));
  const status = (['draft', 'published'].includes(statusParam ?? '') ? statusParam : 'all') as StatusFilter;

  const res = await listNewsAction(page, 20, status === 'all' ? undefined : status);
  const articles = res.success === true ? (res.data?.articles ?? []) : [];
  const total = res.success === true ? (res.data?.total ?? 0) : 0;

  return <NewsManagement articles={articles} total={total} page={page} status={status} />;
}
