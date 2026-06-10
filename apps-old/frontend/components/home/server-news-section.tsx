import { getHomepageNews } from '@/app/actions/news';
import type { NewsArticle } from '@/shared/api/news';
import { resolveNewsImageUrl } from '@/shared/api/news';
import { ArrowRight, Newspaper, Pin } from 'lucide-react';
import Image from 'next/image';
import Link from 'next/link';

const BLUR = 'data:image/gif;base64,R0lGODlhAQABAIAAAP///wAAACH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==';

function formatDate(d: string | null): string {
  if (d === null) { return ''; }
  return new Date(d).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' });
}

function FeaturedCard({ article }: { article: NewsArticle }) {
  const resolvedCover = resolveNewsImageUrl(article.cover_image_url);
  const hasCover = resolvedCover !== null && resolvedCover !== '';
  return (
    <Link href={`/news/${article.slug}`} className="group block">
      <div className="relative rounded-3xl overflow-hidden h-[320px] sm:h-[400px] bg-gradient-to-br from-[#7645d9]/20 via-[#1fc7d4]/10 to-slate-900/60 border border-white/10 dark:border-white/5">
        {hasCover && (
          <Image
            src={resolvedCover ?? ''}
            alt={article.title}
            fill
            className="object-cover group-hover:scale-105 transition-transform duration-700"
            sizes="(max-width: 768px) 100vw, 66vw"
            placeholder="blur"
            blurDataURL={BLUR}
          />
        )}
        <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent" />
        <div className="absolute bottom-0 left-0 right-0 p-6 sm:p-8">
          {article.is_pinned && (
            <div className="flex items-center gap-1.5 mb-3">
              <Pin className="w-3.5 h-3.5 text-[#1fc7d4]" />
              <span className="text-xs font-medium text-[#1fc7d4] uppercase tracking-wider">Featured</span>
            </div>
          )}
          {article.tags.length > 0 && (
            <div className="flex flex-wrap gap-2 mb-3">
              {article.tags.slice(0, 2).map((tag) => (
                <span key={tag} className="px-2 py-0.5 rounded-full text-xs bg-white/10 text-white/80 backdrop-blur-sm">{tag}</span>
              ))}
            </div>
          )}
          <h3 className="text-xl sm:text-2xl font-bold text-white group-hover:text-[#1fc7d4] transition-colors line-clamp-2 mb-2">
            {article.title}
          </h3>
          {article.summary !== null && article.summary !== '' && (
            <p className="text-white/70 text-sm line-clamp-2 mb-3">{article.summary}</p>
          )}
          <span className="text-xs text-white/50">{formatDate(article.published_at)}</span>
        </div>
      </div>
    </Link>
  );
}

function SmallCard({ article }: { article: NewsArticle }) {
  const resolvedCover = resolveNewsImageUrl(article.cover_image_url);
  const hasCover = resolvedCover !== null && resolvedCover !== '';
  return (
    <Link href={`/news/${article.slug}`} className="group block">
      <div className="relative rounded-2xl overflow-hidden h-[180px] bg-gradient-to-br from-[#7645d9]/20 via-[#1fc7d4]/10 to-slate-900/60 border border-white/10 dark:border-white/5">
        {hasCover && (
          <Image
            src={resolvedCover ?? ''}
            alt={article.title}
            fill
            className="object-cover group-hover:scale-105 transition-transform duration-700"
            sizes="(max-width: 768px) 100vw, 33vw"
            placeholder="blur"
            blurDataURL={BLUR}
          />
        )}
        <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/30 to-transparent" />
        <div className="absolute bottom-0 left-0 right-0 p-4">
          {article.is_pinned && <Pin className="w-3 h-3 text-[#1fc7d4] mb-1.5" />}
          <h3 className="text-sm font-semibold text-white group-hover:text-[#1fc7d4] transition-colors line-clamp-2">
            {article.title}
          </h3>
          <span className="text-xs text-white/50 mt-1 block">{formatDate(article.published_at)}</span>
        </div>
      </div>
    </Link>
  );
}

export default async function ServerNewsSection() {
  const res = await getHomepageNews();
  const articles: NewsArticle[] = res.success && Array.isArray(res.data) ? res.data : [];
  if (articles.length === 0) { return null; }

  const [featured, ...rest] = articles;

  return (
    <div className="container mx-auto px-4 py-16 sm:py-24 lg:py-32">
      <div className="mb-6 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Newspaper className="w-5 h-5 text-[#1fc7d4]" />
          <h2 className="text-xl font-bold text-foreground">Latest News</h2>
        </div>
        <Link href="/news" className="flex items-center gap-1 text-sm text-[#1fc7d4] hover:text-[#1fc7d4]/80 transition-colors font-medium">
          View all <ArrowRight className="w-4 h-4" />
        </Link>
      </div>

      <div className="space-y-4">
        <FeaturedCard article={featured} />
        {rest.length > 0 && (
          <div className={`grid gap-4 ${rest.length === 1 ? 'grid-cols-1' : 'grid-cols-2'}`}>
            {rest.map((a) => <SmallCard key={a.id} article={a} />)}
          </div>
        )}
      </div>
    </div>
  );
}
