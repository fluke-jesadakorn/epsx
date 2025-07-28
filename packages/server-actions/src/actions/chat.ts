'use server';

import { serverGet, serverPost } from '../core/request';

// Chat Types
export interface ChatMessage {
  id: string;
  userId: string;
  content: string;
  timestamp: string;
  type: 'user' | 'system' | 'ai';
  metadata?: Record<string, any>;
}

export interface ChatSession {
  id: string;
  userId: string;
  title?: string;
  createdAt: string;
  updatedAt: string;
  isActive: boolean;
  messageCount: number;
}

// Chat Actions
export async function getChatSessions(): Promise<ChatSession[]> {
  try {
    const response = await serverGet('/api/v1/chat/sessions');
    return response || [];
  } catch (error) {
    console.error('Error fetching chat sessions:', error);
    return [];
  }
}

export async function createChatSession(title?: string): Promise<ChatSession> {
  try {
    return await serverPost('/api/v1/chat/sessions', { title });
  } catch (error) {
    console.error('Error creating chat session:', error);
    throw error;
  }
}

export async function getChatMessages(sessionId: string, params?: {
  page?: number;
  limit?: number;
  since?: string;
}): Promise<{
  messages: ChatMessage[];
  pagination?: {
    page: number;
    limit: number;
    total: number;
    hasMore: boolean;
  };
}> {
  try {
    const response = await serverGet(`/api/v1/chat/sessions/${sessionId}/messages`, params);
    return response || { messages: [] };
  } catch (error) {
    console.error('Error fetching chat messages:', error);
    return { messages: [] };
  }
}

export async function sendChatMessage(sessionId: string, content: string, metadata?: Record<string, any>) {
  try {
    return await serverPost(`/api/v1/chat/sessions/${sessionId}/messages`, {
      content,
      metadata,
    });
  } catch (error) {
    console.error('Error sending chat message:', error);
    throw error;
  }
}

export async function deleteChatSession(sessionId: string) {
  try {
    return await serverPost(`/api/v1/chat/sessions/${sessionId}/delete`);
  } catch (error) {
    console.error('Error deleting chat session:', error);
    throw error;
  }
}

export async function updateChatSession(sessionId: string, data: {
  title?: string;
  isActive?: boolean;
}) {
  try {
    return await serverPost(`/api/v1/chat/sessions/${sessionId}/update`, data);
  } catch (error) {
    console.error('Error updating chat session:', error);
    throw error;
  }
}

export async function getChatAnalytics(): Promise<{
  totalSessions: number;
  totalMessages: number;
  averageSessionLength: number;
  mostActiveHours: number[];
}> {
  try {
    const response = await serverGet('/api/v1/chat/analytics');
    return response || {
      totalSessions: 0,
      totalMessages: 0,
      averageSessionLength: 0,
      mostActiveHours: [],
    };
  } catch (error) {
    console.error('Error fetching chat analytics:', error);
    return {
      totalSessions: 0,
      totalMessages: 0,
      averageSessionLength: 0,
      mostActiveHours: [],
    };
  }
}