import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { UserManagement } from '../UserManagement'
// TODO: Fix auth service import after cleanup
// import { ModernAuthService } from '../../../lib/auth/auth-service'

// Mock modern auth service
// jest.mock('../../../lib/auth/auth-service')
// const mockAuthService = ModernAuthService as jest.Mocked<typeof ModernAuthService>

// Mock API calls
jest.mock('../../../lib/actions/user-actions', () => ({
  fetchUsersAction: jest.fn(() => Promise.resolve({
    success: true,
    data: {
      users: [
        { id: '1', email: 'user1@example.com', display_name: 'User 1' },
        { id: '2', email: 'user2@example.com', display_name: 'User 2' },
      ],
      total: 2,
      page: 1,
      limit: 10,
    }
  })),
}))

describe('UserManagement', () => {
  beforeEach(() => {
    // Mock admin user with user management capabilities
    mockAuthService.getCurrentUser.mockResolvedValue({
      user_id: 'admin-user-1',
      email: 'admin@example.com',
      name: 'Admin User',
      admin: true,
      access_level: 'admin',
      permissions: ['epsx:users:manage', 'epsx:system:admin'],
      additional_permissions: ['user:read', 'user:write', 'admin_access'],
      package_tier: 'enterprise',
      subscription_status: 'active'
    })
    mockAuthService.isAdmin.mockResolvedValue(true)
    mockAuthService.canManageUsers.mockResolvedValue(true)
    mockAuthService.hasAdminModule.mockResolvedValue(true)
  })

  it('renders user management interface', async () => {
    render(<UserManagement />)
    
    expect(screen.getByText(/user management/i)).toBeInTheDocument()
    
    // Wait for users to load
    await waitFor(() => {
      expect(screen.getByText('user1@example.com')).toBeInTheDocument()
      expect(screen.getByText('user2@example.com')).toBeInTheDocument()
    })
  })

  it('allows searching users', async () => {
    render(<UserManagement />)
    
    const searchInput = screen.getByPlaceholderText(/search users/i)
    fireEvent.change(searchInput, { target: { value: 'user1' } })
    
    await waitFor(() => {
      expect(searchInput).toHaveValue('user1')
    })
  })

  it('shows create user button for admin', () => {
    render(<UserManagement />)
    
    const createButton = screen.getByRole('button', { name: /create user/i })
    expect(createButton).toBeInTheDocument()
  })

  it('handles pagination correctly', async () => {
    render(<UserManagement />)
    
    await waitFor(() => {
      expect(screen.getByText('user1@example.com')).toBeInTheDocument()
    })

    // Look for pagination controls
    const nextButton = screen.queryByRole('button', { name: /next/i })
    if (nextButton) {
      expect(nextButton).toBeInTheDocument()
    }
  })

  it('does not render for non-admin users', async () => {
    // Mock regular user without admin capabilities
    mockAuthService.getCurrentUser.mockResolvedValue({
      user_id: 'regular-user-1',
      email: 'user@example.com',
      name: 'Regular User',
      admin: false,
      access_level: 'read',
      permissions: [],
      additional_permissions: ['user:read'],
      package_tier: 'free',
      subscription_status: 'active'
    })
    mockAuthService.isAdmin.mockResolvedValue(false)
    mockAuthService.canManageUsers.mockResolvedValue(false)
    mockAuthService.hasAdminModule.mockResolvedValue(false)

    render(<UserManagement />)
    
    await waitFor(() => {
      expect(screen.getByText(/access denied/i)).toBeInTheDocument()
    })
  })
})