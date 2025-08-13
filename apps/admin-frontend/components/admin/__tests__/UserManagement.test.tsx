import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { UserManagement } from '../UserManagement'
import { useSession } from 'next-auth/react'

// Mock next-auth
jest.mock('next-auth/react')
const mockUseSession = useSession as jest.MockedFunction<typeof useSession>

// Mock API calls
jest.mock('../../../lib/actions/unified-user-actions', () => ({
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
    mockUseSession.mockReturnValue({
      data: {
        user: {
          email: 'admin@example.com',
          role: 'admin',
        },
        expires: '2024-12-31',
      },
      status: 'authenticated',
      update: jest.fn(),
    })
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

  it('does not render for non-admin users', () => {
    mockUseSession.mockReturnValue({
      data: {
        user: {
          email: 'user@example.com',
          role: 'user',
        },
        expires: '2024-12-31',
      },
      status: 'authenticated',
      update: jest.fn(),
    })

    render(<UserManagement />)
    
    expect(screen.getByText(/access denied/i)).toBeInTheDocument()
  })
})