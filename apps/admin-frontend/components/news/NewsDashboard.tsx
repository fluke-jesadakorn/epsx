'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { NewsTable } from './NewsTable';
import { NewsStats } from './NewsStats';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';

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

export function NewsDashboard() {
  const router = useRouter();
  const [articles, setArticles] = useState<Article[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [categoryFilter, setCategoryFilter] = useState<string>('all');

  // Mock data for now - will be replaced with actual TinaCMS queries
  useEffect(() => {
    const loadArticles = async () => {
      try {
        setLoading(true);
        // TODO: Replace with actual TinaCMS query
        const mockArticles: Article[] = [
          {
            id: '1',
            title: 'Welcome to EPSX News - Your Source for Market Intelligence',
            excerpt: 'Introducing EPSX News, your comprehensive source for market analysis...',
            status: 'published',
            author: 'admin-user',
            category: 'platform-updates',
            publishedAt: '2025-01-25T10:00:00.000Z',
            updatedAt: '2025-01-25T10:00:00.000Z',
            _sys: {
              filename: '2025/01/welcome-to-epsx-news.mdx'
            }
          }
        ];
        
        setArticles(mockArticles);
      } catch (error) {
        console.error('Error loading articles:', error);
      } finally {
        setLoading(false);
      }
    };

    loadArticles();
  }, []);

  const filteredArticles = articles.filter(article => {
    const matchesSearch = article.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         article.excerpt?.toLowerCase().includes(searchTerm.toLowerCase());
    const matchesStatus = statusFilter === 'all' || article.status === statusFilter;
    const matchesCategory = categoryFilter === 'all' || article.category === categoryFilter;
    
    return matchesSearch && matchesStatus && matchesCategory;
  });

  const stats = {
    total: articles.length,
    published: articles.filter(a => a.status === 'published').length,
    draft: articles.filter(a => a.status === 'draft').length,
    archived: articles.filter(a => a.status === 'archived').length,
  };

  const handleCreateNew = () => {
    router.push('/news/create');
  };

  const handleEdit = (articleId: string) => {
    // URL encode the filename for the edit route
    const encodedId = encodeURIComponent(articleId);
    router.push(`/news/edit/${encodedId}`);
  };

  const handleDelete = async (articleId: string) => {
    if (confirm('Are you sure you want to delete this article?')) {
      try {
        // TODO: Implement delete functionality
        console.log('Deleting article:', articleId);
        // Remove from local state for now
        setArticles(prev => prev.filter(a => a.id !== articleId));
      } catch (error) {
        console.error('Error deleting article:', error);
      }
    }
  };

  const handleStatusChange = async (articleId: string, newStatus: string) => {
    try {
      // TODO: Implement status change
      console.log('Changing status:', articleId, newStatus);
      setArticles(prev => prev.map(a => 
        a.id === articleId ? { ...a, status: newStatus as any } : a
      ));
    } catch (error) {
      console.error('Error updating status:', error);
    }
  };

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          {[...Array(4)].map((_, i) => (
            <div key={i} className="bg-white p-6 rounded-lg border border-gray-200 animate-pulse">
              <div className="h-4 bg-gray-200 rounded w-1/2 mb-2"></div>
              <div className="h-8 bg-gray-200 rounded w-1/3"></div>
            </div>
          ))}
        </div>
        <div className="bg-white p-6 rounded-lg border border-gray-200 animate-pulse">
          <div className="h-6 bg-gray-200 rounded w-1/4 mb-4"></div>
          <div className="space-y-3">
            {[...Array(5)].map((_, i) => (
              <div key={i} className="h-4 bg-gray-200 rounded"></div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Stats Cards */}
      <NewsStats stats={stats} />

      {/* Actions and Filters */}
      <div className="bg-white p-6 rounded-lg border border-gray-200">
        <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
          <div className="flex flex-col sm:flex-row gap-4">
            <Input
              type="text"
              placeholder="Search articles..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="sm:w-64"
            />
            
            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger className="sm:w-40">
                <SelectValue placeholder="Status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Status</SelectItem>
                <SelectItem value="published">Published</SelectItem>
                <SelectItem value="draft">Draft</SelectItem>
                <SelectItem value="archived">Archived</SelectItem>
              </SelectContent>
            </Select>

            <Select value={categoryFilter} onValueChange={setCategoryFilter}>
              <SelectTrigger className="sm:w-48">
                <SelectValue placeholder="Category" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Categories</SelectItem>
                <SelectItem value="market-analysis">Market Analysis</SelectItem>
                <SelectItem value="earnings-reports">Earnings Reports</SelectItem>
                <SelectItem value="eps-insights">EPS Insights</SelectItem>
                <SelectItem value="platform-updates">Platform Updates</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <Button onClick={handleCreateNew} className="bg-blue-600 hover:bg-blue-700">
            <svg className="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
            Create Article
          </Button>
        </div>
      </div>

      {/* Articles Table */}
      <NewsTable
        articles={filteredArticles}
        onEdit={handleEdit}
        onDelete={handleDelete}
        onStatusChange={handleStatusChange}
      />
    </div>
  );
}