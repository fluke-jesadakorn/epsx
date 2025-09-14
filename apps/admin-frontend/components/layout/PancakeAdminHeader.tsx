'use client'

import { useState } from 'react'
import { useRouter } from 'next/navigation'
import { ThemeToggle } from '@/components/ui/ThemeToggle'

interface User {
  id: string
  email: string
  name?: string
  role: string
}

interface PancakeAdminHeaderProps {
  user?: User
}

export function PancakeAdminHeader({ user }: PancakeAdminHeaderProps) {
  const router = useRouter()
  const [showUserMenu, setShowUserMenu] = useState(false)
  const [showNotifications, setShowNotifications] = useState(false)

  const handleLogout = () => {
    router.push('/login')
  }

  return (
    <header className="bg-gradient-to-r from-white via-yellow-50 to-orange-50 dark:from-slate-900 dark:via-slate-800 dark:to-slate-900 border-b border-yellow-200/50 dark:border-slate-700/50 backdrop-blur-sm shadow-lg sticky top-0 z-40">
      <div className="flex items-center justify-between h-16 px-6">
        {/* Search Section */}
        <div className="flex items-center gap-6 flex-1">
          <div className="relative max-w-md w-full">
            <input
              type="search"
              placeholder="Search users, permissions..."
              className="w-full h-12 pl-12 pr-4 bg-gradient-to-r from-white to-yellow-50 dark:from-slate-800 dark:to-slate-700 border-2 border-yellow-200/50 dark:border-slate-600/50 rounded-2xl text-gray-900 dark:text-slate-100 placeholder:text-gray-500 dark:placeholder:text-slate-400 focus:border-orange-400 dark:focus:border-slate-500 focus:ring-2 focus:ring-orange-200 dark:focus:ring-slate-500/20 focus:outline-none transition-all duration-200 shadow-lg"
            />
            <div className="absolute left-4 top-1/2 -translate-y-1/2">
              <span className="text-xl">🔍</span>
            </div>
          </div>

          {/* Quick Actions */}
          <div className="hidden md:flex items-center gap-3">
            <button className="h-12 px-4 bg-gradient-to-r from-green-400 to-teal-500 text-white rounded-2xl font-semibold hover:from-green-500 hover:to-teal-600 shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-200">
              <span className="mr-2">➕</span>
              Add User
            </button>
            <button className="h-12 px-4 bg-gradient-to-r from-blue-400 to-purple-500 text-white rounded-2xl font-semibold hover:from-blue-500 hover:to-purple-600 shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-200">
              <span className="mr-2">🔑</span>
              Grant Access
            </button>
          </div>
        </div>

        {/* Right Section */}
        <div className="flex items-center gap-4">
          {/* Notifications */}
          <div className="relative">
            <button
              onClick={() => setShowNotifications(!showNotifications)}
              className="h-12 w-12 bg-gradient-to-r from-orange-400 to-red-500 text-white rounded-2xl font-semibold hover:from-orange-500 hover:to-red-600 shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-200 relative"
            >
              <span className="text-xl">🔔</span>
              <div className="absolute -top-2 -right-2 h-6 w-6 bg-gradient-to-r from-pink-500 to-red-500 rounded-full flex items-center justify-center text-xs text-white shadow-lg animate-pulse">
                3
              </div>
            </button>

            {showNotifications && (
              <div className="absolute right-0 top-14 w-80 bg-white dark:bg-slate-800 rounded-3xl shadow-2xl border border-yellow-200 dark:border-slate-700/50 p-6 z-50">
                <div className="space-y-4">
                  <h3 className="font-bold text-lg bg-gradient-to-r from-orange-600 to-red-600 bg-clip-text text-transparent">
                    🔥 Recent Notifications
                  </h3>
                  
                  <div className="space-y-3">
                    <div className="p-4 bg-gradient-to-r from-green-50 to-teal-50 dark:from-green-900/20 dark:to-teal-900/20 rounded-2xl border border-green-200 dark:border-green-500/30">
                      <div className="flex items-start gap-3">
                        <span className="text-xl">✅</span>
                        <div>
                          <div className="font-semibold text-green-800 dark:text-green-300">New user registered</div>
                          <div className="text-sm text-green-600 dark:text-green-400">sarah@example.com just signed up</div>
                          <div className="text-xs text-gray-500 mt-1">2 minutes ago</div>
                        </div>
                      </div>
                    </div>

                    <div className="p-4 bg-gradient-to-r from-blue-50 to-purple-50 dark:from-blue-900/20 dark:to-purple-900/20 rounded-2xl border border-blue-200 dark:border-blue-500/30">
                      <div className="flex items-start gap-3">
                        <span className="text-xl">🔑</span>
                        <div>
                          <div className="font-semibold text-blue-800 dark:text-blue-300">Permission request</div>
                          <div className="text-sm text-blue-600 dark:text-blue-400">mike@example.com wants admin access</div>
                          <div className="text-xs text-gray-500 mt-1">5 minutes ago</div>
                        </div>
                      </div>
                    </div>

                    <div className="p-4 bg-gradient-to-r from-orange-50 to-red-50 dark:from-orange-900/20 dark:to-red-900/20 rounded-2xl border border-orange-200 dark:border-orange-500/30">
                      <div className="flex items-start gap-3">
                        <span className="text-xl">⚠️</span>
                        <div>
                          <div className="font-semibold text-orange-800 dark:text-orange-300">Permission expiring</div>
                          <div className="text-sm text-orange-600 dark:text-orange-400">john@example.com access expires in 2 days</div>
                          <div className="text-xs text-gray-500 mt-1">10 minutes ago</div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <button className="w-full bg-gradient-to-r from-orange-400 to-red-500 text-white py-2 rounded-2xl font-semibold hover:from-orange-500 hover:to-red-600 transition-all duration-200">
                    View All Notifications
                  </button>
                </div>
              </div>
            )}
          </div>

          {/* Theme Toggle */}
          <div className="h-12 w-12 bg-gradient-to-r from-purple-400 to-pink-500 hover:from-purple-500 hover:to-pink-600 rounded-2xl shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-200 flex items-center justify-center">
            <ThemeToggle />
          </div>

          {/* User Menu */}
          <div className="relative">
            <button
              onClick={() => setShowUserMenu(!showUserMenu)}
              className="flex items-center gap-3 h-12 pl-4 pr-5 bg-gradient-to-r from-yellow-400 to-orange-500 text-white rounded-2xl font-semibold hover:from-yellow-500 hover:to-orange-600 shadow-lg hover:shadow-xl hover:scale-105 transition-all duration-200"
            >
              <div className="h-8 w-8 bg-white/20 rounded-xl flex items-center justify-center">
                <span className="text-lg">👤</span>
              </div>
              <span className="hidden md:block">
                {user?.name || user?.email || 'Admin'}
              </span>
              <span className="text-sm">↓</span>
            </button>

            {showUserMenu && (
              <div className="absolute right-0 top-14 w-64 bg-white dark:bg-slate-800 rounded-3xl shadow-2xl border border-yellow-200 dark:border-slate-700/50 p-4 z-50">
                <div className="space-y-3">
                  <div className="p-4 bg-gradient-to-r from-yellow-50 to-orange-50 dark:from-yellow-900/20 dark:to-orange-900/20 rounded-2xl">
                    <div className="flex items-center gap-3">
                      <div className="h-12 w-12 bg-gradient-to-r from-yellow-400 to-orange-500 rounded-2xl flex items-center justify-center">
                        <span className="text-white text-lg">👤</span>
                      </div>
                      <div>
                        <div className="font-semibold text-gray-900 dark:text-white">
                          {user?.name || 'Admin User'}
                        </div>
                        <div className="text-sm text-gray-600 dark:text-gray-400">
                          {user?.email || 'admin@epsx.io'}
                        </div>
                        <div className="text-xs bg-gradient-to-r from-yellow-600 to-orange-600 bg-clip-text text-transparent font-semibold">
                          {user?.role || 'Administrator'}
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="space-y-1">
                    <button className="w-full text-left p-3 rounded-2xl hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 dark:hover:from-gray-700 dark:hover:to-gray-600 transition-all duration-200 flex items-center gap-3">
                      <span>👤</span>
                      <span>Profile Settings</span>
                    </button>
                    <button className="w-full text-left p-3 rounded-2xl hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 dark:hover:from-gray-700 dark:hover:to-gray-600 transition-all duration-200 flex items-center gap-3">
                      <span>🔒</span>
                      <span>Security</span>
                    </button>
                    <button className="w-full text-left p-3 rounded-2xl hover:bg-gradient-to-r hover:from-gray-50 hover:to-gray-100 dark:hover:from-gray-700 dark:hover:to-gray-600 transition-all duration-200 flex items-center gap-3">
                      <span>❓</span>
                      <span>Help & Support</span>
                    </button>
                  </div>

                  <div className="border-t border-gray-200 dark:border-gray-600 pt-3">
                    <button
                      onClick={handleLogout}
                      className="w-full p-3 bg-gradient-to-r from-red-400 to-pink-500 text-white rounded-2xl font-semibold hover:from-red-500 hover:to-pink-600 transition-all duration-200 flex items-center justify-center gap-2"
                    >
                      <span>🚪</span>
                      <span>Sign Out</span>
                    </button>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </header>
  )
}