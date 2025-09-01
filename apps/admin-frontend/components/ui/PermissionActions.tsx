'use client'

interface PermissionActionsProps {
  userEmail: string
  permission: string
  status: string
  onExtend: (userEmail: string, permission: string) => void
  onRevoke: (userEmail: string, permission: string) => void
}

export default function PermissionActions({ 
  userEmail, 
  permission, 
  status, 
  onExtend, 
  onRevoke 
}: PermissionActionsProps) {
  return (
    <div className="flex items-center gap-2 md:justify-end">
      {status !== 'never' && (
        <button 
          onClick={() => onExtend(userEmail, permission)}
          className="px-3 py-1 text-sm bg-blue-100 text-blue-800 hover:bg-blue-200 rounded transition-colors"
        >
          ⏰ Extend
        </button>
      )}
      <button 
        onClick={() => onRevoke(userEmail, permission)}
        className="px-3 py-1 text-sm bg-red-100 text-red-800 hover:bg-red-200 rounded transition-colors"
      >
        🗑️ Revoke
      </button>
    </div>
  )
}