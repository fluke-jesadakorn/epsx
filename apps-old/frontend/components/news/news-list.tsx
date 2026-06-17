import type { NewsArticle, NewsListResponse } from '@/shared/api/news';
import { resolveNewsImageUrl } from '@/shared/api/news';
import { ArrowRight, ChevronLeft, ChevronRight, Newspaper } from 'lucide-react';
import Image from 'next/image';
import Link from 'next/link';

const BLUR_PLACEHOLDER = 'data:image/gif;base64,R0lGODlhAQABAIAAAP///wAAACH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==';

interface Props {
  data: NewsListResponse;
}

function formatDate(dateStr: string | null): string {
  if (dateStr === null) { return ''; }
  return new Date(dateStr).toLocaleDateString('en-US', { year: 'numeric', month: 'long', day: 'numeric' });
}

function FeaturedCard({ article }: { article: NewsArticle }) {
  const resolvedCover = resolveNewsImageUrl(article.cover_image_url);
  const hasCover = resolvedCover !== null;
  return (
    <Link href={`/news/${article.slug}`} className="group block">
      <div className="relative rounded-3xl overflow-hidden h-[360px] sm:h-[480px] bg-gradient-to-br from-[#7645d9]/20 via-[#1fc7d4]/10 to-slate-900/50">
        {hasCover && (
          <Image
            src={resolvedCover ?? ''}
            alt={article.title}
            fill
            className="object-cover group-hover:scale-105 transition-transform duration-700"
            sizes="100vw"
            placeholder="blur"
            blurDataURL={BLUR_PLACEHOLDER}
            priority
          />
        )}
        {!hasCover && (
          <div className="absolute top-8 right-8 opacity-10">
            <Newspaper className="w-24 h-24 text-[#1fc7d4]" />
          </div>
        )}
        <div className="absolute inset-0 bg-gradient-to-t from-black/85 via-black/30 to-transparent" />
        <div className="absolute bottom-0 left-0 right-0 p-6 sm:p-10">
          <div className="flex flex-wrap gap-2 mb-4">
            <span className="px-3 py-1 rounded-full text-xs font-semibold bg-[#1fc7d4]/20 text-[#1fc7d4] backdrop-blur-sm border border-[#1fc7d4]/30">
              Featured
            </span>
            {article.tags.slice(0, 2).map((tag) => (
              <span key={tag} className="px-3 py-1 rounded-full text-xs font-medium bg-white/10 text-white/80 backdrop-blur-sm">
                {tag}
              </span>
            ))}
          </div>
          <h2 className="text-2xl sm:text-3xl font-extrabold text-white mb-3 group-hover:text-[#1fc7d4] transition-colors line-clamp-2">
            {article.title}
          </h2>
          {article.summary !== null && (
            <p className="text-white/70 text-sm sm:text-base line-clamp-2 max-w-3xl">{article.summary}</p>
          )}
          <div className="mt-5 flex items-center gap-4">
            <span className="text-xs text-white/40">{formatDate(article.published_at)}</span>
            <span className="flex items-center gap-1.5 text-xs font-semibold text-[#1fc7d4] group-hover:gap-2.5 transition-all">
              Read article <ArrowRight className="w-3.5 h-3.5" />
            </span>
          </div>
        </div>
      </div>
    </Link>
  );
}

function ArticleCard({ article, priority }: { article: NewsArticle; priority?: boolean }) {
  const resolvedCover = resolveNewsImageUrl(article.cover_image_url);
  const hasCover = resolvedCover !== null;
  return (
    <Link href={`/news/${article.slug}`} className="group block h-full">
      <article className="rounded-2xl bg-card border border-border/20 overflow-hidden hover:border-[#1fc7d4]/40 transition-all duration-300 hover:shadow-2xl hover:shadow-[#7645d9]/5 hover:-translate-y-0.5 h-full flex flex-col">
        <div className="relative w-full h-48 overflow-hidden">
          {hasCover ? (
            <Image
              src={resolvedCover ?? ''}
              alt={article.title}
              fill
              className="object-cover group-hover:scale-105 transition-transform duration-500"
              sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw"
              placeholder="blur"
              blurDataURL={BLUR_PLACEHOLDER}
              priority={priority}
            />
          ) : (
            <div className="w-full h-full bg-gradient-to-br from-[#7645d9]/15 via-[#1fc7d4]/5 to-transparent flex items-center justify-center">
              <Newspaper className="w-10 h-10 text-muted-foreground/20" />
            </div>
          )}
        </div>
        <div className="p-5 flex flex-col flex-1">
          <div className="flex flex-wrap gap-1.5 mb-3">
            {article.tags.slice(0, 2).map((tag) => (
              <span key={tag} className="px-2 py-0.5 rounded-full text-xs font-medium bg-[#1fc7d4]/10 text-[#1fc7d4]">
                {tag}
              </span>
            ))}
          </div>
          <h2 className="font-bold text-foreground group-hover:text-[#1fc7d4] transition-colors line-clamp-2 mb-2 leading-snug">
            {article.title}
          </h2>
          {article.summary !== null && (
            <p className="text-sm text-muted-foreground line-clamp-3 flex-1 leading-relaxed">{article.summary}</p>
          )}
          <div className="mt-4 pt-4 border-t border-border/10 flex items-center justify-between">
            <span className="text-xs text-muted-foreground">{formatDate(article.published_at)}</span>
            <span className="text-xs text-[#1fc7d4] font-semibold opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-1">
              Read <ArrowRight className="w-3 h-3" />
            </span>
          </div>
        </div>
      </article>
    </Link>
  );
}

function EmptyState() {
  return (
    <div className="flex flex-col items-center justify-center py-24 gap-5">
      <div className="p-6 rounded-full bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent border border-border/20">
        <Newspaper className="w-10 h-10 text-muted-foreground/30" />
      </div>
      <div className="text-center">
        <p className="font-semibold text-foreground text-lg">No articles yet</p>
        <p className="text-sm text-muted-foreground mt-1.5 max-w-xs leading-relaxed">
          Check back soon for updates, announcements, and insights from the EPSX team.
        </p>
      </div>
    </div>
  );
}

function Pagination({ page, totalPages }: { page: number; totalPages: number }) {
  if (totalPages <= 1) { return null; }
  return (
    <div className="flex items-center justify-center gap-3 mt-12">
      <Link
        href={`/news?page=${page - 1}`}
        aria-disabled={page === 1}
        className={`flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium border border-border/20 bg-card hover:bg-muted/50 transition-colors ${page === 1 ? 'pointer-events-none opacity-30' : ''}`}
      >
        <ChevronLeft className="w-4 h-4" />
        Previous
      </Link>
      <span className="px-4 py-2 rounded-xl text-sm text-muted-foreground bg-muted/20 border border-border/10">
        {page} of {totalPages}
      </span>
      <Link
        href={`/news?page=${page + 1}`}
        aria-disabled={page === totalPages}
        className={`flex items-center gap-1.5 px-4 py-2 rounded-xl text-sm font-medium border border-border/20 bg-card hover:bg-muted/50 transition-colors ${page === totalPages ? 'pointer-events-none opacity-30' : ''}`}
      >
        Next
        <ChevronRight className="w-4 h-4" />
      </Link>
    </div>
  );
}

export function NewsList({ data }: Props) {
  const { page, total, limit } = data;
  const totalPages = Math.ceil(total / limit);
  const featured = data.articles.at(0);
  const rest = data.articles.slice(1);

  if (featured === undefined) { return <EmptyState />; }

  return (
    <div className="space-y-8">
      <FeaturedCard article={featured} />
      {rest.length > 0 && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
          {rest.map((article, idx) => (
            <ArticleCard key={article.id} article={article} priority={idx < 2} />
          ))}
        </div>
      )}
      <Pagination page={page} totalPages={totalPages} />
    </div>
  );
}
