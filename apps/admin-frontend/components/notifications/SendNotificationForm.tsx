'use client';

import { Bell, Send, User, Users, AlertCircle } from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue
} from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { createNotificationsClient } from '@/shared/api/notifications';
import type { NotificationType, NotificationPriority } from '@/shared/api/notifications';
import { createAdminApiClient } from '@/shared/utils/api-client';

interface SendNotificationFormProps {
  onSuccess?: () => void;
  onCancel?: () => void;
}

/**
 *
 * @param root0
 * @param root0.onSuccess
 * @param root0.onCancel
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

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Validation
    if (!title.trim()) {
      setError('Title is required');
      return;
    }
    if (!message.trim()) {
      setError('Message is required');
      return;
    }
    if (recipientType === 'specific' && !walletAddress.trim()) {
      setError('Wallet address is required for specific recipient');
      return;
    }

    try {
      setLoading(true);
      setError(null);

      const client = createNotificationsClient(createAdminApiClient());

      const result = await client.sendNotification({
        ...(recipientType === 'specific'
          ? { recipient_wallet_address: walletAddress }
          : { broadcast: true }
        ),
        notification_type: notificationType,
        priority,
        title,
        message,
        ...(actionUrl && { action_url: actionUrl }),
        ...(imageUrl && { image_url: imageUrl }),
      });

      // Reset form
      setRecipientType('specific');
      setWalletAddress('');
      setTitle('');
      setMessage('');
      setActionUrl('');
      setImageUrl('');

      onSuccess?.();
    } catch (err) {
      // Extract error message from various error types
      let errorMessage = 'Failed to send notification';

      console.error('❌ Failed to send notification - Full error:', err);

      if (err instanceof Error) {
        errorMessage = err.message;
        console.error('Error message:', err.message);
        console.error('Error stack:', err.stack);
      } else if (err && typeof err === 'object') {
        // Handle API error objects with message property
        const apiErr = err as any;
        console.error('API error object:', {
          message: apiErr.message,
          error: apiErr.error,
          data: apiErr.data,
          status: apiErr.status,
          statusText: apiErr.statusText,
          response: apiErr.response,
          details: apiErr.details,
          fullError: apiErr
        });
        errorMessage = apiErr.message || apiErr.error || apiErr.statusText || errorMessage;

        // Check if it's a network error
        if (apiErr.message && apiErr.message.includes('fetch')) {
          errorMessage = 'Network error: Unable to connect to backend server';
        }
      }

      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {/* Error Alert */}
      {error && (
        <div className="flex items-center gap-2 p-4 text-sm text-red-800 bg-red-50 border border-red-200 rounded-lg dark:bg-red-900/20 dark:text-red-300 dark:border-red-800">
          <AlertCircle className="h-4 w-4" />
          {error}
        </div>
      )}

      {/* Recipient Type */}
      <div className="space-y-3">
        <Label>Recipient Type</Label>
        <div className="grid grid-cols-2 gap-3">
          <button
            type="button"
            onClick={() => setRecipientType('specific')}
            className={`flex items-center gap-3 p-3 border rounded-lg hover:bg-slate-50 dark:hover:bg-slate-800 ${
              recipientType === 'specific'
                ? 'border-blue-500 bg-blue-50 dark:bg-blue-950 dark:border-blue-600'
                : 'border-slate-200 dark:border-slate-700'
            }`}
          >
            <User className="h-4 w-4 text-blue-500" />
            <div className="text-left">
              <div className="font-medium text-sm">Specific Wallet</div>
              <div className="text-xs text-slate-500 dark:text-slate-400">Send to single wallet</div>
            </div>
          </button>

          <button
            type="button"
            onClick={() => setRecipientType('broadcast')}
            className={`flex items-center gap-3 p-3 border rounded-lg hover:bg-slate-50 dark:hover:bg-slate-800 ${
              recipientType === 'broadcast'
                ? 'border-orange-500 bg-orange-50 dark:bg-orange-950 dark:border-orange-600'
                : 'border-slate-200 dark:border-slate-700'
            }`}
          >
            <Users className="h-4 w-4 text-orange-500" />
            <div className="text-left">
              <div className="font-medium text-sm">Broadcast to All</div>
              <div className="text-xs text-slate-500 dark:text-slate-400">Send to all users</div>
            </div>
          </button>
        </div>
      </div>

      {/* Wallet Address Input (only for specific) */}
      {recipientType === 'specific' && (
        <div className="space-y-2">
          <Label htmlFor="walletAddress">Wallet Address *</Label>
          <Input
            id="walletAddress"
            type="text"
            placeholder="0x..."
            value={walletAddress}
            onChange={(e) => setWalletAddress(e.target.value)}
            required={recipientType === 'specific'}
            className="font-mono"
          />
        </div>
      )}

      {/* Notification Type */}
      <div className="space-y-2">
        <Label htmlFor="type">Notification Type *</Label>
        <Select value={notificationType} onValueChange={(v) => setNotificationType(v as NotificationType)}>
          <SelectTrigger id="type">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="system">⚙️ System</SelectItem>
            <SelectItem value="security">🔒 Security</SelectItem>
            <SelectItem value="permission">🔑 Permission</SelectItem>
            <SelectItem value="wallet_management">👥 Wallet Management</SelectItem>
            <SelectItem value="wallet">💼 Wallet</SelectItem>
            <SelectItem value="payment">💳 Payment</SelectItem>
            <SelectItem value="general">📬 General</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Priority */}
      <div className="space-y-2">
        <Label htmlFor="priority">Priority *</Label>
        <Select value={priority} onValueChange={(v) => setPriority(v as NotificationPriority)}>
          <SelectTrigger id="priority">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="low">🟢 Low</SelectItem>
            <SelectItem value="normal">🔵 Normal</SelectItem>
            <SelectItem value="high">🟠 High</SelectItem>
            <SelectItem value="critical">🔴 Critical</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Title */}
      <div className="space-y-2">
        <Label htmlFor="title">Title *</Label>
        <Input
          id="title"
          type="text"
          placeholder="Notification title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          required
        />
      </div>

      {/* Message */}
      <div className="space-y-2">
        <Label htmlFor="message">Message *</Label>
        <Textarea
          id="message"
          placeholder="Notification message"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          required
          rows={4}
        />
      </div>

      {/* Optional: Action URL */}
      <div className="space-y-2">
        <Label htmlFor="actionUrl">Action URL (Optional)</Label>
        <Input
          id="actionUrl"
          type="url"
          placeholder="https://..."
          value={actionUrl}
          onChange={(e) => setActionUrl(e.target.value)}
        />
        <p className="text-xs text-muted-foreground">
          URL to navigate when notification is clicked
        </p>
      </div>

      {/* Optional: Image URL */}
      <div className="space-y-2">
        <Label htmlFor="imageUrl">Image URL (Optional)</Label>
        <Input
          id="imageUrl"
          type="url"
          placeholder="https://..."
          value={imageUrl}
          onChange={(e) => setImageUrl(e.target.value)}
        />
        <p className="text-xs text-muted-foreground">
          Image to display with notification
        </p>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-3 pt-4 border-t">
        {onCancel && (
          <Button type="button" variant="outline" onClick={onCancel} disabled={loading}>
            Cancel
          </Button>
        )}
        <Button type="submit" disabled={loading} className="flex-1">
          {loading ? (
            <>
              <div className="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin mr-2" />
              Sending...
            </>
          ) : (
            <>
              <Send className="h-4 w-4 mr-2" />
              Send Notification
            </>
          )}
        </Button>
      </div>
    </form>
  );
}
