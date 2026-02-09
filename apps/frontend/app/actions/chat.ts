/* eslint-disable @typescript-eslint/no-unsafe-member-access */
"use server";

import { chatApiService } from "@/lib/services/chat-api.service";

import type { ChatRequest, ChatResponse } from "@/types/chat.d";

export async function chatQuery(request: ChatRequest): Promise<ChatResponse> {
  return await chatApiService.sendMsg(request.messages, {
    temp: request.temperature ?? undefined,
    maxTokens: request.maxTokens ?? undefined,
  });
}

export async function getChatHistory(conversationId: string) {
  return await chatApiService.getHistory(conversationId);
}

export async function streamChat(request: ChatRequest): Promise<ReadableStream<Uint8Array>> {
  return await chatApiService.streamMsg(request.messages, {
    temp: request.temperature ?? undefined,
    maxTokens: request.maxTokens ?? undefined,
  });
}
