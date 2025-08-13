import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { useSession, signIn } from 'next-auth/react'
import { OIDCLoginButton } from '../OIDCLoginButton'

// Mock next-auth
jest.mock('next-auth/react')
const mockUseSession = useSession as jest.MockedFunction<typeof useSession>
const mockSignIn = signIn as jest.MockedFunction<typeof signIn>

describe('OIDCLoginButton', () => {
  beforeEach(() => {
    jest.clearAllMocks()
  })

  it('renders login button when user is not authenticated', () => {
    mockUseSession.mockReturnValue({
      data: null,
      status: 'unauthenticated',
      update: jest.fn(),
    })

    render(<OIDCLoginButton provider="google" />)
    
    const loginButton = screen.getByRole('button', { name: /sign in with google/i })
    expect(loginButton).toBeInTheDocument()
  })

  it('does not render when user is authenticated', () => {
    mockUseSession.mockReturnValue({
      data: {
        user: { email: 'test@example.com' },
        expires: '2024-12-31',
      },
      status: 'authenticated',
      update: jest.fn(),
    })

    const { container } = render(<OIDCLoginButton provider="google" />)
    expect(container.firstChild).toBeNull()
  })

  it('calls signIn when clicked', async () => {
    mockUseSession.mockReturnValue({
      data: null,
      status: 'unauthenticated',
      update: jest.fn(),
    })

    mockSignIn.mockResolvedValue({ ok: true, error: null })

    render(<OIDCLoginButton provider="google" />)
    
    const loginButton = screen.getByRole('button', { name: /sign in with google/i })
    fireEvent.click(loginButton)

    await waitFor(() => {
      expect(mockSignIn).toHaveBeenCalledWith('google', {
        callbackUrl: '/',
        redirect: true,
      })
    })
  })

  it('shows loading state during authentication', async () => {
    mockUseSession.mockReturnValue({
      data: null,
      status: 'loading',
      update: jest.fn(),
    })

    render(<OIDCLoginButton provider="google" />)
    
    const loadingElement = screen.getByText(/loading/i)
    expect(loadingElement).toBeInTheDocument()
  })
})