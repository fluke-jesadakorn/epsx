'use client'

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, UserPlus } from 'lucide-react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { useState } from 'react'
import { toast } from 'sonner'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { WalletAutocomplete } from '@/components/ui/WalletAutocomplete'
import { groupMgmt } from '@/lib/api/group-management-client'

/**
 *
 */
export default function AssignWalletPage() {
    const router = useRouter()
    const queryClient = useQueryClient()

    const [formData, setFormData] = useState({
        wallet_address: '',
        group_id: '',
        expires_at: '',
        reason: ''
    })

    const { data: permissionGroups = [] } = useQuery({
        queryKey: ['permission-groups'],
        queryFn: () => groupMgmt.getPermissionGroups()
    })

    const assignWalletMutation = useMutation({
        mutationFn: async (data: typeof formData) => {
            return groupMgmt.assignUserToGroup({
                user_id: data.wallet_address,
                group_id: data.group_id,
                expires_at: data.expires_at || null,
                reason: data.reason
            })
        },
        onSuccess: () => {
            toast.success('Wallet assigned to group successfully')
            queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
            queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
            router.push('/permissions')
        },
        onError: (error: any) => {
            toast.error(error.message || 'Failed to assign wallet to group')
        }
    })

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (!formData.wallet_address || !formData.group_id) {
            toast.error('Wallet address and group are required')
            return
        }
        assignWalletMutation.mutate(formData)
    }

    return (
        <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-green-400/20 to-emerald-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-teal-400/20 to-cyan-500/20 rounded-full blur-lg"></div>
            </div>

            <div className="relative max-w-2xl mx-auto space-y-6">
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
                            Assign Wallet to Group
                        </h1>
                        <p className="text-sm text-gray-600 dark:text-gray-400">
                            Add a wallet to a permission group
                        </p>
                    </div>
                </div>

                {/* Form Card */}
                <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-green-300/50 dark:border-green-700/50">
                    <form onSubmit={handleSubmit} className="space-y-4">
                        <div>
                            <Label htmlFor="wallet_address">Wallet Address</Label>
                            <WalletAutocomplete
                                value={formData.wallet_address}
                                onChange={(value) => setFormData(prev => ({ ...prev, wallet_address: value }))}
                                placeholder="Enter wallet address (0x...)"
                                excludeGroupId={formData.group_id}
                            />
                        </div>

                        <div>
                            <Label htmlFor="group_id">Permission Group</Label>
                            <Select value={formData.group_id} onValueChange={(value) => setFormData(prev => ({ ...prev, group_id: value }))}>
                                <SelectTrigger>
                                    <SelectValue placeholder="Select permission group" />
                                </SelectTrigger>
                                <SelectContent>
                                    {permissionGroups.map(group => (
                                        <SelectItem key={group.id} value={group.id}>{group.name}</SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
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
                            />
                        </div>

                        <div className="flex gap-4 pt-4">
                            <Link href="/group-and-permission" className="flex-1">
                                <Button type="button" variant="outline" className="w-full">
                                    Cancel
                                </Button>
                            </Link>
                            <Button type="submit" disabled={assignWalletMutation.isPending} className="flex-1">
                                {assignWalletMutation.isPending ? 'Assigning...' : 'Assign Wallet'}
                            </Button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    )
}
