'use client';

import { useCallback, useState } from 'react';
import Link from 'next/link';
import type { NewsArticle, NewsListResponse } from '@/shared/api/news';
import { getPublicNews } from '@/app/actions/news';

interface Props {
  initial: NewsListResponse;
}

function formatDate(dateStr: string | null): string {
  if (dateStr === null) { return ''; }
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}

function ArticleCard({ article }: { article: NewsArticle }) {
  return (
    <Link href={`/news/${article.slug}`} className="group block">
      <div className="rounded-2xl bg-card border border-border/20 shadow-xl overflow-hidden hover:border-[#1fc7d4]/30 transition-colors h-full flex flex-col">
        {article.cover_image_url !== null && (
          <img
            src={article.cover_image_url}
            alt={article.title}
            className="w-full h-48 object-cover"
          />
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

export function NewsList({ initial }: Props) {
  const [articles, setArticles] = useState(initial.articles);
  const [page, setPage] = useState(initial.page);
  const [total] = useState(initial.total);
  const [loading, setLoading] = useState(false);

  const limit = initial.limit;
  const totalPages = Math.ceil(total / limit);

  const loadPage = useCallback(async (p: number) => {
    setLoading(true);
    const res = await getPublicNews(p, limit);
    if (res.success === true && res.data !== undefined) {
      setArticles(res.data.articles);
      setPage(p);
    }
    setLoading(false);
  }, [limit]);

  if (articles.length === 0) {
    return (
      <div className="text-center py-20 text-muted-foreground">
        No articles published yet.
      </div>
    );
  }

  return (
    <div className="space-y-8">
      <div className={`grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 ${loading ? 'opacity-60 pointer-events-none' : ''}`}>
        {articles.map((article) => (
          <ArticleCard key={article.id} article={article} />
        ))}
      </div>

      {totalPages > 1 && (
        <div className="flex items-center justify-center gap-3">
          <button
            onClick={() => void loadPage(page - 1)}
            disabled={page === 1 || loading}
            className="px-4 py-2 rounded-xl text-sm border border-border/20 disabled:opacity-40 hover:bg-muted/50 transition-colors"
          >
            Previous
          </button>
          <span className="text-sm text-muted-foreground">{page} / {totalPages}</span>
          <button
            onClick={() => void loadPage(page + 1)}
            disabled={page === totalPages || loading}
            className="px-4 py-2 rounded-xl text-sm border border-border/20 disabled:opacity-40 hover:bg-muted/50 transition-colors"
          >
            Next
          </button>
        </div>
      )}
    </div>
  );
}
