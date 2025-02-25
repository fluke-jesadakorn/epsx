# Authentication Components

This directory contains components related to user authentication.

## Components

- **AuthForm.tsx**: The main authentication form component handling login and registration

## Usage

Import components using:
```tsx
import { AuthForm } from '@/components/auth/AuthForm';
```

## Props

The AuthForm component accepts the following props:
- `type`: 'login' | 'register' - Determines which form variant to display
- `onSuccess`: () => void - Callback for successful authentication
- `onError`: (error: string) => void - Callback for authentication errors
