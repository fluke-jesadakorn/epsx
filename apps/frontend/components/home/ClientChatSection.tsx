"use client";

import { useRef, useState, useEffect, useTransition } from "react";
import { Layout, Input, Button, Avatar, Spin, message, theme } from "antd";
import { SendOutlined, UserOutlined, RobotOutlined } from "@ant-design/icons";
import { ChatRequest, Message, ChatHistoryResponse } from "@/types/chat";
import { chatQuery, getChatHistory, streamChat } from "@/app/actions/chat";
import styled from "@emotion/styled";
import { v4 as uuidv4 } from 'uuid';
const { Content, Footer } = Layout;
const { useToken } = theme;

interface StyledProps {
  theme: {
    colorBgContainer: string;
    colorBgElevated: string;
    colorPrimary: string;
    colorText: string;
    colorBorder: string;
    borderRadius: number;
    borderRadiusLG: number;
  };
}

interface MessageBubbleProps extends StyledProps {
  isUser: boolean;
}

// Styled components for chat UI
const ChatContainer = styled(Layout)<StyledProps>`
  height: calc(100vh - 6rem);
  background: ${(props: StyledProps) => props.theme.colorBgContainer};
  border-radius: ${(props: StyledProps) => props.theme.borderRadiusLG}px;
  overflow: hidden;
`;

const MessagesContainer = styled(Content)<StyledProps>`
  padding: 24px;
  overflow-y: auto;
  background: ${(props: StyledProps) => props.theme.colorBgContainer};
`;

const MessageBubble = styled.div<MessageBubbleProps>`
  display: flex;
  align-items: start;
  margin-bottom: 24px;
  flex-direction: ${(props: MessageBubbleProps) =>
    props.isUser ? "row-reverse" : "row"};

  .message-content {
    max-width: 80%;
    margin: ${(props: MessageBubbleProps) =>
      props.isUser ? "0 16px 0 0" : "0 0 0 16px"};
    padding: 12px 16px;
    border-radius: ${(props: MessageBubbleProps) => props.theme.borderRadius}px;
    background: ${(props: MessageBubbleProps) =>
      props.isUser ? props.theme.colorPrimary : props.theme.colorBgElevated};
    color: ${(props: MessageBubbleProps) =>
      props.isUser ? "#fff" : props.theme.colorText};
  }
`;

const InputContainer = styled(Footer)<StyledProps>`
  padding: 16px 24px;
  background: ${(props: StyledProps) => props.theme.colorBgContainer};
  border-top: 1px solid ${(props: StyledProps) => props.theme.colorBorder};
`;

const InputWrapper = styled.div`
  max-width: 800px;
  margin: 0 auto;
  display: flex;
  gap: 8px;

  .ant-input {
    border-radius: ${(props: any) => props.theme.borderRadius}px;
  }

  .ant-btn {
    border-radius: ${(props: any) => props.theme.borderRadius}px;
  }
`;

interface Props {
  initialConversationId?: string;
}

export default function ClientChatSection({ initialConversationId }: Props) {
  const { token } = useToken();
  const [messages, setMessages] = useState<Message[]>([]);
  const [query, setQuery] = useState("");
  const [loading, setLoading] = useState(false);
  const [conversationId, setConversationId] = useState<string>(initialConversationId || uuidv4());
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const [messageApi, contextHolder] = message.useMessage();
  const [isPending, startTransition] = useTransition();
  const reader = useRef<ReadableStreamDefaultReader<Uint8Array> | null>(null);
  const decoder = useRef(new TextDecoder());

  useEffect(() => {
    if (conversationId) {
      loadChatHistory();
    }
  }, [conversationId]);

  // Scroll to bottom when messages change
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  // Cleanup stream reader on unmount
  useEffect(() => {
    return () => {
      if (reader.current) {
        reader.current.cancel();
        reader.current = null;
      }
    };
  }, []);

  const loadChatHistory = async () => {
    try {
      const messages = await getChatHistory(conversationId);
      if (messages && messages.length > 0) {
        setMessages(messages);
      }
    } catch (error) {
      console.error("Failed to load chat history:", error);
      messageApi.error({
        content: "Failed to load chat history.",
        duration: 3,
      });
    }
  };

  const handleStream = async (stream: ReadableStream<Uint8Array>) => {
    const streamReader = stream.getReader();
    reader.current = streamReader;
    let accumulatedContent = '';
    const messageId = uuidv4();
    
    try {
      // Initialize the assistant message
      setMessages(prev => [
        ...prev,
        {
          id: messageId,
          role: "assistant",
          content: '',
          timestamp: new Date()
        }
      ]);

      while (true) {
        const { done, value } = await streamReader.read();
        if (done) break;
        
        const chunk = decoder.current.decode(value, { stream: true });
        accumulatedContent += chunk;

        // Update message with accumulated content
        setMessages(prev => {
          const messageIndex = prev.findIndex(msg => msg.id === messageId);
          if (messageIndex === -1) return prev;

          const newMessages = [...prev];
          newMessages[messageIndex] = {
            ...newMessages[messageIndex],
            content: accumulatedContent
          };
          return newMessages;
        });
      }

      // Handle final chunk if any
      const finalChunk = decoder.current.decode();
      if (finalChunk) {
        accumulatedContent += finalChunk;
        setMessages(prev => {
          const messageIndex = prev.findIndex(msg => msg.id === messageId);
          if (messageIndex === -1) return prev;

          const newMessages = [...prev];
          newMessages[messageIndex] = {
            ...newMessages[messageIndex],
            content: accumulatedContent
          };
          return newMessages;
        });
      }
    } catch (error) {
      console.error("Stream reading error:", error);
      messageApi.error({
        content: "Error receiving message stream",
        duration: 3,
      });
    } finally {
      reader.current = null;
    }
  };

  const handleSubmit = async () => {
    if (!query.trim() || loading) return;

    const userMessage: Message = {
      id: uuidv4(),
      role: "user",
      content: query.trim(),
      timestamp: new Date(),
      conversation_id: conversationId
    };

    setMessages((prev) => [...prev, userMessage]);
    setQuery("");
    setLoading(true);


    startTransition(async () => {
      try {
        const chatRequest: ChatRequest = {
          messages: [...messages, userMessage],
          temperature: 0.7,
          maxTokens: 10000
        };

        // Start streaming response
        const stream = await streamChat(chatRequest);
        handleStream(stream);

      } catch (error) {
        messageApi.error({
          content: "Failed to send message. Please try again.",
          duration: 3,
        });
        console.error("Chat query failed:", error);
      } finally {
        setLoading(false);
      }
    });
  };

  const MessageComponent = ({ message }: { message: Message }) => (
    <MessageBubble isUser={message.role === "user"} theme={token}>
      <Avatar
        icon={message.role === "user" ? <UserOutlined /> : <RobotOutlined />}
        style={{
          backgroundColor:
            message.role === "user" ? token.colorPrimary : token.colorSuccess,
        }}
      />
      <div className="message-content">
        {message.role === "assistant" ? (
          <div dangerouslySetInnerHTML={{ __html: message.content }} />
        ) : (
          <div>{message.content}</div>
        )}
      </div>
    </MessageBubble>
  );

  return (
    <ChatContainer theme={token}>
      {contextHolder}
      <MessagesContainer theme={token}>
        {messages.map((message) => (
          <MessageComponent key={message.id} message={message} />
        ))}
        <div ref={messagesEndRef} />
        {loading && (
          <MessageBubble isUser={false} theme={token}>
            <Avatar
              icon={<RobotOutlined />}
              style={{ backgroundColor: token.colorSuccess }}
            />
            <div className="message-content">
              <Spin size="small" />
            </div>
          </MessageBubble>
        )}
      </MessagesContainer>

      <InputContainer theme={token}>
        <InputWrapper>
          <Input.TextArea
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Type your message here..."
            autoSize={{ minRows: 1, maxRows: 4 }}
            onPressEnter={(e) => {
              if (!e.shiftKey) {
                e.preventDefault();
                handleSubmit();
              }
            }}
            disabled={loading}
            style={{ flex: 1 }}
          />
          <Button
            type="primary"
            icon={<SendOutlined />}
            onClick={handleSubmit}
            disabled={loading || !query.trim()}
            loading={loading}
          >
            Send
          </Button>
        </InputWrapper>
      </InputContainer>
    </ChatContainer>
  );
}
