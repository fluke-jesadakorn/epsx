/**
 * Chat API Service
 *
 * Re-exports from shared/api/chat for backward compatibility.
 * @deprecated Import directly from @/shared/api/chat instead.
 */

import {
  ChatApi,
  chatApi,
  createChatClient,
  type ChatHistoryResponse,
  type ChatOptions,
  type ChatRequest,
  type ChatResponse,
  type Message,
} from '@/shared/api/chat';

// Re-export types
export type { ChatHistoryResponse, ChatOptions, ChatRequest, ChatResponse, Message };

// ChatApiService class for backward compatibility
export class ChatApiService extends ChatApi {
  constructor() {
    super('/api');
  }

  // Alias methods for backward compatibility
  buildReq(
    messages: Message[],
    opts?: { temp?: number; maxTokens?: number },
  ): ChatRequest {
    return {
      messages,
      temperature: opts?.temp || 0.7,
      maxTokens: opts?.maxTokens || 1000,
    };
  }

  async sendMsg(
    messages: Message[],
    opts?: { temp?: number; maxTokens?: number },
  ): Promise<ChatResponse> {
    return this.sendMessage(messages, opts);
  }

  async getHistory(convId: string): Promise<Message[]> {
    return super.getHistory(convId);
  }

  async streamMsg(
    messages: Message[],
    opts?: { temp?: number; maxTokens?: number },
  ): Promise<ReadableStream<Uint8Array>> {
    return this.streamMessage(messages, opts);
  }
}

// Create a singleton instance
export const chatApiService = new ChatApiService();

// Also export shared client
export { chatApi, createChatClient };
