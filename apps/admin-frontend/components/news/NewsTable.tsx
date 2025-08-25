import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ConfirmDialog } from '@/components/ui/ConfirmDialog';

interface Article {
  id: string;
  title: string;
  excerpt?: string;
  status: 'draft' | 'published' | 'archived';
  author?: string;
  category?: string;
  publishedAt: string;
  updatedAt?: string;
  _sys: {
    filename: string;
  };
}

interface NewsTableProps {
  articles: Article[];
  onEdit: (articleId: string) => void;
  onDelete: (articleId: string) => void;
  onStatusChange: (articleId: string, newStatus: string) => void;
}

export function NewsTable({ articles, onEdit, onDelete, onStatusChange }: NewsTableProps) {
  const [deleteDialog, setDeleteDialog] = useState<{
    open: boolean;
    articleId: string;
    articleTitle: string;
  }>({ open: false, articleId: '', articleTitle: '' });

  const getStatusBadge = (status: string) => {
    const variants = {
      published: 'bg-green-100 text-green-800',
      draft: 'bg-yellow-100 text-yellow-800',
      archived: 'bg-gray-100 text-gray-800',
    };

    return (
      <Badge className={variants[status as keyof typeof variants]}>
        {status.charAt(0).toUpperCase() + status.slice(1)}
      </Badge>
    );
  };

  const getCategoryName = (category?: string) => {
    if (!category) return 'Uncategorized';
    return category.split('-').map(word => 
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join(' ');
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  const handleDeleteClick = (article: Article) => {
    setDeleteDialog({
      open: true,
      articleId: article.id,
      articleTitle: article.title,
    });
  };

  const handleDeleteConfirm = () => {
    onDelete(deleteDialog.articleId);
    setDeleteDialog({ open: false, articleId: '', articleTitle: '' });
  };

  if (articles.length === 0) {
    return (
      <div className="bg-white rounded-lg border border-gray-200">
        <div className="p-12 text-center">
          <div className="w-24 h-24 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg className="w-12 h-12 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z" />
            </svg>
          </div>
          <h3 className="text-lg font-medium text-gray-900 mb-2">No articles found</h3>
          <p className="text-gray-600 mb-6">
            Get started by creating your first news article.
          </p>
          <Button onClick={() => window.location.href = '/news/create'}>
            Create First Article
          </Button>
        </div>
      </div>
    );
  }

  return (
    <>
      <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Article
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Category
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Author
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Date
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {articles.map((article) => (
                <tr key={article.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4">
                    <div>
                      <div className="text-sm font-medium text-gray-900 line-clamp-2">
                        {article.title}
                      </div>
                      {article.excerpt && (
                        <div className="text-sm text-gray-500 mt-1 line-clamp-2">
                          {article.excerpt}
                        </div>
                      )}
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    {getStatusBadge(article.status)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                    {getCategoryName(article.category)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {article.author || 'Unknown'}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {formatDate(article.publishedAt)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                    <div className="flex items-center justify-end space-x-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => onEdit(article._sys.filename)}
                        className="text-blue-600 hover:text-blue-800"
                      >
                        Edit
                      </Button>
                      
                      {article.status !== 'published' && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => onStatusChange(article.id, 'published')}
                          className="text-green-600 hover:text-green-800"
                        >
                          Publish
                        </Button>
                      )}
                      
                      {article.status === 'published' && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => onStatusChange(article.id, 'draft')}
                          className="text-yellow-600 hover:text-yellow-800"
                        >
                          Unpublish
                        </Button>
                      )}
                      
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => handleDeleteClick(article)}
                        className="text-red-600 hover:text-red-800"
                      >
                        Delete
                      </Button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      <ConfirmDialog
        open={deleteDialog.open}
        onOpenChange={(open) => setDeleteDialog(prev => ({ ...prev, open }))}
        onConfirm={handleDeleteConfirm}
        title="Delete Article"
        description={`Are you sure you want to delete "${deleteDialog.articleTitle}"? This action cannot be undone.`}
        confirmText="Delete"
        cancelText="Cancel"
      />
    </>
  );
}