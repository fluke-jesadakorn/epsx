'use client';

import { memo, useCallback, useState } from 'react';
import Link from 'next/link';
import { toast } from 'sonner';
import { Newspaper, Plus, Edit, Trash2, Eye, EyeOff } from 'lucide-react';
import type { NewsArticle } from '@/shared/api/news';
import {
  deleteNewsAction,
  publishNewsAction,
  unpublishNewsAction,
} from '@/app/news/actions';

type StatusFilter = 'all' | 'draft' | 'published';

interface Props {
  articles: NewsArticle[];
  total: number;
  page: number;
  status: StatusFilter;
}

function formatDate(dateStr: string | null): string {
  if (dateStr === null) { return '—'; }
  return new Date(dateStr).toLocaleDateString('en-US', {
    year: 'numeric', month: 'short', day: 'numeric',
  });
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
}

const RowActions = memo(function RowActions({ article, onDelete, onTogglePublish }: RowActionsProps) {
  return (
    <div className="flex items-center gap-2 justify-end">
      <button
        title={article.status === 'published' ? 'Unpublish' : 'Publish'}
        onClick={onTogglePublish}
        className="p-1.5 rounded-lg hover:bg-muted/50 text-muted-foreground hover:text-foreground transition-colors"
      >
        {article.status === 'published' ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
      </button>
      <Link
        href={`/news/${article.id}/edit`}
        title="Edit"
        className="p-1.5 rounded-lg hover:bg-muted/50 text-muted-foreground hover:text-foreground transition-colors"
      >
        <Edit className="w-4 h-4" />
      </Link>
      <button
        title="Delete"
        onClick={onDelete}
        className="p-1.5 rounded-lg hover:bg-red-500/10 text-muted-foreground hover:text-red-400 transition-colors"
      >
        <Trash2 className="w-4 h-4" />
      </button>
    </div>
  );
});

const ArticleRow = memo(function ArticleRow({ article, onDelete, onTogglePublish }: {
  article: NewsArticle;
  onDelete: (id: string, title: string) => void;
  onTogglePublish: (article: NewsArticle) => void;
}) {
  return (
    <tr className="border-b border-border/10 hover:bg-muted/30 transition-colors">
      <td className="px-4 py-3">
        <div className="font-medium text-foreground line-clamp-1">{article.title}</div>
        <div className="text-xs text-muted-foreground">{article.slug}</div>
      </td>
      <td className="px-4 py-3"><StatusBadge status={article.status} /></td>
      <td className="px-4 py-3 text-muted-foreground">{formatDate(article.published_at)}</td>
      <td className="px-4 py-3">
        <div className="flex flex-wrap gap-1">
          {article.tags.slice(0, 3).map((tag) => (
            <span key={tag} className="px-1.5 py-0.5 rounded text-xs bg-muted text-muted-foreground">{tag}</span>
          ))}
        </div>
      </td>
      <td className="px-4 py-3">
        <RowActions
          article={article}
          onDelete={() => onDelete(article.id, article.title)}
          onTogglePublish={() => onTogglePublish(article)}
        />
      </td>
    </tr>
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
        <p className="text-sm text-muted-foreground">
          &ldquo;{target.title}&rdquo; will be permanently deleted.
        </p>
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
      <Link
        href={`${base}&page=${page - 1}`}
        aria-disabled={page === 1}
        className={`px-3 py-1.5 rounded-lg text-sm border border-border/20 transition-colors ${page === 1 ? 'pointer-events-none opacity-40' : 'hover:bg-muted/50'}`}
      >
        Previous
      </Link>
      <span className="text-sm text-muted-foreground">{page} / {totalPages}</span>
      <Link
        href={`${base}&page=${page + 1}`}
        aria-disabled={page === totalPages}
        className={`px-3 py-1.5 rounded-lg text-sm border border-border/20 transition-colors ${page === totalPages ? 'pointer-events-none opacity-40' : 'hover:bg-muted/50'}`}
      >
        Next
      </Link>
    </div>
  );
});

export function NewsManagement({ articles, total, page, status }: Props) {
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; title: string } | null>(null);

  const limit = 20;
  const totalPages = Math.ceil(total / limit);

  const handleDelete = useCallback(async (id: string) => {
    const res = await deleteNewsAction(id);
    if (res.success) { toast.success('Article deleted'); }
    else { toast.error('Failed to delete article'); }
    setDeleteTarget(null);
  }, []);

  const handleTogglePublish = useCallback(async (article: NewsArticle) => {
    const action = article.status === 'published' ? unpublishNewsAction : publishNewsAction;
    const res = await action(article.id);
    if (res.success) {
      toast.success(article.status === 'published' ? 'Article unpublished' : 'Article published');
    } else {
      toast.error('Failed to update status');
    }
  }, []);

  const handleDeleteClick = useCallback((id: string, title: string) => setDeleteTarget({ id, title }), []);
  const handleDeleteCancel = useCallback(() => setDeleteTarget(null), []);
  const handleConfirmDelete = useCallback((id: string) => void handleDelete(id), [handleDelete]);
  const handleTogglePublishRow = useCallback((a: NewsArticle) => void handleTogglePublish(a), [handleTogglePublish]);

  return (
    <div className="space-y-6">
      <DeleteModal target={deleteTarget} onCancel={handleDeleteCancel} onConfirm={handleConfirmDelete} />

      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="h-[3px] w-8 bg-[#1fc7d4] rounded-full" />
          <Newspaper className="w-5 h-5 text-[#1fc7d4]" />
          <h1 className="text-xl font-bold text-foreground">News Management</h1>
        </div>
        <Link
          href="/news/create"
          className="flex items-center gap-2 px-4 py-2 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] text-white text-sm font-semibold hover:opacity-90 transition-opacity"
        >
          <Plus className="w-4 h-4" />
          Create Article
        </Link>
      </div>

      {/* Filters */}
      <div className="flex gap-2">
        {(['all', 'draft', 'published'] as StatusFilter[]).map((s) => (
          <Link
            key={s}
            href={`/news?status=${s}&page=1`}
            className={`px-3 py-1.5 rounded-lg text-sm font-medium transition-colors capitalize ${status === s ? 'bg-[#7645d9] text-white' : 'bg-card border border-border/20 text-muted-foreground hover:text-foreground'}`}
          >
            {s}
          </Link>
        ))}
        <span className="ml-auto text-sm text-muted-foreground self-center">{total} articles</span>
      </div>

      {/* Table */}
      <div className="rounded-2xl bg-card border border-border/20 shadow-xl overflow-hidden">
        {articles.length === 0 ? (
          <div className="p-12 text-center text-muted-foreground">No articles found.</div>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border/20 text-muted-foreground text-left">
                <th className="px-4 py-3 font-medium">Title</th>
                <th className="px-4 py-3 font-medium">Status</th>
                <th className="px-4 py-3 font-medium">Published</th>
                <th className="px-4 py-3 font-medium">Tags</th>
                <th className="px-4 py-3 font-medium text-right">Actions</th>
              </tr>
            </thead>
            <tbody>
              {articles.map((article) => (
                <ArticleRow
                  key={article.id}
                  article={article}
                  onDelete={handleDeleteClick}
                  onTogglePublish={handleTogglePublishRow}
                />
              ))}
            </tbody>
          </table>
        )}
      </div>

      <Pagination page={page} totalPages={totalPages} status={status} />
    </div>
  );
}
