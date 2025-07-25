'use client';

import { useState } from 'react';
import { loginFormAction } from '@/app/actions/auth.server';

interface LoginFormServerProps {
  redirectTo?: string;
}

export function LoginFormServer({ redirectTo = '/dashboard' }: LoginFormServerProps) {
  const [showPassword, setShowPassword] = useState(false);

  return (
    <form action={loginFormAction} className="space-y-6">
      <input type="hidden" name="redirectTo" value={redirectTo} />
      <div className="space-y-4">
        <div>
          <label htmlFor="email" className="block text-sm font-medium text-foreground mb-2">
            Email Address
          </label>
          <input
            id="email"
            name="email"
            type="email"
            autoComplete="email"
            required
            className="block w-full px-4 py-3 rounded-xl border border-border bg-background/50 text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-orange-500 transition-all duration-300"
            placeholder="Enter your email"
          />
        </div>
        
        <div>
          <label htmlFor="password" className="block text-sm font-medium text-foreground mb-2">
            🔒 Password
          </label>
          <div className="relative">
            <input
              id="password"
              name="password"
              type={showPassword ? 'text' : 'password'}
              autoComplete="current-password"
              required
              // Security: Password input with visibility toggle
              className="block w-full px-4 py-3 pr-12 rounded-xl border border-border bg-background/50 text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-orange-500 transition-all duration-300"
              placeholder="Enter your password"
            />
            <button
              type="button"
              onClick={() => setShowPassword(!showPassword)}
              className="absolute right-3 top-1/2 transform -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors duration-200"
              aria-label={showPassword ? 'Hide password' : 'Show password'}
            >
              <span className="text-lg">{showPassword ? '🙈' : '👁️'}</span>
            </button>
          </div>
        </div>
      </div>

      <div className="pt-4">
        <button
          type="submit"
          className="w-full flex justify-center items-center gap-2 py-4 px-6 text-base font-bold bg-gradient-to-r from-orange-500 to-yellow-500 hover:from-orange-600 hover:to-yellow-600 text-white rounded-2xl shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-300"
        >
          Sign in
        </button>
      </div>
    </form>
  );
}