/**
 * User Creation Page
 * Server-side rendered user creation form
 */

import { User, Mail, Shield, ArrowLeft } from 'lucide-react'
import { UserCreateForm } from '@/components/users/UserCreateForm'

export default async function CreateUserPage() {
  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      <div className="max-w-2xl mx-auto px-4 py-8">
        {/* Header */}
        <div className="mb-8">
          <div className="flex items-center gap-3 mb-4">
            <div className="w-12 h-12 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center">
              <User className="h-6 w-6 text-green-600 dark:text-green-400" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                Create New User
              </h1>
              <p className="text-gray-600 dark:text-gray-400">
                Add a new user to the system
              </p>
            </div>
          </div>
          
          <nav className="flex items-center text-sm text-gray-500 dark:text-gray-400">
            <a href="/users" className="hover:text-blue-600 dark:hover:text-blue-400 flex items-center gap-1">
              <ArrowLeft className="h-4 w-4" />
              Users
            </a>
            <span className="mx-2">/</span>
            <span className="text-gray-900 dark:text-gray-100">Create</span>
          </nav>
        </div>

        {/* Form Card */}
        <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
          <UserCreateForm />
        </div>
      </div>
    </div>
  )
}