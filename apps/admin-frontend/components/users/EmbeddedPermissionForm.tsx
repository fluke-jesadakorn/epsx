'use client';

import { useState, memo, useCallback } from 'react';
import { Clock, Plus, Calendar } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useToast } from '@/components/ui/use-toast';
import { format, addMinutes } from 'date-fns';

// Quick time selection options
const QUICK_TIME_OPTIONS = [
  { label: '1 Hour', minutes: 60 },
  { label: '4 Hours', minutes: 240 },
  { label: '8 Hours', minutes: 480 },
  { label: '1 Day', minutes: 1440 },
  { label: '3 Days', minutes: 4320 },
  { label: '1 Week', minutes: 10080 },
  { label: '1 Month', minutes: 43200 },
] as const;

// Common permission templates with embedded timestamps
const PERMISSION_TEMPLATES = [
  { 
    name: 'Temporary Analytics Access', 
    basePermission: 'epsx:analytics:view',
    defaultDuration: 240, // 4 hours
    description: 'Temporary access to analytics dashboard'
  },
  { 
    name: 'Premium Rankings (1 Day)', 
    basePermission: 'epsx:rankings:view:100',
    defaultDuration: 1440, // 1 day
    description: 'Temporary access to top 100 rankings'
  },
  { 
    name: 'Admin Dashboard Access', 
    basePermission: 'admin:dashboard:view',
    defaultDuration: 480, // 8 hours
    description: 'Temporary admin dashboard access'
  },
] as const;

interface EmbeddedPermissionData {
  basePermission: string;
  expiryTimestamp: number;
  reason?: string;
}

interface EmbeddedPermissionFormProps {
  userId: string;
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (data: EmbeddedPermissionData) => Promise<void>;
  loading?: boolean;
}

function EmbeddedPermissionForm({
  userId,
  isOpen,
  onClose,
  onSubmit,
  loading = false
}: EmbeddedPermissionFormProps) {
  const { toast } = useToast();
  
  const [formData, setFormData] = useState<EmbeddedPermissionData>({
    basePermission: '',
    expiryTimestamp: 0,
    reason: '',
  });
  
  const [selectedTemplate, setSelectedTemplate] = useState<string>('');
  const [quickTimeSelection, setQuickTimeSelection] = useState<string>('240'); // 4 hours default
  const [customDateTime, setCustomDateTime] = useState<string>('');
  const [timeInputMode, setTimeInputMode] = useState<'quick' | 'custom'>('quick');

  const handleTemplateSelect = useCallback((templateName: string) => {
    const template = PERMISSION_TEMPLATES.find(t => t.name === templateName);
    if (template) {
      setFormData(prev => ({
        ...prev,
        basePermission: template.basePermission,
      }));
      setQuickTimeSelection(template.defaultDuration.toString());
      setSelectedTemplate(templateName);
    }
  }, []);

  const calculateExpiryTimestamp = useCallback(() => {
    if (timeInputMode === 'quick') {
      const minutes = parseInt(quickTimeSelection);
      return Math.floor(addMinutes(new Date(), minutes).getTime() / 1000);
    } else if (customDateTime) {
      return Math.floor(new Date(customDateTime).getTime() / 1000);
    }
    return 0;
  }, [timeInputMode, quickTimeSelection, customDateTime]);

  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!formData.basePermission) {
      toast({
        title: 'Validation Error',
        description: 'Please enter a base permission',
        variant: 'destructive',
      });
      return;
    }

    const expiryTimestamp = calculateExpiryTimestamp();
    if (expiryTimestamp <= Math.floor(Date.now() / 1000)) {
      toast({
        title: 'Validation Error',
        description: 'Expiry time must be in the future',
        variant: 'destructive',
      });
      return;
    }

    const submissionData = {
      ...formData,
      expiryTimestamp,
    };

    try {
      await onSubmit(submissionData);
      
      // Reset form
      setFormData({
        basePermission: '',
        expiryTimestamp: 0,
        reason: '',
      });
      setSelectedTemplate('');
      setQuickTimeSelection('240');
      setCustomDateTime('');
      setTimeInputMode('quick');
    } catch (error) {
      // Error handling is done in parent component
    }
  }, [formData, calculateExpiryTimestamp, onSubmit, toast]);

  const previewExpiry = calculateExpiryTimestamp();
  const previewDate = previewExpiry > 0 ? new Date(previewExpiry * 1000) : null;

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Clock className="h-5 w-5" />
            Grant Embedded Timestamp Permission
          </DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Permission Templates */}
          <div className="space-y-3">
            <Label className="text-sm font-medium">Quick Templates</Label>
            <div className="grid grid-cols-1 gap-2">
              {PERMISSION_TEMPLATES.map((template) => (
                <Card 
                  key={template.name}
                  className={`cursor-pointer border transition-colors ${
                    selectedTemplate === template.name 
                      ? 'border-primary bg-primary/5' 
                      : 'border-border hover:border-primary/50'
                  }`}
                  onClick={() => handleTemplateSelect(template.name)}
                >
                  <CardContent className="p-3">
                    <div className="flex justify-between items-start">
                      <div>
                        <p className="font-medium text-sm">{template.name}</p>
                        <p className="text-xs text-muted-foreground">{template.description}</p>
                        <p className="text-xs font-mono text-primary mt-1">{template.basePermission}</p>
                      </div>
                      <div className="text-xs text-muted-foreground">
                        {template.defaultDuration >= 1440 
                          ? `${Math.floor(template.defaultDuration / 1440)}d`
                          : `${Math.floor(template.defaultDuration / 60)}h`
                        }
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>

          {/* Base Permission */}
          <div className="space-y-2">
            <Label htmlFor="basePermission" className="text-sm font-medium">
              Base Permission
            </Label>
            <Input
              id="basePermission"
              placeholder="e.g., epsx:analytics:view"
              value={formData.basePermission}
              onChange={(e) => setFormData(prev => ({ ...prev, basePermission: e.target.value }))}
              className="font-mono text-sm"
              required
            />
          </div>

          {/* Time Selection Mode */}
          <div className="space-y-3">
            <Label className="text-sm font-medium">Expiry Time</Label>
            <div className="flex gap-2">
              <Button
                type="button"
                variant={timeInputMode === 'quick' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setTimeInputMode('quick')}
              >
                Quick Select
              </Button>
              <Button
                type="button"
                variant={timeInputMode === 'custom' ? 'default' : 'outline'}
                size="sm"
                onClick={() => setTimeInputMode('custom')}
              >
                <Calendar className="h-4 w-4 mr-1" />
                Custom Date
              </Button>
            </div>

            {timeInputMode === 'quick' ? (
              <Select value={quickTimeSelection} onValueChange={setQuickTimeSelection}>
                <SelectTrigger>
                  <SelectValue placeholder="Select duration" />
                </SelectTrigger>
                <SelectContent>
                  {QUICK_TIME_OPTIONS.map((option) => (
                    <SelectItem key={option.minutes} value={option.minutes.toString()}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            ) : (
              <Input
                type="datetime-local"
                value={customDateTime}
                onChange={(e) => setCustomDateTime(e.target.value)}
                min={format(new Date(), "yyyy-MM-dd'T'HH:mm")}
                required={timeInputMode === 'custom'}
              />
            )}
          </div>

          {/* Preview */}
          {previewDate && (
            <Card className="border-dashed">
              <CardContent className="p-3">
                <div className="text-sm">
                  <p className="text-muted-foreground">Permission Preview:</p>
                  <p className="font-mono text-primary break-all">
                    {formData.basePermission}:{Math.floor(previewExpiry)}
                  </p>
                  <p className="text-xs text-muted-foreground mt-1">
                    Expires: {format(previewDate, 'PPpp')}
                  </p>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Reason */}
          <div className="space-y-2">
            <Label htmlFor="reason" className="text-sm font-medium">
              Reason (Optional)
            </Label>
            <Textarea
              id="reason"
              placeholder="Brief explanation for granting this permission..."
              value={formData.reason}
              onChange={(e) => setFormData(prev => ({ ...prev, reason: e.target.value }))}
              rows={3}
            />
          </div>

          {/* Actions */}
          <div className="flex gap-2 pt-4">
            <Button
              type="submit"
              disabled={loading}
              className="flex-1"
            >
              <Plus className="h-4 w-4 mr-2" />
              {loading ? 'Creating...' : 'Grant Permission'}
            </Button>
            <Button
              type="button"
              variant="outline"
              onClick={onClose}
            >
              Cancel
            </Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
}

export default memo(EmbeddedPermissionForm);