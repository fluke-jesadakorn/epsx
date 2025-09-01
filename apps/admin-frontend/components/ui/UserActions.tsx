'use client'

import { Edit, Trash2, Settings } from 'lucide-react'

interface UserActionsProps {
  userId: string
  onEdit: (userId: string) => void
  onDelete: (userId: string) => void
}

export default function UserActions({ userId, onEdit, onDelete }: UserActionsProps) {
  return (
    <div className="flex items-center gap-2 md:justify-end">
      <button 
        onClick={() => onEdit(userId)}
        className="p-2 text-blue-600 hover:bg-blue-50 dark:hover:bg-blue-900 rounded-md transition-colors"
        title="Edit User"
      >
        <Edit size={16} />
      </button>
      <button 
        onClick={() => onDelete(userId)}
        className="p-2 text-red-600 hover:bg-red-50 dark:hover:bg-red-900 rounded-md transition-colors"
        title="Delete User"
      >
        <Trash2 size={16} />
      </button>
      <button className="p-2 text-gray-600 hover:bg-gray-50 dark:hover:bg-gray-700 rounded-md transition-colors">
        <Settings size={16} />
      </button>
    </div>
  )
}