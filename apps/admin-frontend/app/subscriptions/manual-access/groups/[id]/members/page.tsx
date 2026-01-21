'use client'

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Trash2, UserPlus, Users } from 'lucide-react'
import Link from 'next/link'
import { useParams } from 'next/navigation'
import { useState } from 'react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { WalletAutocomplete } from '@/components/ui/WalletAutocomplete'
import { useGroupMembers } from '@/hooks/useGroupPermissions'
import { groupMgmt, PermissionGroup, UserGroupMembership } from '@/lib/api/group-management-client'

/**
 * Group Members Page
 * Part of the Subscription & Access hub > Manual Access
 */
export default function GroupMembersPage() {
    const params = useParams()
    const groupId = params['id'] as string
    const queryClient = useQueryClient()

    const [newWalletAddress, setNewWalletAddress] = useState('')
    const [showAddMember, setShowAddMember] = useState(false)

    // Fetch group details
    const { data: group, isLoading: groupLoading, error: groupError } = useQuery({
        queryKey: ['permission-group', groupId],
        queryFn: async () => {
            const groups = await groupMgmt.getPermissionGroups()
            return groups.find((g: PermissionGroup) => g.id === groupId) || null
        },
        enabled: !!groupId
    })

    // Use the hook to fetch members
    const { members, isLoading: membersLoading, refreshMembers } = useGroupMembers(groupId)

    const addMemberMutation = useMutation({
        mutationFn: async (walletAddress: string) => {
            return groupMgmt.assignUserToGroup({
                user_id: walletAddress,
                group_id: groupId,
                reason: 'Added via Members page'
            })
        },
        onSuccess: () => {
            toast.success('Member added successfully')
            setNewWalletAddress('')
            setShowAddMember(false)
            queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
            queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
            refreshMembers()
        },
        onError: (error: any) => {
            toast.error(error.message || 'Failed to add member')
        }
    })

    const removeMemberMutation = useMutation({
        mutationFn: async (walletAddress: string) => {
            return groupMgmt.removeUserFromGroup(walletAddress, groupId)
        },
        onSuccess: () => {
            toast.success('Member removed successfully')
            queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
            queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
            refreshMembers()
        },
        onError: (error: any) => {
            toast.error(error.message || 'Failed to remove member')
        }
    })

    const handleAddMember = (e: React.FormEvent) => {
        e.preventDefault()
        if (!newWalletAddress?.startsWith('0x')) {
            toast.error('Please enter a valid wallet address')
            return
        }
        addMemberMutation.mutate(newWalletAddress)
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
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-indigo-400/20 to-purple-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-blue-400/20 to-indigo-500/20 rounded-full blur-lg"></div>
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
                        <h1 className="text-2xl font-bold text-indigo-600 dark:text-indigo-400 flex items-center gap-2">
                            <Users className="w-6 h-6" />
                            Members of &quot;{group.name}&quot;
                        </h1>
                        <p className="text-sm text-muted-foreground">
                            Total members: {members.length} {membersLoading && '(Loading...)'}
                        </p>
                    </div>
                </div>

                {/* Main Content */}
                <div className="bg-card rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-indigo-500/20">
                    <div className="space-y-4">
                        {/* Add Member Section */}
                        <div className="border-b border-border pb-4">
                            <div className="flex justify-between items-center mb-3">
                                <h3 className="font-semibold text-foreground">Manage Members</h3>
                                <Button
                                    onClick={() => setShowAddMember(!showAddMember)}
                                    variant="outline"
                                    size="sm"
                                >
                                    <UserPlus className="w-4 h-4 mr-2" />
                                    Add Member
                                </Button>
                            </div>

                            {showAddMember && (
                                <form onSubmit={handleAddMember} className="flex gap-2">
                                    <WalletAutocomplete
                                        value={newWalletAddress}
                                        onChange={setNewWalletAddress}
                                        placeholder="Enter wallet address (0x...)"
                                        className="flex-1"
                                        excludeGroupId={groupId}
                                    />
                                    <Button
                                        type="submit"
                                        disabled={addMemberMutation.isPending}
                                        size="sm"
                                    >
                                        {addMemberMutation.isPending ? 'Adding...' : 'ADD'}
                                    </Button>
                                    <Button
                                        type="button"
                                        variant="outline"
                                        onClick={() => {
                                            setShowAddMember(false)
                                            setNewWalletAddress('')
                                        }}
                                        size="sm"
                                    >
                                        Cancel
                                    </Button>
                                </form>
                            )}
                        </div>

                        {/* Members List */}
                        <div className="space-y-3">
                            <h3 className="font-semibold text-foreground">Current Members</h3>

                            {membersLoading ? (
                                <div className="text-center py-8">
                                    <div className="h-8 w-8 border-b-2 border-indigo-600 mx-auto mb-4 animate-spin rounded-full"></div>
                                    <p className="text-muted-foreground">Loading members...</p>
                                </div>
                            ) : members.length === 0 ? (
                                <div className="text-center py-8 text-muted-foreground">
                                    <Users className="w-12 h-12 mx-auto mb-4 opacity-30" />
                                    <p className="text-lg font-medium">No members in this group</p>
                                    <p className="text-sm">Add members to grant them the group&apos;s permissions</p>
                                </div>
                            ) : (
                                <div className="space-y-2 max-h-[400px] overflow-y-auto pr-2">
                                    {members.map((member: UserGroupMembership) => (
                                        <div key={member.id} className="flex items-center justify-between p-3 bg-muted/50 border border-border rounded-lg hover:shadow-sm transition-shadow">
                                            <div className="flex items-center gap-3">
                                                <div className="h-8 w-8 rounded-full bg-indigo-500/10 flex items-center justify-center text-indigo-600 dark:text-indigo-400 font-mono text-xs">
                                                    {member.user_id.substring(2, 4)}
                                                </div>
                                                <div>
                                                    <div className="font-medium font-mono text-sm text-foreground">
                                                        {member.user_id}
                                                    </div>
                                                    <div className="text-xs text-muted-foreground flex gap-2">
                                                        <span>Added: {new Date(member.granted_at).toLocaleDateString()}</span>
                                                        {member.expires_at && (
                                                            <span className="text-orange-600">
                                                                Expires: {new Date(member.expires_at).toLocaleDateString()}
                                                            </span>
                                                        )}
                                                    </div>
                                                </div>
                                            </div>
                                            <Button
                                                variant="ghost"
                                                size="sm"
                                                className="text-destructive hover:text-destructive hover:bg-destructive/10"
                                                onClick={() => {
                                                    if (confirm('Remove this user from the group?')) {
                                                        removeMemberMutation.mutate(member.user_id)
                                                    }
                                                }}
                                                disabled={removeMemberMutation.isPending}
                                            >
                                                <Trash2 className="w-4 h-4" />
                                            </Button>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </div>

                        {/* Group Info */}
                        <div className="border-t border-border pt-4">
                            <h3 className="font-semibold mb-2 text-foreground">Group Permissions</h3>
                            <div className="flex flex-wrap gap-2">
                                {group.permissions.map((permission: string) => (
                                    <Badge key={permission} variant="secondary">
                                        {permission}
                                    </Badge>
                                ))}
                            </div>
                        </div>
                    </div>
                </div>

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
