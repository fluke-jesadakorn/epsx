'use client'

import { useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Edit3 } from 'lucide-react'
import Link from 'next/link'
import { useParams, useRouter } from 'next/navigation'
import { toast } from 'sonner'

import { GroupEditor } from '@/components/groups/GroupEditor'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { groupMgmt, PermissionGroup } from '@/lib/api/group-management-client'

/**
 * Edit Group Page
 * Part of the Subscription & Access hub > Manual Access
 */
export default function EditGroupPage() {
    const router = useRouter()
    const params = useParams()
    const groupId = params['id'] as string
    const queryClient = useQueryClient()

    // Fetch group details
    const { data: group, isLoading: groupLoading, error: groupError } = useQuery({
        queryKey: ['permission-group', groupId],
        queryFn: async () => {
            try {
                return await groupMgmt.getPermissionGroup(groupId)
            } catch (e) {
                const groups = await groupMgmt.getPermissionGroups()
                return groups.find((g: PermissionGroup) => g.id === groupId) || null
            }
        },
        enabled: !!groupId
    })

    const handleSave = (updatedGroup: PermissionGroup) => {
        toast.success('Permission group updated successfully')
        queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
        queryClient.invalidateQueries({ queryKey: ['permission-group', groupId] })
        queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
        router.push('/subscriptions/manual-access')
    }

    const handleCancel = () => {
        router.push('/subscriptions/manual-access')
    }

    if (groupLoading) {
        return (
            <div className="min-h-screen bg-background p-3 sm:p-6">
                <div className="relative max-w-4xl mx-auto space-y-6">
                    <Skeleton className="h-10 w-64" />
                    <Skeleton className="h-96 w-full rounded-3xl" />
                </div>
            </div>
        )
    }

    if (groupError || !group) {
        return (
            <div className="min-h-screen bg-background p-3 sm:p-6">
                <div className="relative max-w-4xl mx-auto space-y-6">
                    <div className="text-center py-12">
                        <p className="text-destructive mb-4">Group not found or failed to load.</p>
                        <Link href="/subscriptions/manual-access">
                            <Button variant="outline">Back to Manual Access</Button>
                        </Link>
                    </div>
                </div>
            </div>
        )
    }

    return (
        <div className="min-h-screen bg-background p-3 sm:p-6">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-purple-400/20 to-pink-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-indigo-400/20 to-purple-500/20 rounded-full blur-lg"></div>
            </div>

            <div className="relative max-w-4xl mx-auto space-y-6">
                {/* Header */}
                <div className="flex items-center gap-4 mb-6">
                    <Link
                        href="/subscriptions/manual-access"
                        className="p-2 rounded-xl bg-card hover:bg-muted transition-colors border border-border"
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Link>
                    <div className="flex-1">
                        <h1 className="text-2xl font-bold text-purple-600 dark:text-purple-400 flex items-center gap-2">
                            <Edit3 className="w-6 h-6" />
                            Edit Permission Group
                        </h1>
                        <p className="text-sm text-muted-foreground">
                            Editing: {group.name}
                        </p>
                    </div>
                </div>

                {/* Edit Form Component */}
                <GroupEditor
                    group={group}
                    onSave={handleSave}
                    onCancel={handleCancel}
                />

                {/* Back Button */}
                <Link href="/subscriptions/manual-access">
                    <Button variant="outline" className="w-full">
                        <ArrowLeft className="h-4 w-4 mr-2" />
                        Back to Manual Access
                    </Button>
                </Link>
            </div>
        </div>
    )
}
