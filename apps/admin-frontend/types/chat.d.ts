export interface Message {
  id?: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp?: Date;
  user_id?: string;
  conversation_id?: string;
  created_at?: string;
  updated_at?: string;
}

export interface Usage {
  totalTokens: number;
  promptTokens: number;
  completionTokens: number;
}

export interface ChatRequest {
  messages: Message[];
  temperature?: number;
  maxTokens?: number;
}

export interface ChatResponse {
  message: Message;
  usage?: Usage;
}

export interface ChatHistoryResponse {
  id: string;
  start_time: string;
  end_time: string;
  messages: Message[];
  status: string;
}

export interface ChatMessage {
  id: string;
  role: string;
  content: string;
  timestamp: string;
  user_id?: string;
  conversation_id?: string;
  created_at?: string;
  updated_at?: string;
}
