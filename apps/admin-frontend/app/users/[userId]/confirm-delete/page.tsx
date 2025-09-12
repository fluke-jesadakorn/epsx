import { Suspense } from 'react'
import { redirect } from 'next/navigation'
import { ArrowLeft, AlertTriangle, Trash2 } from 'lucide-react'
import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { getServerSession } from '@/lib/server/auth'

interface ConfirmDeletePageProps {
  params: {
    userId: string
  }
  searchParams: {
    permissionName?: string
    action?: string
    returnUrl?: string
  }
}

async function ConfirmDeleteForm({ params, searchParams }: ConfirmDeletePageProps) {
  const { userId } = params
  const { permissionName, action, returnUrl = `/users/${userId}` } = searchParams

  const handleConfirmDelete = async (formData: FormData) => {
    'use server'
    
    const confirmedAction = formData.get('action') as string
    const permissionName = formData.get('permissionName') as string
    const userId = formData.get('userId') as string
    
    if (confirmedAction === 'delete_permission') {
      try {
        // Here you would call the actual permission removal API
        // const result = await removeUserPermission(userId, permissionName)
        
        const searchParams = new URLSearchParams({
          success: 'true',
          message: 'Permission removed successfully'
        })
        redirect(`${returnUrl}?${searchParams.toString()}`)
      } catch (error) {
        const searchParams = new URLSearchParams({
          error: 'delete-failed',
          message: 'Failed to remove permission'
        })
        redirect(`${returnUrl}?${searchParams.toString()}`)
      }
    } else {
      // Just redirect back if not confirmed
      redirect(returnUrl)
    }
  }

  const getDeleteContent = () => {
    if (permissionName) {
      return {
        title: 'Remove Permission',
        description: `Are you sure you want to remove the "${permissionName}" permission? This action cannot be undone.`,
        buttonText: 'Remove Permission',
        actionType: 'delete_permission'
      }
    }
    
    // Default generic delete
    return {
      title: 'Confirm Deletion',
      description: 'Are you sure you want to delete this item? This action cannot be undone.',
      buttonText: 'Delete',
      actionType: 'delete_item'
    }
  }

  const content = getDeleteContent()

  return (
    <div className="max-w-md mx-auto">
      <Card className="p-6">
        <div className="flex items-center gap-3 mb-6">
          <div className="p-3 bg-red-100 rounded-full">
            <AlertTriangle className="w-6 h-6 text-red-600" />
          </div>
          <div>
            <h1 className="text-xl font-semibold text-gray-900">{content.title}</h1>
            <p className="text-sm text-gray-600">This action cannot be undone</p>
          </div>
        </div>

        <div className="mb-6">
          <p className="text-gray-700 leading-relaxed">
            {content.description}
          </p>
        </div>

        <form action={handleConfirmDelete} className="space-y-4">
          <input type="hidden" name="userId" value={userId} />
          <input type="hidden" name="permissionName" value={permissionName || ''} />
          
          <div className="flex gap-3">
            <Link href={returnUrl} className="flex-1">
              <Button type="button" variant="outline" className="w-full">
                <ArrowLeft className="w-4 h-4 mr-2" />
                Cancel
              </Button>
            </Link>
            
            <Button
              type="submit"
              name="action"
              value={content.actionType}
              variant="destructive"
              className="flex-1"
            >
              <Trash2 className="w-4 h-4 mr-2" />
              {content.buttonText}
            </Button>
          </div>
        </form>

        <div className="mt-6 pt-4 border-t">
          <div className="flex items-start gap-2 text-sm text-gray-500">
            <AlertTriangle className="w-4 h-4 text-amber-500 mt-0.5 flex-shrink-0" />
            <p>
              This confirmation page provides a secure way to confirm destructive actions 
              without requiring JavaScript modals.
            </p>
          </div>
        </div>
      </Card>
    </div>
  )
}

export default async function ConfirmDeletePage(props: ConfirmDeletePageProps) {
  // Verify admin session
  const session = await getServerSession()
  if (!session?.user?.permissions?.some(p => p.startsWith('admin:'))) {
    redirect('/unauthorized')
  }

  return (
    <div className="min-h-screen bg-gray-50 py-12">
      <div className="container max-w-2xl mx-auto px-4">
        <Suspense fallback={<div>Loading confirmation...</div>}>
          <ConfirmDeleteForm {...props} />
        </Suspense>
      </div>
    </div>
  )
}