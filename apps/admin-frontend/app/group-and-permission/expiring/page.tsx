'use client'

import { useMutation, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Clock, Trash2 } from 'lucide-react'
import Link from 'next/link'
import { useEffect, useState } from 'react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { groupMgmt } from '@/lib/api/group-management-client'

export default function ExpiringAssignmentsPage() {
    const queryClient = useQueryClient()
    const [assignments, setAssignments] = useState<any[]>([])
    const [isLoading, setIsLoading] = useState(true)

    // Load expiring assignments
    useEffect(() => {
        const loadAssignments = async () => {
            setIsLoading(true)
            try {
                const data = await groupMgmt.getExpiringMemberships(7)
                setAssignments(data)
            } catch (error) {
                console.error('Failed to load expiring assignments:', error)
                setAssignments([])
            } finally {
                setIsLoading(false)
            }
        }
        loadAssignments()
    }, [])

    const handleRefresh = async () => {
        setIsLoading(true)
        try {
            const data = await groupMgmt.getExpiringMemberships(7)
            setAssignments(data)
        } catch (error) {
            console.error('Failed to refresh:', error)
        } finally {
            setIsLoading(false)
        }
    }

    const extendAssignmentMutation = useMutation({
        mutationFn: async ({ userId, groupId, days }: { userId: string; groupId: string; days: number }) => {
            const newExpiryDate = new Date()
            newExpiryDate.setDate(newExpiryDate.getDate() + days)

            return groupMgmt.assignUserToGroup({
                user_id: userId,
                group_id: groupId,
                expires_at: newExpiryDate.toISOString(),
                reason: `Extended expiry by ${days} days`
            })
        },
        onSuccess: () => {
            toast.success('Assignment extended successfully')
            queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
            handleRefresh()
        },
        onError: (error: any) => {
            toast.error(error.message || 'Failed to extend assignment')
        }
    })

    const removeAssignmentMutation = useMutation({
        mutationFn: async ({ userId, groupId }: { userId: string; groupId: string }) => {
            return groupMgmt.removeUserFromGroup(userId, groupId)
        },
        onSuccess: () => {
            toast.success('Assignment removed successfully')
            queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
            handleRefresh()
        },
        onError: (error: any) => {
            toast.error(error.message || 'Failed to remove assignment')
        }
    })

    return (
        <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-orange-400/20 to-pink-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-amber-400/20 to-orange-500/20 rounded-full blur-lg"></div>
            </div>

            <div className="relative max-w-4xl mx-auto space-y-6">
                {/* Header */}
                <div className="flex items-center gap-4 mb-6">
                    <Link
                        href="/group-and-permission"
                        className="p-2 rounded-xl bg-white/80 dark:bg-gray-800/80 hover:bg-white dark:hover:bg-gray-800 transition-colors border border-gray-200 dark:border-gray-700"
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Link>
                    <div className="flex-1">
                        <h1 className="text-2xl font-bold text-orange-600 dark:text-orange-400 flex items-center gap-2">
                            <Clock className="w-6 h-6" />
                            Expiring Assignments
                        </h1>
                        <p className="text-sm text-gray-600 dark:text-gray-400">
                            Assignments expiring in the next 7 days
                        </p>
                    </div>
                    <Button
                        variant="outline"
                        onClick={handleRefresh}
                        disabled={isLoading}
                    >
                        Refresh
                    </Button>
                </div>

                {/* Main Content */}
                <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-orange-300/50 dark:border-orange-700/50">
                    <div className="space-y-4">
                        {isLoading ? (
                            <div className="space-y-4">
                                {[1, 2, 3].map((i) => (
                                    <div key={i} className="p-4 border rounded-lg">
                                        <Skeleton className="h-5 w-48 mb-2" />
                                        <Skeleton className="h-4 w-32" />
                                    </div>
                                ))}
                            </div>
                        ) : assignments.length > 0 ? (
                            <>
                                <div className="flex justify-between items-center pb-2">
                                    <p className="text-sm font-medium text-gray-700 dark:text-gray-300">
                                        Found {assignments.length} expiring assignment{assignments.length !== 1 ? 's' : ''}
                                    </p>
                                </div>

                                {assignments.map((assignment, index) => {
                                    const daysUntilExpiry = assignment.expires_at ?
                                        Math.ceil((new Date(assignment.expires_at).getTime() - Date.now()) / (1000 * 60 * 60 * 24)) : null

                                    return (
                                        <div key={index} className="p-4 border-orange-200 bg-orange-50/50 dark:bg-orange-900/10 rounded-lg border">
                                            <div className="flex justify-between items-start">
                                                <div className="space-y-2">
                                                    <div className="font-mono text-sm font-medium">
                                                        {assignment.user_email || `${(assignment.user_id?.slice(0, 6)) || ''}...${(assignment.user_id?.slice(-4)) || ''}`}
                                                    </div>
                                                    <div className="flex items-center gap-2 text-sm">
                                                        <Badge variant="outline" className="text-xs">
                                                            {assignment.group?.name || assignment.group_id}
                                                        </Badge>
                                                        {daysUntilExpiry !== null && (
                                                            <span className="text-orange-600 font-medium">
                                                                {daysUntilExpiry === 0 ? 'Expires today' :
                                                                    daysUntilExpiry === 1 ? 'Expires tomorrow' :
                                                                        `Expires in ${daysUntilExpiry} days`}
                                                            </span>
                                                        )}
                                                    </div>
                                                    <div className="text-xs text-gray-500">
                                                        Granted: {new Date(assignment.granted_at).toLocaleDateString()}
                                                        {assignment.expires_at && (
                                                            <> • Expires: {new Date(assignment.expires_at).toLocaleDateString()}</>
                                                        )}
                                                    </div>
                                                </div>

                                                <div className="flex gap-2">
                                                    <Button
                                                        size="sm"
                                                        variant="outline"
                                                        onClick={() => extendAssignmentMutation.mutate({ userId: assignment.user_id, groupId: assignment.group_id, days: 30 })}
                                                        disabled={extendAssignmentMutation.isPending}
                                                    >
                                                        <Clock className="w-3 h-3 mr-1" />
                                                        +30d
                                                    </Button>
                                                    <Button
                                                        size="sm"
                                                        variant="outline"
                                                        onClick={() => extendAssignmentMutation.mutate({ userId: assignment.user_id, groupId: assignment.group_id, days: 90 })}
                                                        disabled={extendAssignmentMutation.isPending}
                                                    >
                                                        <Clock className="w-3 h-3 mr-1" />
                                                        +90d
                                                    </Button>
                                                    <Button
                                                        size="sm"
                                                        variant="outline"
                                                        className="text-red-600 hover:text-red-700"
                                                        onClick={() => {
                                                            if (confirm('Remove this assignment?')) {
                                                                removeAssignmentMutation.mutate({ userId: assignment.user_id, groupId: assignment.group_id })
                                                            }
                                                        }}
                                                        disabled={removeAssignmentMutation.isPending}
                                                    >
                                                        <Trash2 className="w-3 h-3" />
                                                    </Button>
                                                </div>
                                            </div>
                                        </div>
                                    )
                                })}
                            </>
                        ) : (
                            <div className="text-center py-12 text-gray-500">
                                <Clock className="w-16 h-16 mx-auto mb-4 text-gray-300" />
                                <p className="text-lg font-medium">No expiring assignments</p>
                                <p className="text-sm">All assignments are valid for more than 7 days</p>
                            </div>
                        )}
                    </div>
                </div>

                {/* Back Button */}
                <Link href="/group-and-permission">
                    <Button variant="outline" className="w-full">
                        <ArrowLeft className="h-4 w-4 mr-2" />
                        Back to Permission Management
                    </Button>
                </Link>
            </div>
        </div>
    )
}
