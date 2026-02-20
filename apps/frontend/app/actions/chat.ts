'use server';

import { createSupportChatClient } from '@/shared/api/chat';
import type { ChatTopic, ChatConversation, ChatMessage } from '@/shared/api/chat';
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

export async function createConversationAction(
  topicId: string,
  subject: string,
  message: string
): Promise<ChatConversation | null> {
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
      return res.data;
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
      return res.data;
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
