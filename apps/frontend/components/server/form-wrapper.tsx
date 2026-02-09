import type { ReactNode } from 'react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { AlertCircle, CheckCircle } from 'lucide-react';

interface FormWrapperProps {
  children: ReactNode;
  action: (formData: FormData) => Promise<void>;
  error?: string;
  success?: string;
  fieldErrors?: Record<string, string[]>;
  className?: string;
}

export function FormWrapper({
  children,
  action,
  error,
  success,
  fieldErrors,
  className = ''
}: FormWrapperProps) {
  return (
    <form action={action} className={className}>
      {/* Global success message */}
      {success && (
        <Alert className="mb-4 border-green-200 bg-green-50 text-green-800">
          <CheckCircle className="h-4 w-4" />
          <AlertDescription>{success}</AlertDescription>
        </Alert>
      )}
      
      {/* Global error message */}
      {error && (
        <Alert className="mb-4 border-red-200 bg-red-50 text-red-800">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      
      {/* Field errors summary */}
      {fieldErrors && Object.keys(fieldErrors).length > 0 && (
        <Alert className="mb-4 border-red-200 bg-red-50 text-red-800">
          <AlertCircle className="h-4 w-4" />
          <AlertDescription>
            <div className="space-y-1">
              <p className="font-medium">Please fix the following errors:</p>
              <ul className="list-disc list-inside space-y-1 text-sm">
                {Object.entries(fieldErrors).map(([field, errors]) =>
                  errors.map((fieldError) => (
                    <li key={`${field}-${fieldError}`}>
                      <span className="capitalize">{field}</span>: {fieldError}
                    </li>
                  ))
                )}
              </ul>
            </div>
          </AlertDescription>
        </Alert>
      )}
      
      {children}
    </form>
  );
}

interface FieldErrorProps {
  fieldName: string;
  fieldErrors?: Record<string, string[]>;
}

export function FieldError({ fieldName, fieldErrors }: FieldErrorProps) {
  const errors = fieldErrors?.[fieldName];
  if (!errors || errors.length === 0) {return null;}
  
  return (
    <div className="mt-1 text-sm text-red-600">
      {errors.map((error) => (
        <p key={error}>{error}</p>
      ))}
    </div>
  );
}