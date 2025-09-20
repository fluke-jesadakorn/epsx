'use client';

import { useState } from 'react';
import { User, Save, X, AlertCircle } from 'lucide-react';
import { type User as UserType } from '../../types/auth';

interface ProfileFormProps {
  user: UserType;
  onSave: (userData: Partial<UserType>) => Promise<void>;
  onCancel: () => void;
  className?: string;
  fields?: ('name' | 'email')[];
}

interface ProfileFormState {
  name: string;
  email: string;
  isSaving: boolean;
  error?: string;
}

export function ProfileForm({ 
  user, 
  onSave, 
  onCancel, 
  className = '',
  fields = ['name', 'email']
}: ProfileFormProps) {
  const [state, setState] = useState<ProfileFormState>({
    name: user.name || '',
    email: user.email,
    isSaving: false,
  });

  const hasChanges = () => {
    return state.name !== (user.name || '') || state.email !== user.email;
  };

  const handleSave = async () => {
    if (!state.email) {
      setState(prev => ({ ...prev, error: 'Email is required' }));
      return;
    }

    if (!isValidEmail(state.email)) {
      setState(prev => ({ ...prev, error: 'Please enter a valid email address' }));
      return;
    }

    setState(prev => ({ ...prev, isSaving: true, error: undefined }));

    try {
      const updates: Partial<UserType> = {};
      
      if (fields.includes('name') && state.name !== (user.name || '')) {
        updates.name = state.name;
      }
      
      if (fields.includes('email') && state.email !== user.email) {
        updates.email = state.email;
      }

      await onSave(updates);
      setState(prev => ({ ...prev, isSaving: false }));
    } catch (error: any) {
      setState(prev => ({ 
        ...prev, 
        isSaving: false, 
        error: error.message || 'Failed to save changes' 
      }));
    }
  };

  const handleCancel = () => {
    setState({
      name: user.name || '',
      email: user.email,
      isSaving: false,
      error: undefined,
    });
    onCancel();
  };

  const isValidEmail = (email: string): boolean => {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  };

  return (
    <div className={`space-y-6 p-6 bg-white dark:bg-slate-800 rounded-lg border border-slate-200 dark:border-slate-700 ${className}`}>
      <div className="flex items-center gap-3 mb-6">
        <div className="p-2 bg-orange-100 dark:bg-orange-900/20 rounded-lg">
          <User className="h-5 w-5 text-orange-600 dark:text-orange-400" />
        </div>
        <div>
          <h3 className="text-lg font-semibold text-slate-900 dark:text-slate-100">
            Edit Profile
          </h3>
          <p className="text-sm text-slate-600 dark:text-slate-400">
            Update your account information
          </p>
        </div>
      </div>

      <div className="space-y-4">
        {fields.includes('name') && (
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
              Display Name
            </label>
            <input
              type="text"
              value={state.name}
              onChange={(e) => setState(prev => ({ 
                ...prev, 
                name: e.target.value,
                error: undefined 
              }))}
              placeholder="Your display name"
              className="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500 dark:bg-slate-700 dark:text-slate-100"
              disabled={state.isSaving}
            />
            <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
              This name will be displayed throughout the platform
            </p>
          </div>
        )}

        {fields.includes('email') && (
          <div>
            <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
              Email Address
            </label>
            <input
              type="email"
              value={state.email}
              onChange={(e) => setState(prev => ({ 
                ...prev, 
                email: e.target.value,
                error: undefined 
              }))}
              placeholder="your.email@example.com"
              className="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500 dark:bg-slate-700 dark:text-slate-100"
              disabled={state.isSaving}
            />
            <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
              Email changes may require verification
            </p>
          </div>
        )}
      </div>

      {state.error && (
        <div className="flex items-center gap-2 p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
          <AlertCircle className="h-4 w-4 text-red-500" />
          <span className="text-sm text-red-700 dark:text-red-300">{state.error}</span>
        </div>
      )}

      <div className="flex gap-3 pt-4 border-t border-slate-200 dark:border-slate-700">
        <button
          onClick={handleSave}
          disabled={state.isSaving || !hasChanges()}
          className="flex items-center gap-2 px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Save className="h-4 w-4" />
          {state.isSaving ? 'Saving...' : 'Save Changes'}
        </button>
        
        <button
          onClick={handleCancel}
          disabled={state.isSaving}
          className="flex items-center gap-2 px-4 py-2 bg-slate-200 hover:bg-slate-300 text-slate-700 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed dark:bg-slate-700 dark:hover:bg-slate-600 dark:text-slate-200"
        >
          <X className="h-4 w-4" />
          Cancel
        </button>
      </div>
    </div>
  );
}