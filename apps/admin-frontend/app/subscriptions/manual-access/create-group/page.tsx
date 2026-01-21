'use client'

import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { ArrowLeft, Calendar, FileText, Hash, Plus, Shield, Type } from 'lucide-react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { useMemo, useState } from 'react'
import { toast } from 'sonner'

import { PermissionTransferList } from '@/components/groups/PermissionTransferList'
import { Label } from '@/components/ui/label'
import { groupMgmt, PermissionDefinitionDto } from '@/lib/api/group-management-client'
import {
    Input,
    Textarea
} from '@/shared/components'

/**
 * Create Group Page
 * Part of the Subscription & Access hub > Manual Access
 */
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

    // Fetch permission definitions from database
    const { data: permissionDefinitions = [], isLoading: isLoadingPermissions } = useQuery({
        queryKey: ['permission-definitions'],
        queryFn: () => groupMgmt.getPermissionDefinitions(),
        staleTime: 30000,
    })

    // Extract permission strings and identify system permissions
    const availablePermissions = useMemo(() => {
        return permissionDefinitions.map((p: PermissionDefinitionDto) => {
            if (!p || !p.permission) {
                console.warn('Invalid permission definition:', p)
            }
            return p.permission
        })
    }, [permissionDefinitions])

    const systemPermissions = useMemo(() =>
        new Set<string>(permissionDefinitions.filter((p: PermissionDefinitionDto) => p.is_system).map((p: PermissionDefinitionDto) => p.permission)),
        [permissionDefinitions]
    )

    // Handler to create a new permission in the database
    const handleCreatePermission = async (permission: string) => {
        await groupMgmt.createPermissionDefinition({
            permission,
            platform: permission.split(':')[0],
            category: permission.split(':')[1],
        })
        queryClient.invalidateQueries({ queryKey: ['permission-definitions'] })
    }

    // Handler to delete a permission from the database
    const handleDeletePermission = async (permission: string) => {
        await groupMgmt.deletePermissionByName(permission)
        queryClient.invalidateQueries({ queryKey: ['permission-definitions'] })
    }

    const createGroupMutation = useMutation({
        mutationFn: async (data: typeof formData) => {
            return groupMgmt.createPermissionGroup({
                name: data.name,
                description: data.description,
                permissions: data.permissions,
                default_expiry_days: data.default_expiry_days ? parseInt(data.default_expiry_days) : undefined,
                priority_level: data.priority_level,
            })
        },
        onSuccess: () => {
            toast.success('Permission group created successfully')
            queryClient.invalidateQueries({ queryKey: ['permission-groups'] })
            queryClient.invalidateQueries({ queryKey: ['group-analytics'] })
            router.push('/subscriptions/manual-access')
        },
        onError: (error: any) => {
            toast.error(error.message || 'Failed to create group')
        }
    })

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (!formData.name.trim()) {
            toast.error('Group name is required')
            return
        }
        if (formData.permissions.length === 0) {
            toast.error('At least one permission is required')
            return
        }
        createGroupMutation.mutate(formData)
    }

    return (
        <div className="min-h-screen bg-background p-3 sm:p-6">
            {/* Background Decorations */}
            <div className="fixed inset-0 overflow-hidden pointer-events-none">
                <div className="absolute top-20 left-20 w-32 h-32 bg-gradient-to-r from-blue-400/20 to-cyan-500/20 rounded-full blur-xl"></div>
                <div className="absolute top-40 right-32 w-24 h-24 bg-gradient-to-r from-indigo-400/20 to-blue-500/20 rounded-full blur-lg"></div>
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
                    <div>
                        <h1 className="text-2xl font-bold text-blue-600 dark:text-blue-400 flex items-center gap-2">
                            <Plus className="w-6 h-6" />
                            Create Permission Group
                        </h1>
                        <p className="text-sm text-muted-foreground">
                            Create a new group with custom permissions
                        </p>
                    </div>
                </div>

                <form onSubmit={handleSubmit} className="space-y-6">
                    {/* Basic Info Card */}
                    <div className="bg-card rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-blue-500/20">
                        <h3 className="text-lg font-semibold mb-4 text-foreground flex items-center gap-2">
                            <FileText className="w-5 h-5 text-blue-500" />
                            Basic Information
                        </h3>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div className="space-y-2">
                                <Label htmlFor="name" className="flex items-center gap-2">
                                    <Type className="w-4 h-4" />
                                    Group Name *
                                </Label>
                                <Input
                                    id="name"
                                    value={formData.name}
                                    onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                                    placeholder="e.g., Beta Testers"
                                    required
                                />
                            </div>
                            <div className="space-y-2">
                                <Label htmlFor="priority_level" className="flex items-center gap-2">
                                    <Hash className="w-4 h-4" />
                                    Priority Level
                                </Label>
                                <Input
                                    id="priority_level"
                                    type="number"
                                    min="0"
                                    value={formData.priority_level}
                                    onChange={(e) => setFormData(prev => ({ ...prev, priority_level: parseInt(e.target.value) || 0 }))}
                                    placeholder="0"
                                />
                                <p className="text-xs text-muted-foreground">Higher priority groups take precedence</p>
                            </div>
                            <div className="md:col-span-2 space-y-2">
                                <Label htmlFor="description">Description</Label>
                                <Textarea
                                    id="description"
                                    value={formData.description}
                                    onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
                                    placeholder="Describe the purpose of this group..."
                                    rows={3}
                                />
                            </div>
                            <div className="space-y-2">
                                <Label htmlFor="default_expiry_days" className="flex items-center gap-2">
                                    <Calendar className="w-4 h-4" />
                                    Default Expiry (Days)
                                </Label>
                                <Input
                                    id="default_expiry_days"
                                    type="number"
                                    min="0"
                                    value={formData.default_expiry_days}
                                    onChange={(e) => setFormData(prev => ({ ...prev, default_expiry_days: e.target.value }))}
                                    placeholder="Leave empty for no expiry"
                                />
                                <p className="text-xs text-muted-foreground">Auto-expire assignments after this many days</p>
                            </div>
                        </div>
                    </div>

                    {/* Permissions Card */}
                    <div className="bg-card rounded-2xl sm:rounded-3xl p-6 shadow-xl border-2 border-purple-500/20">
                        <h3 className="text-lg font-semibold mb-4 text-foreground flex items-center gap-2">
                            <Shield className="w-5 h-5 text-purple-500" />
                            Permissions
                        </h3>
                        <PermissionTransferList
                            available={availablePermissions}
                            selected={formData.permissions}
                            onChange={(permissions) => setFormData(prev => ({ ...prev, permissions }))}
                            onCreatePermission={handleCreatePermission}
                            onDeletePermission={handleDeletePermission}
                            systemPermissions={systemPermissions}
                            isLoading={isLoadingPermissions}
                        />
                    </div>

                    {/* Actions */}
                    <div className="flex gap-4">
                        <Link href="/subscriptions/manual-access" className="flex-1">
                            <button
                                type="button"
                                className="w-full px-6 py-3 rounded-xl font-semibold bg-muted text-foreground hover:bg-muted/80 transition-colors"
                            >
                                Cancel
                            </button>
                        </Link>
                        <button
                            type="submit"
                            disabled={createGroupMutation.isPending}
                            className="flex-1 px-6 py-3 rounded-xl font-semibold bg-gradient-to-r from-blue-500 to-cyan-500 text-white hover:from-blue-600 hover:to-cyan-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                        >
                            {createGroupMutation.isPending ? 'Creating...' : 'Create Group'}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    )
}
