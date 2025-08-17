export interface Message {
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

export interface ChatRequest {
  messages: Message[];
  temperature?: number;
  maxTokens?: number;
}

export interface ChatResponse {
  message: string;
  error?: string;
}