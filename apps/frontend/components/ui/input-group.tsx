'use client';

import * as React from 'react';

import { cn } from '@/lib/utils';

import { Input  } from './input';

import type {InputProps} from './input';

interface InputGroupContextValue {
  id: string;
  error: boolean | undefined;
  disabled: boolean | undefined;
}

const InputGroupContext = React.createContext<InputGroupContextValue | undefined>(
  undefined
);

interface InputGroupProps extends React.HTMLAttributes<HTMLDivElement> {
  id: string;
  error?: boolean;
  disabled?: boolean;
  children: React.ReactNode;
}

function InputGroup({ id, error, disabled, className, children, ...props }: InputGroupProps) {
  return (
    <InputGroupContext.Provider value={{ id, error: error ?? undefined, disabled: disabled ?? undefined }}>
      <div className={cn('space-y-2', className)} {...props}>
        {children}
      </div>
    </InputGroupContext.Provider>
  );
}

interface InputGroupLabelProps extends React.LabelHTMLAttributes<HTMLLabelElement> {
  required?: boolean;
}

function InputGroupLabel({ required, className, children, ...props }: InputGroupLabelProps) {
  const context = React.useContext(InputGroupContext);
  if (!context) throw new Error('InputGroupLabel must be used within an InputGroup');

  return (
    <label
      htmlFor={context.id}
      className={cn(
        'text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70',
        className
      )}
      {...props}
    >
      {children}
      {required && <span className="text-destructive ml-1">*</span>}
    </label>
  );
}

function InputGroupField(props: InputProps) {
  const context = React.useContext(InputGroupContext);
  if (!context) throw new Error('InputGroupField must be used within an InputGroup');

  const { error: _, ...restProps } = props;
  return (
    <Input
      id={context.id}
      error={context.error ?? undefined}
      disabled={context.disabled ?? undefined}
      aria-describedby={`${context.id}-error`}
      {...restProps}
    />
  );
}

interface InputGroupErrorProps extends React.HTMLAttributes<HTMLParagraphElement> {}

function InputGroupError({ className, children, ...props }: InputGroupErrorProps) {
  const context = React.useContext(InputGroupContext);
  if (!context) throw new Error('InputGroupError must be used within an InputGroup');

  if (!context.error || !children) {
    return null;
  }

  return (
    <p
      id={`${context.id}-error`}
      className={cn('text-sm font-medium text-destructive', className)}
      {...props}
    >
      {children}
    </p>
  );
}

interface InputGroupDescriptionProps extends React.HTMLAttributes<HTMLParagraphElement> {}

function InputGroupDescription({ className, children, ...props }: InputGroupDescriptionProps) {
  const context = React.useContext(InputGroupContext);
  if (!context) throw new Error('InputGroupDescription must be used within an InputGroup');

  if (!children) {
    return null;
  }

  return (
    <p
      id={`${context.id}-description`}
      className={cn('text-sm text-muted-foreground', className)}
      {...props}
    >
      {children}
    </p>
  );
}

export {
  InputGroup,
  InputGroupLabel,
  InputGroupField,
  InputGroupError,
  InputGroupDescription,
};

export type {
  InputGroupProps,
  InputGroupLabelProps,
  InputGroupErrorProps,
  InputGroupDescriptionProps,
};
