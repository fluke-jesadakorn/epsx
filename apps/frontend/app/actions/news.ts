'use server';

import { createNewsClient } from '@/shared/api/news';
import type { NewsArticle, NewsListResponse } from '@/shared/api/news';
import type { ApiResponse } from '@/shared/utils/api-client';
import { getServerActionClient } from '@/shared/utils/server-fetch';

function newsClient() {
  return createNewsClient(getServerActionClient());
}

export async function getPublicNews(page = 1, limit = 10): Promise<ApiResponse<NewsListResponse>> {
  return newsClient().listPublished(page, limit);
}

export async function getNewsBySlug(slug: string): Promise<ApiResponse<NewsArticle>> {
  return newsClient().getBySlug(slug);
}
