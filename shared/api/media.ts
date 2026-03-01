import type { ApiResponse, UnifiedApiClient } from '../utils/api-client';

// ============================================================================
// TYPES
// ============================================================================

export interface FileInfo {
  key: string;
  url: string;
  size: number;
  last_modified: string | null;
}

export interface UploadResponse {
  url: string;
  thumb_url: string | null;
  filename: string;
  mime: string;
  size: number;
}

export type BucketName = 'chat' | 'news' | 'notifications' | 'public';

// ============================================================================
// API CLIENT
// ============================================================================

export class MediaApi {
  constructor(private client: UnifiedApiClient) {}

  // Notification image upload
  async uploadNotificationImage(formData: FormData): Promise<ApiResponse<UploadResponse>> {
    return this.client.post('/api/admin/notifications/upload-image', formData);
  }

  // Public file management
  async uploadPublicFile(formData: FormData): Promise<ApiResponse<UploadResponse>> {
    return this.client.post('/api/admin/files/upload', formData);
  }

  async listPublicFiles(prefix?: string, limit?: number): Promise<ApiResponse<FileInfo[]>> {
    const params = new URLSearchParams();
    if (prefix !== undefined) { params.set('prefix', prefix); }
    if (limit !== undefined) { params.set('limit', String(limit)); }
    const qs = params.toString();
    return this.client.get(`/api/admin/files${qs !== '' ? `?${qs}` : ''}`);
  }

  async deletePublicFile(key: string): Promise<ApiResponse<void>> {
    return this.client.delete(`/api/admin/files/${key}`);
  }

  // Generic media management (any bucket)
  async listMedia(bucket: BucketName, prefix?: string, limit?: number): Promise<ApiResponse<FileInfo[]>> {
    const params = new URLSearchParams();
    if (prefix !== undefined) { params.set('prefix', prefix); }
    if (limit !== undefined) { params.set('limit', String(limit)); }
    const qs = params.toString();
    return this.client.get(`/api/admin/media/${bucket}${qs !== '' ? `?${qs}` : ''}`);
  }

  async deleteMedia(bucket: BucketName, key: string): Promise<ApiResponse<void>> {
    return this.client.delete(`/api/admin/media/${bucket}/${key}`);
  }
}

export function createMediaClient(client: UnifiedApiClient): MediaApi {
  return new MediaApi(client);
}
