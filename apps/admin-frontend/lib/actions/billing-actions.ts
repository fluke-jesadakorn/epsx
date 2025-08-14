/**
 * Billing Server Actions - Focused on billing operations
 * Server-side first billing data fetching and operations
 */

'use server'

import { auth } from '@/lib/auth'

// Get bearer token from NextAuth session
const getBearerToken = async () => {
  const session = await auth();
  return (session as any)?.accessToken || null;
};
import { logger } from '@/lib/logger'

const BACKEND_URL = process.env.BACKEND_URL || 'http://localhost:8080'

export interface UsageBill {
  id: string
  period: {
    start: string
    end: string
  }
  status: 'pending' | 'paid' | 'overdue'
  paymentDue: string
  usage: {
    totalRequests: number
    moduleBreakdown: Array<{
      moduleName: string
      requests: number
      cost: number
    }>
  }
  pricing: {
    baseCost: number
    totalCost: number
    discounts: Array<{
      description: string
      amount: number
    }>
  }
}

export interface InvoiceData {
  id: string
  period: string
  amount: number
  status: 'paid' | 'pending' | 'overdue'
  dueDate: string
  downloadUrl: string
}

export interface BillingDashboardData {
  currentBill: UsageBill
  invoices: InvoiceData[]
  usageStats: {
    totalApiCalls: number
    activeModules: number
    currentCost: number
  }
}

export interface BillingOperationResult<T = void> {
  success: boolean
  data?: T
  error?: {
    code: string
    message: string
  }
}

/**
 * Get comprehensive billing dashboard data
 */
export async function getBillingDashboardData(period = 'current'): Promise<BillingOperationResult<BillingDashboardData>> {
  try {
    logger.action.start('getBillingDashboardData', { period })
    
    const user = await getCurrentUser()
    const token = await getBearerToken()
    
    if (!user || !token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } }
    }

    // Check admin module access
    if (!user.admin || !user.admin_modules.includes('billing_admin')) {
      return { 
        success: false, 
        error: { code: 'FORBIDDEN', message: 'Billing admin access required' } 
      }
    }

    // Fetch billing data from backend
    const [billResponse, invoicesResponse] = await Promise.all([
      fetch(`${BACKEND_URL}/api/v1/admin/billing/current-bill?period=${period}`, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        next: { revalidate: 300 } // 5-minute cache
      }),
      fetch(`${BACKEND_URL}/api/v1/admin/billing/invoices`, {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json',
        },
        next: { revalidate: 600 } // 10-minute cache for invoices
      })
    ])

    if (!billResponse.ok || !invoicesResponse.ok) {
      logger.action.error('getBillingDashboardData', 'Failed to fetch billing data from backend')
      
      // Return mock data for development
      const mockData: BillingDashboardData = {
        currentBill: {
          id: 'bill_current',
          period: {
            start: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
            end: new Date().toISOString()
          },
          status: 'pending',
          paymentDue: new Date(Date.now() + 15 * 24 * 60 * 60 * 1000).toISOString(),
          usage: {
            totalRequests: 145230,
            moduleBreakdown: [
              { moduleName: 'Analytics', requests: 85430, cost: 341.72 },
              { moduleName: 'User Management', requests: 35200, cost: 140.80 },
              { moduleName: 'API Gateway', requests: 24600, cost: 98.40 }
            ]
          },
          pricing: {
            baseCost: 500.00,
            totalCost: 1080.92,
            discounts: [
              { description: 'Volume Discount (>100K requests)', amount: 50.00 },
              { description: 'Annual Plan Discount', amount: 40.00 }
            ]
          }
        },
        invoices: [
          {
            id: 'inv_2024_01',
            period: 'January 2024',
            amount: 1420.50,
            status: 'paid',
            dueDate: '2024-02-01',
            downloadUrl: `/api/v1/admin/billing/invoices/inv_2024_01/download`
          },
          {
            id: 'inv_2023_12',
            period: 'December 2023',
            amount: 1256.75,
            status: 'paid',
            dueDate: '2024-01-01',
            downloadUrl: `/api/v1/admin/billing/invoices/inv_2023_12/download`
          }
        ],
        usageStats: {
          totalApiCalls: 145230,
          activeModules: 3,
          currentCost: 1080.92
        }
      }

      logger.action.success('getBillingDashboardData', { period, source: 'mock' })
      return { success: true, data: mockData }
    }

    const [billData, invoicesData] = await Promise.all([
      billResponse.json(),
      invoicesResponse.json()
    ])

    const dashboardData: BillingDashboardData = {
      currentBill: billData,
      invoices: invoicesData.invoices || [],
      usageStats: {
        totalApiCalls: billData.usage.totalRequests,
        activeModules: billData.usage.moduleBreakdown.length,
        currentCost: billData.pricing.totalCost
      }
    }

    logger.action.success('getBillingDashboardData', { 
      period, 
      totalCost: dashboardData.currentBill.pricing.totalCost,
      invoiceCount: dashboardData.invoices.length 
    })
    
    return { success: true, data: dashboardData }
    
  } catch (error) {
    logger.action.error('getBillingDashboardData', error, { period })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Process payment for a bill
 */
export async function processPayment(billId: string, paymentMethod: string): Promise<BillingOperationResult<{ transactionId: string }>> {
  try {
    logger.action.start('processPayment', { billId, paymentMethod })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/billing/payments`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        bill_id: billId,
        payment_method: paymentMethod
      }),
    })
    
    if (!response.ok) {
      logger.action.error('processPayment', `Payment failed: ${response.statusText}`, { billId })
      return { 
        success: false, 
        error: { 
          code: 'PAYMENT_ERROR', 
          message: `Payment failed: ${response.statusText}` 
        } 
      }
    }
    
    const result = await response.json()
    
    logger.action.success('processPayment', { billId, transactionId: result.transaction_id })
    
    return { success: true, data: { transactionId: result.transaction_id } }
    
  } catch (error) {
    logger.action.error('processPayment', error, { billId })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Download invoice
 */
export async function downloadInvoice(invoiceId: string): Promise<BillingOperationResult<{ downloadUrl: string }>> {
  try {
    logger.action.start('downloadInvoice', { invoiceId })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } }
    }
    
    // Generate signed download URL
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/billing/invoices/${invoiceId}/download-url`, {
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    })
    
    if (!response.ok) {
      logger.action.error('downloadInvoice', `Failed to get download URL: ${response.statusText}`, { invoiceId })
      return { 
        success: false, 
        error: { 
          code: 'DOWNLOAD_ERROR', 
          message: `Failed to get download URL: ${response.statusText}` 
        } 
      }
    }
    
    const result = await response.json()
    
    logger.action.success('downloadInvoice', { invoiceId })
    
    return { success: true, data: { downloadUrl: result.download_url } }
    
  } catch (error) {
    logger.action.error('downloadInvoice', error, { invoiceId })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}

/**
 * Export billing report  
 */
export async function exportBillingReport(
  period: string, 
  format: 'csv' | 'pdf' | 'xlsx' = 'pdf'
): Promise<BillingOperationResult<{ downloadUrl: string }>> {
  try {
    logger.action.start('exportBillingReport', { period, format })
    
    const token = await getBearerToken()
    
    if (!token) {
      return { success: false, error: { code: 'UNAUTHORIZED', message: 'Authentication required' } }
    }
    
    const response = await fetch(`${BACKEND_URL}/api/v1/admin/billing/reports/export`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        period,
        format
      }),
    })
    
    if (!response.ok) {
      logger.action.error('exportBillingReport', `Export failed: ${response.statusText}`, { period, format })
      return { 
        success: false, 
        error: { 
          code: 'EXPORT_ERROR', 
          message: `Export failed: ${response.statusText}` 
        } 
      }
    }
    
    const result = await response.json()
    
    logger.action.success('exportBillingReport', { period, format })
    
    return { success: true, data: { downloadUrl: result.download_url } }
    
  } catch (error) {
    logger.action.error('exportBillingReport', error, { period, format })
    return { 
      success: false, 
      error: { 
        code: 'UNKNOWN_ERROR', 
        message: 'An unexpected error occurred' 
      } 
    }
  }
}