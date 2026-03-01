import type { NewsArticle } from '@/shared/api/news';
import { resolveNewsImageUrl } from '@/shared/api/news';
import { ArrowLeft } from 'lucide-react';
import Link from 'next/link';
import { MarkdownAsync } from 'react-markdown';
import remarkGfm from 'remark-gfm';

function formatDate(dateStr: string | null): string {
  if (dateStr === null) { return ''; }
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}

interface Props {
  article: NewsArticle;
}

export function NewsDetail({ article }: Props) {
  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-12 max-w-3xl">
        {/* Back */}
        <Link
          href="/news"
          className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors mb-8"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to News
        </Link>

        {/* Cover image */}
        {resolveNewsImageUrl(article.cover_image_url) !== null && (
          <img
            src={resolveNewsImageUrl(article.cover_image_url) ?? ''}
            alt={article.title}
            className="w-full rounded-2xl mb-8 object-cover max-h-96"
          />
        )}

        {/* Meta */}
        <div className="flex flex-wrap gap-2 mb-4">
          {article.tags.map((tag) => (
            <span
              key={tag}
              className="px-2.5 py-1 rounded-full text-xs font-medium bg-[#1fc7d4]/10 text-[#1fc7d4]"
            >
              {tag}
            </span>
          ))}
        </div>

        <h1 className="text-3xl font-bold text-foreground mb-4">{article.title}</h1>

        <div className="text-sm text-muted-foreground mb-8">
          {formatDate(article.published_at)}
        </div>

        {/* Markdown content */}
        <div className="prose prose-neutral dark:prose-invert max-w-none">
          <MarkdownAsync remarkPlugins={[remarkGfm]}>
            {article.content}
          </MarkdownAsync>
        </div>
      </div>
    </div>
  );
}
