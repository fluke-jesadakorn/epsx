'use client';

import { useState, useEffect, useCallback } from 'react';
import type { ChatTopic, ChatConversation, ChatMessage } from '@/shared/api/chat';
import { ChatTopicSelector } from './chat-topic-selector';
import { ChatConversationList } from './chat-conversation-list';
import { ChatHeader } from './chat-header';
import { ChatMessageList } from './chat-message-list';
import { ChatInput } from './chat-input';
import { X, MessageCircle } from 'lucide-react';
import {
  getTopicsAction,
  listConversationsAction,
  createConversationAction,
  getMessagesAction,
  sendMessageAction,
  updateConversationStatusAction,
  markConversationReadAction
} from '@/app/actions/chat';
import { useChatSSE } from '@/shared/hooks/use-chat-sse';
import type { ChatSSEEvent } from '@/shared/hooks/use-chat-sse';

type View = 'topics' | 'list' | 'conversation';

interface PanelProps {
  isOpen: boolean;
  onClose: () => void;
  walletAddr?: string;
}

export function ChatPanel({ isOpen, onClose, walletAddr }: PanelProps) {
  const [view, setView] = useState<View>('list');
  const [topics, setTopics] = useState<ChatTopic[]>([]);
  const [convos, setConvos] = useState<ChatConversation[]>([]);
  const [activeConvo, setActiveConvo] = useState<ChatConversation | null>(null);
  const [msgs, setMsgs] = useState<ChatMessage[]>([]);
  const [loading, setLoading] = useState(false);

  const loadData = useCallback(async () => {
    try {
      const [topicsData, convosData] = await Promise.all([getTopicsAction(), listConversationsAction()]);
      const t = Array.isArray(topicsData) ? topicsData : [];
      const c = Array.isArray(convosData) ? convosData : [];
      setTopics(t);
      setConvos(c);
      if (c.length === 0) {
        setView('topics');
      }
    } catch {
      setTopics([]);
      setConvos([]);
      setView('topics');
    }
  }, []);

  useEffect(() => {
    if (isOpen) {
      void loadData();
    }
  }, [isOpen, loadData]);

  const handleCreateConvo = useCallback(async (topicId: string, subject: string, message: string) => {
    setLoading(true);
    const convo = await createConversationAction(topicId, subject, message);
    if (convo) {
      setConvos((prev) => [convo, ...prev]);
      setActiveConvo(convo);
      const newMsgs = await getMessagesAction(convo.id);
      setMsgs(Array.isArray(newMsgs) ? newMsgs : []);
      setView('conversation');
    }
    setLoading(false);
  }, []);

  const handleSelectConvo = useCallback(async (id: string) => {
    setLoading(true);
    const convo = convos.find((c) => c.id === id);
    if (convo) {
      setActiveConvo(convo);
      const newMsgs = await getMessagesAction(id);
      setMsgs(Array.isArray(newMsgs) ? newMsgs : []);
      void markConversationReadAction(id);
      setView('conversation');
    }
    setLoading(false);
  }, [convos]);

  const handleSendMsg = useCallback(async (content: string) => {
    if (!activeConvo) {
      return;
    }
    const msg = await sendMessageAction(activeConvo.id, content);
    if (msg) {
      setMsgs((prev) => [...prev, msg]);
    }
  }, [activeConvo]);

  const handleResolve = useCallback(async () => {
    if (!activeConvo) {
      return;
    }
    const updated = await updateConversationStatusAction(activeConvo.id, 'resolved');
    if (updated) {
      setActiveConvo(updated);
      setConvos((prev) => prev.map((c) => (c.id === updated.id ? updated : c)));
    }
  }, [activeConvo]);

  const handleBack = useCallback(() => {
    setView('list');
    setActiveConvo(null);
    setMsgs([]);
  }, []);

  // SSE: real-time messages & status updates
  const handleSSE = useCallback((evt: ChatSSEEvent) => {
    if (evt.type === 'new_message' && evt.message) {
      if (activeConvo && evt.conversation_id === activeConvo.id) {
        setMsgs((prev) => prev.some((m) => m.id === evt.message?.id) ? prev : [...prev, evt.message as ChatMessage]);
      } else {
        // Update unread for other conversations
        setConvos((prev) => prev.map((c) =>
          c.id === evt.conversation_id ? { ...c, unread_user: c.unread_user + 1 } : c
        ));
      }
    }
    if (evt.type === 'status_changed' && evt.conversation) {
      setConvos((prev) => prev.map((c) => c.id === evt.conversation_id ? { ...c, status: evt.conversation?.status ?? c.status } : c));
      if (activeConvo?.id === evt.conversation_id) {
        setActiveConvo(evt.conversation);
      }
    }
  }, [activeConvo]);

  useChatSSE({ enabled: isOpen, mode: 'user', onEvent: handleSSE });

  return (
    <div
      className={`fixed bottom-2 right-2 md:bottom-24 md:right-6 w-[calc(100vw-1rem)] md:w-[400px] max-w-[400px] h-[calc(100vh-4rem)] md:h-[580px] max-h-[580px] bg-white dark:bg-slate-950 backdrop-blur-xl border border-slate-200 dark:border-white/8 rounded-3xl shadow-2xl shadow-black/10 flex flex-col z-50 overflow-hidden transition-all duration-300 origin-bottom-right ${
        isOpen ? 'scale-100 opacity-100 translate-y-0' : 'scale-95 opacity-0 translate-y-4 pointer-events-none'
      }`}
    >
      {view === 'topics' && (
        <>
          <PanelHeader onClose={onClose} title="New Conversation" />
          {convos.length > 0 && (
            <button
              onClick={() => setView('list')}
              className="mx-4 mt-1 text-xs text-blue-400 hover:text-blue-300 text-left transition-colors"
            >
              View existing conversations
            </button>
          )}
          <ChatTopicSelector topics={topics} onSelect={handleCreateConvo} compact />
        </>
      )}

      {view === 'list' && (
        <>
          <PanelHeader onClose={onClose} />
          <ChatConversationList convos={convos} onSelect={handleSelectConvo} onNew={() => setView('topics')} />
        </>
      )}

      {view === 'conversation' && activeConvo && (
        <>
          <ChatHeader
            subject={activeConvo.subject}
            status={activeConvo.status}
            onBack={handleBack}
            onClose={onClose}
            onResolve={handleResolve}
          />
          <ChatMessageList msgs={msgs} userAddr={walletAddr} />
          <ChatInput onSend={handleSendMsg} disabled={loading || activeConvo.status === 'closed'} />
        </>
      )}
    </div>
  );
}

function PanelHeader({ onClose, title }: { onClose: () => void; title?: string }) {
  return (
    <div className="flex items-center justify-between px-4 py-3.5 border-b border-slate-200 dark:border-slate-700 bg-slate-50 dark:bg-slate-900">
      <div className="flex items-center gap-3">
        <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-blue-500 to-indigo-600 flex items-center justify-center shadow-sm shadow-blue-500/20">
          <MessageCircle className="w-4.5 h-4.5 text-white" />
        </div>
        <div>
          <h2 className="font-bold text-sm">{title ?? 'Support Chat'}</h2>
          <div className="flex items-center gap-1 mt-0.5">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-400" />
            <p className="text-[10px] text-muted-foreground leading-none">Online - replies within minutes</p>
          </div>
        </div>
      </div>
      <button
        onClick={onClose}
        className="w-8 h-8 rounded-lg hover:bg-muted flex items-center justify-center transition-colors"
      >
        <X className="w-4 h-4 text-muted-foreground" />
      </button>
    </div>
  );
}
