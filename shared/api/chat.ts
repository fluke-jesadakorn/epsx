/**
 * SUPPORT CHAT API CLIENT
 *
 * Human-first support chat with topic selection, conversations, and messaging.
 * AI-ready architecture via sender_type and metadata fields.
 */

import type { ApiResponse, UnifiedApiClient } from '../utils/api-client';

// ============================================================================
// TYPES
// ============================================================================

export interface ChatTopic {
  id: string;
  name: string;
  label: string;
  description: string | null;
  icon: string | null;
  sort_order: number;
  is_active: boolean;
  created_at: string;
}

export interface ChatConversation {
  id: string;
  topic_id: string;
  wallet_address: string;
  subject: string;
  status: 'open' | 'in_progress' | 'resolved' | 'closed';
  assigned_agent: string | null;
  last_message_at: string;
  unread_user: number;
  unread_agent: number;
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface ChatAttachment {
  url: string;
  thumb_url?: string | null;
  filename: string;
  file_type: string;
  size: number;
}

export interface ChatMessage {
  id: string;
  conversation_id: string;
  sender_type: 'user' | 'agent' | 'system' | 'ai';
  sender_address: string | null;
  content: string;
  is_read: boolean;
  metadata: { attachments?: ChatAttachment[] } & Record<string, unknown>;
  created_at: string;
}

export interface ChatStats {
  total_open: number;
  total_in_progress: number;
  total_resolved: number;
  total_unassigned: number;
}

export interface CreateConversationReq {
  topic_id: string;
  subject: string;
  message: string;
}

export interface SendMessageReq {
  content: string;
}

export interface UpdateStatusReq {
  status: string;
}

export interface AssignAgentReq {
  agent_address?: string;
}

export interface UnreadCountResp {
  count: number;
}

export interface ChatInboxResp {
  topics: ChatTopic[];
  conversations: ChatConversation[];
}

export interface ChatFullResp {
  conversation: ChatConversation;
  messages: ChatMessage[];
}

export interface AdminChatOverviewResp {
  stats: ChatStats;
  conversations: ChatConversation[];
  topics: ChatTopic[];
}

// ============================================================================
// SUPPORT CHAT API CLASS
// ============================================================================

export class SupportChatApi {
  constructor(private client: UnifiedApiClient) { }

  // Topics
  async getTopics(): Promise<ApiResponse<ChatTopic[]>> {
    return this.client.get('/api/chat/topics');
  }

  // User conversations
  async createConversation(data: CreateConversationReq): Promise<ApiResponse<ChatConversation>> {
    return this.client.post('/api/chat/conversations', data);
  }

  async listConversations(): Promise<ApiResponse<ChatConversation[]>> {
    return this.client.get('/api/chat/conversations');
  }

  async getConversation(id: string): Promise<ApiResponse<ChatConversation>> {
    return this.client.get(`/api/chat/conversations/${id}`);
  }

  async getMessages(id: string): Promise<ApiResponse<ChatMessage[]>> {
    return this.client.get(`/api/chat/conversations/${id}/messages`);
  }

  async sendMessage(id: string, content: string): Promise<ApiResponse<ChatMessage>> {
    return this.client.post(`/api/chat/conversations/${id}/messages`, { content });
  }

  async updateStatus(id: string, status: string): Promise<ApiResponse<ChatConversation>> {
    return this.client.put(`/api/chat/conversations/${id}/status`, { status });
  }

  async markRead(id: string): Promise<ApiResponse<void>> {
    return this.client.put(`/api/chat/conversations/${id}/read`, {});
  }

  async getUnreadCount(): Promise<ApiResponse<UnreadCountResp>> {
    return this.client.get('/api/chat/unread');
  }

  async getInbox(): Promise<ApiResponse<ChatInboxResp>> {
    return this.client.get('/api/chat/inbox');
  }

  async getConversationFull(id: string): Promise<ApiResponse<ChatFullResp>> {
    return this.client.get(`/api/chat/conversations/${id}/full`);
  }

  // Admin methods
  async adminListConversations(params?: {
    status?: string;
    topic_id?: string;
    agent?: string;
  }): Promise<ApiResponse<ChatConversation[]>> {
    const searchParams = new URLSearchParams();
    if (params?.status !== undefined && params.status !== '') { searchParams.set('status', params.status); }
    if (params?.topic_id !== undefined && params.topic_id !== '') { searchParams.set('topic_id', params.topic_id); }
    if (params?.agent !== undefined && params.agent !== '') { searchParams.set('agent', params.agent); }
    const qs = searchParams.toString();
    return this.client.get(`/api/admin/chat/conversations${qs !== '' ? `?${qs}` : ''}`);
  }

  async adminGetConversation(id: string): Promise<ApiResponse<ChatConversation>> {
    return this.client.get(`/api/admin/chat/conversations/${id}`);
  }

  async adminGetMessages(id: string): Promise<ApiResponse<ChatMessage[]>> {
    return this.client.get(`/api/admin/chat/conversations/${id}/messages`);
  }

  async adminSendReply(id: string, content: string): Promise<ApiResponse<ChatMessage>> {
    return this.client.post(`/api/admin/chat/conversations/${id}/messages`, { content });
  }

  async adminAssignAgent(id: string, agentAddress?: string): Promise<ApiResponse<ChatConversation>> {
    return this.client.put(`/api/admin/chat/conversations/${id}/assign`, { agent_address: agentAddress });
  }

  async adminUpdateStatus(id: string, status: string): Promise<ApiResponse<ChatConversation>> {
    return this.client.put(`/api/admin/chat/conversations/${id}/status`, { status });
  }

  async adminMarkRead(id: string): Promise<ApiResponse<void>> {
    return this.client.put(`/api/admin/chat/conversations/${id}/read`, {});
  }

  async adminGetStats(): Promise<ApiResponse<ChatStats>> {
    return this.client.get('/api/admin/chat/stats');
  }

  async adminGetTopics(): Promise<ApiResponse<ChatTopic[]>> {
    return this.client.get('/api/admin/chat/topics');
  }

  async adminGetOverview(): Promise<ApiResponse<AdminChatOverviewResp>> {
    return this.client.get('/api/admin/chat/overview');
  }
}

// ============================================================================
// FACTORY
// ============================================================================

export function createSupportChatClient(client: UnifiedApiClient): SupportChatApi {
  return new SupportChatApi(client);
}
