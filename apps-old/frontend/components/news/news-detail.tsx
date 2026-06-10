import type { NewsArticle } from '@/shared/api/news';
import { resolveNewsImageUrl } from '@/shared/api/news';
import { ArrowLeft, Calendar, Clock } from 'lucide-react';
import Link from 'next/link';
import { MarkdownAsync } from 'react-markdown';
import remarkGfm from 'remark-gfm';

function fmtDate(d: string | null): string {
  if (d === null) { return ''; }
  return new Date(d).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}

function readMin(content: string): number {
  return Math.max(1, Math.ceil(content.trim().split(/\s+/).length / 200));
}

interface Props {
  article: NewsArticle;
}

export function NewsDetail({ article }: Props) {
  const coverUrl = resolveNewsImageUrl(article.cover_image_url);
  const mins = readMin(article.content);
  const hasCover = coverUrl !== null;

  return (
    <div className="min-h-screen bg-background">
      {/* ── Hero ── */}
      <section className="relative w-full overflow-hidden isolate">
        {hasCover ? (
          <img
            src={coverUrl}
            alt=""
            className="absolute inset-0 w-full h-full object-cover"
          />
        ) : (
          <div className="absolute inset-0 bg-gradient-to-br from-[#1fc7d4]/8 via-background to-[#7645d9]/8" />
        )}

        {/* Scrim */}
        {hasCover && (
          <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-black/20" />
        )}

        <div
          className={`relative z-10 max-w-4xl mx-auto px-4 sm:px-6 pt-8 pb-12 flex flex-col ${
            hasCover
              ? 'min-h-[340px] sm:min-h-[440px] lg:min-h-[500px]'
              : 'min-h-[240px] sm:min-h-[300px]'
          }`}
        >
          {/* Back (pinned top) */}
          <Link
            href="/news"
            className={`inline-flex items-center gap-2 text-sm mb-auto transition-colors group ${
              hasCover
                ? 'text-white/60 hover:text-white'
                : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            <ArrowLeft className="w-4 h-4 transition-transform group-hover:-translate-x-0.5" />
            Back to News
          </Link>

          {/* Title block (anchored bottom) */}
          <div>
            {article.tags.length > 0 && (
              <div className="flex flex-wrap gap-2 mb-5">
                {article.tags.map((tag) => (
                  <span
                    key={tag}
                    className="px-3 py-1 rounded-full text-[11px] font-bold tracking-[0.15em] uppercase bg-[#1fc7d4]/15 text-[#1fc7d4] border border-[#1fc7d4]/25 backdrop-blur-sm"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            )}

            <h1
              className={`text-3xl sm:text-4xl lg:text-[2.75rem] font-extrabold leading-[1.1] tracking-tight mb-5 ${
                hasCover ? 'text-white' : 'text-foreground'
              }`}
            >
              {article.title}
            </h1>

            <div
              className={`flex items-center gap-5 text-sm ${
                hasCover ? 'text-white/50' : 'text-muted-foreground'
              }`}
            >
              <span className="flex items-center gap-1.5">
                <Calendar className="w-3.5 h-3.5" />
                {fmtDate(article.published_at)}
              </span>
              <span className="flex items-center gap-1.5">
                <Clock className="w-3.5 h-3.5" />
                {mins} min read
              </span>
            </div>
          </div>
        </div>
      </section>

      {/* ── Accent bar ── */}
      <div className="h-[3px] bg-gradient-to-r from-[#1fc7d4] via-[#7645d9] to-[#1fc7d4]" />

      {/* ── Article body ── */}
      <article className="max-w-3xl mx-auto px-4 sm:px-6 pt-12 pb-20">
        <div
          className={[
            'prose prose-lg prose-neutral dark:prose-invert max-w-none',
            'prose-headings:font-bold prose-headings:tracking-tight',
            'prose-h2:text-2xl prose-h2:mt-12 prose-h2:mb-4 prose-h2:pb-3 prose-h2:border-b prose-h2:border-[#1fc7d4]/20',
            'prose-h3:text-lg prose-h3:mt-8',
            'prose-p:leading-[1.8] prose-p:text-muted-foreground',
            'prose-strong:text-foreground prose-strong:font-semibold',
            'prose-a:text-[#1fc7d4] prose-a:no-underline prose-a:font-medium hover:prose-a:underline',
            'prose-blockquote:border-l-[3px] prose-blockquote:border-[#7645d9] prose-blockquote:bg-[#7645d9]/5 prose-blockquote:rounded-r-lg prose-blockquote:py-1 prose-blockquote:italic',
            'prose-hr:border-border/20 prose-hr:my-10',
            'prose-li:text-muted-foreground prose-li:marker:text-[#1fc7d4]',
            'prose-code:text-[#1fc7d4] prose-code:bg-[#1fc7d4]/10 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-code:text-sm prose-code:font-normal prose-code:before:content-none prose-code:after:content-none',
          ].join(' ')}
        >
          <MarkdownAsync remarkPlugins={[remarkGfm]}>
            {article.content}
          </MarkdownAsync>
        </div>

        {/* ── Footer ── */}
        <div className="mt-16 pt-8 border-t border-border/20">
          <Link
            href="/news"
            className="inline-flex items-center gap-3 px-5 py-3 rounded-xl text-sm font-medium text-muted-foreground hover:text-foreground bg-card/50 hover:bg-card border border-border/20 hover:border-border/40 transition-all group"
          >
            <ArrowLeft className="w-4 h-4 transition-transform group-hover:-translate-x-1" />
            Back to all articles
          </Link>
        </div>
      </article>
    </div>
  );
}
