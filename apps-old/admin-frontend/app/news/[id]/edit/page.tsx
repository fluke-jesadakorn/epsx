import { NewsEditor } from '@/components/news/news-editor';
import { getNewsAction } from '@/app/news/actions';

interface Props {
  params: Promise<{ id: string }>;
}

export default async function EditNewsPage({ params }: Props) {
  const { id } = await params;
  const res = await getNewsAction(id);
  const article = res.success ? (res.data ?? null) : null;
  return <NewsEditor article={article} />;
}
