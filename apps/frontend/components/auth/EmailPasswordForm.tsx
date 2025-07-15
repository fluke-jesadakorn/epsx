'use client';

import { zodResolver } from '@hookform/resolvers/zod';
import { useForm, Controller } from 'react-hook-form';
import { z } from 'zod';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useAuth } from '@/context/auth-context';

const formSchema = z
  .object({
    email: z.string().email('Please enter a valid email address'),
    password: z.string().min(6, 'Password must be at least 6 characters'),
    confirmPassword: z.string().optional(),
  })
  .refine(
    (data) => !data.confirmPassword || data.password === data.confirmPassword,
    {
      message: "Passwords don't match",
      path: ['confirmPassword'],
    },
  );

type FormValues = z.infer<typeof formSchema>;

interface EmailPasswordFormProps {
  isSignUp?: boolean;
}

export function EmailPasswordForm({ isSignUp }: EmailPasswordFormProps) {
  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<FormValues>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      email: '',
      password: '',
      confirmPassword: '',
    },
  });

  const { signInWithEmailPassword, signUp, error } = useAuth();

  const onSubmit = async (data: FormValues) => {
    console.log('Form submission triggered', data);
    try {
      if (isSignUp) {
        console.log('Attempting sign up with email:', data.email);
        await signUp(data.email, data.password);
        console.log('Sign up successful for email:', data.email);
      } else {
        console.log('Attempting sign in with email:', data.email);
        await signInWithEmailPassword(data.email, data.password);
        console.log('Sign in successful for email:', data.email);
        // Add a small delay to ensure auth state is updated
        setTimeout(() => {
          console.log('Post sign-in delay completed');
        }, 500);
      }
    } catch (error) {
      console.error('Authentication error:', error);
      // Don't throw to prevent form submission errors
    }
  };

  const handleForm = async (e: React.FormEvent) => {
    e.preventDefault();
    console.log('Form submit event triggered');
    console.log('Form state before submission:', { errors, isSubmitting });
    const isValid = await handleSubmit(onSubmit)();
    console.log('Form validation result:', isValid);
  };

  return (
    <form onSubmit={handleForm} className="space-y-6">
      <div className="space-y-2">
        <Label htmlFor="email" className="text-gray-700 dark:text-gray-300 font-medium">
          Email
        </Label>
        <Controller
          name="email"
          control={control}
          render={({ field }) => (
            <Input
              {...field}
              id="email"
              type="email"
              placeholder="name@example.com"
              error={!!errors.email}
              className="border-gray-300 dark:border-gray-600 focus:border-orange-400 focus:ring-orange-400 rounded-lg py-3 px-4 text-base transition-colors duration-200"
            />
          )}
        />
        {errors.email && (
          <p className="text-sm text-red-500 font-medium">{errors.email.message}</p>
        )}
      </div>
      <div className="space-y-2">
        <Label htmlFor="password" className="text-gray-700 dark:text-gray-300 font-medium">
          Password
        </Label>
        <Controller
          name="password"
          control={control}
          render={({ field }) => (
            <Input
              {...field}
              id="password"
              type="password"
              error={!!errors.password}
              className="border-gray-300 dark:border-gray-600 focus:border-orange-400 focus:ring-orange-400 rounded-lg py-3 px-4 text-base transition-colors duration-200"
            />
          )}
        />
        {errors.password && (
          <p className="text-sm text-red-500 font-medium">{errors.password.message}</p>
        )}
      </div>
      {isSignUp && (
        <div className="space-y-2">
          <Label htmlFor="confirmPassword" className="text-gray-700 dark:text-gray-300 font-medium">
            Confirm Password
          </Label>
          <Controller
            name="confirmPassword"
            control={control}
            render={({ field }) => (
              <Input
                {...field}
                id="confirmPassword"
                type="password"
                error={!!errors.confirmPassword}
                className="border-gray-300 dark:border-gray-600 focus:border-orange-400 focus:ring-orange-400 rounded-lg py-3 px-4 text-base transition-colors duration-200"
              />
            )}
          />
          {errors.confirmPassword && (
            <p className="text-sm text-red-500 font-medium">
              {errors.confirmPassword.message}
            </p>
          )}
        </div>
      )}
      {error && (
        <div className="p-3 rounded-lg bg-red-50 dark:bg-red-950 border border-red-200 dark:border-red-800">
          <p className="text-sm text-red-600 dark:text-red-400 font-medium">{error}</p>
        </div>
      )}
      <Button
        type="submit"
        className="w-full cursor-pointer bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white font-semibold py-3 px-4 rounded-lg shadow-lg hover:shadow-xl transform hover:scale-105 transition-all duration-200 border-0 relative overflow-hidden group"
        disabled={isSubmitting}
      >
        <span className="relative z-10 flex items-center justify-center gap-2">
          {isSubmitting ? (
            <>
              <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"></div>
              {isSignUp ? 'Creating account...' : 'Signing in...'}
            </>
          ) : (
            <>
              {isSignUp ? '🥞 Create Account' : '🥞 Sign In'}
            </>
          )}
        </span>
        {/* Shimmer effect */}
        <div className="absolute inset-0 -top-2 -bottom-2 bg-gradient-to-r from-transparent via-white/20 to-transparent group-hover:animate-shimmer opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
      </Button>
    </form>
  );
}
