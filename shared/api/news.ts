import type { ApiResponse, UnifiedApiClient } from '../utils/api-client';

// ============================================================================
// TYPES
// ============================================================================

export interface NewsArticle {
  id: string;
  title: string;
  slug: string;
  summary: string | null;
  content: string;
  cover_image_url: string | null;
  author_wallet: string;
  status: 'draft' | 'published';
  tags: string[];
  published_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface NewsListResponse {
  articles: NewsArticle[];
  total: number;
  page: number;
  limit: number;
}

export interface CreateNewsReq {
  title: string;
  content: string;
  summary?: string;
  cover_image_url?: string;
  tags?: string[];
  status?: 'draft' | 'published';
}

export interface UpdateNewsReq {
  title?: string;
  slug?: string;
  content?: string;
  summary?: string;
  cover_image_url?: string;
  tags?: string[];
  status?: 'draft' | 'published';
}

export interface NewsImageUploadResponse {
  url: string;
  filename: string;
  mime: string;
  size: number;
}

// ============================================================================
// API CLIENT
// ============================================================================

export class NewsApi {
  constructor(private client: UnifiedApiClient) {}

  // Public
  async listPublished(page = 1, limit = 10): Promise<ApiResponse<NewsListResponse>> {
    return this.client.get(`/api/public/news?page=${page}&limit=${limit}`);
  }

  async getBySlug(slug: string): Promise<ApiResponse<NewsArticle>> {
    return this.client.get(`/api/public/news/${slug}`);
  }

  // Admin
  async adminList(page = 1, limit = 20, status?: string): Promise<ApiResponse<NewsListResponse>> {
    const params = new URLSearchParams({ page: String(page), limit: String(limit) });
    if (status !== undefined) { params.set('status', status); }
    return this.client.get(`/api/admin/news?${params.toString()}`);
  }

  async adminGet(id: string): Promise<ApiResponse<NewsArticle>> {
    return this.client.get(`/api/admin/news/${id}`);
  }

  async adminCreate(data: CreateNewsReq): Promise<ApiResponse<NewsArticle>> {
    return this.client.post('/api/admin/news', data);
  }

  async adminUpdate(id: string, data: UpdateNewsReq): Promise<ApiResponse<NewsArticle>> {
    return this.client.put(`/api/admin/news/${id}`, data);
  }

  async adminDelete(id: string): Promise<ApiResponse<void>> {
    return this.client.delete(`/api/admin/news/${id}`);
  }

  async adminPublish(id: string): Promise<ApiResponse<NewsArticle>> {
    return this.client.put(`/api/admin/news/${id}/publish`, {});
  }

  async adminUnpublish(id: string): Promise<ApiResponse<NewsArticle>> {
    return this.client.put(`/api/admin/news/${id}/unpublish`, {});
  }

  async adminUploadImage(formData: FormData): Promise<ApiResponse<NewsImageUploadResponse>> {
    return this.client.post('/api/admin/news/upload-image', formData);
  }
}

export function createNewsClient(client: UnifiedApiClient): NewsApi {
  return new NewsApi(client);
}
