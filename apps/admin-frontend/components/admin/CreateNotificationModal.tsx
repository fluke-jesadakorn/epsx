'use client';

import React, { useState } from 'react';
import { X, Send, Users, User, Globe } from 'lucide-react';
import { apiClient } from '@/lib/api-client';
import type { NotificationCreateRequest } from '@/types';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Dialog } from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';

interface CreateNotificationModalProps {
  open: boolean;
  onClose: () => void;
  onSuccess?: () => void;
}

type DeliveryMode = 'single' | 'multiple' | 'broadcast';

export function CreateNotificationModal({ open, onClose, onSuccess }: CreateNotificationModalProps) {
  const [formData, setFormData] = useState<NotificationCreateRequest>({
    title: '',
    message: '',
    type: 'system',
    priority: 'medium',
    actionUrl: '',
    metadata: {}
  });
  
  const [deliveryMode, setDeliveryMode] = useState<DeliveryMode>('broadcast');
  const [targetUserId, setTargetUserId] = useState('');
  const [targetUserIds, setTargetUserIds] = useState<string[]>([]);
  const [newUserId, setNewUserId] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!formData.title.trim() || !formData.message.trim()) {
      setError('Title and message are required');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      let response;
      
      if (deliveryMode === 'broadcast') {
        // Broadcast to all users
        response = await apiClient.broadcastNotification({
          ...formData,
          allUsers: true
        });
      } else if (deliveryMode === 'multiple') {
        // Send to multiple specific users
        if (targetUserIds.length === 0) {
          setError('Please specify at least one user');
          setLoading(false);
          return;
        }
        response = await apiClient.broadcastNotification({
          ...formData,
          userIds: targetUserIds
        });
      } else {
        // Send to single user
        if (!targetUserId.trim()) {
          setError('Please specify a user ID');
          setLoading(false);
          return;
        }
        response = await apiClient.createNotification({
          ...formData,
          userId: targetUserId
        });
      }

      // Reset form
      setFormData({
        title: '',
        message: '',
        type: 'system',
        priority: 'medium',
        actionUrl: '',
        metadata: {}
      });
      setTargetUserId('');
      setTargetUserIds([]);
      setNewUserId('');
      setDeliveryMode('broadcast');
      
      onSuccess?.();
      onClose();
    } catch (err: any) {
      console.error('Failed to create notification:', err);
      setError(err?.response?.data?.error || 'Failed to create notification');
    } finally {
      setLoading(false);
    }
  };

  const addUserId = () => {
    if (newUserId.trim() && !targetUserIds.includes(newUserId.trim())) {
      setTargetUserIds([...targetUserIds, newUserId.trim()]);
      setNewUserId('');
    }
  };

  const removeUserId = (userId: string) => {
    setTargetUserIds(targetUserIds.filter(id => id !== userId));
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 bg-black/50 flex items-center justify-center p-4">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-auto">
        <div className="flex items-center justify-between p-6 border-b">
          <h2 className="text-xl font-semibold">Create Notification</h2>
          <Button variant="ghost" size="sm" onClick={onClose}>
            <X className="h-4 w-4" />
          </Button>
        </div>

        <form onSubmit={handleSubmit} className="p-6 space-y-6">
          {error && (
            <div className="p-4 bg-red-50 border border-red-200 rounded-md text-red-700">
              {error}
            </div>
          )}

          {/* Delivery Mode Selection */}
          <div>
            <Label className="text-base font-medium mb-3 block">Delivery Mode</Label>
            <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
              <button
                type="button"
                onClick={() => setDeliveryMode('single')}
                className={`p-4 border rounded-lg text-left transition-colors ${
                  deliveryMode === 'single' 
                    ? 'border-primary bg-primary/5' 
                    : 'border-border hover:border-primary/50'
                }`}
              >
                <div className="flex items-center gap-2 mb-2">
                  <User className="h-5 w-5" />
                  <span className="font-medium">Single User</span>
                </div>
                <p className="text-sm text-muted-foreground">
                  Send to one specific user
                </p>
              </button>

              <button
                type="button"
                onClick={() => setDeliveryMode('multiple')}
                className={`p-4 border rounded-lg text-left transition-colors ${
                  deliveryMode === 'multiple' 
                    ? 'border-primary bg-primary/5' 
                    : 'border-border hover:border-primary/50'
                }`}
              >
                <div className="flex items-center gap-2 mb-2">
                  <Users className="h-5 w-5" />
                  <span className="font-medium">Multiple Users</span>
                </div>
                <p className="text-sm text-muted-foreground">
                  Send to specific users
                </p>
              </button>

              <button
                type="button"
                onClick={() => setDeliveryMode('broadcast')}
                className={`p-4 border rounded-lg text-left transition-colors ${
                  deliveryMode === 'broadcast' 
                    ? 'border-primary bg-primary/5' 
                    : 'border-border hover:border-primary/50'
                }`}
              >
                <div className="flex items-center gap-2 mb-2">
                  <Globe className="h-5 w-5" />
                  <span className="font-medium">Broadcast</span>
                </div>
                <p className="text-sm text-muted-foreground">
                  Send to all users
                </p>
              </button>
            </div>
          </div>

          {/* Target Users */}
          {deliveryMode === 'single' && (
            <div>
              <Label htmlFor="targetUserId">Target User ID</Label>
              <Input
                id="targetUserId"
                value={targetUserId}
                onChange={(e) => setTargetUserId(e.target.value)}
                placeholder="Enter user ID"
                required
              />
            </div>
          )}

          {deliveryMode === 'multiple' && (
            <div>
              <Label>Target Users</Label>
              <div className="flex gap-2 mb-2">
                <Input
                  value={newUserId}
                  onChange={(e) => setNewUserId(e.target.value)}
                  placeholder="Enter user ID"
                  onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), addUserId())}
                />
                <Button type="button" onClick={addUserId} variant="outline">
                  Add
                </Button>
              </div>
              {targetUserIds.length > 0 && (
                <div className="flex flex-wrap gap-2">
                  {targetUserIds.map(userId => (
                    <Badge key={userId} variant="secondary" className="flex items-center gap-1">
                      {userId}
                      <button
                        type="button"
                        onClick={() => removeUserId(userId)}
                        className="ml-1 hover:text-destructive"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </Badge>
                  ))}
                </div>
              )}
            </div>
          )}

          {/* Notification Details */}
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <Label htmlFor="type">Type</Label>
              <select
                id="type"
                value={formData.type}
                onChange={(e) => setFormData({ ...formData, type: e.target.value as any })}
                className="w-full px-3 py-2 border border-input rounded-md bg-background"
                required
              >
                <option value="system">System</option>
                <option value="trading">Trading</option>
                <option value="security">Security</option>
                <option value="compliance">Compliance</option>
                <option value="account">Account</option>
                <option value="price_alert">Price Alert</option>
              </select>
            </div>

            <div>
              <Label htmlFor="priority">Priority</Label>
              <select
                id="priority"
                value={formData.priority}
                onChange={(e) => setFormData({ ...formData, priority: e.target.value as any })}
                className="w-full px-3 py-2 border border-input rounded-md bg-background"
                required
              >
                <option value="low">Low</option>
                <option value="medium">Medium</option>
                <option value="high">High</option>
                <option value="critical">Critical</option>
              </select>
            </div>
          </div>

          <div>
            <Label htmlFor="title">Title</Label>
            <Input
              id="title"
              value={formData.title}
              onChange={(e) => setFormData({ ...formData, title: e.target.value })}
              placeholder="Notification title"
              required
            />
          </div>

          <div>
            <Label htmlFor="message">Message</Label>
            <Textarea
              id="message"
              value={formData.message}
              onChange={(e) => setFormData({ ...formData, message: e.target.value })}
              placeholder="Notification message"
              rows={4}
              required
            />
          </div>

          <div>
            <Label htmlFor="actionUrl">Action URL (optional)</Label>
            <Input
              id="actionUrl"
              value={formData.actionUrl}
              onChange={(e) => setFormData({ ...formData, actionUrl: e.target.value })}
              placeholder="https://example.com"
              type="url"
            />
          </div>

          {/* Actions */}
          <div className="flex justify-end gap-3 pt-4 border-t">
            <Button type="button" variant="outline" onClick={onClose} disabled={loading}>
              Cancel
            </Button>
            <Button type="submit" disabled={loading} className="flex items-center gap-2">
              {loading ? (
                <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
              ) : (
                <Send className="h-4 w-4" />
              )}
              {deliveryMode === 'broadcast' ? 'Broadcast' : 'Send'} Notification
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}