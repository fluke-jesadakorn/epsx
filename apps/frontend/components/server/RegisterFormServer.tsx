import { redirect } from 'next/navigation';
import { registerAction } from '@/app/actions/auth';
import { validateFormData, extractFormData, authSchemas, checkRateLimit } from '@/lib/server-validation';
import { Card, CardContent, CardDescription, CardHeader, CardTitle , Button , Input , Label } from '@/packages/ui';
import { FormWrapper, FieldError } from './FormWrapper';

interface RegisterFormServerProps {
  redirectTo?: string;
  error?: string;
  fieldErrors?: Record<string, string[]>;
}

export async function RegisterFormServer({ 
  redirectTo, 
  error, 
  fieldErrors 
}: RegisterFormServerProps) {
  async function handleRegister(formData: FormData) {
    'use server';
    
    try {
      // Extract and validate form data
      const rawData = extractFormData(formData);
      const validation = await validateFormData(authSchemas.register, rawData);
      
      if (!validation.success) {
        const searchParams = new URLSearchParams();
        searchParams.set('error', validation.error || 'Validation failed');
        if (validation.fieldErrors) {
          searchParams.set('fieldErrors', JSON.stringify(validation.fieldErrors));
        }
        redirect(`/register?${searchParams.toString()}`);
      }
      
      // Rate limiting check
      const clientIP = '127.0.0.1'; // In production, get real IP from headers
      if (!checkRateLimit(`register_${clientIP}`, 3, 15 * 60 * 1000)) {
        redirect('/register?error=' + encodeURIComponent('Too many registration attempts. Please try again later.'));
      }
      
      const { email, password, confirmPassword, name } = validation.data!;
      
      // Call the registration action
      const result = await registerAction(email, password, confirmPassword, { name });
      
      if (!result.success) {
        const searchParams = new URLSearchParams();
        searchParams.set('error', result.error || 'Registration failed');
        if (result.fieldErrors) {
          searchParams.set('fieldErrors', JSON.stringify(result.fieldErrors));
        }
        redirect(`/register?${searchParams.toString()}`);
      }
      
      // Success - redirect to dashboard or login
      redirect(redirectTo || '/login?success=' + encodeURIComponent('Registration successful! Please log in.'));
      
    } catch (error) {
      console.error('Registration form error:', error);
      redirect('/register?error=' + encodeURIComponent('Registration failed. Please try again.'));
    }
  }

  return (
    <Card className="w-full max-w-md">
      <CardHeader>
        <CardTitle>Create Account</CardTitle>
        <CardDescription>
          Sign up for a new account to get started
        </CardDescription>
      </CardHeader>
      <CardContent>
        <FormWrapper 
          action={handleRegister}
          error={error}
          fieldErrors={fieldErrors}
          className="space-y-4"
        >
          <div className="space-y-2">
            <Label htmlFor="name">Full Name (Optional)</Label>
            <Input
              id="name"
              name="name"
              type="text"
              placeholder="Enter your full name"
            />
            <FieldError fieldName="name" fieldErrors={fieldErrors} />
          </div>
          
          <div className="space-y-2">
            <Label htmlFor="email">
              Email Address <span className="text-red-500">*</span>
            </Label>
            <Input
              id="email"
              name="email"
              type="email"
              placeholder="Enter your email"
              required
            />
            <FieldError fieldName="email" fieldErrors={fieldErrors} />
          </div>
          
          <div className="space-y-2">
            <Label htmlFor="password">
              Password <span className="text-red-500">*</span>
            </Label>
            <Input
              id="password"
              name="password"
              type="password"
              placeholder="Create a strong password"
              required
            />
            <FieldError fieldName="password" fieldErrors={fieldErrors} />
            <div className="text-xs text-gray-600 space-y-1">
              <p>Password must contain:</p>
              <ul className="list-disc list-inside ml-2 space-y-0.5">
                <li>At least 8 characters</li>
                <li>One uppercase letter</li>
                <li>One lowercase letter</li>
                <li>One number</li>
              </ul>
            </div>
          </div>
          
          <div className="space-y-2">
            <Label htmlFor="confirmPassword">
              Confirm Password <span className="text-red-500">*</span>
            </Label>
            <Input
              id="confirmPassword"
              name="confirmPassword"
              type="password"
              placeholder="Confirm your password"
              required
            />
            <FieldError fieldName="confirmPassword" fieldErrors={fieldErrors} />
          </div>

          <div className="pt-4">
            <Button type="submit" className="w-full">
              Create Account
            </Button>
          </div>
          
          <div className="text-center text-sm text-gray-600">
            <p>
              Already have an account?{' '}
              <a href="/login" className="text-blue-600 hover:underline">
                Sign in
              </a>
            </p>
          </div>
        </FormWrapper>
      </CardContent>
    </Card>
  );
}