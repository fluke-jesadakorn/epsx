'use client';

import { useState } from 'react';
import { 
  Shield, 
  Lock, 
  Crown, 
  User, 
  Mail, 
  MessageSquare, 
  ExternalLink,
  CheckCircle,
  XCircle
} from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Textarea } from '@/components/ui/textarea';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';

interface PermissionRequestModalProps {
  isOpen: boolean;
  onClose: () => void;
  
  // Permission details
  requiredPermission?: string;
  requiredTier?: string;
  featureName: string;
  restrictionReason: string;
  
  // User context
  userEmail?: string;
  currentTier?: string;
  canRequest?: boolean;
  isAdmin?: boolean;
  
  // Actions
  onRequestPermission?: (data: PermissionRequestData) => Promise<void>;
  onContactAdmin?: () => void;
  onUpgrade?: () => void;
}

export interface PermissionRequestData {
  featureName: string;
  requiredPermission?: string;
  reason: string;
  businessJustification?: string;
  urgency: 'low' | 'medium' | 'high';
}

type RequestStep = 'reason' | 'submitting' | 'success' | 'error';

export function PermissionRequestModal({
  isOpen,
  onClose,
  requiredPermission,
  requiredTier,
  featureName,
  restrictionReason,
  userEmail,
  currentTier,
  canRequest = true,
  isAdmin = false,
  onRequestPermission,
  onContactAdmin,
  onUpgrade
}: PermissionRequestModalProps) {
  const [currentStep, setCurrentStep] = useState<RequestStep>('reason');
  const [requestReason, setRequestReason] = useState('');
  const [businessJustification, setBusinessJustification] = useState('');
  const [urgency, setUrgency] = useState<'low' | 'medium' | 'high'>('medium');
  const [errorMessage, setErrorMessage] = useState('');

  // Reset form when modal opens
  const handleModalOpenChange = (open: boolean) => {
    if (!open) {
      onClose();
      // Reset form after animation completes
      setTimeout(() => {
        setCurrentStep('reason');
        setRequestReason('');
        setBusinessJustification('');
        setUrgency('medium');
        setErrorMessage('');
      }, 300);
    }
  };

  const handleSubmitRequest = async () => {
    if (!requestReason.trim()) return;

    setCurrentStep('submitting');

    try {
      await onRequestPermission?.({
        featureName,
        requiredPermission,
        reason: requestReason,
        businessJustification,
        urgency
      });
      setCurrentStep('success');
    } catch (error) {
      setCurrentStep('error');
      setErrorMessage(error instanceof Error ? error.message : 'Failed to submit request');
    }
  };

  const getIcon = () => {
    if (requiredTier) return <Crown className="h-8 w-8 text-yellow-500" />;
    if (isAdmin) return <Shield className="h-8 w-8 text-red-500" />;
    return <Lock className="h-8 w-8 text-gray-500" />;
  };

  const getTitle = () => {
    switch (currentStep) {
      case 'submitting': return 'Submitting Request...';
      case 'success': return 'Request Submitted!';
      case 'error': return 'Request Failed';
      default: return `Access Required: ${featureName}`;
    }
  };

  const renderReasonStep = () => (
    <div className="space-y-6">
      {/* Feature info */}
      <div className="flex items-start space-x-4 p-4 bg-muted/50 rounded-lg">
        {getIcon()}
        <div className="flex-1">
          <h3 className="font-semibold text-lg">{featureName}</h3>
          <p className="text-muted-foreground text-sm mt-1">{restrictionReason}</p>
          
          {requiredPermission && (
            <div className="mt-2">
              <Badge variant="outline" className="text-xs">
                {requiredPermission}
              </Badge>
            </div>
          )}
          
          {requiredTier && (
            <div className="mt-2">
              <Badge variant="secondary" className="text-xs bg-yellow-100 text-yellow-800">
                Requires: {requiredTier}
              </Badge>
              {currentTier && (
                <Badge variant="outline" className="text-xs ml-2">
                  Current: {currentTier}
                </Badge>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Request form */}
      {canRequest && onRequestPermission && (
        <div className="space-y-4">
          <div>
            <Label htmlFor="reason">Why do you need access to this feature? *</Label>
            <Textarea
              id="reason"
              placeholder="Please explain your use case and why you need access to this feature..."
              value={requestReason}
              onChange={(e) => setRequestReason(e.target.value)}
              className="mt-1 min-h-[100px]"
            />
          </div>
          
          <div>
            <Label htmlFor="justification">Business justification (optional)</Label>
            <Textarea
              id="justification"
              placeholder="How will this feature help your work or improve business outcomes?"
              value={businessJustification}
              onChange={(e) => setBusinessJustification(e.target.value)}
              className="mt-1 min-h-[60px]"
            />
          </div>

          <div>
            <Label htmlFor="urgency">Urgency Level</Label>
            <select
              id="urgency"
              value={urgency}
              onChange={(e) => setUrgency(e.target.value as 'low' | 'medium' | 'high')}
              className="mt-1 w-full p-2 border rounded-md bg-background"
            >
              <option value="low">Low - Can wait a few days</option>
              <option value="medium">Medium - Needed this week</option>
              <option value="high">High - Urgent, needed ASAP</option>
            </select>
          </div>
        </div>
      )}

      {/* Action buttons */}
      <div className="flex flex-col sm:flex-row gap-3 pt-4">
        {canRequest && onRequestPermission && (
          <Button 
            onClick={handleSubmitRequest}
            disabled={!requestReason.trim()}
            className="flex-1"
          >
            <MessageSquare className="h-4 w-4 mr-2" />
            Submit Request
          </Button>
        )}
        
        {requiredTier && onUpgrade && (
          <Button onClick={onUpgrade} variant="secondary" className="flex-1">
            <Crown className="h-4 w-4 mr-2" />
            Upgrade to {requiredTier}
          </Button>
        )}
        
        {onContactAdmin && (
          <Button onClick={onContactAdmin} variant="outline" className="flex-1">
            <Mail className="h-4 w-4 mr-2" />
            Contact Admin
          </Button>
        )}
      </div>

      {/* Current user info */}
      {userEmail && (
        <div className="text-xs text-muted-foreground border-t pt-4">
          <div className="flex items-center">
            <User className="h-3 w-3 mr-1" />
            Requesting as: {userEmail}
          </div>
        </div>
      )}
    </div>
  );

  const renderSubmittingStep = () => (
    <div className="text-center py-8">
      <div className="animate-spin h-8 w-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-4"></div>
      <p className="text-muted-foreground">Submitting your access request...</p>
    </div>
  );

  const renderSuccessStep = () => (
    <div className="text-center py-8 space-y-4">
      <CheckCircle className="h-12 w-12 text-green-500 mx-auto" />
      <div>
        <h3 className="font-semibold text-lg">Request Submitted Successfully!</h3>
        <p className="text-muted-foreground mt-2">
          Your access request for <strong>{featureName}</strong> has been sent to the administrators.
          You'll receive an email notification when your request is reviewed.
        </p>
      </div>
      <div className="bg-green-50 dark:bg-green-900/20 p-4 rounded-lg">
        <p className="text-sm text-green-700 dark:text-green-300">
          <strong>What happens next:</strong>
        </p>
        <ul className="text-sm text-green-600 dark:text-green-400 mt-2 space-y-1">
          <li>• Admin team will review your request within 1-2 business days</li>
          <li>• You'll receive an email notification with their decision</li>
          <li>• If approved, access will be granted automatically</li>
        </ul>
      </div>
      
      <Button onClick={() => handleModalOpenChange(false)} className="w-full">
        Close
      </Button>
    </div>
  );

  const renderErrorStep = () => (
    <div className="text-center py-8 space-y-4">
      <XCircle className="h-12 w-12 text-red-500 mx-auto" />
      <div>
        <h3 className="font-semibold text-lg">Request Failed</h3>
        <p className="text-muted-foreground mt-2">
          We couldn't submit your access request at this time.
        </p>
        {errorMessage && (
          <div className="bg-red-50 dark:bg-red-900/20 p-4 rounded-lg mt-4">
            <p className="text-sm text-red-700 dark:text-red-300">
              <strong>Error:</strong> {errorMessage}
            </p>
          </div>
        )}
      </div>
      
      <div className="flex gap-2">
        <Button 
          onClick={() => setCurrentStep('reason')} 
          variant="outline"
          className="flex-1"
        >
          Try Again
        </Button>
        <Button onClick={() => handleModalOpenChange(false)} className="flex-1">
          Close
        </Button>
      </div>
    </div>
  );

  return (
    <Dialog open={isOpen} onOpenChange={handleModalOpenChange}>
      <DialogContent className="sm:max-w-[500px] max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="text-center">{getTitle()}</DialogTitle>
          {currentStep === 'reason' && (
            <DialogDescription className="text-center">
              Request access to this feature or contact an administrator for assistance.
            </DialogDescription>
          )}
        </DialogHeader>

        <div className="mt-4">
          {currentStep === 'reason' && renderReasonStep()}
          {currentStep === 'submitting' && renderSubmittingStep()}
          {currentStep === 'success' && renderSuccessStep()}
          {currentStep === 'error' && renderErrorStep()}
        </div>
      </DialogContent>
    </Dialog>
  );
}