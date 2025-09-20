'use client';

import { useState } from 'react';
import { Mail, Edit, Check, X, AlertCircle } from 'lucide-react';

interface EmailEditorProps {
  currentEmail: string;
  onEmailChange: (newEmail: string) => Promise<void>;
  onCancel: () => void;
  className?: string;
  disabled?: boolean;
}

interface EmailEditorState {
  newEmail: string;
  isEditing: boolean;
  isSaving: boolean;
  error?: string;
}

export function EmailEditor({ 
  currentEmail, 
  onEmailChange, 
  onCancel, 
  className = '',
  disabled = false
}: EmailEditorProps) {
  const [state, setState] = useState<EmailEditorState>({
    newEmail: currentEmail,
    isEditing: false,
    isSaving: false,
  });

  const handleStartEdit = () => {
    setState(prev => ({
      ...prev,
      isEditing: true,
      newEmail: currentEmail,
      error: undefined,
    }));
  };

  const handleCancel = () => {
    setState(prev => ({
      ...prev,
      isEditing: false,
      newEmail: currentEmail,
      error: undefined,
    }));
    onCancel();
  };

  const handleSave = async () => {
    if (!state.newEmail || state.newEmail === currentEmail) {
      setState(prev => ({ ...prev, error: 'Please enter a new email address' }));
      return;
    }

    if (!isValidEmail(state.newEmail)) {
      setState(prev => ({ ...prev, error: 'Please enter a valid email address' }));
      return;
    }

    setState(prev => ({ ...prev, isSaving: true, error: undefined }));

    try {
      await onEmailChange(state.newEmail);
      setState(prev => ({
        ...prev,
        isSaving: false,
        isEditing: false,
      }));
    } catch (error: any) {
      setState(prev => ({ 
        ...prev, 
        isSaving: false, 
        error: error.message || 'Failed to update email' 
      }));
    }
  };

  const isValidEmail = (email: string): boolean => {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  };

  if (!state.isEditing) {
    return (
      <div className={`flex items-center justify-between p-4 bg-slate-50 dark:bg-slate-800 rounded-lg ${className}`}>
        <div>
          <div className="font-medium text-slate-900 dark:text-slate-100">
            Email Address
          </div>
          <div className="text-sm text-slate-600 dark:text-slate-400">
            {currentEmail}
          </div>
        </div>
        <button
          onClick={handleStartEdit}
          disabled={disabled}
          className="flex items-center gap-2 px-3 py-1.5 text-sm text-orange-600 hover:text-orange-700 hover:bg-orange-50 dark:text-orange-400 dark:hover:text-orange-300 dark:hover:bg-orange-900/20 rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Edit className="h-4 w-4" />
          Edit
        </button>
      </div>
    );
  }

  return (
    <div className={`space-y-4 p-4 border border-orange-200 dark:border-orange-700 rounded-lg ${className}`}>
      <div>
        <label className="block text-sm font-medium text-slate-700 dark:text-slate-300 mb-2">
          New Email Address
        </label>
        <input
          type="email"
          value={state.newEmail}
          onChange={(e) => setState(prev => ({ 
            ...prev, 
            newEmail: e.target.value,
            error: undefined 
          }))}
          placeholder="your.new.email@example.com"
          className="w-full px-3 py-2 border border-slate-300 dark:border-slate-600 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500 dark:bg-slate-700 dark:text-slate-100"
          disabled={state.isSaving}
        />
        <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
          A verification process will be required for the new email
        </p>
      </div>

      {state.error && (
        <div className="flex items-center gap-2 p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
          <AlertCircle className="h-4 w-4 text-red-500" />
          <span className="text-sm text-red-700 dark:text-red-300">{state.error}</span>
        </div>
      )}

      <div className="flex gap-2">
        <button
          onClick={handleSave}
          disabled={state.isSaving || !state.newEmail || state.newEmail === currentEmail}
          className="flex items-center gap-2 px-4 py-2 bg-orange-600 hover:bg-orange-700 text-white rounded-lg disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Check className="h-4 w-4" />
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