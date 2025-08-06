'use client';

import * as React from 'react';

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?:
    | 'default'
    | 'destructive'
    | 'outline'
    | 'secondary'
    | 'ghost'
    | 'link';
  size?: 'default' | 'sm' | 'lg' | 'icon';
}

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  error?: boolean;
  helperText?: string;
}

interface BadgeProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'secondary' | 'destructive' | 'outline';
}

interface LabelProps extends React.LabelHTMLAttributes<HTMLLabelElement> {
  required?: boolean;
}

interface FormFieldProps {
  label: string;
  id: string;
  required?: boolean;
  error?: string;
  helperText?: string;
  children: React.ReactNode;
}

export const Button: React.FC<ButtonProps> = ({
  className,
  variant = 'default',
  size = 'default',
  ...props
}) => {
  const baseClasses =
    'inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none min-h-[44px] touch-manipulation';

  const variants = {
    default: 'bg-primary text-primary-foreground hover:bg-primary/90',
    destructive:
      'bg-destructive text-destructive-foreground hover:bg-destructive/90',
    outline: 'border border-input hover:bg-accent hover:text-accent-foreground',
    secondary: 'bg-secondary text-secondary-foreground hover:bg-secondary/80',
    ghost: 'hover:bg-accent hover:text-accent-foreground',
    link: 'underline-offset-4 hover:underline text-primary',
  };

  const sizes = {
    default: 'h-10 py-2 px-4',
    sm: 'h-9 px-3 rounded-md',
    lg: 'h-11 px-8 rounded-md',
    icon: 'h-10 w-10',
  };

  return (
    <button
      className={`${baseClasses} ${variants[variant]} ${sizes[size]} ${className || ''}`}
      {...props}
    />
  );
};

export const Input: React.FC<InputProps> = ({
  className,
  type = 'text',
  error,
  helperText,
  'aria-describedby': ariaDescribedby,
  'aria-invalid': _ariaInvalid,
  ...props
}) => {
  const errorId = React.useId();
  const helperId = React.useId();

  const describedBy =
    [ariaDescribedby, error ? errorId : null, helperText ? helperId : null]
      .filter(Boolean)
      .join(' ') || undefined;

  return (
    <div className="space-y-1">
      <input
        type={type}
        aria-invalid={error ? 'true' : 'false'}
        aria-describedby={describedBy}
        className={`input flex h-10 w-full border-2 bg-card px-3 py-2 text-sm rounded-lg placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:border-primary disabled:cursor-not-allowed disabled:opacity-50 transition-colors ${
          error
            ? 'border-destructive focus-visible:ring-destructive'
            : 'border-input'
        } ${className || ''}`}
        {...props}
      />
      {error && (
        <p id={errorId} className="text-sm text-destructive" role="alert">
          {error}
        </p>
      )}
      {helperText && !error && (
        <p id={helperId} className="text-sm text-muted-foreground">
          {helperText}
        </p>
      )}
    </div>
  );
};

export const Label: React.FC<LabelProps> = ({
  className,
  required,
  children,
  ...props
}) => {
  return (
    <label
      className={`block text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 ${
        required
          ? "after:content-['*'] after:ml-0.5 after:text-destructive"
          : ''
      } ${className || ''}`}
      {...props}
    >
      {children}
    </label>
  );
};

export const FormField: React.FC<FormFieldProps> = ({
  label,
  id,
  required,
  error,
  helperText,
  children,
}) => {
  return (
    <div className="space-y-2">
      <Label htmlFor={id} required={required}>
        {label}
      </Label>
      {React.cloneElement(children as React.ReactElement<any>, {
        id,
        'aria-invalid': error ? 'true' : 'false',
        'aria-describedby': error || helperText ? `${id}-helper` : undefined,
      })}
      {error && (
        <p
          id={`${id}-helper`}
          className="text-sm text-destructive"
          role="alert"
        >
          {error}
        </p>
      )}
      {helperText && !error && (
        <p id={`${id}-helper`} className="text-sm text-muted-foreground">
          {helperText}
        </p>
      )}
    </div>
  );
};

export const Badge: React.FC<BadgeProps> = ({
  className,
  variant = 'default',
  ...props
}) => {
  const variants = {
    default: 'bg-primary text-white hover:bg-primary/80',
    secondary: 'bg-secondary text-text hover:bg-secondary/80',
    destructive: 'bg-warning text-white hover:bg-warning/80',
    outline: 'text-text border border-muted',
  };

  return (
    <div
      className={`inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-accent focus:ring-offset-2 ${variants[variant]} ${className || ''}`}
      {...props}
    />
  );
};

// Additional accessible form components
export const Select: React.FC<
  React.SelectHTMLAttributes<HTMLSelectElement> & { error?: boolean }
> = ({ className, error, children, ...props }) => {
  return (
    <select
      className={`flex h-10 w-full items-center justify-between rounded-lg border-2 bg-card px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary disabled:cursor-not-allowed disabled:opacity-50 ${
        error ? 'border-destructive focus:ring-destructive' : 'border-input'
      } ${className || ''}`}
      {...props}
    >
      {children}
    </select>
  );
};

export const Checkbox: React.FC<
  React.InputHTMLAttributes<HTMLInputElement>
> = ({ className, ...props }) => {
  return (
    <input
      type="checkbox"
      className={`h-4 w-4 rounded border-2 border-input bg-card text-primary focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 ${className || ''}`}
      {...props}
    />
  );
};

export const Textarea: React.FC<
  React.TextareaHTMLAttributes<HTMLTextAreaElement> & { error?: boolean }
> = ({ className, error, ...props }) => {
  return (
    <textarea
      className={`flex min-h-[80px] w-full rounded-lg border-2 bg-card px-3 py-2 text-sm placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary disabled:cursor-not-allowed disabled:opacity-50 ${
        error ? 'border-destructive focus:ring-destructive' : 'border-input'
      } ${className || ''}`}
      {...props}
    />
  );
};
