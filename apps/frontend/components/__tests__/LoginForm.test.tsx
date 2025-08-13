// Frontend Components Tests - Presentation Layer (UI Components)
// Tests for React components and user interactions
// Clean Architecture: Presentation Layer - UI behavior with mocked dependencies

import React from 'react'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { LoginForm } from '../LoginForm'
import { AuthService } from '../../lib/auth-service'

// Mock AuthService
jest.mock('../../lib/auth-service')
const mockAuthService = AuthService as jest.MockedClass<typeof AuthService>

// Mock useRouter from Next.js
const mockPush = jest.fn()
jest.mock('next/navigation', () => ({
  useRouter: () => ({
    push: mockPush
  })
}))

describe('LoginForm Component', () => {
  let mockAuthServiceInstance: jest.Mocked<AuthService>
  const user = userEvent.setup()

  beforeEach(() => {
    mockAuthServiceInstance = {
      login: jest.fn(),
      logout: jest.fn(),
      getCurrentUser: jest.fn(),
      hasPermission: jest.fn()
    } as any

    mockAuthService.mockImplementation(() => mockAuthServiceInstance)
    mockPush.mockClear()
  })

  describe('Rendering', () => {
    test('renders login form with all required fields', () => {
      render(<LoginForm />)

      expect(screen.getByLabelText(/email/i)).toBeInTheDocument()
      expect(screen.getByLabelText(/password/i)).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument()
    })

    test('renders remember me checkbox', () => {
      render(<LoginForm />)

      expect(screen.getByLabelText(/remember me/i)).toBeInTheDocument()
    })

    test('renders forgot password link', () => {
      render(<LoginForm />)

      expect(screen.getByRole('link', { name: /forgot password/i })).toBeInTheDocument()
    })
  })

  describe('Form Validation', () => {
    test('shows validation error for invalid email', async () => {
      render(<LoginForm />)

      const emailInput = screen.getByLabelText(/email/i)
      
      await user.type(emailInput, 'invalid-email')
      await user.tab() // Trigger blur event

      await waitFor(() => {
        expect(screen.getByText(/invalid email format/i)).toBeInTheDocument()
      })
    })

    test('shows validation error for empty password', async () => {
      render(<LoginForm />)

      const submitButton = screen.getByRole('button', { name: /sign in/i })
      
      await user.click(submitButton)

      await waitFor(() => {
        expect(screen.getByText(/password is required/i)).toBeInTheDocument()
      })
    })

    test('clears validation errors when user corrects input', async () => {
      render(<LoginForm />)

      const emailInput = screen.getByLabelText(/email/i)
      
      // Enter invalid email
      await user.type(emailInput, 'invalid')
      await user.tab()

      await waitFor(() => {
        expect(screen.getByText(/invalid email format/i)).toBeInTheDocument()
      })

      // Clear and enter valid email
      await user.clear(emailInput)
      await user.type(emailInput, 'test@example.com')
      await user.tab()

      await waitFor(() => {
        expect(screen.queryByText(/invalid email format/i)).not.toBeInTheDocument()
      })
    })
  })

  describe('Form Submission', () => {
    test('successful login redirects to dashboard', async () => {
      const mockUser = {
        id: '123',
        email: 'test@example.com',
        permissionProfile: 'user-premium-002'
      }

      mockAuthServiceInstance.login.mockResolvedValue(mockUser)

      render(<LoginForm />)

      await user.type(screen.getByLabelText(/email/i), 'test@example.com')
      await user.type(screen.getByLabelText(/password/i), 'password123')
      await user.click(screen.getByRole('button', { name: /sign in/i }))

      await waitFor(() => {
        expect(mockAuthServiceInstance.login).toHaveBeenCalledWith(
          'test@example.com',
          'password123'
        )
      })

      expect(mockPush).toHaveBeenCalledWith('/dashboard')
    })

    test('failed login shows error message', async () => {
      mockAuthServiceInstance.login.mockRejectedValue(
        new Error('Authentication failed: 401')
      )

      render(<LoginForm />)

      await user.type(screen.getByLabelText(/email/i), 'wrong@example.com')
      await user.type(screen.getByLabelText(/password/i), 'wrongpassword')
      await user.click(screen.getByRole('button', { name: /sign in/i }))

      await waitFor(() => {
        expect(screen.getByText(/invalid email or password/i)).toBeInTheDocument()
      })

      expect(mockPush).not.toHaveBeenCalled()
    })

    test('network error shows generic error message', async () => {
      mockAuthServiceInstance.login.mockRejectedValue(
        new Error('Network error')
      )

      render(<LoginForm />)

      await user.type(screen.getByLabelText(/email/i), 'test@example.com')
      await user.type(screen.getByLabelText(/password/i), 'password123')
      await user.click(screen.getByRole('button', { name: /sign in/i }))

      await waitFor(() => {
        expect(screen.getByText(/something went wrong/i)).toBeInTheDocument()
      })
    })
  })

  describe('Loading States', () => {
    test('shows loading spinner during submission', async () => {
      // Create a promise that we can control
      let resolveLogin: (value: any) => void
      const loginPromise = new Promise((resolve) => {
        resolveLogin = resolve
      })

      mockAuthServiceInstance.login.mockImplementation(() => loginPromise)

      render(<LoginForm />)

      await user.type(screen.getByLabelText(/email/i), 'test@example.com')
      await user.type(screen.getByLabelText(/password/i), 'password123')
      await user.click(screen.getByRole('button', { name: /sign in/i }))

      // Should show loading state
      expect(screen.getByText(/signing in/i)).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /signing in/i })).toBeDisabled()

      // Resolve the login
      resolveLogin!({ id: '123', email: 'test@example.com' })

      await waitFor(() => {
        expect(screen.queryByText(/signing in/i)).not.toBeInTheDocument()
      })
    })

    test('disables form inputs during submission', async () => {
      let resolveLogin: (value: any) => void
      const loginPromise = new Promise((resolve) => {
        resolveLogin = resolve
      })

      mockAuthServiceInstance.login.mockImplementation(() => loginPromise)

      render(<LoginForm />)

      const emailInput = screen.getByLabelText(/email/i)
      const passwordInput = screen.getByLabelText(/password/i)
      
      await user.type(emailInput, 'test@example.com')
      await user.type(passwordInput, 'password123')
      await user.click(screen.getByRole('button', { name: /sign in/i }))

      // Inputs should be disabled
      expect(emailInput).toBeDisabled()
      expect(passwordInput).toBeDisabled()

      // Resolve the login
      resolveLogin!({ id: '123', email: 'test@example.com' })

      await waitFor(() => {
        expect(emailInput).not.toBeDisabled()
        expect(passwordInput).not.toBeDisabled()
      })
    })
  })

  describe('Accessibility', () => {
    test('form has proper ARIA labels and roles', () => {
      render(<LoginForm />)

      expect(screen.getByRole('form')).toBeInTheDocument()
      expect(screen.getByLabelText(/email/i)).toHaveAttribute('type', 'email')
      expect(screen.getByLabelText(/password/i)).toHaveAttribute('type', 'password')
    })

    test('error messages are associated with form fields', async () => {
      render(<LoginForm />)

      const emailInput = screen.getByLabelText(/email/i)
      
      await user.type(emailInput, 'invalid')
      await user.tab()

      await waitFor(() => {
        const errorMessage = screen.getByText(/invalid email format/i)
        expect(errorMessage).toBeInTheDocument()
        expect(emailInput).toHaveAttribute('aria-describedby', expect.stringContaining(errorMessage.id))
      })
    })

    test('form can be submitted with Enter key', async () => {
      mockAuthServiceInstance.login.mockResolvedValue({
        id: '123',
        email: 'test@example.com'
      })

      render(<LoginForm />)

      await user.type(screen.getByLabelText(/email/i), 'test@example.com')
      await user.type(screen.getByLabelText(/password/i), 'password123')
      
      // Press Enter in password field
      fireEvent.keyDown(screen.getByLabelText(/password/i), {
        key: 'Enter',
        code: 'Enter',
        charCode: 13
      })

      await waitFor(() => {
        expect(mockAuthServiceInstance.login).toHaveBeenCalled()
      })
    })
  })
})