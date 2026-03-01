'use server';

import { revalidatePath } from 'next/cache';
import { createAdminApiClient } from '@/shared/api';
import type { CreateNewsReq, NewsArticle, NewsImageUploadResponse, NewsListResponse, UpdateNewsReq } from '@/shared/api/news';
import type { ApiResponse } from '@/shared/utils/api-client';

function client() {
  return createAdminApiClient({ serverSide: true });
}

export async function listNewsAction(
  page = 1,
  limit = 20,
  status?: string,
): Promise<ApiResponse<NewsListResponse>> {
  const params = new URLSearchParams({ page: String(page), limit: String(limit) });
  if (status !== undefined) { params.set('status', status); }
  return client().get(`/api/admin/news?${params.toString()}`);
}

export async function getNewsAction(id: string): Promise<ApiResponse<NewsArticle>> {
  return client().get(`/api/admin/news/${id}`);
}

export async function createNewsAction(data: CreateNewsReq): Promise<ApiResponse<NewsArticle>> {
  return client().post('/api/admin/news', data);
}

export async function updateNewsAction(id: string, data: UpdateNewsReq): Promise<ApiResponse<NewsArticle>> {
  return client().put(`/api/admin/news/${id}`, data);
}

export async function deleteNewsAction(id: string): Promise<ApiResponse<unknown>> {
  const res = await client().delete(`/api/admin/news/${id}`);
  if (res.success) { revalidatePath('/news'); }
  return res;
}

export async function publishNewsAction(id: string): Promise<ApiResponse<unknown>> {
  const res = await client().put(`/api/admin/news/${id}/publish`, {});
  if (res.success) { revalidatePath('/news'); }
  return res;
}

export async function unpublishNewsAction(id: string): Promise<ApiResponse<unknown>> {
  const res = await client().put(`/api/admin/news/${id}/unpublish`, {});
  if (res.success) { revalidatePath('/news'); }
  return res;
}

export async function uploadNewsImageAction(formData: FormData): Promise<ApiResponse<NewsImageUploadResponse>> {
  return client().post('/api/admin/news/upload-image', formData);
}

export async function pinNewsAction(id: string): Promise<ApiResponse<unknown>> {
  const res = await client().put(`/api/admin/news/${id}/pin`, {});
  if (res.success) { revalidatePath('/news'); }
  return res;
}

export async function unpinNewsAction(id: string): Promise<ApiResponse<unknown>> {
  const res = await client().put(`/api/admin/news/${id}/unpin`, {});
  if (res.success) { revalidatePath('/news'); }
  return res;
}
