'use client'

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, UserPlus } from 'lucide-react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { useState } from 'react'
import { toast } from 'sonner'

import { GroupAssignmentTransferList } from '@/components/groups/GroupAssignmentTransferList'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { WalletAutocomplete } from '@/components/ui/WalletAutocomplete'
import { groupMgmt, PermissionGroup } from '@/lib/api/group-management-client'

/**
 *
 */
export default function AssignWalletPage() {
    const router = useRouter()
    const queryClient = useQueryClient()

    const [formData, setFormData] = useState({
        wallet_address: '',
        expires_at: '',
        reason: ''
    })

    const [selectedGroups, setSelectedGroups] = useState<PermissionGroup[]>([])

    const { data: permissionGroups = [], isLoading: groupsLoading } = useQuery({
        queryKey: ['permission-groups'],
        queryFn: () => groupMgmt.getPermissionGroups()
    })

    const assignWalletMutation = useMutation({
        mutationFn: async (data: typeof formData & { groups: PermissionGroup[] }) => {
            // Assign user to all selected groups
            const promises = data.groups.map(group =>
                groupMgmt.assignUserToGroup({
                    user_id: data.wallet_address,
                    group_id: group.id,
                    expires_at: data.expires_at || null,
                    reason: data.reason
                })
            )
            return Promise.all(promises)
        },
        onSuccess: () => {
            toast.success(`Wallet assigned to ${selectedGroups.length} groups successfully`)
            queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
            queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
            // Wait a bit before redirecting so user can see success message
            setTimeout(() => router.push('/permissions'), 1000)
        },
        onError: (error: any) => {
            toast.error(error.message || 'Failed to assign wallet to groups')
        }
    })

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (!formData.wallet_address) {
            toast.error('Wallet address is required')
            return
        }
        if (selectedGroups.length === 0) {
            toast.error('Please select at least one group')
            return
        }

        assignWalletMutation.mutate({
            ...formData,
            groups: selectedGroups
        })
    }

    return (
        <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-green-400/20 to-emerald-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-teal-400/20 to-cyan-500/20 rounded-full blur-lg"></div>
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
                    <div>
                        <h1 className="text-2xl font-bold text-green-600 dark:text-green-400 flex items-center gap-2">
                            <UserPlus className="w-6 h-6" />
                            Assign Wallet to Groups
                        </h1>
                        <p className="text-sm text-gray-600 dark:text-gray-400">
                            Add a wallet to one or more permission groups
                        </p>
                    </div>
                </div>

                {/* Content Grid */}
                <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                    {/* Left Column: Wallet & Settings */}
                    <div className="lg:col-span-1 space-y-6">
                        <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50 h-full">
                            <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-gray-100">Assignment Details</h3>
                            <form id="assignment-form" onSubmit={handleSubmit} className="space-y-4">
                                <div>
                                    <Label htmlFor="wallet_address">Wallet Address</Label>
                                    <WalletAutocomplete
                                        value={formData.wallet_address}
                                        onChange={(value) => setFormData(prev => ({ ...prev, wallet_address: value }))}
                                        placeholder="Enter wallet address (0x...)"
                                    />
                                </div>

                                <div>
                                    <Label htmlFor="expires_at">Expires At (Optional)</Label>
                                    <Input
                                        id="expires_at"
                                        type="datetime-local"
                                        value={formData.expires_at}
                                        onChange={(e) => setFormData(prev => ({ ...prev, expires_at: e.target.value }))}
                                    />
                                </div>

                                <div>
                                    <Label htmlFor="reason">Reason (Optional)</Label>
                                    <Textarea
                                        id="reason"
                                        value={formData.reason}
                                        onChange={(e) => setFormData(prev => ({ ...prev, reason: e.target.value }))}
                                        placeholder="Reason for assignment..."
                                        rows={4}
                                    />
                                </div>

                                <div className="pt-4 flex gap-3">
                                    <Link href="/group-and-permission" className="flex-1">
                                        <Button type="button" variant="outline" className="w-full">
                                            Cancel
                                        </Button>
                                    </Link>
                                    <Button
                                        type="submit"
                                        disabled={assignWalletMutation.isPending || !formData.wallet_address || selectedGroups.length === 0}
                                        className="flex-1 bg-green-600 hover:bg-green-700 text-white"
                                    >
                                        {assignWalletMutation.isPending ? 'Assigning...' : 'Assign Groups'}
                                    </Button>
                                </div>
                            </form>
                        </div>
                    </div>

                    {/* Right Column: Group Selection */}
                    <div className="lg:col-span-2">
                        <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50 h-full">
                            <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-gray-100 flex justify-between items-center">
                                <span>Select Groups</span>
                                <span className="text-sm font-normal text-gray-500">
                                    {selectedGroups.length} selected
                                </span>
                            </h3>

                            <GroupAssignmentTransferList
                                available={permissionGroups}
                                selected={selectedGroups}
                                onChange={setSelectedGroups}
                                isLoading={groupsLoading}
                            />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    )
}
