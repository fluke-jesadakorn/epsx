'use server';

import type { ChatConversation, ChatFullResp, ChatInboxResp, ChatMessage, ChatTopic } from '@/shared/api/chat';
import {
  createSupportChatClient,
  normalizeChatFullResp,
  normalizeChatMessage,
  normalizeChatMessages,
} from '@/shared/api/chat';
import { logger } from '@/shared/utils/logger';
import { getServerActionClient } from '@/shared/utils/server-fetch';
import { revalidatePath } from 'next/cache';

export async function getTopicsAction(): Promise<ChatTopic[]> {
  try {
    const client = getServerActionClient();
    const api = createSupportChatClient(client);
    const res = await api.getTopics();
    if (res.success && res.data) {
      return res.data;
    }
    logger.debug('Topics fetch failed:', res);
  } catch (e) {
    logger.debug('Failed to fetch topics:', e);
  }
  return [];
}

interface CreateConversationOptions {
  topicId: string;
  subject: string;
  message: string;
}

export async function createConversationAction(
  opts: CreateConversationOptions
): Promise<ChatConversation | null> {
  const { topicId, subject, message } = opts;
  try {
    const client = getServerActionClient();
    const api = createSupportChatClient(client);
    const res = await api.createConversation({ topic_id: topicId, subject, message });
    if (res.success && res.data) {
      revalidatePath('/chat');
      return res.data;
    }
    logger.debug('Conversation create failed:', res);
  } catch (e) {
    logger.debug('Failed to create conversation:', e);
  }
  return null;
}

export async function listConversationsAction(): Promise<ChatConversation[]> {
  try {
    const client = getServerActionClient();
    const api = createSupportChatClient(client);
    const res = await api.listConversations();
    if (res.success && res.data) {
      return res.data;
    }
    logger.debug('Conversations list failed:', res);
  } catch (e) {
    logger.debug('Failed to list conversations:', e);
  }
  return [];
}

export async function getConversationAction(id: string): Promise<ChatConversation | null> {
  try {
    const client = getServerActionClient();
    const api = createSupportChatClient(client);
    const res = await api.getConversation(id);
    if (res.success && res.data) {
      return res.data;
    }
    logger.debug('Conversation fetch failed:', res);
  } catch (e) {
    logger.debug('Failed to fetch conversation:', e);
  }
  return null;
}

export async function getMessagesAction(id: string): Promise<ChatMessage[]> {
  try {
    const client = getServerActionClient();
    const api = createSupportChatClient(client);
    const res = await api.getMessages(id);
    if (res.success && res.data) {
      return normalizeChatMessages(res.data);
    }
    logger.debug('Messages fetch failed:', res);
  } catch (e) {
    logger.debug('Failed to fetch messages:', e);
  }
  return [];
}

export async function sendMessageAction(id: string, content: string): Promise<ChatMessage | null> {
  try {
    const client = getServerActionClient();
    const api = createSupportChatClient(client);
    const res = await api.sendMessage(id, content);
    if (res.success && res.data) {
      revalidatePath(`/chat/${id}`);
      return normalizeChatMessage(res.data);
    }
    logger.debug('Message send failed:', res);
  } catch (e) {
    logger.debug('Failed to send message:', e);
  }
  return null;
}

export async function updateConversationStatusAction(
  id: string,
  status: string
): Promise<ChatConversation | null> {
  try {
    const client = getServerActionClient();
    const api = createSupportChatClient(client);
    const res = await api.updateStatus(id, status);
    if (res.success && res.data) {
      revalidatePath('/chat');
      revalidatePath(`/chat/${id}`);
      return res.data;
    }
    logger.debug('Status update failed:', res);
  } catch (e) {
    logger.debug('Failed to update status:', e);
  }
  return null;
}

export async function markConversationReadAction(id: string): Promise<boolean> {
  try {
    const client = getServerActionClient();
    const api = createSupportChatClient(client);
    const res = await api.markRead(id);
    if (res.success) {
      revalidatePath('/chat');
      return true;
    }
    logger.debug('Mark read failed:', res);
  } catch (e) {
    logger.debug('Failed to mark read:', e);
  }
  return false;
}

export async function notifyTypingAction(id: string, isTyping: boolean): Promise<void> {
  try {
    const client = getServerActionClient();
    await client.post(`/api/chat/conversations/${id}/typing`, { is_typing: isTyping });
  } catch { /* non-critical */ }
}

export async function uploadAttachmentAction(convId: string, formData: FormData): Promise<ChatMessage | null> {
  try {
    const { cookies } = await import('next/headers');
    const { getServerAuthToken } = await import('@/shared/auth/cookies');
    const { getBackendUrl } = await import('@/shared/utils/url-resolver');
    const cookieStore = await cookies();
    const token = getServerAuthToken(cookieStore);
    const res = await fetch(`${getBackendUrl('server')}/api/chat/conversations/${convId}/upload`, {
      method: 'POST',
      headers: token !== null ? { Authorization: `Bearer ${token}` } : {},
      body: formData,
    });
    const json = await res.json() as { success?: boolean; data?: { message?: ChatMessage } };
    if (json.success !== true || json.data?.message === undefined) {
      return null;
    }
    return normalizeChatMessage(json.data.message);
  } catch (e) {
    logger.debug('Failed to upload attachment:', e);
    return null;
  }
}

export async function getUnreadCountAction(): Promise<number> {
  try {
    const client = getServerActionClient();
    const api = createSupportChatClient(client);
    const res = await api.getUnreadCount();
    if (res.success && res.data) {
      return res.data.count;
    }
    logger.debug('Unread count fetch failed:', res);
  } catch (e) {
    logger.debug('Failed to fetch unread count:', e);
  }
  return 0;
}

export async function getChatInboxAction(): Promise<ChatInboxResp> {
  try {
    const client = getServerActionClient();
    const api = createSupportChatClient(client);
    const res = await api.getInbox();
    if (res.success && res.data) {
      return res.data;
    }
    logger.debug('Chat inbox fetch failed:', res);
  } catch (e) {
    logger.debug('Failed to fetch chat inbox:', e);
  }
  return { topics: [], conversations: [] };
}

export async function getChatFullAction(id: string): Promise<ChatFullResp | null> {
  try {
    const client = getServerActionClient();
    const api = createSupportChatClient(client);
    const res = await api.getConversationFull(id);
    if (res.success && res.data) {
      return normalizeChatFullResp(res.data);
    }
    logger.debug('Chat full fetch failed:', res);
  } catch (e) {
    logger.debug('Failed to fetch chat full:', e);
  }
  return null;
}
