'use client';

import { MessageCircle, Send, Minimize2 } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { streamChat } from "@/app/actions/chat";
import { Button, Card, CardContent, CardFooter, CardHeader, CardTitle, Input } from "@/components/ui";
import { ErrorBoundary } from "../common/ErrorBoundary";
import type { Message } from "@/types/chat";

interface ChatDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export function ChatDialog({ isOpen, onClose }: ChatDialogProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const chatEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    chatEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSendMessage = async (message: string) => {
    if (!message.trim()) return;

    try {
      setIsLoading(true);
      setInput("");

      const newUserMessage: Message = {
        role: "user",
        content: message,
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, newUserMessage]);

      const stream = await streamChat({
        messages: [...messages, newUserMessage],
        temperature: 0.7,
        maxTokens: 1000,
      });

      const reader = stream.getReader();
      const decoder = new TextDecoder();
      let accumulatedResponse = "";

      setMessages((prev) => [...prev, { 
        role: "assistant", 
        content: "",
        timestamp: new Date(),
      }]);

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        const chunk = decoder.decode(value);
        accumulatedResponse += chunk;

        setMessages((prev) => {
          const newMessages = [...prev];
          if (newMessages[newMessages.length - 1].timestamp) {
            newMessages[newMessages.length - 1] = {
              role: "assistant",
              content: accumulatedResponse,
              timestamp: newMessages[newMessages.length - 1].timestamp,
            };
          }
          return newMessages;
        });
      }
    } catch (error) {
      console.error("Failed to send message:", error);
      setMessages((prev) => [
        ...prev,
        {
          role: "assistant",
          content: "Sorry, I encountered an error. Please try again.",
          timestamp: new Date(),
        },
      ]);
    } finally {
      setIsLoading(false);
    }
  };

  if (!isOpen) return null;

  return (
    <ErrorBoundary>
      <Card
        className="transform transition-all duration-200 ease-in-out w-[400px] shadow-xl border fixed bottom-6 right-6 z-50"
      >
        <CardHeader>
          <CardTitle className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-2">
              <MessageCircle className="h-5 w-5" />
              Chat with EPSx AI
            </div>
            <Button
              variant="ghost"
              size="icon"
              onClick={onClose}
              className="h-8 w-8 rounded-full hover:bg-primary/10"
            >
              <Minimize2 className="h-4 w-4" />
            </Button>
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="h-[400px] overflow-y-auto pr-4 space-y-4 custom-scrollbar">
            {messages.map((message, i) => (
              <div
                key={i}
                className={`flex flex-col ${
                  message.role === "assistant"
                    ? "items-start"
                    : "items-end"
                } animate-fade-in`}
              >
                <div
                  className={`relative max-w-[80%] p-4 ${
                    message.role === "assistant"
                      ? "bg-primary/10 text-foreground rounded-[20px] rounded-tl-none"
                      : "bg-gradient-to-r from-blue-500 to-purple-500 text-white rounded-[20px] rounded-br-none"
                  } shadow-sm hover:shadow-md transition-shadow duration-200`}
                >
                  {message.role === "assistant" && (
                    <div className="absolute -left-2 -top-0 w-4 h-4 bg-primary/10 clip-bubble-left" />
                  )}
                  {message.role === "user" && (
                    <div className="absolute -right-2 -top-0 w-4 h-4 bg-gradient-to-r from-blue-500 to-purple-500 clip-bubble-right" />
                  )}
                  <div className="whitespace-pre-wrap break-words">
                    {message.content}
                  </div>
                </div>
                <span className="text-xs text-muted-foreground mt-1 px-2">
                  {message.timestamp && 
                    message.timestamp.toLocaleTimeString([], { 
                      hour: '2-digit', 
                      minute: '2-digit' 
                    })
                  }
                </span>
              </div>
            ))}
            <div ref={chatEndRef} />
          </div>
        </CardContent>
        <CardFooter>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              handleSendMessage(input);
            }}
            className="flex w-full gap-2"
          >
            <Input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              placeholder="Ask about EPS trends, data insights, or specific companies..."
              disabled={isLoading}
              className="flex-1"
              error={undefined}
            />
            <Button
              type="submit"
              disabled={isLoading || !input.trim()}
              className="bg-gradient-to-r from-blue-500 to-purple-500 hover:from-blue-600 hover:to-purple-600 text-white transform hover:scale-105 transition-all duration-200"
            >
              <Send className="h-4 w-4" />
            </Button>
          </form>
        </CardFooter>
      </Card>
    </ErrorBoundary>
  );
}