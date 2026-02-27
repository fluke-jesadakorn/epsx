import type { Metadata } from 'next';
import { notFound } from 'next/navigation';
import { getNewsBySlug } from '@/app/actions/news';
import { NewsDetail } from '@/components/news/news-detail';

export const revalidate = 300;

interface Props {
  params: Promise<{ slug: string }>;
}

export async function generateMetadata({ params }: Props): Promise<Metadata> {
  const { slug } = await params;
  const res = await getNewsBySlug(slug);
  if (res.success !== true || res.data === undefined) {
    return { title: 'Article Not Found — EPSX' };
  }
  return {
    title: `${res.data.title} — EPSX News`,
    description: res.data.summary ?? res.data.title,
  };
}

export default async function NewsDetailPage({ params }: Props) {
  const { slug } = await params;
  const res = await getNewsBySlug(slug);
  if (res.success !== true || res.data === undefined) { notFound(); }
  return <NewsDetail article={res.data} />;
}
