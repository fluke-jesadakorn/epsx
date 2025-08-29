'use client';

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { cn } from '@/lib/utils';
import {
  CheckCircle,
  XCircle,
  VolumeX,
  MessageSquare,
  Clock,
  AlertTriangle,
  User,
  Flag,
  Eye,
  Send
} from 'lucide-react';

interface SecurityAlert {
  id: string;
  title: string;
  description: string;
  severity: 'low' | 'medium' | 'high' | 'critical';
  category: 'authentication' | 'access_control' | 'network' | 'system' | 'data_breach';
  status: 'active' | 'acknowledged' | 'resolved' | 'muted';
  createdAt: Date;
  updatedAt: Date;
  acknowledgedBy?: string;
  acknowledgedAt?: Date;
  source: string;
  affectedResources: string[];
  recommendations: string[];
  riskScore: number;
}

interface AlertAcknowledgmentProps {
  alert: SecurityAlert;
  onAcknowledge: (alertId: string, comment?: string) => Promise<void>;
  onResolve: (alertId: string, resolution?: string) => Promise<void>;
  onVolumeX: (alertId: string, duration?: string) => Promise<void>;
}

export function AlertAcknowledgment({ 
  alert, 
  onAcknowledge, 
  onResolve, 
  onVolumeX 
}: AlertAcknowledgmentProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [showAckDialog, setShowAckDialog] = useState(false);
  const [showResolveDialog, setShowResolveDialog] = useState(false);
  const [showVolumeXDialog, setShowVolumeXDialog] = useState(false);
  const [ackComment, setAckComment] = useState('');
  const [resolution, setResolution] = useState('');
  const [muteDuration, setVolumeXDuration] = useState('1h');

  const handleAcknowledge = async () => {
    setIsLoading(true);
    try {
      await onAcknowledge(alert.id, ackComment);
      setShowAckDialog(false);
      setAckComment('');
    } catch (error) {
      // Error handling would be done by parent component
    } finally {
      setIsLoading(false);
    }
  };

  const handleResolve = async () => {
    setIsLoading(true);
    try {
      await onResolve(alert.id, resolution);
      setShowResolveDialog(false);
      setResolution('');
    } catch (error) {
      // Error handling would be done by parent component
    } finally {
      setIsLoading(false);
    }
  };

  const handleVolumeX = async () => {
    setIsLoading(true);
    try {
      await onVolumeX(alert.id, muteDuration);
      setShowVolumeXDialog(false);
      setVolumeXDuration('1h');
    } catch (error) {
      // Error handling would be done by parent component
    } finally {
      setIsLoading(false);
    }
  };

  const getStatusDisplay = () => {
    switch (alert.status) {
      case 'active':
        return (
          <div className="flex items-center gap-2">
            <div className="flex gap-1">
              {/* Acknowledge Button */}
              <Dialog open={showAckDialog} onOpenChange={setShowAckDialog}>
                <DialogTrigger asChild>
                  <Button size="sm" variant="outline">
                    <CheckCircle className="w-4 h-4 mr-1" />
                    Acknowledge
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Acknowledge Alert</DialogTitle>
                  </DialogHeader>
                  <div className="space-y-4">
                    <div>
                      <h4 className="font-semibold mb-2">{alert.title}</h4>
                      <p className="text-sm text-muted-foreground">{alert.description}</p>
                    </div>
                    
                    <div>
                      <Label>Comment (optional)</Label>
                      <Textarea
                        placeholder="Add a comment about your acknowledgment..."
                        value={ackComment}
                        onChange={(e) => setAckComment(e.target.value)}
                        rows={3}
                      />
                    </div>
                    
                    <div className="flex items-center gap-2">
                      <Button onClick={handleAcknowledge} disabled={isLoading}>
                        <CheckCircle className="w-4 h-4 mr-2" />
                        {isLoading ? 'Acknowledging...' : 'Acknowledge Alert'}
                      </Button>
                      <Button variant="outline" onClick={() => setShowAckDialog(false)}>
                        Cancel
                      </Button>
                    </div>
                  </div>
                </DialogContent>
              </Dialog>

              {/* Resolve Button */}
              <Dialog open={showResolveDialog} onOpenChange={setShowResolveDialog}>
                <DialogTrigger asChild>
                  <Button size="sm" variant="default">
                    <XCircle className="w-4 h-4 mr-1" />
                    Resolve
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Resolve Alert</DialogTitle>
                  </DialogHeader>
                  <div className="space-y-4">
                    <div>
                      <h4 className="font-semibold mb-2">{alert.title}</h4>
                      <p className="text-sm text-muted-foreground">{alert.description}</p>
                    </div>
                    
                    <div>
                      <Label>Resolution Details</Label>
                      <Textarea
                        placeholder="Describe how this alert was resolved..."
                        value={resolution}
                        onChange={(e) => setResolution(e.target.value)}
                        rows={4}
                      />
                    </div>
                    
                    <div className="flex items-center gap-2">
                      <Button onClick={handleResolve} disabled={isLoading}>
                        <XCircle className="w-4 h-4 mr-2" />
                        {isLoading ? 'Resolving...' : 'Resolve Alert'}
                      </Button>
                      <Button variant="outline" onClick={() => setShowResolveDialog(false)}>
                        Cancel
                      </Button>
                    </div>
                  </div>
                </DialogContent>
              </Dialog>

              {/* VolumeX Button */}
              <Dialog open={showVolumeXDialog} onOpenChange={setShowVolumeXDialog}>
                <DialogTrigger asChild>
                  <Button size="sm" variant="ghost">
                    <VolumeX className="w-4 h-4" />
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>VolumeX Alert</DialogTitle>
                  </DialogHeader>
                  <div className="space-y-4">
                    <div>
                      <h4 className="font-semibold mb-2">{alert.title}</h4>
                      <p className="text-sm text-muted-foreground">
                        Muting will suppress notifications for this alert temporarily.
                      </p>
                    </div>
                    
                    <div>
                      <Label>VolumeX Duration</Label>
                      <Select value={muteDuration} onValueChange={setVolumeXDuration}>
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="1h">1 hour</SelectItem>
                          <SelectItem value="6h">6 hours</SelectItem>
                          <SelectItem value="24h">24 hours</SelectItem>
                          <SelectItem value="7d">7 days</SelectItem>
                          <SelectItem value="30d">30 days</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    
                    <div className="flex items-center gap-2">
                      <Button onClick={handleVolumeX} disabled={isLoading}>
                        <VolumeX className="w-4 h-4 mr-2" />
                        {isLoading ? 'Muting...' : 'VolumeX Alert'}
                      </Button>
                      <Button variant="outline" onClick={() => setShowVolumeXDialog(false)}>
                        Cancel
                      </Button>
                    </div>
                  </div>
                </DialogContent>
              </Dialog>
            </div>
          </div>
        );
        
      case 'acknowledged':
        return (
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="text-yellow-600 bg-yellow-50 border-yellow-200">
              <CheckCircle className="w-3 h-3 mr-1" />
              Acknowledged
            </Badge>
            {alert.acknowledgedBy && (
              <div className="text-xs text-muted-foreground flex items-center gap-1">
                <User className="w-3 h-3" />
                {alert.acknowledgedBy}
              </div>
            )}
            {alert.acknowledgedAt && (
              <div className="text-xs text-muted-foreground flex items-center gap-1">
                <Clock className="w-3 h-3" />
                {alert.acknowledgedAt.toLocaleString()}
              </div>
            )}
            <Button size="sm" variant="default" onClick={() => setShowResolveDialog(true)}>
              <XCircle className="w-4 h-4 mr-1" />
              Resolve
            </Button>
          </div>
        );
        
      case 'resolved':
        return (
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="text-green-600 bg-green-50 border-green-200">
              <XCircle className="w-3 h-3 mr-1" />
              Resolved
            </Badge>
            <div className="text-xs text-muted-foreground flex items-center gap-1">
              <Clock className="w-3 h-3" />
              {alert.updatedAt.toLocaleString()}
            </div>
            <Button size="sm" variant="ghost">
              <Eye className="w-4 h-4" />
            </Button>
          </div>
        );
        
      case 'muted':
        return (
          <div className="flex items-center gap-2">
            <Badge variant="outline" className="text-gray-600 bg-gray-50 border-gray-200">
              <VolumeX className="w-3 h-3 mr-1" />
              VolumeXd
            </Badge>
            <div className="text-xs text-muted-foreground flex items-center gap-1">
              <Clock className="w-3 h-3" />
              {alert.updatedAt.toLocaleString()}
            </div>
            <Button 
              size="sm" 
              variant="outline"
              onClick={() => onAcknowledge(alert.id)}
            >
              <CheckCircle className="w-4 h-4 mr-1" />
              Unmute
            </Button>
          </div>
        );
        
      default:
        return null;
    }
  };

  // Quick action buttons for different alert severities
  const getQuickActions = () => {
    if (alert.status !== 'active') return null;

    const quickActions = [];

    // Critical alerts get immediate action buttons
    if (alert.severity === 'critical') {
      quickActions.push(
        <Button 
          key="emergency" 
          size="sm" 
          variant="destructive"
          onClick={() => setShowResolveDialog(true)}
        >
          <AlertTriangle className="w-4 h-4 mr-1" />
          Emergency Response
        </Button>
      );
    }

    // High priority alerts get escalation button
    if (alert.severity === 'high' || alert.severity === 'critical') {
      quickActions.push(
        <Button key="escalate" size="sm" variant="outline">
          <Flag className="w-4 h-4 mr-1" />
          Escalate
        </Button>
      );
    }

    return quickActions.length > 0 ? (
      <div className="flex gap-1 ml-2">
        {quickActions}
      </div>
    ) : null;
  };

  return (
    <div className="flex flex-col gap-2">
      {getStatusDisplay()}
      {getQuickActions()}
    </div>
  );
}