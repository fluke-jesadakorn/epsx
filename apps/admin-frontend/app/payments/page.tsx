/**
 * Admin Payments Management Page
 *
 * Comprehensive payment management interface for administrators including:
 * - Payment listing with filtering and search
 * - Payment details and audit logs
 * - Subscription management
 * - Refund processing
 * - Payment analytics and reporting
 * - Dynamic Payment Links management
 */

'use client';

import { useApiClient } from '@/shared/hooks/useApiClient'; // Import Shared Hook
import {
  ChartBarIcon,
  CheckCircleIcon,
  ClipboardDocumentIcon,
  ClockIcon,
  CurrencyDollarIcon,
  EyeIcon,
  LinkIcon,
  PlusIcon,
  QrCodeIcon,
  TrashIcon,
  UserGroupIcon,
  XCircleIcon,
  XMarkIcon
} from '@heroicons/react/24/outline';
import React, { useCallback, useEffect, useState } from 'react';

// Import shared types
import type {
  PaymentResponse,
  PermissionTemplateName,
  PlanAccessData
} from '@/shared/types/payment';

// --- Types from Payments Page ---
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

// --- Types from Payment Links Page ---
type PaymentContextType = 'plan' | 'group' | 'product' | 'campaign' | 'custom';

interface PaymentLink {
  id: string;
  context_type: PaymentContextType;
  context_id?: string;
  slug: string;
  name: string;
  description?: string;
  amount: number;
  currency: string;
  expires_at?: string;
  max_uses?: number;
  current_uses: number;
  is_active: boolean;
  is_usable: boolean;
  url: string;
  link_hash: string;
  created_by: string;
  metadata: Record<string, any>;
  created_at: string;
  updated_at: string;
}

interface CreatePaymentLinkForm {
  context_type: PaymentContextType;
  context_id: string;
  slug: string;
  name: string;
  description: string;
  amount: string;
  currency: string;
  expires_in_hours: string;
  max_uses: string;
}

const CONTEXT_TYPES: { value: PaymentContextType; label: string; description: string }[] = [
  { value: 'plan', label: 'Plan', description: 'Plan payment' },
  { value: 'group', label: 'Group', description: 'Permission group access' },
  { value: 'product', label: 'Product', description: 'One-time product purchase' },
  { value: 'campaign', label: 'Campaign', description: 'Promotional campaign' },
  { value: 'custom', label: 'Custom', description: 'Custom payment link' },
];

const CURRENCIES = ['USDT', 'USDC', 'BNB'];

type TabType = 'payments' | 'user-access' | 'analytics' | 'payment-links';

const PaymentsPage: React.FC = () => {
  // --- Shared API Client ---
  const { base } = useApiClient({ platform: 'admin' });

  // --- Global State ---
  const [selectedTab, setSelectedTab] = useState<TabType>('payments');

  // --- Payments Tab State ---
  const [payments, setPayments] = useState<AdminPayment[]>([]);
  const [userAccess, setUserAccess] = useState<PlanAccessData[]>([]);
  const [stats, setStats] = useState<PaymentStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const [filters, setFilters] = useState<PaymentFilters>({
    status: '',
    payment_method: '',
    date_range: '',
    plan_template: 'BASIC',
    search: '',
  });

  const [currentPage, setCurrentPage] = useState(1);
  const [totalPages, setTotalPages] = useState(1);
  const [itemsPerPage, _setItemsPerPage] = useState(20);

  // --- Payment Links Tab State ---
  const [paymentLinks, setPaymentLinks] = useState<PaymentLink[]>([]);
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [copiedSlug, setCopiedSlug] = useState<string | null>(null);
  const [linkFilterType, setLinkFilterType] = useState<string>('');
  const [linkFilterActive, setLinkFilterActive] = useState<string>('');

  const [form, setForm] = useState<CreatePaymentLinkForm>({
    context_type: 'plan',
    context_id: '',
    slug: '',
    name: '',
    description: '',
    amount: '',
    currency: 'USDT',
    expires_in_hours: '24',
    max_uses: '',
  });
  const [formLoading, setFormLoading] = useState(false);
  const [formError, setFormError] = useState<string | null>(null);


  // --- Data Loading Functions ---

  const loadPayments = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const params = {
        page: currentPage.toString(),
        limit: itemsPerPage.toString(),
        ...(filters.status && { status: filters.status }),
        ...(filters.payment_method && { payment_method: filters.payment_method }),
        ...(filters.search && { search: filters.search }),
      };

      // Direct request to backend using UnifiedApiClient
      const response = await base.get<any>('/api/v1/payments/admin/list', params);

      if (response.success && response.data) {
        setPayments(response.data.payments || []);
        setStats(response.data.summary || null);
        setTotalPages(response.data.pagination?.total_pages || 1);
      } else {
        throw new Error(response.error || response.message || 'Failed to load payments');
      }

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error loading payments');
    } finally {
      setLoading(false);
    }
  }, [base, currentPage, itemsPerPage, filters]);

  const loadUserAccess = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const params = {
        page: currentPage.toString(),
        limit: itemsPerPage.toString(),
      };

      // Direct Payment model: query user plan access from wallet_users
      const response = await base.get<any>('/api/v1/admin/plans/user-access/list', params);

      if (response.success && response.data) {
        setUserAccess(response.data.users || []);
        setStats(response.data.summary || null);
      } else {
        throw new Error(response.error || response.message || 'Failed to load user access');
      }

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error loading user access');
    } finally {
      setLoading(false);
    }
  }, [base, currentPage, itemsPerPage]);

  const loadAnalytics = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const response = await base.get<any>('/api/v1/payments/admin/analytics');

      if (response.success && response.data) {
        setStats(response.data.analytics?.summary || null);
      } else {
        throw new Error(response.error || response.message || 'Failed to load analytics');
      }

    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error loading analytics');
    } finally {
      setLoading(false);
    }
  }, [base]);

  const loadPaymentLinks = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);

      const params: Record<string, string> = {};
      if (linkFilterType) params.context_type = linkFilterType;
      if (linkFilterActive) params.is_active = linkFilterActive;

      // Note: Assuming the backend endpoint for payment links is /api/v1/admin/payment-links
      const response = await base.get<any>('/api/v1/admin/payment-links', params);

      if (response.success && response.data) {
        setPaymentLinks(response.data.payment_links || []);
      } else {
        throw new Error(response.error || response.message || 'Failed to load payment links');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  }, [base, linkFilterType, linkFilterActive]);

  // --- Effects ---

  useEffect(() => {
    switch (selectedTab) {
      case 'payments':
        loadPayments();
        break;
      case 'user-access':
        loadUserAccess();
        break;
      case 'analytics':
        loadAnalytics();
        break;
      case 'payment-links':
        loadPaymentLinks();
        break;
    }
  }, [selectedTab, loadPayments, loadUserAccess, loadAnalytics, loadPaymentLinks]);

  // --- Helpers ---

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

  const formatCurrency = (amount: number, currency: string = 'USD') => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency,
    }).format(amount);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString();
  };

  // --- Payment Links Helpers ---

  const handleCreateLink = async (e: React.FormEvent) => {
    e.preventDefault();
    setFormLoading(true);
    setFormError(null);

    try {
      const expiresAt = form.expires_in_hours
        ? new Date(Date.now() + parseInt(form.expires_in_hours) * 60 * 60 * 1000).toISOString()
        : undefined;

      const payload = {
        context_type: form.context_type,
        context_id: form.context_id || undefined,
        slug: form.slug || undefined,
        name: form.name,
        description: form.description || undefined,
        amount: parseFloat(form.amount),
        currency: form.currency,
        expires_at: expiresAt,
        max_uses: form.max_uses ? parseInt(form.max_uses) : undefined,
      };

      const response = await base.post<any>('/api/v1/admin/payment-links', payload);

      if (response.success && response.data) {
        const newLink = response.data;
        setPaymentLinks((prev) => [newLink, ...prev]);
        setIsModalOpen(false);
        resetForm();
      } else {
        throw new Error(response.error || response.message || 'Failed to create payment link');
      }
    } catch (err) {
      setFormError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setFormLoading(false);
    }
  };

  const handleDeleteLink = async (id: string) => {
    if (!confirm('Are you sure you want to deactivate this payment link?')) return;

    try {
      const response = await base.delete<any>(`/api/v1/admin/payment-links/${id}`);

      if (response.success) {
        setPaymentLinks((prev) =>
          prev.map((link) => (link.id === id ? { ...link, is_active: false, is_usable: false } : link))
        );
      } else {
        throw new Error(response.error || response.message || 'Failed to deactivate payment link');
      }
    } catch (err) {
      alert(err instanceof Error ? err.message : 'Failed to deactivate');
    }
  };

  const handleCopyUrl = async (link: PaymentLink) => {
    try {
      await navigator.clipboard.writeText(link.url);
      setCopiedSlug(link.slug);
      setTimeout(() => setCopiedSlug(null), 2000);
    } catch {
      alert('Failed to copy URL');
    }
  };

  const resetForm = () => {
    setForm({
      context_type: 'plan',
      context_id: '',
      slug: '',
      name: '',
      description: '',
      amount: '',
      currency: 'USDT',
      expires_in_hours: '24',
      max_uses: '',
    });
    setFormError(null);
  };

  const isExpired = (expiresAt?: string) => {
    if (!expiresAt) return false;
    return new Date(expiresAt) < new Date();
  };

  const getLinkStatusBadge = (link: PaymentLink) => {
    if (!link.is_active) {
      return (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
          <XCircleIcon className="w-3 h-3 mr-1" />
          Inactive
        </span>
      );
    }
    if (isExpired(link.expires_at)) {
      return (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
          <ClockIcon className="w-3 h-3 mr-1" />
          Expired
        </span>
      );
    }
    if (link.max_uses && link.current_uses >= link.max_uses) {
      return (
        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800">
          <CheckCircleIcon className="w-3 h-3 mr-1" />
          Max Uses Reached
        </span>
      );
    }
    return (
      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
        <CheckCircleIcon className="w-3 h-3 mr-1" />
        Active
      </span>
    );
  };

  return (
    <div className="p-6">
      <div className="mb-8 flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Payment Management</h1>
          <p className="mt-2 text-gray-600">
            Manage payments, subscriptions, payment links, and financial analytics
          </p>
        </div>

        {/* Action Button for Payment Links */}
        {selectedTab === 'payment-links' && (
          <button
            onClick={() => setIsModalOpen(true)}
            className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          >
            <PlusIcon className="w-5 h-5 mr-2" />
            Create Link
          </button>
        )}
      </div>

      {/* Stats Cards (Reused globally for unified view possibly, or just show active subs etc always) */}
      {/* Currently Logic: Stats are set by all loaders, so it updates depending on tab context which is good */}
      {stats && selectedTab !== 'payment-links' && (
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
                <CurrencyDollarIcon className="w-6 h-6 text-purple-600" />
              </div>
              <div className="ml-4">
                <p className="text-sm text-gray-600">Avg. Payment</p>
                <p className="text-2xl font-bold text-gray-900">
                  {(stats.average_payment_amount || 0).toFixed(1)}
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Tab Navigation */}
      <div className="border-b border-gray-200 mb-6">
        <nav className="flex space-x-8 overflow-x-auto">
          <button
            onClick={() => setSelectedTab('payments')}
            className={`py-2 px-1 border-b-2 font-medium text-sm whitespace-nowrap ${selectedTab === 'payments'
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
            onClick={() => setSelectedTab('user-access')}
            className={`py-2 px-1 border-b-2 font-medium text-sm whitespace-nowrap ${selectedTab === 'user-access'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
          >
            <div className="flex items-center space-x-2">
              <UserGroupIcon className="w-4 h-4" />
              <span>User Access</span>
            </div>
          </button>

          <button
            onClick={() => setSelectedTab('payment-links')}
            className={`py-2 px-1 border-b-2 font-medium text-sm whitespace-nowrap ${selectedTab === 'payment-links'
              ? 'border-blue-500 text-blue-600'
              : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
          >
            <div className="flex items-center space-x-2">
              <LinkIcon className="w-4 h-4" />
              <span>Payment Links</span>
            </div>
          </button>

          <button
            onClick={() => setSelectedTab('analytics')}
            className={`py-2 px-1 border-b-2 font-medium text-sm whitespace-nowrap ${selectedTab === 'analytics'
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

      {/* --- PAYMENTS TAB --- */}
      {selectedTab === 'payments' && (
        <div className="bg-white p-4 rounded-lg shadow mb-6">
          <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Search
              </label>
              <input
                type="text"
                placeholder="Reference, wallet, hash..."
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

      {/* --- PAYMENT LINKS TAB (Filters) --- */}
      {selectedTab === 'payment-links' && (
        <div className="bg-white p-4 rounded-lg shadow mb-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Context Type</label>
              <select
                value={linkFilterType}
                onChange={(e) => setLinkFilterType(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="">All Types</option>
                {CONTEXT_TYPES.map((type) => (
                  <option key={type.value} value={type.value}>
                    {type.label}
                  </option>
                ))}
              </select>
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Status</label>
              <select
                value={linkFilterActive}
                onChange={(e) => setLinkFilterActive(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              >
                <option value="">All Status</option>
                <option value="true">Active</option>
                <option value="false">Inactive</option>
              </select>
            </div>
            <div className="flex items-end">
              <button
                onClick={() => {
                  setLinkFilterType('');
                  setLinkFilterActive('');
                }}
                className="px-4 py-2 text-gray-600 border border-gray-300 rounded-md hover:bg-gray-50"
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
          <p className="mt-4 text-gray-600">Loading {selectedTab.replace('-', ' ')}...</p>
        </div>
      )}

      {/* Error State */}
      {error && !loading && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6">
          {error}
        </div>
      )}

      {/* --- 1. PAYMENTS TABLE --- */}
      {!loading && !error && selectedTab === 'payments' && (
        <>
          {payments.length === 0 ? (
            <div className="text-center py-12">
              <CurrencyDollarIcon className="mx-auto h-12 w-12 text-gray-400" />
              <h3 className="mt-2 text-sm font-medium text-gray-900">No payments found</h3>
              <p className="mt-1 text-sm text-gray-500">
                No payments match your current filters.
              </p>
            </div>
          ) : (
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
        </>
      )}

      {/* --- 2. USER ACCESS TAB --- */}
      {!loading && !error && selectedTab === 'user-access' && (
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Wallet
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Plan
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Days Left
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Expires
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {userAccess.length === 0 ? (
                  <tr>
                    <td colSpan={6} className="px-6 py-12 text-center text-gray-500">
                      <UserGroupIcon className="mx-auto h-12 w-12 text-gray-400 mb-2" />
                      <p>No users with plan access found</p>
                    </td>
                  </tr>
                ) : (
                  userAccess.map((user: PlanAccessData) => (
                    <tr key={user.wallet_address} className="hover:bg-gray-50">
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        <div className="font-mono text-xs" title={user.wallet_address}>
                          {user.wallet_address.substring(0, 10)}...{user.wallet_address.substring(user.wallet_address.length - 4)}
                        </div>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        <span className="font-medium">{user.plan_name || 'No Plan'}</span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap">
                        <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${user.status === 'active' ? 'bg-green-100 text-green-800' :
                          user.status === 'expiring_soon' ? 'bg-yellow-100 text-yellow-800' :
                            user.status === 'expired' ? 'bg-red-100 text-red-800' :
                              'bg-gray-100 text-gray-800'
                          }`}>
                          {user.status === 'no_plan' ? 'No Plan' : user.status}
                        </span>
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {user.days_remaining > 0 ? `${user.days_remaining} days` : '-'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {user.plan_expires_at ? formatDate(user.plan_expires_at) : 'Never'}
                      </td>
                      <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                        <div className="flex space-x-2">
                          <button className="text-blue-600 hover:text-blue-900">
                            View
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* --- 3. ANALYTICS TAB --- */}
      {!loading && !error && selectedTab === 'analytics' && stats && (
        <div className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="bg-white p-6 rounded-lg shadow">
              <h3 className="text-lg font-medium text-gray-900 mb-4">Transaction Overview</h3>
              <dl className="grid grid-cols-2 gap-4">
                <div className="bg-gray-50 p-4 rounded-lg">
                  <dt className="text-sm font-medium text-gray-500">Total Transactions</dt>
                  <dd className="mt-1 text-2xl font-semibold text-gray-900">{stats.total_payments}</dd>
                </div>
                <div className="bg-green-50 p-4 rounded-lg">
                  <dt className="text-sm font-medium text-green-800">Successful</dt>
                  <dd className="mt-1 text-2xl font-semibold text-green-900">{stats.successful_payments}</dd>
                </div>
                <div className="bg-yellow-50 p-4 rounded-lg">
                  <dt className="text-sm font-medium text-yellow-800">Pending</dt>
                  <dd className="mt-1 text-2xl font-semibold text-yellow-900">{stats.pending_payments}</dd>
                </div>
                <div className="bg-red-50 p-4 rounded-lg">
                  <dt className="text-sm font-medium text-red-800">Failed</dt>
                  <dd className="mt-1 text-2xl font-semibold text-red-900">{stats.failed_payments}</dd>
                </div>
              </dl>
            </div>

            <div className="bg-white p-6 rounded-lg shadow">
              <h3 className="text-lg font-medium text-gray-900 mb-4">Revenue Overview</h3>
              <dl className="space-y-4">
                <div className="flex items-center justify-between p-4 bg-blue-50 rounded-lg">
                  <dt className="text-sm font-medium text-blue-800">Total Revenue</dt>
                  <dd className="text-2xl font-bold text-blue-900">{formatCurrency(stats.total_amount)}</dd>
                </div>
                <div className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                  <dt className="text-sm font-medium text-gray-600">Revenue Today</dt>
                  <dd className="text-xl font-semibold text-gray-900">{formatCurrency(stats.revenue_today)}</dd>
                </div>
                <div className="flex items-center justify-between p-4 bg-gray-50 rounded-lg">
                  <dt className="text-sm font-medium text-gray-600">Avg. Transaction</dt>
                  <dd className="text-xl font-semibold text-gray-900">{formatCurrency(stats.average_payment_amount)}</dd>
                </div>
              </dl>
            </div>
          </div>

          <div className="bg-white p-6 rounded-lg shadow">
            <h3 className="text-lg font-medium text-gray-900 mb-4">Recent Activity</h3>
            <p className="text-gray-500 text-sm">Detailed historical analytics and charts coming soon.</p>
          </div>
        </div>
      )}

      {/* --- 4. PAYMENT LINKS TAB --- */}
      {!loading && !error && selectedTab === 'payment-links' && (
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Name / Slug
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Type
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Amount
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Usage
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Expires
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {paymentLinks.length === 0 ? (
                  <tr>
                    <td colSpan={7} className="px-6 py-12 text-center text-gray-500">
                      <LinkIcon className="mx-auto h-12 w-12 text-gray-400 mb-2" />
                      <p className="text-lg font-medium">No payment links yet</p>
                      <p className="text-sm">Create your first payment link to get started</p>
                    </td>
                  </tr>
                ) : (
                  paymentLinks.map((link) => (
                    <tr key={link.id} className="hover:bg-gray-50">
                      <td className="px-6 py-4">
                        <div className="text-sm font-medium text-gray-900">{link.name}</div>
                        <div className="text-xs text-gray-500 font-mono">{link.slug}</div>
                      </td>
                      <td className="px-6 py-4">
                        <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 capitalize">
                          {link.context_type}
                        </span>
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-900">
                        {link.amount} {link.currency}
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-900">
                        {link.current_uses}
                        {link.max_uses ? ` / ${link.max_uses}` : ' / ∞'}
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-500">
                        {link.expires_at ? formatDate(link.expires_at) : 'Never'}
                      </td>
                      <td className="px-6 py-4">{getLinkStatusBadge(link)}</td>
                      <td className="px-6 py-4">
                        <div className="flex space-x-2">
                          <button
                            onClick={() => handleCopyUrl(link)}
                            className="p-1 text-gray-500 hover:text-blue-600"
                            title="Copy URL"
                          >
                            {copiedSlug === link.slug ? (
                              <CheckCircleIcon className="w-5 h-5 text-green-500" />
                            ) : (
                              <ClipboardDocumentIcon className="w-5 h-5" />
                            )}
                          </button>
                          <button
                            className="p-1 text-gray-500 hover:text-purple-600"
                            title="Show QR Code"
                          >
                            <QrCodeIcon className="w-5 h-5" />
                          </button>
                          {link.is_active && (
                            <button
                              onClick={() => handleDeleteLink(link.id)}
                              className="p-1 text-gray-500 hover:text-red-600"
                              title="Deactivate"
                            >
                              <TrashIcon className="w-5 h-5" />
                            </button>
                          )}
                        </div>
                      </td>
                    </tr>
                  ))
                )}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* --- PAYMENT LINK MODAL --- */}
      {isModalOpen && (
        <div className="fixed inset-0 z-50 overflow-y-auto">
          <div className="flex items-center justify-center min-h-screen px-4 pt-4 pb-20 text-center sm:block sm:p-0">
            <div className="fixed inset-0 transition-opacity" aria-hidden="true">
              <div className="absolute inset-0 bg-gray-500 opacity-75"></div>
            </div>
            <div className="inline-block align-bottom bg-white rounded-lg text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full">
              <form onSubmit={handleCreateLink}>
                <div className="bg-white px-4 pt-5 pb-4 sm:p-6 sm:pb-4">
                  <div className="flex items-center justify-between mb-4">
                    <h3 className="text-lg leading-6 font-medium text-gray-900">
                      Create Payment Link
                    </h3>
                    <button
                      type="button"
                      onClick={() => {
                        setIsModalOpen(false);
                        resetForm();
                      }}
                      className="text-gray-400 hover:text-gray-500"
                    >
                      <XMarkIcon className="w-6 h-6" />
                    </button>
                  </div>

                  {formError && (
                    <div className="mb-4 bg-red-50 border border-red-200 text-red-700 px-3 py-2 rounded text-sm">
                      {formError}
                    </div>
                  )}

                  <div className="space-y-4">
                    {/* Context Type */}
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Context Type *
                      </label>
                      <select
                        value={form.context_type}
                        onChange={(e) =>
                          setForm({ ...form, context_type: e.target.value as PaymentContextType })
                        }
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                        required
                      >
                        {CONTEXT_TYPES.map((type) => (
                          <option key={type.value} value={type.value}>
                            {type.label} - {type.description}
                          </option>
                        ))}
                      </select>
                    </div>

                    {/* Context ID (for plan/group) */}
                    {(form.context_type === 'plan' || form.context_type === 'group') && (
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          {form.context_type === 'plan' ? 'Plan ID' : 'Group ID'}
                        </label>
                        <input
                          type="text"
                          value={form.context_id}
                          onChange={(e) => setForm({ ...form, context_id: e.target.value })}
                          placeholder="UUID of the linked entity"
                          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                        />
                      </div>
                    )}

                    {/* Name */}
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">Name *</label>
                      <input
                        type="text"
                        value={form.name}
                        onChange={(e) => setForm({ ...form, name: e.target.value })}
                        placeholder="e.g., Pro Plan Monthly"
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                        required
                      />
                    </div>

                    {/* Description */}
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Description
                      </label>
                      <textarea
                        value={form.description}
                        onChange={(e) => setForm({ ...form, description: e.target.value })}
                        placeholder="Optional description"
                        rows={2}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                      />
                    </div>

                    {/* Amount & Currency */}
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          Amount *
                        </label>
                        <input
                          type="number"
                          step="0.01"
                          min="0.01"
                          value={form.amount}
                          onChange={(e) => setForm({ ...form, amount: e.target.value })}
                          placeholder="0.00"
                          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                          required
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          Currency
                        </label>
                        <select
                          value={form.currency}
                          onChange={(e) => setForm({ ...form, currency: e.target.value })}
                          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                        >
                          {CURRENCIES.map((c) => (
                            <option key={c} value={c}>
                              {c}
                            </option>
                          ))}
                        </select>
                      </div>
                    </div>

                    {/* Expiration & Max Uses */}
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          Expires In (hours)
                        </label>
                        <input
                          type="number"
                          min="1"
                          value={form.expires_in_hours}
                          onChange={(e) => setForm({ ...form, expires_in_hours: e.target.value })}
                          placeholder="24"
                          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                        />
                        <p className="text-xs text-gray-500 mt-1">Leave empty for no expiration</p>
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          Max Uses
                        </label>
                        <input
                          type="number"
                          min="1"
                          value={form.max_uses}
                          onChange={(e) => setForm({ ...form, max_uses: e.target.value })}
                          placeholder="Unlimited"
                          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                        />
                        <p className="text-xs text-gray-500 mt-1">Leave empty for unlimited</p>
                      </div>
                    </div>

                    {/* Custom Slug */}
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Custom Slug (optional)
                      </label>
                      <input
                        type="text"
                        value={form.slug}
                        onChange={(e) => setForm({ ...form, slug: e.target.value })}
                        placeholder="Auto-generated if empty"
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-blue-500 focus:border-blue-500"
                      />
                    </div>
                  </div>
                </div>

                <div className="bg-gray-50 px-4 py-3 sm:px-6 sm:flex sm:flex-row-reverse">
                  <button
                    type="submit"
                    disabled={formLoading}
                    className="w-full inline-flex justify-center rounded-md border border-transparent shadow-sm px-4 py-2 bg-blue-600 text-base font-medium text-white hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:ml-3 sm:w-auto sm:text-sm disabled:opacity-50"
                  >
                    {formLoading ? 'Creating...' : 'Create Link'}
                  </button>
                  <button
                    type="button"
                    onClick={() => {
                      setIsModalOpen(false);
                      resetForm();
                    }}
                    className="mt-3 w-full inline-flex justify-center rounded-md border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 sm:mt-0 sm:ml-3 sm:w-auto sm:text-sm"
                  >
                    Cancel
                  </button>
                </div>
              </form>
            </div>
          </div>
        </div>
      )}

    </div>
  );
};

export default PaymentsPage;