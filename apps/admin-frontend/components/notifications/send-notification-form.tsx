'use client';

import {
  AlertCircle,
  CreditCard,
  ExternalLink,
  Image as ImageIcon,
  Key,
  MessageSquare,
  Send,
  Settings,
  Shield,
  User,
  Users,
  Zap
} from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import type { NotificationPriority, NotificationType } from '@/shared/api/notifications';
import { createNotificationsClient } from '@/shared/api/notifications';
import { createAdminApiClient } from '@/shared/utils/api-client';

interface SendNotificationFormProps {
  onSuccess?: () => void;
  onCancel?: () => void;
}

interface RecipientSelectorProps {
  value: 'specific' | 'broadcast';
  onChange: (v: 'specific' | 'broadcast') => void;
}

function RecipientSelector({ value, onChange }: RecipientSelectorProps) {
  return (
    <div className="space-y-4">
      <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">Transmission Logic</label>
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        <button
          type="button"
          onClick={() => onChange('specific')}
          className={`flex items-center gap-6 p-6 rounded-xl border transition-all ${value === 'specific' ? 'bg-[#1fc7d4]/10 border-[#1fc7d4] shadow-[0_0_20px_rgba(31,199,212,0.1)]' : 'bg-muted/30 border-border/40 hover:bg-muted/50'}`}
        >
          <div className={`w-12 h-12 rounded-xl flex items-center justify-center ${value === 'specific' ? 'bg-[#1fc7d4] text-white' : 'bg-muted/50 text-muted-foreground/30'}`}>
            <User className="h-6 w-6" />
          </div>
          <div className="text-left">
            <div className="font-black text-foreground uppercase tracking-tight text-sm">Targeted Client</div>
            <div className="text-[10px] font-bold text-muted-foreground uppercase opacity-50">Single Node Access</div>
          </div>
        </button>
        <button
          type="button"
          onClick={() => onChange('broadcast')}
          className={`flex items-center gap-6 p-6 rounded-xl border transition-all ${value === 'broadcast' ? 'bg-amber-500/10 border-amber-500 shadow-[0_0_20px_rgba(245,158,11,0.1)]' : 'bg-muted/30 border-border/40 hover:bg-muted/50'}`}
        >
          <div className={`w-12 h-12 rounded-xl flex items-center justify-center ${value === 'broadcast' ? 'bg-amber-500 text-white' : 'bg-muted/50 text-muted-foreground/30'}`}>
            <Users className="h-6 w-6" />
          </div>
          <div className="text-left">
            <div className="font-black text-foreground uppercase tracking-tight text-sm">Global Broadcast</div>
            <div className="text-[10px] font-bold text-muted-foreground uppercase opacity-50">Network-wide Delivery</div>
          </div>
        </button>
      </div>
    </div>
  );
}

interface TypeSelectorProps {
  value: NotificationType;
  onChange: (v: NotificationType) => void;
}

function TypeSelector({ value, onChange }: TypeSelectorProps) {
  return (
    <div className="space-y-3">
      <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">Classification</label>
      <Select value={value} onValueChange={(v) => onChange(v as NotificationType)}>
        <SelectTrigger className="h-14 bg-muted/30 border-border/40 rounded-2xl px-6 text-sm font-black uppercase tracking-widest hover:bg-muted/50">
          <SelectValue />
        </SelectTrigger>
        <SelectContent className="bg-card border-border/40 rounded-2xl overflow-hidden backdrop-blur-3xl">
          <SelectItem value="system" className="p-4 focus:bg-[#1fc7d4]/10">
            <div className="flex items-center font-black uppercase tracking-widest text-[10px]"><Settings className="w-4 h-4 mr-3 text-cyan-400" /> System Alert</div>
          </SelectItem>
          <SelectItem value="security" className="p-4 focus:bg-[#1fc7d4]/10">
            <div className="flex items-center font-black uppercase tracking-widest text-[10px]"><Shield className="w-4 h-4 mr-3 text-red-400" /> Security Event</div>
          </SelectItem>
          <SelectItem value="permission" className="p-4 focus:bg-[#1fc7d4]/10">
            <div className="flex items-center font-black uppercase tracking-widest text-[10px]"><Key className="w-4 h-4 mr-3 text-amber-400" /> Permission Auth</div>
          </SelectItem>
          <SelectItem value="payment" className="p-4 focus:bg-[#1fc7d4]/10">
            <div className="flex items-center font-black uppercase tracking-widest text-[10px]"><CreditCard className="w-4 h-4 mr-3 text-green-400" /> Payment Transaction</div>
          </SelectItem>
          <SelectItem value="general" className="p-4 focus:bg-[#1fc7d4]/10">
            <div className="flex items-center font-black uppercase tracking-widest text-[10px]"><MessageSquare className="w-4 h-4 mr-3 text-purple-400" /> General Message</div>
          </SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}

interface PrioritySelectorProps {
  value: NotificationPriority;
  onChange: (v: NotificationPriority) => void;
}

function PrioritySelector({ value, onChange }: PrioritySelectorProps) {
  return (
    <div className="space-y-3">
      <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">Priority Vector</label>
      <Select value={value} onValueChange={(v) => onChange(v as NotificationPriority)}>
        <SelectTrigger className="h-14 bg-muted/30 border-border/40 rounded-2xl px-6 text-sm font-black uppercase tracking-widest hover:bg-muted/50">
          <SelectValue />
        </SelectTrigger>
        <SelectContent className="bg-card border-border/40 rounded-2xl overflow-hidden backdrop-blur-3xl">
          <SelectItem value="low" className="p-4 focus:bg-[#1fc7d4]/10 font-black uppercase tracking-widest text-[10px] text-green-400">Low Clearance</SelectItem>
          <SelectItem value="normal" className="p-4 focus:bg-[#1fc7d4]/10 font-black uppercase tracking-widest text-[10px] text-cyan-400">Normal Operation</SelectItem>
          <SelectItem value="high" className="p-4 focus:bg-[#1fc7d4]/10 font-black uppercase tracking-widest text-[10px] text-amber-400">High Priority</SelectItem>
          <SelectItem value="critical" className="p-4 focus:bg-[#1fc7d4]/10 font-black uppercase tracking-widest text-[10px] text-red-400">Critical Override</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
}

/**
 * Modernized Send Notification Form with PancakeSwap aesthetic
 */
export function SendNotificationForm({ onSuccess, onCancel }: SendNotificationFormProps) {
  const [recipientType, setRecipientType] = useState<'specific' | 'broadcast'>('specific');
  const [walletAddress, setWalletAddress] = useState('');
  const [notificationType, setNotificationType] = useState<NotificationType>('system');
  const [priority, setPriority] = useState<NotificationPriority>('normal');
  const [title, setTitle] = useState('');
  const [message, setMessage] = useState('');
  const [actionUrl, setActionUrl] = useState('');
  const [imageUrl, setImageUrl] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (title.trim() === '') { setError('Title is mandatory for identification'); return; }
    if (message.trim() === '') { setError('Message payload cannot be empty'); return; }
    if (recipientType === 'specific' && walletAddress.trim() === '') { setError('Protocol requires a destination wallet address'); return; }

    setLoading(true);
    setError(null);
    void (async () => {
      try {
        const client = createNotificationsClient(createAdminApiClient());
        await client.sendNotification({
          ...(recipientType === 'specific' ? { recipient_wallet_address: walletAddress } : { broadcast: true }),
          notification_type: notificationType,
          priority,
          title,
          message,
          ...(actionUrl !== '' && { action_url: actionUrl }),
          ...(imageUrl !== '' && { image_url: imageUrl }),
        });
        setRecipientType('specific');
        setWalletAddress('');
        setTitle('');
        setMessage('');
        setActionUrl('');
        setImageUrl('');
        onSuccess?.();
      } catch (err: unknown) {
        setError(err instanceof Error ? err.message : 'Signal transmission failed.');
      } finally {
        setLoading(false);
      }
    })();
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-10">
      {error !== null && (
        <div className="flex items-center gap-4 p-6 text-sm font-bold text-red-400 bg-red-500/10 border border-red-500/20 rounded-[20px] animate-in slide-in-from-top-4">
          <AlertCircle className="h-5 w-5 flex-shrink-0" />
          {error}
        </div>
      )}

      <RecipientSelector value={recipientType} onChange={setRecipientType} />

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-x-10 gap-y-8">
        {recipientType === 'specific' && (
          <div className="space-y-3 lg:col-span-2">
            <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">Destination Node</label>
            <div className="relative group">
              <Input
                type="text"
                placeholder="0x..."
                value={walletAddress}
                onChange={(e) => setWalletAddress(e.target.value)}
                className="h-14 bg-muted/30 border-border/40 rounded-2xl px-6 font-mono text-sm group-hover:bg-muted/50 focus:border-[#1fc7d4]/50 transition-all"
              />
              <div className="absolute right-4 top-1/2 -translate-y-1/2 opacity-20 pointer-events-none font-black text-[10px] uppercase">Required</div>
            </div>
          </div>
        )}

        <TypeSelector value={notificationType} onChange={setNotificationType} />
        <PrioritySelector value={priority} onChange={setPriority} />

        <div className="space-y-3 lg:col-span-2">
          <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">Subject Heading</label>
          <Input value={title} onChange={(e) => setTitle(e.target.value)} placeholder="Payload designation..." className="h-14 bg-muted/30 border-border/40 rounded-2xl px-6 font-bold text-sm tracking-tight hover:bg-muted/50 focus:border-[#1fc7d4]/50 transition-all" />
        </div>

        <div className="space-y-3 lg:col-span-2">
          <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2">Message Payload</label>
          <Textarea value={message} onChange={(e) => setMessage(e.target.value)} placeholder="Enter transmission data..." rows={5} className="bg-muted/30 border-border/40 rounded-2xl p-6 font-bold text-sm tracking-tight hover:bg-muted/50 focus:border-[#1fc7d4]/50 transition-all resize-none" />
        </div>

        <div className="space-y-3">
          <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2 inline-flex items-center">
            Action URL <ExternalLink className="w-2.5 h-2.5 ml-2 opacity-30" />
          </label>
          <Input type="url" value={actionUrl} onChange={(e) => setActionUrl(e.target.value)} placeholder="https://..." className="h-12 bg-muted/30 border-border/40 rounded-2xl px-5 text-xs font-bold hover:bg-muted/50 focus:border-[#1fc7d4]/50 transition-all" />
        </div>

        <div className="space-y-3">
          <label className="text-[10px] font-black uppercase tracking-[0.2em] text-muted-foreground ml-2 inline-flex items-center">
            Asset URL <ImageIcon className="w-2.5 h-2.5 ml-2 opacity-30" />
          </label>
          <Input type="url" value={imageUrl} onChange={(e) => setImageUrl(e.target.value)} placeholder="https://..." className="h-12 bg-muted/30 border-border/40 rounded-2xl px-5 text-xs font-bold hover:bg-muted/50 focus:border-[#1fc7d4]/50 transition-all" />
        </div>
      </div>

      <div className="flex items-center gap-6 pt-10 border-t border-border/20">
        {onCancel !== undefined && (
          <Button type="button" variant="ghost" onClick={onCancel} disabled={loading} className="flex-1 py-7 rounded-2xl text-[10px] font-black uppercase tracking-[0.2em] opacity-40 hover:opacity-100 transition-all">
            Abort
          </Button>
        )}
        <Button type="submit" disabled={loading} className="flex-[2] py-7 rounded-xl bg-gradient-to-r from-[#7645d9] to-[#5a33b8] hover:opacity-90 text-white font-black uppercase tracking-[0.2em] shadow-lg active:scale-95 transition-all">
          {loading ? (
            <div className="flex items-center justify-center"><Zap className="h-4 w-4 mr-3 animate-pulse text-white" />Transmitting...</div>
          ) : (
            <div className="flex items-center justify-center"><Send className="h-4 w-4 mr-3" />Execute Broadcast</div>
          )}
        </Button>
      </div>
    </form>
  );
}
