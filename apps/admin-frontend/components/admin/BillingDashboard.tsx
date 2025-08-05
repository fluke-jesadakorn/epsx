'use client';

import React, { useState, useEffect } from 'react';
import { 
  DollarSign,
  CreditCard,
  TrendingUp,
  Download,
  Calendar,
  AlertTriangle,
  Settings,
  Users,
  Activity,
  FileText
} from 'lucide-react';
import { Button } from '@/components/ui/button';
import { FormField, Select } from '@/components/ui/form-components';
import { ModuleAnalyticsDashboard } from './ModuleAnalyticsDashboard';
import { BillingAlerts } from './BillingAlerts';
import { BillingService, type UsageBill } from '@/services/billingService';
import { useModuleAuth } from '@/auth/module-ctx';
import { toast } from 'react-hot-toast';
import { fmtCurrency } from '@epsx/shared-utils/formatting';

interface InvoiceData {
  id: string;
  period: string;
  amount: number;
  status: 'paid' | 'pending' | 'overdue';
  dueDate: string;
  downloadUrl: string;
}

export const BillingDashboard: React.FC = () => {
  const { hasModuleAccess, canPerformAction, user } = useModuleAuth();
  const [activeTab, setActiveTab] = useState<'overview' | 'analytics' | 'invoices' | 'alerts'>('overview');
  const [loading, setLoading] = useState(true);
  const [currentBill, setCurrentBill] = useState<UsageBill | null>(null);
  const [invoices, setInvoices] = useState<InvoiceData[]>([]);
  const [billingPeriod, setBillingPeriod] = useState('current');
  
  const billingService = BillingService.getInstance();

  useEffect(() => {
    const fetchBillingData = async () => {
      setLoading(true);
      try {
        // Generate current bill
        const bill = await billingService.generateUsageBill(
          user?.uid || 'demo_user',
          {
            start: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
            end: new Date().toISOString()
          }
        );
        setCurrentBill(bill);

        // Mock invoice data
        setInvoices([
          {
            id: 'inv_2024_01',
            period: 'January 2024',
            amount: 1420.50,
            status: 'paid',
            dueDate: '2024-02-01',
            downloadUrl: '/api/invoices/inv_2024_01/download'
          },
          {
            id: 'inv_2023_12',
            period: 'December 2023',
            amount: 1256.75,
            status: 'paid',
            dueDate: '2024-01-01',
            downloadUrl: '/api/invoices/inv_2023_12/download'
          },
          {
            id: 'inv_2023_11',
            period: 'November 2023',
            amount: 1089.25,
            status: 'paid',
            dueDate: '2023-12-01',
            downloadUrl: '/api/invoices/inv_2023_11/download'
          }
        ]);

      } catch (error) {
        console.error('Failed to fetch billing data:', error);
        toast.error('Failed to load billing data');
      } finally {
        setLoading(false);
      }
    };

    fetchBillingData();
  }, [user, billingPeriod]);

  const handleDownloadInvoice = async (invoice: InvoiceData) => {
    try {
      // In a real implementation, this would trigger a download
      toast.success(`Downloading invoice for ${invoice.period}`);
      window.open(invoice.downloadUrl, '_blank');
    } catch (error) {
      console.error('Failed to download invoice:', error);
      toast.error('Failed to download invoice');
    }
  };

  const handleMakePayment = async (billId: string) => {
    try {
      const result = await billingService.processPayment(billId, 'credit_card');
      if (result.success) {
        toast.success('Payment processed successfully');
        // Refresh billing data
      } else {
        toast.error(result.error || 'Payment failed');
      }
    } catch (error) {
      console.error('Payment error:', error);
      toast.error('Failed to process payment');
    }
  };


  const getStatusColor = (status: string) => {
    switch (status) {
      case 'paid': return 'bg-green-100 text-green-800';
      case 'pending': return 'bg-yellow-100 text-yellow-800';
      case 'overdue': return 'bg-red-100 text-red-800';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  if (!hasModuleAccess('billing') || !canPerformAction('billing', 'read')) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <AlertTriangle className="w-12 h-12 text-yellow-500 mx-auto mb-4" />
          <h3 className="text-lg font-semibold text-gray-900 mb-2">Access Restricted</h3>
          <p className="text-gray-600">You don't have permission to view billing information.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-7xl mx-auto p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Billing & Analytics</h1>
          <p className="text-gray-600">Monitor usage, costs, and billing across all modules</p>
        </div>
        
        <div className="flex items-center space-x-3">
          <FormField label="">
            <Select value={billingPeriod} onChange={(e) => setBillingPeriod(e.target.value)}>
              <option value="current">Current Period</option>
              <option value="last_month">Last Month</option>
              <option value="last_quarter">Last Quarter</option>
              <option value="last_year">Last Year</option>
            </Select>
          </FormField>
          
          <Button variant="outline">
            <Download className="w-4 h-4 mr-2" />
            Export Report
          </Button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {[
            { id: 'overview', name: 'Overview', icon: DollarSign },
            { id: 'analytics', name: 'Analytics', icon: TrendingUp },
            { id: 'invoices', name: 'Invoices', icon: FileText },
            { id: 'alerts', name: 'Alerts', icon: AlertTriangle }
          ].map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id as any)}
                className={`group inline-flex items-center py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === tab.id
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <Icon className="w-4 h-4 mr-2" />
                {tab.name}
              </button>
            );
          })}
        </nav>
      </div>

      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        </div>
      ) : (
        <>
          {/* Overview Tab */}
          {activeTab === 'overview' && currentBill && (
            <div className="space-y-6">
              {/* Current Bill Summary */}
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div className="bg-white border rounded-lg p-6">
                  <div className="flex items-center justify-between">
                    <div>
                      <h3 className="text-lg font-semibold text-gray-900">Current Bill</h3>
                      <div className="text-3xl font-bold text-gray-900 mt-2">
                        {fmtCurrency(currentBill.pricing.totalCost)}
                      </div>
                      <p className="text-sm text-gray-600 mt-1">
                        Due: {new Date(currentBill.paymentDue).toLocaleDateString()}
                      </p>
                    </div>
                    <DollarSign className="w-8 h-8 text-blue-600" />
                  </div>
                  
                  {currentBill.status === 'pending' && (
                    <Button 
                      className="w-full mt-4"
                      onClick={() => handleMakePayment(currentBill.id)}
                    >
                      <CreditCard className="w-4 h-4 mr-2" />
                      Make Payment
                    </Button>
                  )}
                </div>

                <div className="bg-white border rounded-lg p-6">
                  <div className="flex items-center justify-between">
                    <div>
                      <h3 className="text-lg font-semibold text-gray-900">Total Usage</h3>
                      <div className="text-3xl font-bold text-gray-900 mt-2">
                        {currentBill.usage.totalRequests.toLocaleString()}
                      </div>
                      <p className="text-sm text-gray-600 mt-1">API requests</p>
                    </div>
                    <Activity className="w-8 h-8 text-green-600" />
                  </div>
                </div>

                <div className="bg-white border rounded-lg p-6">
                  <div className="flex items-center justify-between">
                    <div>
                      <h3 className="text-lg font-semibold text-gray-900">Active Modules</h3>
                      <div className="text-3xl font-bold text-gray-900 mt-2">
                        {currentBill.usage.moduleBreakdown.length}
                      </div>
                      <p className="text-sm text-gray-600 mt-1">modules in use</p>
                    </div>
                    <Settings className="w-8 h-8 text-purple-600" />
                  </div>
                </div>
              </div>

              {/* Usage Breakdown */}
              <div className="bg-white border rounded-lg overflow-hidden">
                <div className="px-6 py-4 border-b border-gray-200">
                  <h3 className="text-lg font-semibold text-gray-900">Usage Breakdown</h3>
                </div>
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Module</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Requests</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Cost</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Per Request</th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {currentBill.usage.moduleBreakdown.map((module, index) => (
                        <tr key={index}>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <div className="font-medium text-gray-900">{module.moduleName}</div>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-gray-900">
                            {module.requests.toLocaleString()}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-gray-900">
                            {fmtCurrency(module.cost)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-gray-600">
                            ${(module.cost / module.requests * 1000).toFixed(4)}/1K
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>

              {/* Discounts Applied */}
              {currentBill.pricing.discounts.length > 0 && (
                <div className="bg-green-50 border border-green-200 rounded-lg p-6">
                  <h3 className="text-lg font-semibold text-green-900 mb-4">Discounts Applied</h3>
                  <div className="space-y-2">
                    {currentBill.pricing.discounts.map((discount, index) => (
                      <div key={index} className="flex justify-between items-center">
                        <span className="text-green-800">{discount.description}</span>
                        <span className="font-semibold text-green-900">
                          -{fmtCurrency(discount.amount)}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Analytics Tab */}
          {activeTab === 'analytics' && (
            <ModuleAnalyticsDashboard />
          )}

          {/* Invoices Tab */}
          {activeTab === 'invoices' && (
            <div className="space-y-6">
              <div className="bg-white border rounded-lg overflow-hidden">
                <div className="px-6 py-4 border-b border-gray-200">
                  <h3 className="text-lg font-semibold text-gray-900">Invoice History</h3>
                </div>
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Period</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Amount</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Due Date</th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {invoices.map((invoice) => (
                        <tr key={invoice.id}>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <div className="font-medium text-gray-900">{invoice.period}</div>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-gray-900">
                            {fmtCurrency(invoice.amount)}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <span className={`px-2 py-1 text-xs font-medium rounded-full ${getStatusColor(invoice.status)}`}>
                              {invoice.status}
                            </span>
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-gray-900">
                            {new Date(invoice.dueDate).toLocaleDateString()}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap">
                            <Button
                              size="sm"
                              variant="outline"
                              onClick={() => handleDownloadInvoice(invoice)}
                            >
                              <Download className="w-4 h-4 mr-1" />
                              Download
                            </Button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          )}

          {/* Alerts Tab */}
          {activeTab === 'alerts' && (
            <BillingAlerts 
              userId={user?.uid || 'demo_user'}
              onAlertAction={(alertId, action) => {
                // TODO: Implement alert action handling
              }}
            />
          )}
        </>
      )}
    </div>
  );
};