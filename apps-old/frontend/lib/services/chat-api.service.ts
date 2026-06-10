/**
 * Chat API Service
 *
 * Re-exports from shared/api/chat for backward compatibility.
 * @deprecated Import directly from @/shared/api/chat instead.
 */

export {
  SupportChatApi as ChatApiService,
  createSupportChatClient as createChatClient,
} from '@/shared/api/chat';

export type {
  ChatConversation,
  ChatMessage,
  ChatTopic,
  CreateConversationReq,
  SendMessageReq,
} from '@/shared/api/chat';
