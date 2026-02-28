import Image from 'next/image';
import Link from 'next/link';
import type { NewsListResponse, NewsArticle } from '@/shared/api/news';

const BLUR_PLACEHOLDER = 'data:image/gif;base64,R0lGODlhAQABAIAAAP///wAAACH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==';

interface Props {
  data: NewsListResponse;
}

function formatDate(dateStr: string | null): string {
  if (dateStr === null) { return ''; }
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}

function ArticleCard({ article, priority }: { article: NewsArticle; priority?: boolean }) {
  return (
    <Link href={`/news/${article.slug}`} className="group block">
      <div className="rounded-2xl bg-card border border-border/20 shadow-xl overflow-hidden hover:border-[#1fc7d4]/30 transition-colors h-full flex flex-col">
        {article.cover_image_url !== null && (
          <div className="relative w-full h-48">
            <Image
              src={article.cover_image_url}
              alt={article.title}
              fill
              className="object-cover"
              sizes="(max-width: 640px) 100vw, (max-width: 1024px) 50vw, 33vw"
              placeholder="blur"
              blurDataURL={BLUR_PLACEHOLDER}
              priority={priority}
            />
          </div>
        )}
        <div className="p-5 flex flex-col flex-1">
          <div className="flex flex-wrap gap-1.5 mb-3">
            {article.tags.slice(0, 3).map((tag) => (
              <span
                key={tag}
                className="px-2 py-0.5 rounded-full text-xs font-medium bg-[#1fc7d4]/10 text-[#1fc7d4]"
              >
                {tag}
              </span>
            ))}
          </div>
          <h2 className="font-bold text-foreground group-hover:text-[#1fc7d4] transition-colors line-clamp-2 mb-2">
            {article.title}
          </h2>
          {article.summary !== null && (
            <p className="text-sm text-muted-foreground line-clamp-3 flex-1">
              {article.summary}
            </p>
          )}
          <div className="mt-4 text-xs text-muted-foreground">
            {formatDate(article.published_at)}
          </div>
        </div>
      </div>
    </Link>
  );
}

export function NewsList({ data }: Props) {
  const articles = data.articles ?? [];
  const { page, total, limit } = data;
  const totalPages = Math.ceil(total / limit);

  if (articles.length === 0) {
    return (
      <div className="text-center py-20 text-muted-foreground">
        No articles published yet.
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
        {articles.map((article, idx) => (
          <ArticleCard key={article.id} article={article} priority={idx === 0} />
        ))}
      </div>

      {totalPages > 1 && (
        <div className="flex items-center justify-center gap-3">
          <Link
            href={`/news?page=${page - 1}`}
            aria-disabled={page === 1}
            className={`px-4 py-2 rounded-xl text-sm border border-border/20 hover:bg-muted/50 transition-colors ${page === 1 ? 'pointer-events-none opacity-40' : ''}`}
          >
            Previous
          </Link>
          <span className="text-sm text-muted-foreground">{page} / {totalPages}</span>
          <Link
            href={`/news?page=${page + 1}`}
            aria-disabled={page === totalPages}
            className={`px-4 py-2 rounded-xl text-sm border border-border/20 hover:bg-muted/50 transition-colors ${page === totalPages ? 'pointer-events-none opacity-40' : ''}`}
          >
            Next
          </Link>
        </div>
      )}
    </div>
  );
}
