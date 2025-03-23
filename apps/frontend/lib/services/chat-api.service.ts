import type {
  ChatRequest,
  ChatResponse,
  Message,
  ChatHistoryResponse,
} from "@/types/chat";

export class ChatApiService {
  private baseUrl: string;

  constructor() {
    this.baseUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001";
  }

  private buildRequest(
    messages: Message[],
    options?: { temperature?: number; maxTokens?: number }
  ): ChatRequest {
    return {
      messages,
      temperature: options?.temperature || 0.7,
      maxTokens: options?.maxTokens || 1000,
    };
  }

  async sendMessage(
    messages: Message[],
    options?: { temperature?: number; maxTokens?: number }
  ): Promise<ChatResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/chat`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(this.buildRequest(messages, options)),
      });

      if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
      }

      const result = await response.json();
      return {
        message: {
          role: "assistant",
          content: result.message.content,
        },
        usage: result.usage || {
          totalTokens: 0,
          promptTokens: 0,
          completionTokens: 0,
        },
      };
    } catch (error) {
      console.error("Chat API error:", error);
      throw error;
    }
  }

  async getChatHistory(conversationId: string): Promise<Message[]> {
    try {
      const response = await fetch(
        `${this.baseUrl}/chat/history/${conversationId}`
      );

      if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
      }

      const data: ChatHistoryResponse = await response.json();
      return data.messages;
    } catch (error) {
      console.error("Chat history API error:", error);
      throw error;
    }
  }

  async streamMessage(
    messages: Message[],
    options?: { temperature?: number; maxTokens?: number }
  ): Promise<ReadableStream<Uint8Array>> {
    try {
      const response = await fetch(`${this.baseUrl}/chat/stream`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(this.buildRequest(messages, options)),
      });

      if (!response.ok) {
        throw new Error(`API error: ${response.status}`);
      }

      if (!response.body) {
        throw new Error("No response body received");
      }

      return response.body;
    } catch (error) {
      console.error("Chat stream error:", error);
      throw error;
    }
  }
}

// Create a singleton instance
export const chatApiService = new ChatApiService();
