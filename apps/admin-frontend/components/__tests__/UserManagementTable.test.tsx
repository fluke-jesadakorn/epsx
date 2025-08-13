// Admin Frontend Components Tests - Presentation Layer (Admin UI Components)
// Tests for admin user management interface components
// Clean Architecture: Presentation Layer - UI behavior with mocked dependencies

import React from 'react'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { UserManagementTable } from '../UserManagementTable'
import { PermissionService } from '../../lib/permission-service'

// Mock PermissionService
jest.mock('../../lib/permission-service')
const mockPermissionService = PermissionService as jest.MockedClass<typeof PermissionService>

const mockUsers = [
  {
    id: '123',
    email: 'user1@example.com',
    permissionProfile: 'user-basic-001',
    createdAt: '2024-01-01T00:00:00Z',
    lastLoginAt: '2024-08-01T10:00:00Z',
    subscriptionStatus: 'active'
  },
  {
    id: '456',
    email: 'user2@example.com',
    permissionProfile: 'user-premium-002',
    createdAt: '2024-01-02T00:00:00Z',
    lastLoginAt: '2024-08-02T11:00:00Z',
    subscriptionStatus: 'trial'
  },
  {
    id: '789',
    email: 'moderator@example.com',
    permissionProfile: 'moderator-standard-003',
    createdAt: '2024-01-03T00:00:00Z',
    lastLoginAt: '2024-08-13T09:00:00Z',
    subscriptionStatus: 'active'
  }
]

const mockPermissionProfiles = [
  { id: 'user-basic-001', name: 'Basic User', permissions: ['basic-analytics'] },
  { id: 'user-premium-002', name: 'Premium User', permissions: ['basic-analytics', 'advanced-analytics'] },
  { id: 'moderator-standard-003', name: 'Moderator', permissions: ['user-management'] }
]

describe('UserManagementTable Component', () => {
  let mockPermissionServiceInstance: jest.Mocked<PermissionService>
  const user = userEvent.setup()

  beforeEach(() => {
    mockPermissionServiceInstance = {
      getAvailableProfiles: jest.fn(),
      updateUserPermissions: jest.fn(),
      processBulkPermissionUpdate: jest.fn(),
      validatePermissionUpgrade: jest.fn()
    } as any

    mockPermissionService.mockImplementation(() => mockPermissionServiceInstance)
    mockPermissionServiceInstance.getAvailableProfiles.mockResolvedValue(mockPermissionProfiles)
  })

  describe('Table Rendering', () => {
    test('renders user data in table format', () => {
      render(<UserManagementTable users={mockUsers} />)

      // Check table headers
      expect(screen.getByText('Email')).toBeInTheDocument()
      expect(screen.getByText('Permission Profile')).toBeInTheDocument()
      expect(screen.getByText('Last Login')).toBeInTheDocument()
      expect(screen.getByText('Actions')).toBeInTheDocument()

      // Check user data
      expect(screen.getByText('user1@example.com')).toBeInTheDocument()
      expect(screen.getByText('user2@example.com')).toBeInTheDocument()
      expect(screen.getByText('moderator@example.com')).toBeInTheDocument()
    })

    test('displays permission profiles with proper formatting', () => {
      render(<UserManagementTable users={mockUsers} />)

      expect(screen.getByText('Basic User')).toBeInTheDocument()
      expect(screen.getByText('Premium User')).toBeInTheDocument()
      expect(screen.getByText('Moderator')).toBeInTheDocument()
    })

    test('shows subscription status badges', () => {
      render(<UserManagementTable users={mockUsers} />)

      const activeStatuses = screen.getAllByText('Active')
      const trialStatus = screen.getByText('Trial')

      expect(activeStatuses).toHaveLength(2)
      expect(trialStatus).toBeInTheDocument()
    })

    test('formats last login dates correctly', () => {
      render(<UserManagementTable users={mockUsers} />)

      expect(screen.getByText('Aug 1, 2024')).toBeInTheDocument()
      expect(screen.getByText('Aug 2, 2024')).toBeInTheDocument()
      expect(screen.getByText('Today')).toBeInTheDocument() // For today's login
    })
  })

  describe('User Selection', () => {
    test('allows selecting individual users', async () => {
      const mockOnSelectionChange = jest.fn()
      
      render(<UserManagementTable users={mockUsers} onSelectionChange={mockOnSelectionChange} />)

      const firstCheckbox = screen.getAllByRole('checkbox')[1] // Skip header checkbox
      await user.click(firstCheckbox)

      expect(mockOnSelectionChange).toHaveBeenCalledWith(['123'])
    })

    test('allows selecting all users via header checkbox', async () => {
      const mockOnSelectionChange = jest.fn()
      
      render(<UserManagementTable users={mockUsers} onSelectionChange={mockOnSelectionChange} />)

      const headerCheckbox = screen.getAllByRole('checkbox')[0]
      await user.click(headerCheckbox)

      expect(mockOnSelectionChange).toHaveBeenCalledWith(['123', '456', '789'])
    })

    test('shows selected count when users are selected', async () => {
      render(<UserManagementTable users={mockUsers} selectedUserIds={['123', '456']} />)

      expect(screen.getByText('2 users selected')).toBeInTheDocument()
    })
  })

  describe('Bulk Operations', () => {
    test('shows bulk actions toolbar when users are selected', async () => {
      render(<UserManagementTable users={mockUsers} selectedUserIds={['123', '456']} />)

      expect(screen.getByText('Bulk Actions')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /update permissions/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /export selected/i })).toBeInTheDocument()
    })

    test('opens bulk permission update dialog', async () => {
      render(<UserManagementTable users={mockUsers} selectedUserIds={['123', '456']} />)

      const bulkUpdateButton = screen.getByRole('button', { name: /update permissions/i })
      await user.click(bulkUpdateButton)

      await waitFor(() => {
        expect(screen.getByText('Bulk Update Permissions')).toBeInTheDocument()
        expect(screen.getByText('Update 2 selected users')).toBeInTheDocument()
      })
    })

    test('processes bulk permission update', async () => {
      mockPermissionServiceInstance.processBulkPermissionUpdate.mockResolvedValue({
        summary: { successful: 2, failed: 0, total: 2 },
        failures: []
      })

      const mockOnUpdate = jest.fn()
      
      render(
        <UserManagementTable 
          users={mockUsers} 
          selectedUserIds={['123', '456']} 
          onUsersUpdate={mockOnUpdate}
        />
      )

      const bulkUpdateButton = screen.getByRole('button', { name: /update permissions/i })
      await user.click(bulkUpdateButton)

      await waitFor(() => {
        expect(screen.getByText('Bulk Update Permissions')).toBeInTheDocument()
      })

      // Select new permission profile
      const profileSelect = screen.getByRole('combobox', { name: /permission profile/i })
      await user.selectOptions(profileSelect, 'user-premium-002')

      // Enter reason
      const reasonInput = screen.getByLabelText(/reason/i)
      await user.type(reasonInput, 'Bulk upgrade for promotion')

      // Submit
      const submitButton = screen.getByRole('button', { name: /update permissions/i })
      await user.click(submitButton)

      await waitFor(() => {
        expect(mockPermissionServiceInstance.processBulkPermissionUpdate).toHaveBeenCalledWith({
          userIds: ['123', '456'],
          permissionProfile: 'user-premium-002',
          reason: 'Bulk upgrade for promotion'
        })
        expect(mockOnUpdate).toHaveBeenCalled()
      })
    })
  })

  describe('Individual User Actions', () => {
    test('opens permission edit dialog for individual user', async () => {
      render(<UserManagementTable users={mockUsers} />)

      const editButtons = screen.getAllByRole('button', { name: /edit permissions/i })
      await user.click(editButtons[0])

      await waitFor(() => {
        expect(screen.getByText('Edit User Permissions')).toBeInTheDocument()
        expect(screen.getByText('user1@example.com')).toBeInTheDocument()
      })
    })

    test('updates individual user permissions', async () => {
      mockPermissionServiceInstance.updateUserPermissions.mockResolvedValue({ success: true })
      mockPermissionServiceInstance.validatePermissionUpgrade.mockResolvedValue({
        isValid: true,
        reason: 'Valid upgrade'
      })

      const mockOnUpdate = jest.fn()
      
      render(<UserManagementTable users={mockUsers} onUsersUpdate={mockOnUpdate} />)

      const editButtons = screen.getAllByRole('button', { name: /edit permissions/i })
      await user.click(editButtons[0])

      await waitFor(() => {
        expect(screen.getByText('Edit User Permissions')).toBeInTheDocument()
      })

      // Change permission profile
      const profileSelect = screen.getByRole('combobox', { name: /permission profile/i })
      await user.selectOptions(profileSelect, 'user-premium-002')

      // Enter reason
      const reasonInput = screen.getByLabelText(/reason/i)
      await user.type(reasonInput, 'Individual upgrade')

      // Submit
      const submitButton = screen.getByRole('button', { name: /save changes/i })
      await user.click(submitButton)

      await waitFor(() => {
        expect(mockPermissionServiceInstance.updateUserPermissions).toHaveBeenCalledWith({
          userId: '123',
          permissionProfile: 'user-premium-002',
          reason: 'Individual upgrade'
        })
        expect(mockOnUpdate).toHaveBeenCalled()
      })
    })

    test('shows impersonation button for admin users', () => {
      render(<UserManagementTable users={mockUsers} currentAdminProfile="admin-full-004" />)

      const impersonateButtons = screen.getAllByRole('button', { name: /impersonate/i })
      expect(impersonateButtons).toHaveLength(3) // Should show for all users
    })

    test('disables impersonation for equal/higher permission users', () => {
      render(<UserManagementTable users={mockUsers} currentAdminProfile="moderator-standard-003" />)

      const impersonateButtons = screen.getAllByRole('button', { name: /impersonate/i })
      
      // Should be disabled for moderator (same level)
      const moderatorImpersonateButton = impersonateButtons[2]
      expect(moderatorImpersonateButton).toBeDisabled()
    })
  })

  describe('Filtering and Sorting', () => {
    test('allows filtering by permission profile', async () => {
      const mockOnFilter = jest.fn()
      
      render(<UserManagementTable users={mockUsers} onFilter={mockOnFilter} />)

      const filterSelect = screen.getByRole('combobox', { name: /filter by permission/i })
      await user.selectOptions(filterSelect, 'user-premium-002')

      expect(mockOnFilter).toHaveBeenCalledWith({
        permissionProfile: 'user-premium-002'
      })
    })

    test('allows searching users by email', async () => {
      const mockOnFilter = jest.fn()
      
      render(<UserManagementTable users={mockUsers} onFilter={mockOnFilter} />)

      const searchInput = screen.getByRole('textbox', { name: /search users/i })
      await user.type(searchInput, 'user1@example.com')

      // Should debounce the search
      await waitFor(() => {
        expect(mockOnFilter).toHaveBeenCalledWith({
          search: 'user1@example.com'
        })
      }, { timeout: 1000 })
    })

    test('allows sorting by different columns', async () => {
      const mockOnSort = jest.fn()
      
      render(<UserManagementTable users={mockUsers} onSort={mockOnSort} />)

      const emailHeader = screen.getByRole('button', { name: /sort by email/i })
      await user.click(emailHeader)

      expect(mockOnSort).toHaveBeenCalledWith({
        column: 'email',
        direction: 'asc'
      })

      // Click again for descending
      await user.click(emailHeader)

      expect(mockOnSort).toHaveBeenCalledWith({
        column: 'email',
        direction: 'desc'
      })
    })
  })

  describe('Loading and Error States', () => {
    test('shows skeleton loader when loading', () => {
      render(<UserManagementTable users={[]} loading={true} />)

      expect(screen.getAllByTestId('user-row-skeleton')).toHaveLength(5) // Default skeleton rows
    })

    test('shows empty state when no users', () => {
      render(<UserManagementTable users={[]} />)

      expect(screen.getByText('No users found')).toBeInTheDocument()
      expect(screen.getByText('Try adjusting your search or filter criteria')).toBeInTheDocument()
    })

    test('shows error state when data fetch fails', () => {
      render(<UserManagementTable users={[]} error="Failed to load users" />)

      expect(screen.getByText('Error loading users')).toBeInTheDocument()
      expect(screen.getByText('Failed to load users')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument()
    })
  })

  describe('Accessibility', () => {
    test('has proper ARIA labels and roles', () => {
      render(<UserManagementTable users={mockUsers} />)

      expect(screen.getByRole('table')).toHaveAttribute('aria-label', 'User management table')
      expect(screen.getAllByRole('columnheader')).toHaveLength(5)
      expect(screen.getAllByRole('row')).toHaveLength(4) // 3 users + header
    })

    test('supports keyboard navigation for actions', async () => {
      render(<UserManagementTable users={mockUsers} />)

      const firstEditButton = screen.getAllByRole('button', { name: /edit permissions/i })[0]
      
      // Tab to focus the button
      await user.tab()
      expect(firstEditButton).toHaveFocus()

      // Press Enter to activate
      fireEvent.keyDown(firstEditButton, { key: 'Enter', code: 'Enter' })
      
      await waitFor(() => {
        expect(screen.getByText('Edit User Permissions')).toBeInTheDocument()
      })
    })

    test('provides screen reader feedback for selection changes', async () => {
      render(<UserManagementTable users={mockUsers} />)

      const firstCheckbox = screen.getAllByRole('checkbox')[1]
      await user.click(firstCheckbox)

      // Should announce selection change
      expect(screen.getByText('1 user selected')).toBeInTheDocument()
    })
  })
})