// Admin Frontend Components Tests - Presentation Layer (Admin UI Components)
// Tests for permission profile management interface components
// Clean Architecture: Presentation Layer - UI behavior with mocked dependencies

import React from 'react'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { PermissionProfileManager } from '../PermissionProfileManager'
import { AdminApiClient } from '../../lib/admin-api-client'

// Mock AdminApiClient
jest.mock('../../lib/admin-api-client')
const mockAdminApiClient = AdminApiClient as jest.MockedClass<typeof AdminApiClient>

const mockPermissionProfiles = [
  {
    id: 'user-basic-001',
    name: 'Basic User',
    description: 'Basic trading features',
    permissions: ['basic-analytics', 'stock-ranking'],
    userCount: 45,
    isSystemProfile: true
  },
  {
    id: 'user-premium-002',
    name: 'Premium User',
    description: 'Premium features + advanced analytics',
    permissions: ['basic-analytics', 'stock-ranking', 'advanced-analytics', 'export-data'],
    userCount: 23,
    isSystemProfile: true
  },
  {
    id: 'custom-profile-001',
    name: 'Custom Department Profile',
    description: 'Custom profile for marketing department',
    permissions: ['basic-analytics', 'user-insights'],
    userCount: 8,
    isSystemProfile: false,
    createdBy: 'admin-123',
    createdAt: '2024-08-01T00:00:00Z'
  }
]

const mockAvailablePermissions = [
  { id: 'basic-analytics', name: 'Basic Analytics', category: 'Analytics' },
  { id: 'advanced-analytics', name: 'Advanced Analytics', category: 'Analytics' },
  { id: 'stock-ranking', name: 'Stock Ranking', category: 'Trading' },
  { id: 'export-data', name: 'Export Data', category: 'Data' },
  { id: 'user-insights', name: 'User Insights', category: 'Analytics' },
  { id: 'user-management', name: 'User Management', category: 'Admin' }
]

describe('PermissionProfileManager Component', () => {
  let mockApiClientInstance: jest.Mocked<AdminApiClient>
  const user = userEvent.setup()

  beforeEach(() => {
    mockApiClientInstance = {
      fetchPermissionProfiles: jest.fn(),
      createPermissionProfile: jest.fn(),
      updatePermissionProfile: jest.fn(),
      deletePermissionProfile: jest.fn(),
      fetchAvailablePermissions: jest.fn()
    } as any

    mockAdminApiClient.mockImplementation(() => mockApiClientInstance)
    mockApiClientInstance.fetchPermissionProfiles.mockResolvedValue(mockPermissionProfiles)
    mockApiClientInstance.fetchAvailablePermissions.mockResolvedValue(mockAvailablePermissions)
  })

  describe('Profile List Display', () => {
    test('renders all permission profiles', async () => {
      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByText('Basic User')).toBeInTheDocument()
        expect(screen.getByText('Premium User')).toBeInTheDocument()
        expect(screen.getByText('Custom Department Profile')).toBeInTheDocument()
      })
    })

    test('shows user counts for each profile', async () => {
      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByText('45 users')).toBeInTheDocument()
        expect(screen.getByText('23 users')).toBeInTheDocument()
        expect(screen.getByText('8 users')).toBeInTheDocument()
      })
    })

    test('displays system vs custom profile badges', async () => {
      render(<PermissionProfileManager />)

      await waitFor(() => {
        const systemBadges = screen.getAllByText('System')
        const customBadges = screen.getAllByText('Custom')
        
        expect(systemBadges).toHaveLength(2)
        expect(customBadges).toHaveLength(1)
      })
    })

    test('shows permissions list for each profile', async () => {
      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByText('basic-analytics')).toBeInTheDocument()
        expect(screen.getByText('advanced-analytics')).toBeInTheDocument()
        expect(screen.getByText('export-data')).toBeInTheDocument()
      })
    })
  })

  describe('Create New Profile', () => {
    test('opens create profile dialog', async () => {
      render(<PermissionProfileManager />)

      const createButton = screen.getByRole('button', { name: /create profile/i })
      await user.click(createButton)

      await waitFor(() => {
        expect(screen.getByText('Create Permission Profile')).toBeInTheDocument()
        expect(screen.getByLabelText(/profile name/i)).toBeInTheDocument()
        expect(screen.getByLabelText(/description/i)).toBeInTheDocument()
      })
    })

    test('creates new custom profile with selected permissions', async () => {
      mockApiClientInstance.createPermissionProfile.mockResolvedValue({
        id: 'new-profile-001',
        name: 'New Test Profile',
        description: 'Test description',
        permissions: ['basic-analytics', 'stock-ranking']
      })

      render(<PermissionProfileManager />)

      const createButton = screen.getByRole('button', { name: /create profile/i })
      await user.click(createButton)

      await waitFor(() => {
        expect(screen.getByText('Create Permission Profile')).toBeInTheDocument()
      })

      // Fill in form
      const nameInput = screen.getByLabelText(/profile name/i)
      const descriptionInput = screen.getByLabelText(/description/i)
      
      await user.type(nameInput, 'New Test Profile')
      await user.type(descriptionInput, 'Test description')

      // Select permissions
      const basicAnalyticsCheckbox = screen.getByLabelText(/basic analytics/i)
      const stockRankingCheckbox = screen.getByLabelText(/stock ranking/i)
      
      await user.click(basicAnalyticsCheckbox)
      await user.click(stockRankingCheckbox)

      // Submit
      const submitButton = screen.getByRole('button', { name: /create profile/i })
      await user.click(submitButton)

      await waitFor(() => {
        expect(mockApiClientInstance.createPermissionProfile).toHaveBeenCalledWith({
          name: 'New Test Profile',
          description: 'Test description',
          permissions: ['basic-analytics', 'stock-ranking']
        })
      })
    })

    test('validates required fields', async () => {
      render(<PermissionProfileManager />)

      const createButton = screen.getByRole('button', { name: /create profile/i })
      await user.click(createButton)

      await waitFor(() => {
        expect(screen.getByText('Create Permission Profile')).toBeInTheDocument()
      })

      // Try to submit without filling required fields
      const submitButton = screen.getByRole('button', { name: /create profile/i })
      await user.click(submitButton)

      await waitFor(() => {
        expect(screen.getByText('Profile name is required')).toBeInTheDocument()
        expect(screen.getByText('At least one permission must be selected')).toBeInTheDocument()
      })
    })
  })

  describe('Edit Profile', () => {
    test('opens edit dialog for custom profiles', async () => {
      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByText('Custom Department Profile')).toBeInTheDocument()
      })

      const editButtons = screen.getAllByRole('button', { name: /edit/i })
      const customProfileEditButton = editButtons.find(button => 
        button.closest('[data-profile-id="custom-profile-001"]')
      )
      
      await user.click(customProfileEditButton!)

      await waitFor(() => {
        expect(screen.getByText('Edit Permission Profile')).toBeInTheDocument()
        expect(screen.getByDisplayValue('Custom Department Profile')).toBeInTheDocument()
      })
    })

    test('disables edit for system profiles', async () => {
      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByText('Basic User')).toBeInTheDocument()
      })

      const editButtons = screen.getAllByRole('button', { name: /edit/i })
      const systemProfileEditButton = editButtons.find(button => 
        button.closest('[data-profile-id="user-basic-001"]')
      )
      
      expect(systemProfileEditButton).toBeDisabled()
    })

    test('updates profile with new permissions', async () => {
      mockApiClientInstance.updatePermissionProfile.mockResolvedValue({
        success: true
      })

      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByText('Custom Department Profile')).toBeInTheDocument()
      })

      const editButtons = screen.getAllByRole('button', { name: /edit/i })
      const customProfileEditButton = editButtons.find(button => 
        button.closest('[data-profile-id="custom-profile-001"]')
      )
      
      await user.click(customProfileEditButton!)

      await waitFor(() => {
        expect(screen.getByText('Edit Permission Profile')).toBeInTheDocument()
      })

      // Add a new permission
      const exportDataCheckbox = screen.getByLabelText(/export data/i)
      await user.click(exportDataCheckbox)

      // Save changes
      const saveButton = screen.getByRole('button', { name: /save changes/i })
      await user.click(saveButton)

      await waitFor(() => {
        expect(mockApiClientInstance.updatePermissionProfile).toHaveBeenCalledWith(
          'custom-profile-001',
          {
            name: 'Custom Department Profile',
            description: 'Custom profile for marketing department',
            permissions: expect.arrayContaining(['basic-analytics', 'user-insights', 'export-data'])
          }
        )
      })
    })
  })

  describe('Delete Profile', () => {
    test('opens delete confirmation dialog', async () => {
      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByText('Custom Department Profile')).toBeInTheDocument()
      })

      const deleteButtons = screen.getAllByRole('button', { name: /delete/i })
      const customProfileDeleteButton = deleteButtons.find(button => 
        button.closest('[data-profile-id="custom-profile-001"]')
      )
      
      await user.click(customProfileDeleteButton!)

      await waitFor(() => {
        expect(screen.getByText('Delete Permission Profile')).toBeInTheDocument()
        expect(screen.getByText(/are you sure you want to delete/i)).toBeInTheDocument()
        expect(screen.getByText('8 users will be affected')).toBeInTheDocument()
      })
    })

    test('prevents deletion of system profiles', async () => {
      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByText('Basic User')).toBeInTheDocument()
      })

      const deleteButtons = screen.getAllByRole('button', { name: /delete/i })
      const systemProfileDeleteButton = deleteButtons.find(button => 
        button.closest('[data-profile-id="user-basic-001"]')
      )
      
      expect(systemProfileDeleteButton).toBeDisabled()
    })

    test('deletes profile after confirmation', async () => {
      mockApiClientInstance.deletePermissionProfile.mockResolvedValue({
        success: true
      })

      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByText('Custom Department Profile')).toBeInTheDocument()
      })

      const deleteButtons = screen.getAllByRole('button', { name: /delete/i })
      const customProfileDeleteButton = deleteButtons.find(button => 
        button.closest('[data-profile-id="custom-profile-001"]')
      )
      
      await user.click(customProfileDeleteButton!)

      await waitFor(() => {
        expect(screen.getByText('Delete Permission Profile')).toBeInTheDocument()
      })

      // Confirm deletion
      const confirmButton = screen.getByRole('button', { name: /delete profile/i })
      await user.click(confirmButton)

      await waitFor(() => {
        expect(mockApiClientInstance.deletePermissionProfile).toHaveBeenCalledWith('custom-profile-001')
      })
    })
  })

  describe('Permission Categories', () => {
    test('groups permissions by category', async () => {
      render(<PermissionProfileManager />)

      const createButton = screen.getByRole('button', { name: /create profile/i })
      await user.click(createButton)

      await waitFor(() => {
        expect(screen.getByText('Analytics')).toBeInTheDocument()
        expect(screen.getByText('Trading')).toBeInTheDocument()
        expect(screen.getByText('Data')).toBeInTheDocument()
        expect(screen.getByText('Admin')).toBeInTheDocument()
      })
    })

    test('allows selecting all permissions in a category', async () => {
      render(<PermissionProfileManager />)

      const createButton = screen.getByRole('button', { name: /create profile/i })
      await user.click(createButton)

      await waitFor(() => {
        expect(screen.getByText('Analytics')).toBeInTheDocument()
      })

      // Click "Select All" for Analytics category
      const selectAllAnalytics = screen.getByRole('button', { name: /select all analytics/i })
      await user.click(selectAllAnalytics)

      // Verify analytics permissions are selected
      expect(screen.getByLabelText(/basic analytics/i)).toBeChecked()
      expect(screen.getByLabelText(/advanced analytics/i)).toBeChecked()
      expect(screen.getByLabelText(/user insights/i)).toBeChecked()
    })
  })

  describe('Search and Filter', () => {
    test('filters profiles by name', async () => {
      render(<PermissionProfileManager />)

      const searchInput = screen.getByRole('textbox', { name: /search profiles/i })
      await user.type(searchInput, 'Custom')

      await waitFor(() => {
        expect(screen.getByText('Custom Department Profile')).toBeInTheDocument()
        expect(screen.queryByText('Basic User')).not.toBeInTheDocument()
        expect(screen.queryByText('Premium User')).not.toBeInTheDocument()
      })
    })

    test('filters by system vs custom profiles', async () => {
      render(<PermissionProfileManager />)

      const filterSelect = screen.getByRole('combobox', { name: /filter by type/i })
      await user.selectOptions(filterSelect, 'custom')

      await waitFor(() => {
        expect(screen.getByText('Custom Department Profile')).toBeInTheDocument()
        expect(screen.queryByText('Basic User')).not.toBeInTheDocument()
        expect(screen.queryByText('Premium User')).not.toBeInTheDocument()
      })
    })
  })

  describe('Loading and Error States', () => {
    test('shows loading skeleton while fetching profiles', () => {
      mockApiClientInstance.fetchPermissionProfiles.mockImplementation(
        () => new Promise(resolve => setTimeout(() => resolve(mockPermissionProfiles), 1000))
      )

      render(<PermissionProfileManager />)

      expect(screen.getAllByTestId('profile-skeleton')).toHaveLength(3) // Default skeleton count
    })

    test('shows error state when fetch fails', async () => {
      mockApiClientInstance.fetchPermissionProfiles.mockRejectedValue(
        new Error('Failed to load profiles')
      )

      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByText('Error loading permission profiles')).toBeInTheDocument()
        expect(screen.getByText('Failed to load profiles')).toBeInTheDocument()
        expect(screen.getByRole('button', { name: /retry/i })).toBeInTheDocument()
      })
    })
  })

  describe('Accessibility', () => {
    test('has proper ARIA labels and roles', async () => {
      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByRole('main')).toHaveAttribute('aria-label', 'Permission profile management')
        expect(screen.getByRole('button', { name: /create profile/i })).toBeInTheDocument()
      })
    })

    test('supports keyboard navigation', async () => {
      render(<PermissionProfileManager />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /create profile/i })).toBeInTheDocument()
      })

      // Tab through create button
      await user.tab()
      expect(screen.getByRole('button', { name: /create profile/i })).toHaveFocus()

      // Enter should open dialog
      fireEvent.keyDown(screen.getByRole('button', { name: /create profile/i }), {
        key: 'Enter',
        code: 'Enter'
      })

      await waitFor(() => {
        expect(screen.getByText('Create Permission Profile')).toBeInTheDocument()
      })
    })
  })
})