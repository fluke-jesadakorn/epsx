'use client';

import { useState, useTransition } from 'react';
import { Send, Loader2 } from 'lucide-react';
import { toast } from 'sonner';
import { sendNotification, sendBroadcastNotification } from '@/app/actions/admin-server';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

interface NotificationFormData {
  title: string;
  message: string;
  target: 'broadcast' | 'user';
  userEmail?: string;
  priority: 'normal' | 'high';
}

export function NotificationSendForm() {
  const [isPending, startTransition] = useTransition();
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  async function handleSendNotification(formData: FormData) {
    const data: NotificationFormData = {
      title: formData.get('title') as string,
      message: formData.get('message') as string,
      target: formData.get('target') as 'broadcast' | 'user',
      userEmail: formData.get('userEmail') as string,
      priority: formData.get('priority') as 'normal' | 'high',
    };

    if (!data.title.trim() || !data.message.trim()) {
      setError('Title and message are required');
      return;
    }

    if (data.target === 'user' && !data.userEmail?.trim()) {
      setError('User email is required for user-specific notifications');
      return;
    }

    startTransition(async () => {
      setError(null);
      setSuccess(null);

      try {
        let result;
        if (data.target === 'broadcast') {
          result = await sendBroadcastNotification(data.title, data.message, data.priority);
        } else {
          result = await sendNotification(data.userEmail!, data.title, data.message, data.priority);
        }

        if (result.success) {
          setSuccess(`Notification sent successfully!`);
          toast.success('Notification sent successfully');
          
          // Reset form
          const form = document.getElementById('notification-form') as HTMLFormElement;
          form?.reset();
        } else {
          setError(result.error || 'Failed to send notification');
          toast.error(result.error || 'Failed to send notification');
        }
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'An unexpected error occurred';
        setError(errorMessage);
        toast.error(errorMessage);
      }
    });
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Send className="h-5 w-5" />
          Send Notification
        </CardTitle>
      </CardHeader>
      <CardContent>
        <form id="notification-form" action={handleSendNotification} className="space-y-4">
          {error && (
            <div className="p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md text-red-800 dark:text-red-200 text-sm">
              {error}
            </div>
          )}
          
          {success && (
            <div className="p-3 bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-md text-green-800 dark:text-green-200 text-sm">
              {success}
            </div>
          )}

          <div>
            <label htmlFor="title" className="block text-sm font-medium mb-2">
              Title *
            </label>
            <Input
              id="title"
              name="title"
              placeholder="Notification title"
              required
              disabled={isPending}
            />
          </div>

          <div>
            <label htmlFor="message" className="block text-sm font-medium mb-2">
              Message *
            </label>
            <Textarea
              id="message"
              name="message"
              rows={4}
              placeholder="Notification message"
              required
              disabled={isPending}
            />
          </div>

          <div>
            <label htmlFor="target" className="block text-sm font-medium mb-2">
              Target
            </label>
            <Select name="target" defaultValue="broadcast" disabled={isPending}>
              <SelectTrigger>
                <SelectValue placeholder="Select target" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="broadcast">All Users (Broadcast)</SelectItem>
                <SelectItem value="user">Specific User</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div>
            <label htmlFor="userEmail" className="block text-sm font-medium mb-2">
              User Email (for specific user notifications)
            </label>
            <Input
              id="userEmail"
              name="userEmail"
              type="email"
              placeholder="user@example.com"
              disabled={isPending}
            />
          </div>

          <div>
            <label htmlFor="priority" className="block text-sm font-medium mb-2">
              Priority
            </label>
            <Select name="priority" defaultValue="normal" disabled={isPending}>
              <SelectTrigger>
                <SelectValue placeholder="Select priority" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="normal">Normal</SelectItem>
                <SelectItem value="high">High</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <Button 
            type="submit" 
            disabled={isPending}
            className="w-full"
          >
            {isPending ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Sending...
              </>
            ) : (
              <>
                <Send className="mr-2 h-4 w-4" />
                Send Notification
              </>
            )}
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}