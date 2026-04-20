'use server';

import { redirectOnForbidden } from '@/lib/api-error';
import { createAdminApiClient } from '@/shared/api';
import {
  normalizeChatMessage,
  normalizeChatMessages,
  type AdminChatOverviewResp,
  type ChatConversation,
  type ChatMessage,
  type ChatStats,
  type ChatTopic,
} from '@/shared/api/chat';
import type { ApiResponse } from '@/shared/utils/api-client';

async function check403<T>(
  res: ApiResponse<T>,
  route = '/chat'
): Promise<ApiResponse<T>> {
  await redirectOnForbidden(res, route);
  return res;
}

export async function getAdminChatStats(): Promise<ApiResponse<ChatStats>> {
  const client = createAdminApiClient({ serverSide: true });
  return await check403(await client.get('/api/admin/chat/stats'));
}

export async function listAllConversations(
  status?: string,
  topicId?: string,
  agent?: string
): Promise<ApiResponse<ChatConversation[]>> {
  const client = createAdminApiClient({ serverSide: true });
  const params = new URLSearchParams();
  if (status !== undefined && status !== '') {
    params.set('status', status);
  }
  if (topicId !== undefined && topicId !== '') {
    params.set('topic_id', topicId);
  }
  if (agent !== undefined && agent !== '') {
    params.set('agent', agent);
  }
  const qs = params.toString();
  return await check403(
    await client.get(
      `/api/admin/chat/conversations${qs !== '' ? `?${qs}` : ''}`
    )
  );
}

export async function getConversation(
  id: string
): Promise<ApiResponse<ChatConversation>> {
  const client = createAdminApiClient({ serverSide: true });
  return await check403(
    await client.get(`/api/admin/chat/conversations/${id}`)
  );
}

export async function getMessages(
  id: string
): Promise<ApiResponse<ChatMessage[]>> {
  const client = createAdminApiClient({ serverSide: true });
  const res = await check403(
    await client.get<ChatMessage[]>(
      `/api/admin/chat/conversations/${id}/messages`
    )
  );
  if (res.success && Array.isArray(res.data)) {
    return { ...res, data: normalizeChatMessages(res.data) };
  }
  return res;
}

export async function sendReply(
  id: string,
  content: string
): Promise<ApiResponse<ChatMessage>> {
  const client = createAdminApiClient({ serverSide: true });
  const res = await check403(
    await client.post<ChatMessage>(
      `/api/admin/chat/conversations/${id}/messages`,
      { content }
    )
  );
  if (res.success && res.data) {
    return { ...res, data: normalizeChatMessage(res.data) };
  }
  return res;
}

export async function assignAgent(
  id: string,
  agentAddress?: string
): Promise<ApiResponse<ChatConversation>> {
  const client = createAdminApiClient({ serverSide: true });
  return await check403(
    await client.put(`/api/admin/chat/conversations/${id}/assign`, {
      agent_address: agentAddress,
    })
  );
}

export async function updateStatus(
  id: string,
  status: string
): Promise<ApiResponse<ChatConversation>> {
  const client = createAdminApiClient({ serverSide: true });
  return await check403(
    await client.put(`/api/admin/chat/conversations/${id}/status`, { status })
  );
}

export async function markAsRead(id: string): Promise<ApiResponse<void>> {
  const client = createAdminApiClient({ serverSide: true });
  return await check403(
    await client.put(`/api/admin/chat/conversations/${id}/read`, {})
  );
}

export async function getTopics(): Promise<ApiResponse<ChatTopic[]>> {
  const client = createAdminApiClient({ serverSide: true });
  return await check403(await client.get('/api/admin/chat/topics'));
}

export async function getAdminChatOverview(): Promise<
  ApiResponse<AdminChatOverviewResp>
> {
  const client = createAdminApiClient({ serverSide: true });
  return await check403(await client.get('/api/admin/chat/overview'));
}

export interface AdminAgent {
  wallet_address: string;
  tier: string;
  status: string;
}

export async function listAdminAgents(
  search?: string
): Promise<ApiResponse<AdminAgent[]>> {
  const client = createAdminApiClient({ serverSide: true });
  const params: Record<string, string> = { limit: '50', status: 'active' };
  if (search !== undefined && search !== '') {
    params.search = search;
  }
  const qs = new URLSearchParams(params).toString();
  const res = await client.get<{ users: AdminAgent[]; total_count: number }>(
    `/api/admin/users?${qs}`
  );
  if (res.success && res.data) {
    return { ...res, data: res.data.users };
  }
  return { ...res, data: [] };
}
