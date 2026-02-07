/**
 * UNIFIED CHAT API CLIENT
 *
 * Chat and AI assistant endpoints.
 * Migrated from frontend/lib/services/chat-api.service.ts
 *
 * Features:
 * - Send chat messages
 * - Get chat history
 * - Stream chat responses
 */

// ============================================================================
// TYPES
// ============================================================================

export interface Message {
    role: 'user' | 'assistant' | 'system';
    content: string;
}

export interface ChatRequest {
    messages: Message[];
    temperature: number;
    maxTokens: number;
}

export interface ChatResponse {
    message: Message;
    usage: {
        totalTokens: number;
        promptTokens: number;
        completionTokens: number;
    };
}

export interface ChatHistoryResponse {
    messages: Message[];
}

export interface ChatOptions {
    temp?: number;
    maxTokens?: number;
}

// ============================================================================
// CHAT API CLASS
// ============================================================================

export class ChatApi {
    private baseUrl: string;

    constructor(baseUrl = '/api') {
        this.baseUrl = baseUrl;
    }

    private buildRequest(messages: Message[], opts?: ChatOptions): ChatRequest {
        return {
            messages,
            temperature: opts?.temp || 0.7,
            maxTokens: opts?.maxTokens || 1000,
        };
    }

    /**
     * Send a chat message and get a response
     */
    async sendMessage(messages: Message[], opts?: ChatOptions): Promise<ChatResponse> {
        const response = await fetch(`${this.baseUrl}/chat`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(this.buildRequest(messages, opts)),
        });

        if (!response.ok) {
            throw new Error(`Chat API error: ${response.status}`);
        }

        const result = await response.json();
        return {
            message: {
                role: 'assistant',
                content: result.message.content,
            },
            usage: result.usage || {
                totalTokens: 0,
                promptTokens: 0,
                completionTokens: 0,
            },
        };
    }

    /**
     * Get chat history for a conversation
     */
    async getHistory(conversationId: string): Promise<Message[]> {
        const response = await fetch(`${this.baseUrl}/chat/history/${conversationId}`);

        if (!response.ok) {
            throw new Error(`Chat history API error: ${response.status}`);
        }

        const data: ChatHistoryResponse = await response.json();
        return data.messages;
    }

    /**
     * Stream a chat message response
     */
    async streamMessage(messages: Message[], opts?: ChatOptions): Promise<ReadableStream<Uint8Array>> {
        const response = await fetch(`${this.baseUrl}/chat/stream`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(this.buildRequest(messages, opts)),
        });

        if (!response.ok) {
            throw new Error(`Chat stream API error: ${response.status}`);
        }

        if (!response.body) {
            throw new Error('No response body received');
        }

        return response.body;
    }
}

// ============================================================================
// FACTORY FUNCTION
// ============================================================================

/**
 * Create a chat API client
 */
export function createChatClient(baseUrl = '/api'): ChatApi {
    return new ChatApi(baseUrl);
}

// ============================================================================
// SINGLETON INSTANCE
// ============================================================================

/**
 * Default chat API client instance
 */
export const chatApi = new ChatApi();
