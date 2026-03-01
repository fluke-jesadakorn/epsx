'use client';

import {
  deleteNewsAction,
  pinNewsAction,
  publishNewsAction,
  unpinNewsAction,
  unpublishNewsAction,
} from '@/app/news/actions';
import type { NewsArticle } from '@/shared/api/news';
import { resolveNewsImageUrl } from '@/shared/api/news';
import { Edit, Eye, EyeOff, FileText, Newspaper, Pin, PinOff, Plus, Trash2 } from 'lucide-react';
import Link from 'next/link';
import { memo, useCallback, useState } from 'react';
import { toast } from 'sonner';

type StatusFilter = 'all' | 'draft' | 'published';

interface Props {
  articles: NewsArticle[];
  total: number;
  page: number;
  status: StatusFilter;
}

const CREATE_BTN = 'flex items-center gap-2 px-4 py-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white text-sm font-semibold hover:opacity-90 transition-opacity';
const ACTION_BTN = 'p-1.5 rounded-lg hover:bg-muted/50 text-muted-foreground hover:text-foreground transition-colors';

function formatDate(dateStr: string | null): string {
  if (dateStr === null) { return '—'; }
  return new Date(dateStr).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
}

const StatusBadge = memo(function StatusBadge({ status }: { status: string }) {
  const cls = status === 'published'
    ? 'bg-green-500/10 text-green-400 border border-green-500/20'
    : 'bg-yellow-500/10 text-yellow-400 border border-yellow-500/20';
  return (
    <span className={`inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium ${cls}`}>
      {status}
    </span>
  );
});

interface RowActionsProps {
  article: NewsArticle;
  onDelete: () => void;
  onTogglePublish: () => void;
  onTogglePin: () => void;
}

const RowActions = memo(function RowActions({ article, onDelete, onTogglePublish, onTogglePin }: RowActionsProps) {
  return (
    <div className="flex items-center gap-1 shrink-0">
      <button
        title={article.is_pinned ? 'Unpin' : 'Pin to homepage'}
        onClick={onTogglePin}
        className={`${ACTION_BTN} ${article.is_pinned ? 'text-[#1fc7d4]' : ''}`}
      >
        {article.is_pinned ? <PinOff className="w-4 h-4" /> : <Pin className="w-4 h-4" />}
      </button>
      <button title={article.status === 'published' ? 'Unpublish' : 'Publish'} onClick={onTogglePublish} className={ACTION_BTN}>
        {article.status === 'published' ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
      </button>
      <Link href={`/news/${article.id}/edit`} title="Edit" className={ACTION_BTN}>
        <Edit className="w-4 h-4" />
      </Link>
      <button title="Delete" onClick={onDelete} className="p-1.5 rounded-lg hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition-colors">
        <Trash2 className="w-4 h-4" />
      </button>
    </div>
  );
});

const ArticleCard = memo(function ArticleCard({ article, onDelete, onTogglePublish, onTogglePin }: {
  article: NewsArticle;
  onDelete: (id: string, title: string) => void;
  onTogglePublish: (article: NewsArticle) => void;
  onTogglePin: (article: NewsArticle) => void;
}) {
  const resolvedCover = resolveNewsImageUrl(article.cover_image_url);
  const hasCover = resolvedCover !== null && resolvedCover !== '';
  return (
    <div className={`flex items-start gap-4 p-4 rounded-2xl bg-card border transition-colors ${article.is_pinned ? 'border-[#1fc7d4]/30 hover:border-[#1fc7d4]/50' : 'border-border/20 hover:border-border/40'}`}>
      <div className="shrink-0 w-20 h-14 rounded-xl overflow-hidden bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent border border-border/20 flex items-center justify-center">
        {hasCover ? (
          // eslint-disable-next-line @next/next/no-img-element
          <img src={resolvedCover} alt="" className="w-full h-full object-cover" />
        ) : (
          <FileText className="w-5 h-5 text-muted-foreground/40" />
        )}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-start justify-between gap-3">
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2">
              <p className="font-semibold text-foreground truncate">{article.title}</p>
              {article.is_pinned && <Pin className="w-3.5 h-3.5 text-[#1fc7d4] shrink-0" />}
            </div>
            <p className="text-xs text-muted-foreground/60 font-mono truncate">{article.slug}</p>
          </div>
          <div className="flex items-center gap-2 shrink-0">
            <StatusBadge status={article.status} />
            <span className="text-xs text-muted-foreground whitespace-nowrap hidden sm:block">{formatDate(article.published_at)}</span>
            <RowActions
              article={article}
              onDelete={() => onDelete(article.id, article.title)}
              onTogglePublish={() => onTogglePublish(article)}
              onTogglePin={() => onTogglePin(article)}
            />
          </div>
        </div>
        {article.summary !== null && article.summary !== '' && (
          <p className="text-sm text-muted-foreground mt-1 line-clamp-1">{article.summary}</p>
        )}
        {article.tags.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-2">
            {article.tags.slice(0, 4).map((tag) => (
              <span key={tag} className="px-2 py-0.5 rounded-full text-xs bg-[#7645d9]/10 text-[#7645d9]/70 font-medium">{tag}</span>
            ))}
            {article.tags.length > 4 && (
              <span className="text-xs text-muted-foreground/50">+{article.tags.length - 4}</span>
            )}
          </div>
        )}
      </div>
    </div>
  );
});

interface DeleteModalProps {
  target: { id: string; title: string } | null;
  onCancel: () => void;
  onConfirm: (id: string) => void;
}

const DeleteModal = memo(function DeleteModal({ target, onCancel, onConfirm }: DeleteModalProps) {
  if (target === null) { return null; }
  return (
    <div className="fixed inset-0 bg-black/60 z-50 flex items-center justify-center p-4">
      <div className="rounded-2xl bg-card border border-border/20 shadow-2xl p-6 max-w-sm w-full space-y-4">
        <h3 className="font-bold text-foreground">Delete Article?</h3>
        <p className="text-sm text-muted-foreground">&ldquo;{target.title}&rdquo; will be permanently deleted.</p>
        <div className="flex gap-3 justify-end">
          <button onClick={onCancel} className="px-4 py-2 rounded-xl text-sm border border-border/20 hover:bg-muted/30">Cancel</button>
          <button onClick={() => onConfirm(target.id)} className="px-4 py-2 rounded-xl text-sm bg-red-500/10 text-red-400 border border-red-500/20 hover:bg-red-500/20">Delete</button>
        </div>
      </div>
    </div>
  );
});

const Pagination = memo(function Pagination({ page, totalPages, status }: { page: number; totalPages: number; status: string }) {
  if (totalPages <= 1) { return null; }
  const base = `/news?status=${status}`;
  return (
    <div className="flex items-center justify-center gap-2">
      <Link href={`${base}&page=${page - 1}`} aria-disabled={page === 1} className={`px-3 py-1.5 rounded-lg text-sm border border-border/20 transition-colors ${page === 1 ? 'pointer-events-none opacity-40' : 'hover:bg-muted/50'}`}>
        Previous
      </Link>
      <span className="text-sm text-muted-foreground">{page} / {totalPages}</span>
      <Link href={`${base}&page=${page + 1}`} aria-disabled={page === totalPages} className={`px-3 py-1.5 rounded-lg text-sm border border-border/20 transition-colors ${page === totalPages ? 'pointer-events-none opacity-40' : 'hover:bg-muted/50'}`}>
        Next
      </Link>
    </div>
  );
});

function EmptyState() {
  return (
    <div className="rounded-2xl bg-card border border-border/20 shadow-xl flex flex-col items-center justify-center py-20 gap-4">
      <div className="p-5 rounded-full bg-gradient-to-br from-[#7645d9]/10 via-[#1fc7d4]/5 to-transparent border border-border/20">
        <Newspaper className="w-8 h-8 text-muted-foreground/40" />
      </div>
      <div className="text-center">
        <p className="font-semibold text-foreground">No articles yet</p>
        <p className="text-sm text-muted-foreground mt-1">Create your first article to get started.</p>
      </div>
      <Link href="/news/create" className={CREATE_BTN}>
        <Plus className="w-4 h-4" />
        Create Article
      </Link>
    </div>
  );
}

export function NewsManagement({ articles: initialArticles, total, page, status }: Props) {
  const [localArticles, setLocalArticles] = useState<NewsArticle[]>(initialArticles);
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; title: string } | null>(null);

  const limit = 20;
  const totalPages = Math.ceil(total / limit);

  const handleDelete = useCallback(async (id: string) => {
    const res = await deleteNewsAction(id);
    if (res.success) {
      toast.success('Article deleted');
      setLocalArticles((prev) => prev.filter((a) => a.id !== id));
    } else { toast.error('Failed to delete article'); }
    setDeleteTarget(null);
  }, []);

  const handleTogglePublish = useCallback(async (article: NewsArticle) => {
    const action = article.status === 'published' ? unpublishNewsAction : publishNewsAction;
    const res = await action(article.id);
    if (res.success) {
      const next = article.status === 'published' ? 'draft' : 'published';
      toast.success(next === 'published' ? 'Article published' : 'Article unpublished');
      setLocalArticles((prev) => prev.map((a) => a.id === article.id ? { ...a, status: next } : a));
    } else { toast.error('Failed to update status'); }
  }, []);

  const handleTogglePin = useCallback(async (article: NewsArticle) => {
    const action = article.is_pinned ? unpinNewsAction : pinNewsAction;
    const res = await action(article.id);
    if (res.success) {
      toast.success(article.is_pinned ? 'Article unpinned' : 'Article pinned to homepage');
      setLocalArticles((prev) => prev.map((a) => a.id === article.id ? { ...a, is_pinned: !article.is_pinned, pinned_at: (res.data as NewsArticle | null)?.pinned_at ?? null } : a));
    } else { toast.error('Failed to update pin status'); }
  }, []);

  const handleDeleteClick = useCallback((id: string, title: string) => setDeleteTarget({ id, title }), []);
  const handleDeleteCancel = useCallback(() => setDeleteTarget(null), []);
  const handleConfirmDelete = useCallback((id: string) => void handleDelete(id), [handleDelete]);
  const handleTogglePublishRow = useCallback((a: NewsArticle) => void handleTogglePublish(a), [handleTogglePublish]);
  const handleTogglePinRow = useCallback((a: NewsArticle) => void handleTogglePin(a), [handleTogglePin]);

  return (
    <div className="p-4 sm:p-6 lg:p-8 space-y-6">
      <DeleteModal target={deleteTarget} onCancel={handleDeleteCancel} onConfirm={handleConfirmDelete} />

      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="h-[3px] w-8 bg-[#1fc7d4] rounded-full" />
          <Newspaper className="w-5 h-5 text-[#1fc7d4]" />
          <h1 className="text-xl font-bold text-foreground">News Management</h1>
        </div>
        <Link href="/news/create" className={CREATE_BTN}>
          <Plus className="w-4 h-4" />
          Create Article
        </Link>
      </div>

      <div className="flex items-center gap-2 flex-wrap">
        {(['all', 'draft', 'published'] as StatusFilter[]).map((s) => (
          <Link
            key={s}
            href={`/news?status=${s}&page=1`}
            className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors capitalize ${status === s ? 'bg-[#7645d9] text-white shadow-lg shadow-[#7645d9]/20' : 'bg-card border border-border/20 text-muted-foreground hover:text-foreground hover:border-border/40'}`}
          >
            {s}
          </Link>
        ))}
        <span className="ml-auto text-sm text-muted-foreground">{total} {total === 1 ? 'article' : 'articles'}</span>
      </div>

      {localArticles.length === 0 ? (
        <EmptyState />
      ) : (
        <div className="space-y-3">
          {localArticles.map((article) => (
            <ArticleCard
              key={article.id}
              article={article}
              onDelete={handleDeleteClick}
              onTogglePublish={handleTogglePublishRow}
              onTogglePin={handleTogglePinRow}
            />
          ))}
        </div>
      )}

      <Pagination page={page} totalPages={totalPages} status={status} />
    </div>
  );
}
