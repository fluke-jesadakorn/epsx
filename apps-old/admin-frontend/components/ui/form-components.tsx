/**
 * ADMIN FRONTEND FORM COMPONENTS
 * Re-exports from shared UI components
 */

'use client';

import type * as React from 'react';

import { Button } from '@/shared/components/ui/button';
import { Input } from '@/shared/components/ui/input';
import { Select } from '@/shared/components/ui/select';
import { Textarea } from '@/shared/components/ui/textarea';
import { Form, FormField, FormFieldWrapper, FormLabel as Label } from '@/shared/components/ui/form';

interface ButtonProps {
  variant?: 'default' | 'destructive' | 'outline' | 'secondary' | 'ghost' | 'link';
  size?: 'default' | 'sm' | 'lg' | 'icon';
  children?: React.ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  className?: string;
  type?: 'button' | 'submit' | 'reset';
}

interface InputProps {
  error?: boolean | string;
  helperText?: string;
  value?: string;
  onChange?: (e: React.ChangeEvent<HTMLInputElement>) => void;
  placeholder?: string;
  type?: string;
  disabled?: boolean;
  className?: string;
}

interface FormFieldProps {
  label: string;
  id: string;
  required?: boolean;
  error?: string;
  helperText?: string;
  children: React.ReactNode;
  className?: string;
}

export { Button, Form, FormField, FormFieldWrapper, Input, Label, Select, Textarea };
export type { ButtonProps, FormFieldProps, InputProps };
