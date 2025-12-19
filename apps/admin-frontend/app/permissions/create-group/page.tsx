'use client'

import { useMutation, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Plus } from 'lucide-react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { useState } from 'react'
import { toast } from 'sonner'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { groupMgmt } from '@/lib/api/group-management-client'

export default function CreateGroupPage() {
    const router = useRouter()
    const queryClient = useQueryClient()

    const [formData, setFormData] = useState({
        name: '',
        description: '',
        permissions: [] as string[],
        default_expiry_days: '',
        priority_level: 0
    })

    const [selectedPermission, setSelectedPermission] = useState('')

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

    const createGroupMutation = useMutation({
        mutationFn: async (data: typeof formData) => {
            return groupMgmt.createPermissionGroup({
                name: data.name,
                description: data.description,
                permissions: data.permissions,
                default_expiry_days: data.default_expiry_days ? parseInt(data.default_expiry_days) : undefined,
                priority_level: data.priority_level
            })
        },
        onSuccess: () => {
            toast.success('Permission group created successfully')
            queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
            queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
            router.push('/permissions')
        },
        onError: (error: any) => {
            // Check for conflict/duplicate error (409)
            const status = error?.response?.status || error?.status;
            const errorMessage = error?.response?.data?.error?.detail || error?.message || '';

            if (status === 409 || errorMessage.toLowerCase().includes('already exists')) {
                toast.error(`A group with this name already exists. Please choose a different name.`)
            } else {
                toast.error(errorMessage || 'Failed to create permission group')
            }
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
        createGroupMutation.mutate(formData)
    }

    return (
        <div className="min-h-screen bg-gradient-to-br from-yellow-50 via-orange-50 to-pink-50 dark:from-gray-900 dark:via-purple-900 dark:to-gray-900 p-3 sm:p-6">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-blue-400/20 to-cyan-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-pink-400/20 to-purple-500/20 rounded-full blur-lg"></div>
            </div>

            <div className="relative max-w-2xl mx-auto space-y-6">
                {/* Header */}
                <div className="flex items-center gap-4 mb-6">
                    <Link
                        href="/permissions"
                        className="p-2 rounded-xl bg-white/80 dark:bg-gray-800/80 hover:bg-white dark:hover:bg-gray-800 transition-colors border border-gray-200 dark:border-gray-700"
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Link>
                    <div>
                        <h1 className="text-2xl font-bold text-blue-600 dark:text-blue-400 flex items-center gap-2">
                            <Plus className="w-6 h-6" />
                            Create Permission Group
                        </h1>
                        <p className="text-sm text-gray-600 dark:text-gray-400">
                            Create a new permission group with custom access rules
                        </p>
                    </div>
                </div>

                {/* Form Card */}
                <div className="bg-white/80 dark:bg-gray-800/80 backdrop-blur-sm rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-blue-300/50 dark:border-blue-700/50">
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

                        <div className="flex gap-4 pt-4">
                            <Link href="/permissions" className="flex-1">
                                <Button type="button" variant="outline" className="w-full">
                                    Cancel
                                </Button>
                            </Link>
                            <Button type="submit" disabled={createGroupMutation.isPending} className="flex-1">
                                {createGroupMutation.isPending ? 'Creating...' : 'Create Group'}
                            </Button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    )
}
