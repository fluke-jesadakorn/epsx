'use client';

import { useState, useEffect } from 'react';
import { useRouter } from 'next/navigation';
import { TinaCMS, useTina } from 'tinacms';
import { TinaMarkdown } from 'tinacms/dist/rich-text';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

interface NewsEditorProps {
  mode: 'create' | 'edit';
  articleId?: string;
}

interface ArticleForm {
  title: string;
  excerpt: string;
  author: string;
  category: string;
  tags: string[];
  featuredImage: string;
  imageAlt: string;
  status: 'draft' | 'published' | 'archived';
  featured: boolean;
  publishedAt: string;
  seo: {
    title: string;
    description: string;
    keywords: string[];
  };
  body: any;
}

const defaultForm: ArticleForm = {
  title: '',
  excerpt: '',
  author: 'admin-user',
  category: 'market-analysis',
  tags: [],
  featuredImage: '',
  imageAlt: '',
  status: 'draft',
  featured: false,
  publishedAt: new Date().toISOString(),
  seo: {
    title: '',
    description: '',
    keywords: [],
  },
  body: {
    type: 'root',
    children: [
      {
        type: 'p',
        children: [{ type: 'text', text: 'Start writing your article here...' }],
      },
    ],
  },
};

export function NewsEditor({ mode, articleId }: NewsEditorProps) {
  const router = useRouter();
  const [formData, setFormData] = useState<ArticleForm>(defaultForm);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [activeTab, setActiveTab] = useState<'content' | 'settings' | 'seo'>('content');
  const [newTag, setNewTag] = useState('');

  // Load article data in edit mode
  useEffect(() => {
    if (mode === 'edit' && articleId) {
      loadArticle();
    }
  }, [mode, articleId]);

  const loadArticle = async () => {
    try {
      setLoading(true);
      // TODO: Replace with actual TinaCMS query
      console.log('Loading article:', articleId);
      // For now, keep default form
    } catch (error) {
      console.error('Error loading article:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async (publishStatus?: 'draft' | 'published') => {
    try {
      setSaving(true);
      
      const dataToSave = {
        ...formData,
        status: publishStatus || formData.status,
        publishedAt: publishStatus === 'published' ? new Date().toISOString() : formData.publishedAt,
      };

      // TODO: Implement actual TinaCMS save
      console.log('Saving article:', dataToSave);
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));

      if (publishStatus === 'published') {
        alert('Article published successfully!');
      } else {
        alert('Article saved as draft!');
      }

      if (mode === 'create') {
        router.push('/news');
      }
    } catch (error) {
      console.error('Error saving article:', error);
      alert('Error saving article. Please try again.');
    } finally {
      setSaving(false);
    }
  };

  const addTag = () => {
    if (newTag.trim() && !formData.tags.includes(newTag.trim())) {
      setFormData(prev => ({
        ...prev,
        tags: [...prev.tags, newTag.trim()],
      }));
      setNewTag('');
    }
  };

  const removeTag = (tagToRemove: string) => {
    setFormData(prev => ({
      ...prev,
      tags: prev.tags.filter(tag => tag !== tagToRemove),
    }));
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
          <p className="text-gray-600">Loading article...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-6xl mx-auto">
      {/* Header */}
      <div className="flex items-center justify-between mb-6 pb-4 border-b border-gray-200">
        <div>
          <h2 className="text-xl font-semibold">
            {mode === 'create' ? 'Create New Article' : 'Edit Article'}
          </h2>
          <p className="text-gray-600 mt-1">
            {mode === 'create' 
              ? 'Create a new article with rich content and media'
              : 'Edit article content and settings'
            }
          </p>
        </div>

        <div className="flex items-center space-x-3">
          <Button
            variant="outline"
            onClick={() => router.push('/news')}
          >
            Cancel
          </Button>
          
          <Button
            onClick={() => handleSave('draft')}
            disabled={saving}
            variant="outline"
          >
            {saving ? 'Saving...' : 'Save Draft'}
          </Button>
          
          <Button
            onClick={() => handleSave('published')}
            disabled={saving || !formData.title.trim()}
            className="bg-green-600 hover:bg-green-700"
          >
            {saving ? 'Publishing...' : 'Publish'}
          </Button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6">
        {['content', 'settings', 'seo'].map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab as any)}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
              activeTab === tab
                ? 'bg-blue-100 text-blue-700'
                : 'text-gray-600 hover:text-gray-800 hover:bg-gray-100'
            }`}
          >
            {tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Main Content */}
        <div className="lg:col-span-2">
          {activeTab === 'content' && (
            <Card>
              <CardHeader>
                <CardTitle>Article Content</CardTitle>
              </CardHeader>
              <CardContent className="space-y-6">
                <div>
                  <Label htmlFor="title">Title</Label>
                  <Input
                    id="title"
                    value={formData.title}
                    onChange={(e) => setFormData(prev => ({ ...prev, title: e.target.value }))}
                    placeholder="Enter article title..."
                    className="text-lg font-medium"
                  />
                </div>

                <div>
                  <Label htmlFor="excerpt">Excerpt</Label>
                  <Textarea
                    id="excerpt"
                    value={formData.excerpt}
                    onChange={(e) => setFormData(prev => ({ ...prev, excerpt: e.target.value }))}
                    placeholder="Brief description of the article..."
                    rows={3}
                  />
                </div>

                {/* Rich Text Editor Placeholder */}
                <div>
                  <Label>Content</Label>
                  <div className="border border-gray-300 rounded-lg p-4 min-h-96 bg-white">
                    <div className="text-gray-500 text-center py-12">
                      <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
                        <svg className="w-8 h-8 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                        </svg>
                      </div>
                      <p className="text-lg font-medium mb-2">Rich Text Editor</p>
                      <p>TinaCMS rich text editor will be integrated here</p>
                      <p className="text-sm mt-2">Supports custom components: StockChart, EPSTable, InfoBox, Quote</p>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}

          {activeTab === 'settings' && (
            <Card>
              <CardHeader>
                <CardTitle>Article Settings</CardTitle>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div>
                    <Label htmlFor="author">Author</Label>
                    <Select value={formData.author} onValueChange={(value) => setFormData(prev => ({ ...prev, author: value }))}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="admin-user">EPSX Admin</SelectItem>
                        <SelectItem value="market-analyst">Market Analyst</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>

                  <div>
                    <Label htmlFor="category">Category</Label>
                    <Select value={formData.category} onValueChange={(value) => setFormData(prev => ({ ...prev, category: value }))}>
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="market-analysis">Market Analysis</SelectItem>
                        <SelectItem value="earnings-reports">Earnings Reports</SelectItem>
                        <SelectItem value="eps-insights">EPS Insights</SelectItem>
                        <SelectItem value="platform-updates">Platform Updates</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                </div>

                <div>
                  <Label>Tags</Label>
                  <div className="flex flex-wrap gap-2 mb-2">
                    {formData.tags.map((tag) => (
                      <Badge key={tag} variant="secondary" className="pr-1">
                        {tag}
                        <button
                          onClick={() => removeTag(tag)}
                          className="ml-2 hover:text-red-600"
                        >
                          ×
                        </button>
                      </Badge>
                    ))}
                  </div>
                  <div className="flex gap-2">
                    <Input
                      value={newTag}
                      onChange={(e) => setNewTag(e.target.value)}
                      onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addTag())}
                      placeholder="Add a tag..."
                    />
                    <Button onClick={addTag} variant="outline" size="sm">
                      Add
                    </Button>
                  </div>
                </div>

                <div>
                  <Label htmlFor="featuredImage">Featured Image URL</Label>
                  <Input
                    id="featuredImage"
                    value={formData.featuredImage}
                    onChange={(e) => setFormData(prev => ({ ...prev, featuredImage: e.target.value }))}
                    placeholder="https://example.com/image.jpg"
                  />
                </div>

                <div>
                  <Label htmlFor="imageAlt">Image Alt Text</Label>
                  <Input
                    id="imageAlt"
                    value={formData.imageAlt}
                    onChange={(e) => setFormData(prev => ({ ...prev, imageAlt: e.target.value }))}
                    placeholder="Describe the image for accessibility..."
                  />
                </div>

                <div className="flex items-center space-x-2">
                  <input
                    type="checkbox"
                    id="featured"
                    checked={formData.featured}
                    onChange={(e) => setFormData(prev => ({ ...prev, featured: e.target.checked }))}
                    className="rounded border-gray-300"
                  />
                  <Label htmlFor="featured">Featured Article</Label>
                </div>
              </CardContent>
            </Card>
          )}

          {activeTab === 'seo' && (
            <Card>
              <CardHeader>
                <CardTitle>SEO Settings</CardTitle>
              </CardHeader>
              <CardContent className="space-y-6">
                <div>
                  <Label htmlFor="seoTitle">SEO Title</Label>
                  <Input
                    id="seoTitle"
                    value={formData.seo.title}
                    onChange={(e) => setFormData(prev => ({ 
                      ...prev, 
                      seo: { ...prev.seo, title: e.target.value }
                    }))}
                    placeholder="Leave empty to use article title"
                    maxLength={60}
                  />
                  <p className="text-xs text-gray-500 mt-1">{formData.seo.title.length}/60 characters</p>
                </div>

                <div>
                  <Label htmlFor="seoDescription">SEO Description</Label>
                  <Textarea
                    id="seoDescription"
                    value={formData.seo.description}
                    onChange={(e) => setFormData(prev => ({ 
                      ...prev, 
                      seo: { ...prev.seo, description: e.target.value }
                    }))}
                    placeholder="Leave empty to use article excerpt"
                    rows={3}
                    maxLength={160}
                  />
                  <p className="text-xs text-gray-500 mt-1">{formData.seo.description.length}/160 characters</p>
                </div>

                <div>
                  <Label>SEO Keywords</Label>
                  <div className="flex flex-wrap gap-2 mb-2">
                    {formData.seo.keywords.map((keyword) => (
                      <Badge key={keyword} variant="outline" className="pr-1">
                        {keyword}
                        <button
                          onClick={() => setFormData(prev => ({
                            ...prev,
                            seo: {
                              ...prev.seo,
                              keywords: prev.seo.keywords.filter(k => k !== keyword)
                            }
                          }))}
                          className="ml-2 hover:text-red-600"
                        >
                          ×
                        </button>
                      </Badge>
                    ))}
                  </div>
                  <Input
                    placeholder="Add SEO keywords (press Enter)"
                    onKeyPress={(e) => {
                      if (e.key === 'Enter') {
                        e.preventDefault();
                        const keyword = e.currentTarget.value.trim();
                        if (keyword && !formData.seo.keywords.includes(keyword)) {
                          setFormData(prev => ({
                            ...prev,
                            seo: {
                              ...prev.seo,
                              keywords: [...prev.seo.keywords, keyword]
                            }
                          }));
                          e.currentTarget.value = '';
                        }
                      }
                    }}
                  />
                </div>
              </CardContent>
            </Card>
          )}
        </div>

        {/* Sidebar */}
        <div className="lg:col-span-1">
          <Card>
            <CardHeader>
              <CardTitle>Article Status</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div>
                <Label>Current Status</Label>
                <div className="mt-1">
                  <Badge className={
                    formData.status === 'published' ? 'bg-green-100 text-green-800' :
                    formData.status === 'draft' ? 'bg-yellow-100 text-yellow-800' :
                    'bg-gray-100 text-gray-800'
                  }>
                    {formData.status.charAt(0).toUpperCase() + formData.status.slice(1)}
                  </Badge>
                </div>
              </div>

              <div>
                <Label htmlFor="publishDate">Publish Date</Label>
                <Input
                  id="publishDate"
                  type="datetime-local"
                  value={formData.publishedAt.slice(0, 16)}
                  onChange={(e) => setFormData(prev => ({ 
                    ...prev, 
                    publishedAt: e.target.value + ':00.000Z' 
                  }))}
                />
              </div>

              <div className="pt-4 border-t border-gray-200">
                <h4 className="font-medium mb-2">Quick Actions</h4>
                <div className="space-y-2">
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full"
                    onClick={() => {
                      const url = `/news/${formData.publishedAt.slice(0, 4)}/${formData.publishedAt.slice(5, 7)}/${formData.title.toLowerCase().replace(/\s+/g, '-').replace(/[^\w-]/g, '')}`;
                      window.open(url, '_blank');
                    }}
                    disabled={!formData.title}
                  >
                    Preview Article
                  </Button>
                  
                  <Button
                    variant="outline"
                    size="sm"
                    className="w-full"
                    onClick={() => {
                      navigator.clipboard.writeText(formData.title);
                      alert('Title copied to clipboard!');
                    }}
                    disabled={!formData.title}
                  >
                    Copy Title
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}