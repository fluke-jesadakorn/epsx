"use server";

import { chatApiService } from "@/lib/services/chat-api.service";

import type { ChatRequest, ChatResponse } from "@/types/chat.d";

export async function chatQuery(request: ChatRequest): Promise<ChatResponse> {
  try {
    return await chatApiService.sendMsg(request.messages, {
      temp: request.temperature || undefined,
      maxTokens: request.maxTokens || undefined,
    });
  } catch (error) {
    // console.error("Server chat query failed:", error);
    throw error;
  }
}

export async function getChatHistory(conversationId: string) {
  try {
    return await chatApiService.getHistory(conversationId);
  } catch (error) {
    // console.error("Failed to fetch chat history:", error);
    throw error;
  }
}

export async function streamChat(request: ChatRequest): Promise<ReadableStream<Uint8Array>> {
  try {
    return await chatApiService.streamMsg(request.messages, {
      temp: request.temperature || undefined,
      maxTokens: request.maxTokens || undefined,
    });
  } catch (error) {
    // console.error("Stream chat query failed:", error);
    throw error;
  }
}
