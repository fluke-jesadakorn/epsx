'use server';

import { revalidatePath } from 'next/cache';
import { createAdminApiClient } from '@/shared/api';
import type { ApiResponse } from '@/shared/utils/api-client';
import type { FileInfo, UploadResponse, BucketName } from '@/shared/api/media';

function client() {
  return createAdminApiClient({ serverSide: true });
}

export async function listMediaAction(
  bucket: BucketName,
  prefix?: string,
  limit?: number,
): Promise<ApiResponse<FileInfo[]>> {
  const params = new URLSearchParams();
  if (prefix !== undefined && prefix !== '') { params.set('prefix', prefix); }
  if (limit !== undefined) { params.set('limit', String(limit)); }
  const qs = params.toString();
  return client().get(`/api/admin/media/${bucket}${qs !== '' ? `?${qs}` : ''}`);
}

export async function uploadMediaAction(
  bucket: BucketName,
  formData: FormData,
): Promise<ApiResponse<UploadResponse>> {
  // Route to the correct upload endpoint based on bucket
  const endpoint = bucket === 'notifications'
    ? '/api/admin/notifications/upload-image'
    : bucket === 'public'
      ? '/api/admin/files/upload'
      : bucket === 'news'
        ? '/api/admin/news/upload-image'
        : '/api/admin/files/upload'; // chat uses same generic endpoint

  return client().post(endpoint, formData);
}

export async function deleteMediaAction(
  bucket: BucketName,
  key: string,
): Promise<ApiResponse<void>> {
  const res = await client().delete(`/api/admin/media/${bucket}/${key}`) as ApiResponse<void>;
  if (res.success) { revalidatePath('/media'); }
  return res;
}
