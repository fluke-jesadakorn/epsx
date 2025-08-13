'use client';

import React, { useState, useEffect } from 'react';
import { 
  AlertTriangle, 
  DollarSign, 
  TrendingUp, 
  Bell,
  X,
  ExternalLink,
  CreditCard,
  Settings,
  CheckCircle,
  AlertCircle,
  Info
} from 'lucide-react';
import { Button } from '@epsx/ui';
import { BillingService } from '@/services/billingService';
import type { BillingAlert } from '@/services/billingService';
import { toast } from 'react-hot-toast';

interface BillingAlertsProps {
  userId: string;
  onAlertAction?: (alertId: string, action: string) => void;
}

export const BillingAlerts: React.FC<BillingAlertsProps> = ({ 
  userId, 
  onAlertAction 
}) => {
  const [alerts, setAlerts] = useState<BillingAlert[]>([]);
  const [loading, setLoading] = useState(true);
  const [dismissingAlerts, setDismissingAlerts] = useState<Set<string>>(new Set());

  const billingService = BillingService.getInstance();

  useEffect(() => {
    const fetchAlerts = async () => {
      try {
        const alertsData = await billingService.generateBillingAlerts(userId);
        setAlerts(alertsData.filter(alert => !alert.dismissedAt));
      } catch (error) {
        console.error('Failed to fetch billing alerts:', error);
        toast.error('Failed to load billing alerts');
      } finally {
        setLoading(false);
      }
    };

    fetchAlerts();
  }, [userId, billingService]);

  const handleDismissAlert = async (alertId: string) => {
    setDismissingAlerts(prev => new Set([...prev, alertId]));
    
    try {
      // In a real implementation, this would call an API to dismiss the alert
      await new Promise(resolve => setTimeout(resolve, 500)); // Simulate API call
      
      setAlerts(prev => prev.filter(alert => alert.id !== alertId));
      toast.success('Alert dismissed');
      
      if (onAlertAction) {
        onAlertAction(alertId, 'dismiss');
      }
    } catch (error) {
      console.error('Failed to dismiss alert:', error);
      toast.error('Failed to dismiss alert');
    } finally {
      setDismissingAlerts(prev => {
        const newSet = new Set(prev);
        newSet.delete(alertId);
        return newSet;
      });
    }
  };

  const handleAlertAction = (alertId: string, action: string) => {
    if (onAlertAction) {
      onAlertAction(alertId, action);
    }

    // Handle common actions
    switch (action) {
      case 'view_billing':
        // Navigate to billing page
        window.location.href = '/admin/billing';
        break;
      case 'upgrade_plan':
        // Navigate to plan upgrade
        window.location.href = '/admin/billing/upgrade';
        break;
      case 'manage_quota':
        // Navigate to quota management
        window.location.href = '/admin/modules';
        break;
      case 'make_payment':
        // Navigate to payment page
        window.location.href = '/admin/billing/payment';
        break;
    }
  };

  const getAlertIcon = (type: BillingAlert['type'], severity: BillingAlert['severity']) => {
    const iconSize = "w-5 h-5";
    
    switch (type) {
      case 'quota_warning':
      case 'quota_exceeded':
        return severity === 'critical' ? 
          <AlertCircle className={`${iconSize} text-red-500`} /> : 
          <AlertTriangle className={`${iconSize} text-yellow-500`} />;
      case 'payment_due':
        return <DollarSign className={`${iconSize} text-blue-500`} />;
      case 'usage_spike':
        return <TrendingUp className={`${iconSize} text-purple-500`} />;
      default:
        return <Info className={`${iconSize} text-gray-500`} />;
    }
  };

  const getAlertStyles = (severity: BillingAlert['severity']) => {
    switch (severity) {
      case 'critical':
        return 'border-red-200 bg-red-50';
      case 'high':
        return 'border-orange-200 bg-orange-50';
      case 'medium':
        return 'border-yellow-200 bg-yellow-50';
      case 'low':
        return 'border-blue-200 bg-blue-50';
      default:
        return 'border-gray-200 bg-gray-50';
    }
  };

  const getActionButtons = (alert: BillingAlert) => {
    const buttons: Array<{ label: string; action: string; variant?: 'primary' | 'secondary'; icon?: React.ReactNode }> = [];

    switch (alert.type) {
      case 'quota_warning':
        buttons.push(
          { label: 'Upgrade Plan', action: 'upgrade_plan', variant: 'primary', icon: <ExternalLink className="w-4 h-4" /> },
          { label: 'Manage Quotas', action: 'manage_quota', variant: 'secondary', icon: <Settings className="w-4 h-4" /> }
        );
        break;
      case 'quota_exceeded':
        buttons.push(
          { label: 'Upgrade Now', action: 'upgrade_plan', variant: 'primary', icon: <ExternalLink className="w-4 h-4" /> },
          { label: 'View Usage', action: 'view_billing', variant: 'secondary' }
        );
        break;
      case 'payment_due':
        buttons.push(
          { label: 'Make Payment', action: 'make_payment', variant: 'primary', icon: <CreditCard className="w-4 h-4" /> },
          { label: 'View Bill', action: 'view_billing', variant: 'secondary' }
        );
        break;
      case 'usage_spike':
        buttons.push(
          { label: 'View Analytics', action: 'view_billing', variant: 'primary', icon: <TrendingUp className="w-4 h-4" /> }
        );
        break;
    }

    return buttons;
  };

  if (loading) {
    return (
      <div className="bg-white border rounded-lg p-4">
        <div className="animate-pulse flex items-center space-x-3">
          <div className="w-5 h-5 bg-gray-300 rounded"></div>
          <div className="flex-1 space-y-2">
            <div className="h-4 bg-gray-300 rounded w-3/4"></div>
            <div className="h-3 bg-gray-300 rounded w-1/2"></div>
          </div>
        </div>
      </div>
    );
  }

  if (alerts.length === 0) {
    return (
      <div className="bg-white border rounded-lg p-6 text-center">
        <CheckCircle className="w-12 h-12 text-green-500 mx-auto mb-4" />
        <h3 className="text-lg font-semibold text-gray-900 mb-2">All Clear!</h3>
        <p className="text-gray-600">No billing alerts at this time.</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <Bell className="w-5 h-5 text-gray-600" />
          <h2 className="text-lg font-semibold text-gray-900">Billing Alerts</h2>
          <span className="bg-red-100 text-red-800 text-xs font-medium px-2.5 py-0.5 rounded-full">
            {alerts.length}
          </span>
        </div>
      </div>

      <div className="space-y-3">
        {alerts.map((alert) => (
          <div
            key={alert.id}
            className={`border rounded-lg p-4 ${getAlertStyles(alert.severity)}`}
          >
            <div className="flex items-start justify-between">
              <div className="flex items-start space-x-3 flex-1">
                {getAlertIcon(alert.type, alert.severity)}
                
                <div className="flex-1 min-w-0">
                  <div className="flex items-center space-x-2 mb-1">
                    <h3 className="text-sm font-semibold text-gray-900">
                      {alert.title}
                    </h3>
                    <span className={`px-2 py-0.5 text-xs font-medium rounded-full ${
                      alert.severity === 'critical' ? 'bg-red-100 text-red-800' :
                      alert.severity === 'high' ? 'bg-orange-100 text-orange-800' :
                      alert.severity === 'medium' ? 'bg-yellow-100 text-yellow-800' :
                      'bg-blue-100 text-blue-800'
                    }`}>
                      {alert.severity}
                    </span>
                  </div>
                  
                  <p className="text-sm text-gray-700 mb-3">
                    {alert.message}
                  </p>

                  {/* Alert metadata */}
                  {alert.metadata && (
                    <div className="text-xs text-gray-500 mb-3">
                      {alert.type === 'quota_warning' || alert.type === 'quota_exceeded' ? (
                        <div className="flex items-center space-x-4">
                          <span>Module: {alert.metadata.moduleId?.replace('-', ' ')}</span>
                          <span>Usage: {alert.metadata.used?.toLocaleString()} / {alert.metadata.quota?.toLocaleString()}</span>
                          <span>({alert.metadata.usagePercentage?.toFixed(1)}%)</span>
                        </div>
                      ) : alert.type === 'payment_due' ? (
                        <div className="flex items-center space-x-4">
                          <span>Amount: ${alert.metadata.amount}</span>
                          <span>Due: {new Date(alert.metadata.dueDate).toLocaleDateString()}</span>
                        </div>
                      ) : null}
                    </div>
                  )}

                  {/* Action buttons */}
                  {alert.actionRequired && (
                    <div className="flex flex-wrap gap-2">
                      {getActionButtons(alert).map((button, index) => (
                        <Button
                          key={index}
                          size="sm"
                          variant={button.variant === 'primary' ? 'default' : 'outline'}
                          onClick={() => handleAlertAction(alert.id, button.action)}
                          className="text-xs"
                        >
                          {button.icon && <span className="mr-1">{button.icon}</span>}
                          {button.label}
                        </Button>
                      ))}
                    </div>
                  )}
                </div>
              </div>

              {/* Dismiss button */}
              <button
                onClick={() => handleDismissAlert(alert.id)}
                disabled={dismissingAlerts.has(alert.id)}
                className="ml-2 p-1 text-gray-400 hover:text-gray-600 disabled:opacity-50"
                title="Dismiss alert"
              >
                {dismissingAlerts.has(alert.id) ? (
                  <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-gray-400"></div>
                ) : (
                  <X className="w-4 h-4" />
                )}
              </button>
            </div>

            {/* Time stamp */}
            <div className="mt-3 pt-3 border-t border-gray-200">
              <p className="text-xs text-gray-500">
                {new Date(alert.createdAt).toLocaleString()}
              </p>
            </div>
          </div>
        ))}
      </div>

      {/* Quick actions */}
      <div className="bg-gray-50 border rounded-lg p-4">
        <h3 className="text-sm font-semibold text-gray-900 mb-3">Quick Actions</h3>
        <div className="flex flex-wrap gap-2">
          <Button size="sm" variant="outline" onClick={() => window.location.href = '/admin/billing'}>
            <DollarSign className="w-4 h-4 mr-1" />
            View Billing
          </Button>
          <Button size="sm" variant="outline" onClick={() => window.location.href = '/admin/modules'}>
            <Settings className="w-4 h-4 mr-1" />
            Manage Quotas
          </Button>
          <Button size="sm" variant="outline" onClick={() => window.location.href = '/admin/analytics'}>
            <TrendingUp className="w-4 h-4 mr-1" />
            Usage Analytics
          </Button>
        </div>
      </div>
    </div>
  );
};