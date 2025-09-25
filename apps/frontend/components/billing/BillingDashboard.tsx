'use client'

/**
 * Web3 Enterprise Billing Dashboard
 * Comprehensive billing management for Web3 enterprise users
 */

import { useState, useEffect } from 'react';
import { useEnterpriseApi, type BillingSubscription, type PaymentHistory, type ApiKey } from '@/lib/enterprise-api-client';
import { useWeb3Auth } from '@/lib/auth/use-web3-auth';

interface BillingOverview {
  total_spent_usd: number;
  active_subscriptions: number;
  next_payment_due: string;
  billing_status: string;
}

interface UsageMetrics {
  api_calls: Array<{ timestamp: string; count: number }>;
  data_transfer: Array<{ timestamp: string; bytes: number }>;
  error_rate: Array<{ timestamp: string; rate: number }>;
}

interface AnalyticsOverview {
  total_api_calls: number;
  total_data_points: number;
  success_rate: number;
  average_response_time: number;
  top_endpoints: Array<{ endpoint: string; calls: number }>;
}

export default function BillingDashboard() {
  const enterpriseApi = useEnterpriseApi();
  const { walletAddress, permissions, userTier } = useWeb3Auth();
  
  // Map userTier to enterprise tier display name
  const enterpriseTier = userTier === 'dao' ? 'Business' :
                        userTier === 'enterprise' ? 'Enterprise' :
                        userTier === 'token' ? 'Whale' :
                        'Starter';
  const user = walletAddress ? { address: walletAddress } : null;
  
  // State management
  const [activeTab, setActiveTab] = useState<'overview' | 'subscriptions' | 'payments' | 'api-keys' | 'usage'>('overview');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  
  // Data state
  const [billingOverview, setBillingOverview] = useState<BillingOverview | null>(null);
  const [subscriptions, setSubscriptions] = useState<BillingSubscription[]>([]);
  const [paymentHistory, setPaymentHistory] = useState<PaymentHistory[]>([]);
  const [apiKeys, setApiKeys] = useState<ApiKey[]>([]);
  const [usageMetrics, setUsageMetrics] = useState<UsageMetrics | null>(null);
  const [analyticsOverview, setAnalyticsOverview] = useState<AnalyticsOverview | null>(null);

  // Form state
  const [newApiKeyName, setNewApiKeyName] = useState('');
  const [isCreatingApiKey, setIsCreatingApiKey] = useState(false);

  useEffect(() => {
    loadBillingData();
  }, []);

  const loadBillingData = async () => {
    try {
      setLoading(true);
      setError(null);

      // Load billing overview
      const overviewResponse = await enterpriseApi.getBillingOverview();
      if (overviewResponse.success && overviewResponse.data) {
        setBillingOverview(overviewResponse.data);
      }

      // Load subscriptions
      const subscriptionsResponse = await enterpriseApi.getSubscriptions();
      if (subscriptionsResponse.success && subscriptionsResponse.data) {
        setSubscriptions(subscriptionsResponse.data);
      }

      // Load payment history
      const paymentsResponse = await enterpriseApi.getPaymentHistory({ limit: 10 });
      if (paymentsResponse.success && paymentsResponse.data) {
        setPaymentHistory(paymentsResponse.data.items);
      }

      // Load API keys
      const apiKeysResponse = await enterpriseApi.getApiKeys();
      if (apiKeysResponse.success && apiKeysResponse.data) {
        setApiKeys(apiKeysResponse.data);
      }

      // Load usage metrics
      const usageResponse = await enterpriseApi.getUsageMetrics({
        granularity: 'day',
        start_date: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
        end_date: new Date().toISOString().split('T')[0],
      });
      if (usageResponse.success && usageResponse.data) {
        setUsageMetrics(usageResponse.data);
      }

      // Load analytics overview
      const analyticsResponse = await enterpriseApi.getAnalyticsOverview();
      if (analyticsResponse.success && analyticsResponse.data) {
        setAnalyticsOverview(analyticsResponse.data);
      }

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load billing data');
    } finally {
      setLoading(false);
    }
  };

  const handleSubscriptionAction = async (
    subscriptionId: string, 
    action: 'pause' | 'resume' | 'cancel'
  ) => {
    try {
      let response;
      switch (action) {
        case 'pause':
          response = await enterpriseApi.pauseSubscription(subscriptionId);
          break;
        case 'resume':
          response = await enterpriseApi.resumeSubscription(subscriptionId);
          break;
        case 'cancel':
          response = await enterpriseApi.cancelSubscription(subscriptionId);
          break;
      }

      if (response.success) {
        await loadBillingData(); // Refresh data
      } else {
        setError(response.error || `Failed to ${action} subscription`);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : `Failed to ${action} subscription`);
    }
  };

  const handleCreateApiKey = async () => {
    if (!newApiKeyName.trim()) return;

    try {
      setIsCreatingApiKey(true);
      const response = await enterpriseApi.createApiKey(newApiKeyName.trim());
      
      if (response.success) {
        setNewApiKeyName('');
        await loadBillingData(); // Refresh API keys
      } else {
        setError(response.error || 'Failed to create API key');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create API key');
    } finally {
      setIsCreatingApiKey(false);
    }
  };

  const handleDeleteApiKey = async (keyId: string) => {
    if (!confirm('Are you sure you want to delete this API key? This action cannot be undone.')) {
      return;
    }

    try {
      const response = await enterpriseApi.deleteApiKey(keyId);
      
      if (response.success) {
        await loadBillingData(); // Refresh API keys
      } else {
        setError(response.error || 'Failed to delete API key');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete API key');
    }
  };

  const formatCurrency = (amount: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
    }).format(amount);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'active': return 'text-green-600 bg-green-50';
      case 'paused': return 'text-yellow-600 bg-yellow-50';
      case 'cancelled': return 'text-red-600 bg-red-50';
      case 'expired': return 'text-gray-600 bg-gray-50';
      case 'confirmed': return 'text-green-600 bg-green-50';
      case 'pending': return 'text-yellow-600 bg-yellow-50';
      case 'failed': return 'text-red-600 bg-red-50';
      default: return 'text-gray-600 bg-gray-50';
    }
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-white rounded-lg shadow p-8 text-center">
            <div className="text-gray-600">Loading Web3 billing dashboard...</div>
          </div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen bg-gray-50 p-6">
        <div className="max-w-7xl mx-auto">
          <div className="bg-red-50 border border-red-200 rounded-lg p-6">
            <div className="text-red-800 font-medium">Error Loading Billing Data</div>
            <div className="text-red-600 mt-2">{error}</div>
            <button
              onClick={() => {
                setError(null);
                loadBillingData();
              }}
              className="mt-4 bg-red-600 text-white px-4 py-2 rounded hover:bg-red-700"
            >
              Retry
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Web3 Enterprise Billing</h1>
          <p className="text-gray-600 mt-2">
            Manage your enterprise subscriptions, payments, and API usage
          </p>
          {user && (
            <div className="mt-4 inline-flex items-center space-x-4">
              <span className="text-sm text-gray-500">
                Wallet: {user.address?.slice(0, 8)}...{user.address?.slice(-4)}
              </span>
              <span className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${
                enterpriseTier === 'Whale' ? 'text-purple-700 bg-purple-100' :
                enterpriseTier === 'Enterprise' ? 'text-blue-700 bg-blue-100' :
                enterpriseTier === 'Business' ? 'text-green-700 bg-green-100' :
                'text-gray-700 bg-gray-100'
              }`}>
                {enterpriseTier} Tier
              </span>
            </div>
          )}
        </div>

        {/* Navigation Tabs */}
        <div className="bg-white rounded-lg shadow mb-6">
          <div className="border-b border-gray-200">
            <nav className="flex space-x-8" aria-label="Tabs">
              {[
                { key: 'overview', label: 'Overview' },
                { key: 'subscriptions', label: 'Subscriptions' },
                { key: 'payments', label: 'Payment History' },
                { key: 'api-keys', label: 'API Keys' },
                { key: 'usage', label: 'Usage Analytics' },
              ].map((tab) => (
                <button
                  key={tab.key}
                  onClick={() => setActiveTab(tab.key as any)}
                  className={`py-4 px-1 border-b-2 font-medium text-sm ${
                    activeTab === tab.key
                      ? 'border-blue-500 text-blue-600'
                      : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                  }`}
                >
                  {tab.label}
                </button>
              ))}
            </nav>
          </div>
        </div>

        {/* Tab Content */}
        <div className="space-y-6">
          {/* Overview Tab */}
          {activeTab === 'overview' && billingOverview && (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <div className="bg-white rounded-lg shadow p-6">
                <div className="text-sm font-medium text-gray-500">Total Spent</div>
                <div className="text-3xl font-bold text-gray-900 mt-2">
                  {formatCurrency(billingOverview.total_spent_usd)}
                </div>
              </div>
              <div className="bg-white rounded-lg shadow p-6">
                <div className="text-sm font-medium text-gray-500">Active Subscriptions</div>
                <div className="text-3xl font-bold text-gray-900 mt-2">
                  {billingOverview.active_subscriptions}
                </div>
              </div>
              <div className="bg-white rounded-lg shadow p-6">
                <div className="text-sm font-medium text-gray-500">Next Payment</div>
                <div className="text-lg font-medium text-gray-900 mt-2">
                  {billingOverview.next_payment_due ? 
                    formatDate(billingOverview.next_payment_due) : 
                    'No upcoming payments'
                  }
                </div>
              </div>
              <div className="bg-white rounded-lg shadow p-6">
                <div className="text-sm font-medium text-gray-500">Billing Status</div>
                <div className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium mt-2 ${
                  getStatusColor(billingOverview.billing_status)
                }`}>
                  {billingOverview.billing_status}
                </div>
              </div>
            </div>
          )}

          {/* Subscriptions Tab */}
          {activeTab === 'subscriptions' && (
            <div className="bg-white rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200">
                <h3 className="text-lg font-medium text-gray-900">Active Subscriptions</h3>
              </div>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Product
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Status
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Amount
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Next Billing
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Actions
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {subscriptions.map((subscription) => (
                      <tr key={subscription.id}>
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                          {subscription.product_id}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                            getStatusColor(subscription.status)
                          }`}>
                            {subscription.status}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatCurrency(subscription.amount_usd)} / {subscription.billing_cycle}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatDate(subscription.next_billing_date)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                          <div className="flex space-x-2">
                            {subscription.status === 'active' && (
                              <button
                                onClick={() => handleSubscriptionAction(subscription.id, 'pause')}
                                className="text-yellow-600 hover:text-yellow-900"
                              >
                                Pause
                              </button>
                            )}
                            {subscription.status === 'paused' && (
                              <button
                                onClick={() => handleSubscriptionAction(subscription.id, 'resume')}
                                className="text-green-600 hover:text-green-900"
                              >
                                Resume
                              </button>
                            )}
                            {subscription.status !== 'cancelled' && (
                              <button
                                onClick={() => handleSubscriptionAction(subscription.id, 'cancel')}
                                className="text-red-600 hover:text-red-900"
                              >
                                Cancel
                              </button>
                            )}
                          </div>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {subscriptions.length === 0 && (
                  <div className="text-center py-8 text-gray-500">
                    No active subscriptions found
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Payment History Tab */}
          {activeTab === 'payments' && (
            <div className="bg-white rounded-lg shadow">
              <div className="px-6 py-4 border-b border-gray-200">
                <h3 className="text-lg font-medium text-gray-900">Payment History</h3>
              </div>
              <div className="overflow-x-auto">
                <table className="min-w-full divide-y divide-gray-200">
                  <thead className="bg-gray-50">
                    <tr>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Date
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Amount
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Token
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Status
                      </th>
                      <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                        Transaction
                      </th>
                    </tr>
                  </thead>
                  <tbody className="bg-white divide-y divide-gray-200">
                    {paymentHistory.map((payment) => (
                      <tr key={payment.id}>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatDate(payment.created_at)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {formatCurrency(payment.amount_usd)}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          {payment.token_symbol} ({payment.network})
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap">
                          <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                            getStatusColor(payment.status)
                          }`}>
                            {payment.status}
                          </span>
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <a
                            href={`https://etherscan.io/tx/${payment.transaction_hash}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-blue-600 hover:text-blue-900"
                          >
                            {payment.transaction_hash.slice(0, 8)}...{payment.transaction_hash.slice(-4)}
                          </a>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
                {paymentHistory.length === 0 && (
                  <div className="text-center py-8 text-gray-500">
                    No payment history found
                  </div>
                )}
              </div>
            </div>
          )}

          {/* API Keys Tab */}
          {activeTab === 'api-keys' && (
            <div className="space-y-6">
              {/* Create New API Key */}
              <div className="bg-white rounded-lg shadow p-6">
                <h3 className="text-lg font-medium text-gray-900 mb-4">Create New API Key</h3>
                <div className="flex items-center space-x-4">
                  <input
                    type="text"
                    value={newApiKeyName}
                    onChange={(e) => setNewApiKeyName(e.target.value)}
                    placeholder="Enter API key name..."
                    className="flex-1 border border-gray-300 rounded-md px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  />
                  <button
                    onClick={handleCreateApiKey}
                    disabled={!newApiKeyName.trim() || isCreatingApiKey}
                    className="bg-blue-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    {isCreatingApiKey ? 'Creating...' : 'Create Key'}
                  </button>
                </div>
              </div>

              {/* API Keys List */}
              <div className="bg-white rounded-lg shadow">
                <div className="px-6 py-4 border-b border-gray-200">
                  <h3 className="text-lg font-medium text-gray-900">API Keys</h3>
                </div>
                <div className="overflow-x-auto">
                  <table className="min-w-full divide-y divide-gray-200">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Name
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Key
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Rate Limit
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Last Used
                        </th>
                        <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                          Actions
                        </th>
                      </tr>
                    </thead>
                    <tbody className="bg-white divide-y divide-gray-200">
                      {apiKeys.map((apiKey) => (
                        <tr key={apiKey.id}>
                          <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                            {apiKey.name}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900 font-mono">
                            {apiKey.key ? `${apiKey.key.slice(0, 8)}...${apiKey.key.slice(-4)}` : 'Hidden'}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {apiKey.rate_limit_per_minute} / min
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {apiKey.last_used_at ? formatDate(apiKey.last_used_at) : 'Never'}
                          </td>
                          <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                            <button
                              onClick={() => handleDeleteApiKey(apiKey.id)}
                              className="text-red-600 hover:text-red-900"
                            >
                              Delete
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                  {apiKeys.length === 0 && (
                    <div className="text-center py-8 text-gray-500">
                      No API keys found. Create your first API key above.
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* Usage Analytics Tab */}
          {activeTab === 'usage' && analyticsOverview && (
            <div className="space-y-6">
              {/* Analytics Overview */}
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Total API Calls</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {analyticsOverview.total_api_calls.toLocaleString()}
                  </div>
                </div>
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Success Rate</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {(analyticsOverview.success_rate * 100).toFixed(1)}%
                  </div>
                </div>
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Avg Response Time</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {analyticsOverview.average_response_time.toFixed(0)}ms
                  </div>
                </div>
                <div className="bg-white rounded-lg shadow p-6">
                  <div className="text-sm font-medium text-gray-500">Data Points</div>
                  <div className="text-3xl font-bold text-gray-900 mt-2">
                    {analyticsOverview.total_data_points.toLocaleString()}
                  </div>
                </div>
              </div>

              {/* Top Endpoints */}
              <div className="bg-white rounded-lg shadow">
                <div className="px-6 py-4 border-b border-gray-200">
                  <h3 className="text-lg font-medium text-gray-900">Top API Endpoints</h3>
                </div>
                <div className="p-6">
                  <div className="space-y-4">
                    {analyticsOverview.top_endpoints.map((endpoint, index) => (
                      <div key={endpoint.endpoint} className="flex items-center justify-between">
                        <div className="flex items-center space-x-3">
                          <div className="flex-shrink-0 w-6 h-6 bg-blue-100 rounded-full flex items-center justify-center text-xs font-medium text-blue-600">
                            {index + 1}
                          </div>
                          <span className="text-sm font-medium text-gray-900 font-mono">
                            {endpoint.endpoint}
                          </span>
                        </div>
                        <span className="text-sm text-gray-600">
                          {endpoint.calls.toLocaleString()} calls
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}