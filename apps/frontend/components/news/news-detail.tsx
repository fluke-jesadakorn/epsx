import type { NewsArticle } from '@/shared/api/news';
import { resolveNewsImageUrl } from '@/shared/api/news';
import { ArrowLeft, Calendar, Clock } from 'lucide-react';
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

function readingTime(content: string): number {
  const words = content.trim().split(/\s+/).length;
  return Math.max(1, Math.ceil(words / 200));
}

interface Props {
  article: NewsArticle;
}

export function NewsDetail({ article }: Props) {
  const coverUrl = resolveNewsImageUrl(article.cover_image_url);
  const mins = readingTime(article.content);

  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-10 max-w-3xl">
        {/* Back */}
        <Link
          href="/news"
          className="inline-flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors mb-8 group"
        >
          <ArrowLeft className="w-4 h-4 transition-transform group-hover:-translate-x-0.5" />
          Back to News
        </Link>

        {/* Cover image */}
        {coverUrl !== null && (
          <div className="relative w-full mb-8 rounded-2xl overflow-hidden">
            <img
              src={coverUrl}
              alt={article.title}
              className="w-full object-cover max-h-[420px]"
            />
            <div className="absolute inset-0 bg-gradient-to-t from-background/60 via-transparent to-transparent" />
          </div>
        )}

        {/* Tags */}
        {article.tags.length > 0 && (
          <div className="flex flex-wrap gap-2 mb-5">
            {article.tags.map((tag) => (
              <span
                key={tag}
                className="px-3 py-1 rounded-full text-xs font-semibold tracking-wide uppercase bg-[#1fc7d4]/10 text-[#1fc7d4] border border-[#1fc7d4]/20"
              >
                {tag}
              </span>
            ))}
          </div>
        )}

        {/* Title */}
        <h1 className="text-3xl sm:text-4xl font-bold text-foreground leading-tight mb-5">
          {article.title}
        </h1>

        {/* Meta row */}
        <div className="flex items-center gap-5 text-sm text-muted-foreground mb-8 pb-8 border-b border-border/30">
          <span className="flex items-center gap-1.5">
            <Calendar className="w-3.5 h-3.5" />
            {formatDate(article.published_at)}
          </span>
          <span className="flex items-center gap-1.5">
            <Clock className="w-3.5 h-3.5" />
            {mins} min read
          </span>
        </div>

        {/* Markdown content */}
        <div className="prose prose-neutral dark:prose-invert max-w-none prose-headings:font-semibold prose-h2:text-xl prose-h2:mt-8 prose-p:leading-relaxed prose-a:text-[#1fc7d4] prose-a:no-underline hover:prose-a:underline">
          <MarkdownAsync remarkPlugins={[remarkGfm]}>
            {article.content}
          </MarkdownAsync>
        </div>

        {/* Footer CTA */}
        <div className="mt-12 pt-8 border-t border-border/30">
          <Link
            href="/news"
            className="inline-flex items-center gap-2 text-sm font-medium text-muted-foreground hover:text-foreground transition-colors group"
          >
            <ArrowLeft className="w-4 h-4 transition-transform group-hover:-translate-x-0.5" />
            Back to all articles
          </Link>
        </div>
      </div>
    </div>
  );
}
