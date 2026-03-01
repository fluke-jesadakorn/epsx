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
  is_pinned: boolean;
  pinned_at: string | null;
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
  constructor(private client: UnifiedApiClient) { }

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

  async adminPin(id: string): Promise<ApiResponse<NewsArticle>> {
    return this.client.put(`/api/admin/news/${id}/pin`, {});
  }

  async adminUnpin(id: string): Promise<ApiResponse<NewsArticle>> {
    return this.client.put(`/api/admin/news/${id}/unpin`, {});
  }

  async listFeatured(limit = 3): Promise<ApiResponse<NewsArticle[]>> {
    return this.client.get(`/api/public/news/featured?limit=${limit}`);
  }
}

export function createNewsClient(client: UnifiedApiClient): NewsApi {
  return new NewsApi(client);
}

// ============================================================================
// IMAGE URL RESOLVER
// ============================================================================

/**
 * Resolves a news article's cover_image_url to an absolute URL.
 * Old paths like `/api/public/news/images/foo.png` → CDN direct.
 * New uploads already store full CDN URLs → pass through.
 */
const OLD_NEWS_IMAGE_PREFIX = '/api/public/news/images/';

function getCdnUrl(): string {
  return (typeof process !== 'undefined' && process.env.NEXT_PUBLIC_CDN_URL) ?? 'https://cdn.epsx.io';
}

function getBackendUrl(): string {
  return (
    (typeof process !== 'undefined' && process.env.NEXT_PUBLIC_BACKEND_URL) ??
    (typeof window !== 'undefined' && (window as Window & { __ENV__?: { NEXT_PUBLIC_BACKEND_URL?: string } }).__ENV__?.NEXT_PUBLIC_BACKEND_URL) ??
    'http://127.0.0.1:8080'
  );
}

export function resolveNewsImageUrl(coverImageUrl: string | null | undefined): string | null {
  if (coverImageUrl === null || coverImageUrl === undefined || coverImageUrl === '') {
    return null;
  }
  if (coverImageUrl.startsWith('http://') || coverImageUrl.startsWith('https://')) {
    return coverImageUrl;
  }
  if (coverImageUrl.startsWith(OLD_NEWS_IMAGE_PREFIX)) {
    return `${getCdnUrl()}/news/${coverImageUrl.slice(OLD_NEWS_IMAGE_PREFIX.length)}`;
  }
  const cleanPath = coverImageUrl.startsWith('/') ? coverImageUrl : `/${coverImageUrl}`;
  return `${getBackendUrl()}${cleanPath}`;
}

