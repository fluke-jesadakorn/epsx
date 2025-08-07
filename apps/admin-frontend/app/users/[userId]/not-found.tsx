/**
 * User Not Found Page
 * Shows when a user profile cannot be found
 */

import Link from 'next/link'
import { UserX, ArrowLeft, Home } from 'lucide-react'

export default function UserNotFound() {
  return (
    <div className="min-h-[60vh] flex items-center justify-center p-6">
      <div className="max-w-md w-full text-center">
        <div className="flex justify-center mb-6">
          <div className="p-4 rounded-full bg-gray-100">
            <UserX className="h-12 w-12 text-gray-400" />
          </div>
        </div>
        
        <h1 className="text-2xl font-semibold mb-3">User not found</h1>
        
        <p className="text-muted-foreground mb-8">
          The user profile you&apos;re looking for doesn&apos;t exist or you don&apos;t have permission to view it.
        </p>
        
        <div className="flex flex-col sm:flex-row gap-3 justify-center">
          <Link
            href="/users"
            className="inline-flex items-center justify-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            <ArrowLeft className="h-4 w-4" />
            Back to Users
          </Link>
          
          <Link
            href="/"
            className="inline-flex items-center justify-center gap-2 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
          >
            <Home className="h-4 w-4" />
            Dashboard
          </Link>
        </div>
      </div>
    </div>
  )
}