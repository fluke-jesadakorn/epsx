"use server";

import type { ChatRequest, ChatResponse } from "@/types/chat";
import { chatApiService } from "@/lib/services/chat-api.service";

export async function chatQuery(request: ChatRequest): Promise<ChatResponse> {
  try {
    return await chatApiService.sendMessage(request.messages, {
      temperature: request.temperature,
      maxTokens: request.maxTokens,
    });
  } catch (error) {
    console.error("Server chat query failed:", error);
    throw error;
  }
}

export async function getChatHistory(conversationId: string) {
  try {
    return await chatApiService.getChatHistory(conversationId);
  } catch (error) {
    console.error("Failed to fetch chat history:", error);
    throw error;
  }
}

export async function streamChat(request: ChatRequest): Promise<ReadableStream<Uint8Array>> {
  try {
    return await chatApiService.streamMessage(request.messages, {
      temperature: request.temperature,
      maxTokens: request.maxTokens,
    });
  } catch (error) {
    console.error("Stream chat query failed:", error);
    throw error;
  }
}
