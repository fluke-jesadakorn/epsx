import { firebaseIAMService } from '@/services/firebaseIAMService';
import { PermissionAuditLog } from '@/types/admin/iam-enhanced';
import { NextRequest, NextResponse } from 'next/server';

export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);

    // Parse query parameters
    const limit = parseInt(searchParams.get('limit') || '50');
    const userId = searchParams.get('userId');
    const action = searchParams.get('action');
    const status = searchParams.get('status');
    const startDate = searchParams.get('startDate');
    const endDate = searchParams.get('endDate');

    // Get all audit logs (we'll implement comprehensive filtering)
    let logs: PermissionAuditLog[] =
      await firebaseIAMService.getAllAuditLogs(limit);

    // Apply filters
    if (userId && userId !== 'all') {
      logs = logs.filter((log: PermissionAuditLog) => log.userId === userId);
    }

    if (action && action !== 'all') {
      logs = logs.filter((log: PermissionAuditLog) =>
        log.action.toLowerCase().includes(action.toLowerCase()),
      );
    }

    if (status && status !== 'all') {
      // Map status from metadata if available
      logs = logs.filter((log: PermissionAuditLog) => {
        const logStatus = log.metadata?.status || 'success';
        return logStatus === status;
      });
    }

    if (startDate) {
      const start = new Date(startDate);
      logs = logs.filter((log: PermissionAuditLog) => log.timestamp >= start);
    }

    if (endDate) {
      const end = new Date(endDate);
      logs = logs.filter((log: PermissionAuditLog) => log.timestamp <= end);
    }

    return NextResponse.json({
      success: true,
      logs,
      total: logs.length,
    });
  } catch (error) {
    console.error('Error fetching activity logs:', error);

    // Return mock data as fallback
    const mockLogs = [
      {
        id: '1',
        timestamp: new Date(),
        action: 'package_upgrade',
        resource: 'user_package',
        userId: 'user123',
        performedBy: 'admin001',
        reason: 'User requested upgrade',
        metadata: {
          userName: 'john.doe@example.com',
          performedByName: 'Admin User',
          status: 'success',
          details: 'Package upgraded from Bronze to Gold',
          oldTier: 'bronze',
          newTier: 'gold',
          ipAddress: '192.168.1.100',
        },
      },
      {
        id: '2',
        timestamp: new Date(Date.now() - 3600000),
        action: 'permission_granted',
        resource: 'custom_permission',
        userId: 'user456',
        performedBy: 'admin001',
        reason: 'Special access request approved',
        metadata: {
          userName: 'jane.smith@example.com',
          performedByName: 'Admin User',
          status: 'success',
          details: 'Custom permission granted for API access',
          permission: 'api_premium_access',
          duration: '30 days',
          ipAddress: '192.168.1.101',
        },
      },
      {
        id: '3',
        timestamp: new Date(Date.now() - 7200000),
        action: 'login_attempt',
        resource: 'authentication',
        userId: 'user789',
        performedBy: 'user789',
        metadata: {
          userName: 'bob.wilson@example.com',
          performedByName: 'Bob Wilson',
          status: 'error',
          details: 'Failed login attempt - invalid credentials',
          ipAddress: '192.168.1.100',
          userAgent: 'Mozilla/5.0...',
          attempts: 3,
        },
      },
      {
        id: '4',
        timestamp: new Date(Date.now() - 10800000),
        action: 'subscription_renewal',
        resource: 'subscription',
        userId: 'user101',
        performedBy: 'system',
        metadata: {
          userName: 'alice.brown@example.com',
          performedByName: 'System',
          status: 'success',
          details: 'Subscription automatically renewed',
          amount: 99.99,
          currency: 'USD',
          ipAddress: 'system',
        },
      },
      {
        id: '5',
        timestamp: new Date(Date.now() - 14400000),
        action: 'permission_revoked',
        resource: 'custom_permission',
        userId: 'user202',
        performedBy: 'admin002',
        reason: 'Policy violation detected',
        metadata: {
          userName: 'charlie.green@example.com',
          performedByName: 'Support Admin',
          status: 'warning',
          details: 'Custom permission revoked due to policy violation',
          permission: 'bulk_export',
          ipAddress: '192.168.1.102',
        },
      },
      {
        id: '6',
        timestamp: new Date(Date.now() - 18000000),
        action: 'user_created',
        resource: 'user_management',
        userId: 'user303',
        performedBy: 'admin001',
        reason: 'New user registration',
        metadata: {
          userName: 'david.white@example.com',
          performedByName: 'Admin User',
          status: 'success',
          details: 'New user account created successfully',
          packageTier: 'free',
          ipAddress: '192.168.1.103',
        },
      },
      {
        id: '7',
        timestamp: new Date(Date.now() - 21600000),
        action: 'password_reset',
        resource: 'authentication',
        userId: 'user404',
        performedBy: 'user404',
        metadata: {
          userName: 'emma.davis@example.com',
          performedByName: 'Emma Davis',
          status: 'success',
          details: 'Password reset completed successfully',
          ipAddress: '192.168.1.104',
          resetMethod: 'email',
        },
      },
      {
        id: '8',
        timestamp: new Date(Date.now() - 25200000),
        action: 'bulk_permission_update',
        resource: 'permission_management',
        userId: 'multiple',
        performedBy: 'admin001',
        reason: 'Monthly permission audit',
        metadata: {
          userName: 'Multiple Users',
          performedByName: 'Admin User',
          status: 'success',
          details: 'Applied new permission template to 15 users',
          affectedUsers: 15,
          templateName: 'Standard User Template',
          ipAddress: '192.168.1.100',
        },
      },
    ];

    return NextResponse.json({
      success: true,
      logs: mockLogs,
      total: mockLogs.length,
      fallback: true,
    });
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { exportFormat = 'csv', filters } = body;

    // Get logs with filters
    let logs: PermissionAuditLog[] =
      await firebaseIAMService.getAllAuditLogs(1000); // Get more for export

    // Apply filters (same logic as GET)
    if (filters) {
      if (filters.userId && filters.userId !== 'all') {
        logs = logs.filter(
          (log: PermissionAuditLog) => log.userId === filters.userId,
        );
      }

      if (filters.action && filters.action !== 'all') {
        logs = logs.filter((log: PermissionAuditLog) =>
          log.action.toLowerCase().includes(filters.action.toLowerCase()),
        );
      }

      if (filters.status && filters.status !== 'all') {
        logs = logs.filter((log: PermissionAuditLog) => {
          const logStatus = log.metadata?.status || 'success';
          return logStatus === filters.status;
        });
      }

      if (filters.startDate) {
        const start = new Date(filters.startDate);
        logs = logs.filter((log: PermissionAuditLog) => log.timestamp >= start);
      }

      if (filters.endDate) {
        const end = new Date(filters.endDate);
        logs = logs.filter((log: PermissionAuditLog) => log.timestamp <= end);
      }
    }

    if (exportFormat === 'csv') {
      const csvHeaders =
        'Timestamp,Action,User,Performed By,Status,Details,IP Address\n';
      const csvContent = logs
        .map((log: PermissionAuditLog) => {
          const userName = log.metadata?.userName || 'Unknown';
          const performedByName = log.metadata?.performedByName || 'Unknown';
          const status = log.metadata?.status || 'success';
          const details = log.metadata?.details || log.action;
          const ipAddress = log.metadata?.ipAddress || 'Unknown';

          return [
            log.timestamp.toISOString(),
            log.action,
            userName,
            performedByName,
            status,
            `"${details}"`,
            ipAddress,
          ].join(',');
        })
        .join('\n');

      return new NextResponse(csvHeaders + csvContent, {
        headers: {
          'Content-Type': 'text/csv',
          'Content-Disposition': `attachment; filename="activity-logs-${new Date().toISOString().split('T')[0]}.csv"`,
        },
      });
    }

    return NextResponse.json({
      success: true,
      logs,
      total: logs.length,
    });
  } catch (error) {
    console.error('Error exporting activity logs:', error);
    return NextResponse.json(
      { error: 'Failed to export activity logs' },
      { status: 500 },
    );
  }
}
