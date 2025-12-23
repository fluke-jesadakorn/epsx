/**
 * Admin Payments Management Page
 *
 * Comprehensive payment management interface for administrators including:
 * - Payment listing with filtering and search
 * - Payment details and audit logs
 * - Subscription management
 * - Refund processing
 * - Payment analytics and reporting
 */

'use client';

import {
  ChartBarIcon,
  CheckCircleIcon,
  ClockIcon,
  CurrencyDollarIcon,
  EyeIcon,
  PencilIcon,
  TrashIcon,
  UserGroupIcon,
  XCircleIcon
} from '@heroicons/react/24/outline';
import React, { useEffect, useState } from 'react';

// Import shared types
import type {
  PaymentResponse,
  PermissionTemplateName,
  UserSubscription
} from '@/shared/types/payment';

// Enhanced types for admin interface
interface AdminPayment extends PaymentResponse {
  user_wallet_address: string;
  transaction_hash?: string;
  block_number?: number;
  confirmations: number;
  plan_name: string;
  plan_price: number;
  payment_reference: string;
  metadata?: Record<string, any>;
}

interface PaymentStats {
  total_payments: number;
  total_amount: number;
  successful_payments: number;
  failed_payments: number;
  pending_payments: number;
  average_payment_amount: number;
  payments_today: number;
  revenue_today: number;
}

interface PaymentFilters {
  status: string;
  payment_method: string;
  date_range: string;
  plan_template: PermissionTemplateName;
  search: string;
}

const PaymentsPage: React.FC = () => {
  const [payments, setPayments] = useState<AdminPayment[]>([]);
  const [_subscriptions, setSubscriptions] = useState<UserSubscription[]>([]);
  const [stats, setStats] = useState<PaymentStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedTab, setSelectedTab] = useState<'payments' | 'subscriptions' | 'analytics'>('payments');

  // Filters
  const [filters, setFilters] = useState<PaymentFilters>({
    status: '',
    payment_method: '',
    date_range: '',
    plan_template: 'BASIC',
    search: '',
  });

  // Pagination
  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [itemsPerPage, _setItemsPerPage] = useState(20);

  // Load payments data
  const loadPayments = async () => {
    try {
      setLoading(true);
      setError(null);

      const params = new URLSearchParams({
        page: currentPage.toString(),
        limit: itemsPerPage.toString(),
        ...(filters.status && { status: filters.status }),
        ...(filters.payment_method && { payment_method: filters.payment_method }),
        ...(filters.search && { search: filters.search }),
      });

      const response = await fetch(`/api/admin/payments/list?${params}`);
      if (!response.ok) {
        throw new Error('Failed to load payments');
      }

      const data = await response.json();
      setPayments(data.payments || []);
      setStats(data.summary || null);
      setTotalPages(data.pagination?.total_pages || 1);

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error loading payments');
    } finally {
      setLoading(false);
    }
  };

  // Load subscriptions data
  const loadSubscriptions = async () => {
    try {
      setLoading(true);
      setError(null);

      const params = new URLSearchParams({
        page: currentPage.toString(),
        limit: itemsPerPage.toString(),
      });

      const response = await fetch(`/api/admin/payments/subscriptions?${params}`);
      if (!response.ok) {
        throw new Error('Failed to load subscriptions');
      }

      const data = await response.json();
      setSubscriptions(data.subscriptions || []);
      setStats(data.summary || null);

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error loading subscriptions');
    } finally {
      setLoading(false);
    }
  };

  // Load analytics data
  const loadAnalytics = async () => {
    try {
      setLoading(true);
      setError(null);

      const response = await fetch('/api/admin/payments/analytics');
      if (!response.ok) {
        throw new Error('Failed to load analytics');
      }

      const data = await response.json();
      setStats(data.analytics?.summary || null);

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error loading analytics');
    } finally {
      setLoading(false);
    }
  };

  // Load data based on selected tab
  useEffect(() => {
    switch (selectedTab) {
      case 'payments':
        loadPayments();
        break;
      case 'subscriptions':
        loadSubscriptions();
        break;
      case 'analytics':
        loadAnalytics();
        break;
    }
  }, [selectedTab, currentPage, itemsPerPage, filters]);

  // Get status color
  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'succeeded':
      case 'completed':
        return 'text-green-600 bg-green-100';
      case 'failed':
      case 'cancelled':
      case 'expired':
        return 'text-red-600 bg-red-100';
      case 'pending':
      case 'processing':
        return 'text-yellow-600 bg-yellow-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  };

  // Get status icon
  const getStatusIcon = (status: string) => {
    switch (status.toLowerCase()) {
      case 'succeeded':
      case 'completed':
        return <CheckCircleIcon className="w-4 h-4" />;
      case 'failed':
      case 'cancelled':
      case 'expired':
        return <XCircleIcon className="w-4 h-4" />;
      case 'pending':
      case 'processing':
        return <ClockIcon className="w-4 h-4" />;
      default:
        return <ClockIcon className="w-4 h-4" />;
    }
  };

  // Format currency
  const formatCurrency = (amount: number, currency: string = 'USD') => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency,
    }).format(amount);
  };

  // Format date
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  return (
    <div className="p-6">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">Payment Management</h1>
        <p className="mt-2 text-gray-600">
          Manage payments, subscriptions, and financial analytics
        </p>
      </div>

      {/* Stats Cards */}
      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          <div className="bg-white p-6 rounded-lg shadow">
            <div className="flex items-center">
              <div className="p-3 bg-blue-100 rounded-lg">
                <CurrencyDollarIcon className="w-6 h-6 text-blue-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm text-gray-600">Total Revenue</p>
                <p className="text-2xl font-bold text-gray-900">
                  {formatCurrency(stats.total_amount)}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg shadow">
            <div className="flex items-center">
              <div className="p-3 bg-green-100 rounded-lg">
                <CheckCircleIcon className="w-6 h-6 text-green-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm text-gray-600">Successful</p>
                <p className="text-2xl font-bold text-gray-900">
                  {stats.successful_payments}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg shadow">
            <div className="flex items-center">
              <div className="p-3 bg-yellow-100 rounded-lg">
                <ClockIcon className="w-6 h-6 text-yellow-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm text-gray-600">Pending</p>
                <p className="text-2xl font-bold text-gray-900">
                  {stats.pending_payments}
                </p>
              </div>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg shadow">
            <div className="flex items-center">
              <div className="p-3 bg-purple-100 rounded-lg">
                <UserGroupIcon className="w-6 h-6 text-purple-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm text-gray-600">Active Subs</p>
                <p className="text-2xl font-bold text-gray-900">
                  {stats.average_payment_amount.toFixed(1)}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab Navigation */}
      <div className="border-b border-gray-200 mb-6">
        <nav className="flex space-x-8">
          <button
            onClick={() => setSelectedTab('payments')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${selectedTab === 'payments'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
          >
            <div className="flex items-center space-x-2">
              <CurrencyDollarIcon className="w-4 h-4" />
              <span>Payments</span>
            </div>
          </button>

          <button
            onClick={() => setSelectedTab('subscriptions')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${selectedTab === 'subscriptions'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
          >
            <div className="flex items-center space-x-2">
              <UserGroupIcon className="w-4 h-4" />
              <span>Subscriptions</span>
            </div>
          </button>

          <button
            onClick={() => setSelectedTab('analytics')}
            className={`py-2 px-1 border-b-2 font-medium text-sm ${selectedTab === 'analytics'
                ? 'border-blue-500 text-blue-600'
                : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
          >
            <div className="flex items-center space-x-2">
              <ChartBarIcon className="w-4 h-4" />
              <span>Analytics</span>
            </div>
          </button>
        </nav>
      </div>

      {/* Filters */}
      {selectedTab === 'payments' && (
        <div className="bg-white p-4 rounded-lg shadow mb-6">
          <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Search
              </label>
              <input
                type="text"
                placeholder="Search by reference, wallet, or transaction hash..."
                value={filters.search}
                onChange={(e) => setFilters({ ...filters, search: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Status
              </label>
              <select
                value={filters.status}
                onChange={(e) => setFilters({ ...filters, status: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="">All Status</option>
                <option value="succeeded">Succeeded</option>
                <option value="pending">Pending</option>
                <option value="failed">Failed</option>
                <option value="cancelled">Cancelled</option>
                <option value="expired">Expired</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Payment Method
              </label>
              <select
                value={filters.payment_method}
                onChange={(e) => setFilters({ ...filters, payment_method: e.target.value })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="">All Methods</option>
                <option value="on_chain">On Chain</option>
                <option value="on_line">Online</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Plan
              </label>
              <select
                value={filters.plan_template}
                onChange={(e) => setFilters({ ...filters, plan_template: e.target.value as PermissionTemplateName })}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="BASIC">Basic</option>
                <option value="PRO">Pro</option>
                <option value="ENTERPRISE">Enterprise</option>
                <option value="WHALE">Whale</option>
              </select>
            </div>

            <div className="flex items-end">
              <button
                onClick={() => {
                  setFilters({
                    status: '',
                    payment_method: '',
                    date_range: '',
                    plan_template: 'BASIC',
                    search: '',
                  });
                }}
                className="px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                Reset
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Loading State */}
      {loading && (
        <div className="text-center py-12">
          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto"></div>
          <p className="mt-4 text-gray-600">Loading {selectedTab}...</p>
        </div>
      )}

      {/* Error State */}
      {error && !loading && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
          {error}
        </div>
      )}

      {/* Payments Table */}
      {!loading && !error && selectedTab === 'payments' && (
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Reference
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Wallet
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Plan
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Amount
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Created
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {payments.map((payment) => (
                  <tr key={payment.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {payment.payment_reference}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      <div className="font-mono text-xs">
                        {payment.user_wallet_address}
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {payment.plan_name}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {formatCurrency(payment.amount, payment.currency)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getStatusColor(payment.status)}`}>
                        {getStatusIcon(payment.status)}
                        <span className="ml-1">{payment.status}</span>
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {formatDate(payment.created_at)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                      <div className="flex space-x-2">
                        <button className="text-blue-600 hover:text-blue-900">
                          <EyeIcon className="w-4 h-4" />
                        </button>
                        <button className="text-green-600 hover:text-green-900">
                          <PencilIcon className="w-4 h-4" />
                        </button>
                        <button className="text-red-600 hover:text-red-900">
                          <TrashIcon className="w-4 h-4" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Pagination */}
          {totalPages > 1 && (
            <div className="bg-white px-4 py-3 flex items-center justify-between border-t border-gray-200 sm:px-6">
              <div className="flex-1 flex justify-between sm:hidden">
                <button
                  onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
                  disabled={currentPage === 1}
                  className="relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Previous
                </button>
                <button
                  onClick={() => setCurrentPage(Math.min(totalPages, currentPage + 1))}
                  disabled={currentPage === totalPages}
                  className="ml-3 relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Next
                </button>
              </div>
              <div className="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between">
                <div>
                  <p className="text-sm text-gray-700">
                    Showing page <span className="font-medium">{currentPage}</span> of{' '}
                    <span className="font-medium">{totalPages}</span>
                  </p>
                </div>
                <div>
                  <nav className="relative z-0 inline-flex rounded-md shadow-sm -space-x-px" aria-label="Pagination">
                    <button
                      onClick={() => setCurrentPage(Math.max(1, currentPage - 1))}
                      disabled={currentPage === 1}
                      className="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      Previous
                    </button>
                    {/* Page numbers could be added here */}
                    <button
                      onClick={() => setCurrentPage(Math.min(totalPages, currentPage + 1))}
                      disabled={currentPage === totalPages}
                      className="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      Next
                    </button>
                  </nav>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      {/* Empty State */}
      {!loading && !error && selectedTab === 'payments' && payments.length === 0 && (
        <div className="text-center py-12">
          <CurrencyDollarIcon className="mx-auto h-12 w-12 text-gray-400" />
          <h3 className="mt-2 text-sm font-medium text-gray-900">No payments found</h3>
          <p className="mt-1 text-sm text-gray-500">
            No payments match your current filters.
          </p>
          <div className="mt-6">
            <button
              onClick={() => {
                setFilters({
                  status: '',
                  payment_method: '',
                  date_range: '',
                  plan_template: 'BASIC',
                  search: '',
                });
              }}
              className="inline-flex items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Clear filters
            </button>
          </div>
        </div>
      )}

      {/* Subscriptions Tab Content */}
      {!loading && !error && selectedTab === 'subscriptions' && (
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <div className="text-center py-12">
            <UserGroupIcon className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900">Subscription management</h3>
            <p className="mt-1 text-sm text-gray-500">
              Subscription management interface will be implemented here.
            </p>
          </div>
        </div>
      )}

      {/* Analytics Tab Content */}
      {!loading && !error && selectedTab === 'analytics' && (
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <div className="text-center py-12">
            <ChartBarIcon className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900">Payment analytics</h3>
            <p className="mt-1 text-sm text-gray-500">
              Analytics dashboard will be implemented here.
            </p>
          </div>
        </div>
      )}
    </div>
  );
};

export default PaymentsPage;