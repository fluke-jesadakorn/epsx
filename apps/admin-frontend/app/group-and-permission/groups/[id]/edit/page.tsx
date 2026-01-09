'use client'

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Edit3, Plus } from 'lucide-react'
import Link from 'next/link'
import { useParams, useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Skeleton } from '@/components/ui/skeleton'
import { Textarea } from '@/components/ui/textarea'
import { groupMgmt, PermissionGroup } from '@/lib/api/group-management-client'

/**
 *
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
            const groups = await groupMgmt.getPermissionGroups()
            return groups.find((g: PermissionGroup) => g.id === groupId) || null
        },
        enabled: !!groupId
    })

    const [formData, setFormData] = useState({
        name: '',
        description: '',
        permissions: [] as string[],
        default_expiry_days: '',
        priority_level: 0
    })

    const [selectedPermission, setSelectedPermission] = useState('')

    // Update form data when group is loaded
    useEffect(() => {
        if (group) {
            setFormData({
                name: group.name || '',
                description: group.description || '',
                permissions: group.permissions || [],
                default_expiry_days: group.default_expiry_days?.toString() || '',
                priority_level: group.priority_level || 0
            })
        }
    }, [group])

    const availablePermissions = [
        'epsx:analytics:view',
        'epsx:analytics:advanced',
        'epsx:trading:basic',
        'epsx:trading:advanced',
        'epsx:trading:pro',
        'epsx:data:export',
        'epsx:api:read',
        'epsx:api:write',
        'epsx:notifications:manage',
        'admin:users:view',
        'admin:users:manage',
        'admin:permissions:view',
        'admin:permissions:manage',
        'admin:*:*'
    ]

    const updateGroupMutation = useMutation({
        mutationFn: async (data: typeof formData) => {
            return groupMgmt.updatePermissionGroup(groupId, {
                name: data.name,
                description: data.description,
                permissions: data.permissions,
                default_expiry_days: data.default_expiry_days ? parseInt(data.default_expiry_days) : undefined,
                priority_level: data.priority_level
            })
        },
        onSuccess: () => {
            toast.success('Permission group updated successfully')
            queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
            queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
            router.push('/permissions')
        },
        onError: (error: any) => {
            toast.error(error.message || 'Failed to update permission group')
        }
    })

    const addPermission = () => {
        if (selectedPermission && !formData.permissions.includes(selectedPermission)) {
            setFormData(prev => ({
                ...prev,
                permissions: [...prev.permissions, selectedPermission]
            }))
            setSelectedPermission('')
        }
    }

    const removePermission = (permission: string) => {
        setFormData(prev => ({
            ...prev,
            permissions: prev.permissions.filter(p => p !== permission)
        }))
    }

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (!formData.name || formData.permissions.length === 0) {
            toast.error('Name and at least one permission are required')
            return
        }
        updateGroupMutation.mutate(formData)
    }

    if (groupLoading) {
        return (
            <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
                <div className="relative max-w-4xl mx-auto space-y-6">
                    <Skeleton className="h-10 w-64" />
                    <Skeleton className="h-96 w-full rounded-3xl" />
                </div>
            </div>
        )
    }

    if (groupError || !group) {
        return (
            <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
                <div className="relative max-w-4xl mx-auto space-y-6">
                    <div className="text-center py-12">
                        <p className="text-red-600 mb-4">Group not found or failed to load.</p>
                        <Link href="/group-and-permission">
                            <Button variant="outline">Back to Permissions</Button>
                        </Link>
                    </div>
                </div>
            </div>
        )
    }

    return (
        <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-purple-400/20 to-pink-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-indigo-400/20 to-purple-500/20 rounded-full blur-lg"></div>
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
                        <h1 className="text-2xl font-bold text-purple-600 dark:text-purple-400 flex items-center gap-2">
                            <Edit3 className="w-6 h-6" />
                            Edit Permission Group
                        </h1>
                        <p className="text-sm text-gray-600 dark:text-gray-400">
                            Editing: {group.name}
                        </p>
                    </div>
                </div>

                {/* Edit Form */}
                <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-purple-300/50 dark:border-purple-700/50">
                    <form onSubmit={handleSubmit} className="space-y-4">
                        <div>
                            <Label htmlFor="name">Group Name</Label>
                            <Input
                                id="name"
                                value={formData.name}
                                onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                                placeholder="e.g., Premium Users"
                                required
                            />
                        </div>

                        <div>
                            <Label htmlFor="description">Description</Label>
                            <Textarea
                                id="description"
                                value={formData.description}
                                onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
                                placeholder="Describe what this group is for..."
                            />
                        </div>

                        <div>
                            <Label>Permissions</Label>
                            <div className="flex gap-2 mb-2">
                                <Select value={selectedPermission} onValueChange={setSelectedPermission}>
                                    <SelectTrigger className="flex-1">
                                        <SelectValue placeholder="Select permission to add" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        {availablePermissions.map(perm => (
                                            <SelectItem key={perm} value={perm}>{perm}</SelectItem>
                                        ))}
                                    </SelectContent>
                                </Select>
                                <Button type="button" onClick={addPermission} disabled={!selectedPermission}>
                                    <Plus className="w-4 h-4" />
                                </Button>
                            </div>
                            <div className="flex flex-wrap gap-2">
                                {formData.permissions.map(perm => (
                                    <Badge key={perm} variant="secondary" className="flex items-center gap-1">
                                        {perm}
                                        <button
                                            type="button"
                                            onClick={() => removePermission(perm)}
                                            className="ml-1 text-red-500 hover:text-red-700"
                                        >
                                            ×
                                        </button>
                                    </Badge>
                                ))}
                            </div>
                        </div>

                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 sm:gap-4">
                            <div>
                                <Label htmlFor="expiry">Default Expiry (Days)</Label>
                                <Input
                                    id="expiry"
                                    type="number"
                                    value={formData.default_expiry_days}
                                    onChange={(e) => setFormData(prev => ({ ...prev, default_expiry_days: e.target.value }))}
                                    placeholder="30"
                                />
                            </div>
                            <div>
                                <Label htmlFor="priority">Priority Level</Label>
                                <Input
                                    id="priority"
                                    type="number"
                                    value={formData.priority_level}
                                    onChange={(e) => setFormData(prev => ({ ...prev, priority_level: parseInt(e.target.value) || 0 }))}
                                    placeholder="0"
                                />
                            </div>
                        </div>

                        <Button type="submit" disabled={updateGroupMutation.isPending} className="w-full">
                            {updateGroupMutation.isPending ? 'Updating...' : 'Update Group'}
                        </Button>
                    </form>
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
