import { MediaBrowser } from '@/components/media/media-browser';
import { listMediaAction } from '@/app/media/actions';
import type { BucketName } from '@/shared/api/media';

export const revalidate = 60;

const BUCKETS: BucketName[] = ['news', 'chat', 'notifications', 'public'];

export default async function MediaPage({ searchParams }: { searchParams: Promise<{ bucket?: string }> }) {
  const { bucket: bucketParam } = await searchParams;
  const bucket = (BUCKETS.includes(bucketParam as BucketName) ? bucketParam : 'news') as BucketName;

  const res = await listMediaAction(bucket, undefined, 200);
  const files = res.success === true ? (res.data ?? []) : [];

  return <MediaBrowser key={bucket} files={files} bucket={bucket} buckets={BUCKETS} />;
}
