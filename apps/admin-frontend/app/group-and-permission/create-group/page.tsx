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
 *
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
        staleTime: 30000, // 30 seconds
    })

    // Extract permission strings and identify system permissions
    const availablePermissions = useMemo(() => {
        console.log('Permission Definitions:', permissionDefinitions)
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
        await groupMgmt.createPermissionDefinition({ permission })
        // Invalidate the query to refetch
        queryClient.invalidateQueries({ queryKey: ['permission-definitions'] })
    }

    // Handler to delete a permission from the database
    const handleDeletePermission = async (permission: string) => {
        await groupMgmt.deletePermissionByName(permission)
        // Invalidate the query to refetch
        queryClient.invalidateQueries({ queryKey: ['permission-definitions'] })
        // Also remove from selected if it was selected
        setFormData(prev => ({
            ...prev,
            permissions: prev.permissions.filter(p => p !== permission)
        }))
    }

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
            const status = error?.response?.status || error?.status;
            const errorMessage = error?.response?.data?.error?.detail || error?.message || '';

            if (status === 409 || errorMessage.toLowerCase().includes('already exists')) {
                toast.error(`A group with this name already exists. Please choose a different name.`)
            } else {
                toast.error(errorMessage || 'Failed to create permission group')
            }
        }
    })

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault()
        if (!formData.name || formData.permissions.length === 0) {
            toast.error('Name and at least one permission are required')
            return
        }
        createGroupMutation.mutate(formData)
    }

    return (
        <div className="min-h-screen p-4 sm:p-8 relative overflow-hidden">
            {/* Ambient Background Effects */}
            <div className="absolute top-0 left-0 w-full h-full pointer-events-none">
                <div className="absolute top-[-10%] left-[-10%] w-[40%] h-[40%] bg-primary/10 rounded-full blur-[120px]" />
                <div className="absolute bottom-[-10%] right-[-10%] w-[40%] h-[40%] bg-secondary/10 rounded-full blur-[120px]" />
            </div>

            <div className="relative max-w-4xl mx-auto space-y-8 animate-in fade-in slide-in-from-bottom-4 duration-700">
                {/* Header Section */}
                <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                    <div className="flex items-center gap-4">
                        <Link
                            href="/group-and-permission"
                            className="group p-2.5 rounded-xl bg-card hover:bg-muted border border-border/50 transition-all duration-300 backdrop-blur-md"
                        >
                            <ArrowLeft className="h-5 w-5 text-muted-foreground group-hover:text-primary group-hover:-translate-x-0.5 transition-all" />
                        </Link>
                        <div>
                            <div className="flex items-center gap-2 mb-1">
                                <div className="p-1.5 rounded-lg bg-primary/10 text-primary border border-primary/20">
                                    <Shield className="w-5 h-5" />
                                </div>
                                <h1 className="text-2xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                                    Create Permission Group
                                </h1>
                            </div>
                            <p className="text-sm text-muted-foreground font-medium">
                                Design custom access rules for your platform users
                            </p>
                        </div>
                    </div>
                </div>

                {/* Main Form Card */}
                <div className="bg-card/50 backdrop-blur-xl border border-border/50 shadow-2xl rounded-3xl overflow-hidden">
                    <div className="border-b border-border/50 p-6 sm:px-8">
                        <h2 className="text-lg font-semibold flex items-center gap-2 text-foreground">
                            Group Configuration
                        </h2>
                        <p className="text-sm text-muted-foreground mt-1">
                            Provide the details and permissions for this new security group.
                        </p>
                    </div>

                    <div className="p-6 sm:p-8">
                        <form onSubmit={handleSubmit} className="space-y-6">
                            {/* Group Name & Description */}
                            <div className="grid grid-cols-1 gap-6">
                                <div className="space-y-2.5">
                                    <Label htmlFor="name" className="text-muted-foreground flex items-center gap-2 text-xs uppercase tracking-wider font-bold">
                                        <Type className="w-3.5 h-3.5" /> Group Name
                                    </Label>
                                    <Input
                                        id="name"
                                        value={formData.name}
                                        onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                                        placeholder="e.g., Alpha Traders Elite"
                                        className="bg-muted/30 border-border focus:ring-2 focus:ring-primary h-12 rounded-xl"
                                        required
                                    />
                                </div>

                                <div className="space-y-2.5">
                                    <Label htmlFor="description" className="text-muted-foreground flex items-center gap-2 text-xs uppercase tracking-wider font-bold">
                                        <FileText className="w-3.5 h-3.5" /> Description
                                    </Label>
                                    <Textarea
                                        id="description"
                                        value={formData.description}
                                        onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
                                        placeholder="Briefly describe the purpose of this group..."
                                        className="bg-muted/30 border-border focus:ring-2 focus:ring-primary min-h-[100px] rounded-xl resize-none"
                                    />
                                </div>
                            </div>

                            {/* Permissions Selector */}
                            <div className="space-y-4 pt-2">
                                <Label className="text-muted-foreground flex items-center gap-2 text-xs uppercase tracking-wider font-bold px-1">
                                    <Shield className="w-3.5 h-3.5" /> Access Permissions
                                </Label>
                                <PermissionTransferList
                                    available={availablePermissions}
                                    selected={formData.permissions}
                                    onChange={(newPermissions) => setFormData(prev => ({ ...prev, permissions: newPermissions }))}
                                    onCreatePermission={handleCreatePermission}
                                    onDeletePermission={handleDeletePermission}
                                    systemPermissions={systemPermissions}
                                    isLoading={isLoadingPermissions}
                                />
                            </div>

                            {/* Additional Settings Grid */}
                            <div className="grid grid-cols-1 sm:grid-cols-2 gap-6 pt-2">
                                <div className="space-y-2.5">
                                    <Label htmlFor="expiry" className="text-muted-foreground flex items-center gap-2 text-xs uppercase tracking-wider font-bold">
                                        <Calendar className="w-3.5 h-3.5" /> Default Expiry (Days)
                                    </Label>
                                    <Input
                                        id="expiry"
                                        type="number"
                                        value={formData.default_expiry_days}
                                        onChange={(e) => setFormData(prev => ({ ...prev, default_expiry_days: e.target.value }))}
                                        placeholder="e.g., 30"
                                        className="bg-muted/30 border-border focus:ring-2 focus:ring-primary h-12 rounded-xl"
                                    />
                                </div>
                                <div className="space-y-2.5">
                                    <Label htmlFor="priority" className="text-muted-foreground flex items-center gap-2 text-xs uppercase tracking-wider font-bold">
                                        <Hash className="w-3.5 h-3.5" /> Priority Level
                                    </Label>
                                    <Input
                                        id="priority"
                                        type="number"
                                        value={formData.priority_level}
                                        onChange={(e) => setFormData(prev => ({ ...prev, priority_level: parseInt(e.target.value) || 0 }))}
                                        placeholder="0"
                                        className="bg-muted/30 border-border focus:ring-2 focus:ring-primary h-12 rounded-xl"
                                    />
                                </div>
                            </div>

                            {/* Action Buttons */}
                            <div className="flex flex-col sm:flex-row gap-4 pt-6 border-t border-border/50">
                                <Link href="/group-and-permission" className="flex-1">
                                    <button
                                        type="button"
                                        className="w-full h-12 rounded-xl border border-border text-muted-foreground font-semibold hover:bg-muted transition-colors"
                                    >
                                        Cancel
                                    </button>
                                </Link>
                                <button
                                    type="submit"
                                    disabled={createGroupMutation.isPending}
                                    className="flex-1 h-12 rounded-xl bg-primary text-primary-foreground font-bold shadow-lg shadow-primary/20 group relative overflow-hidden transition-all duration-300 hover:opacity-90 disabled:opacity-50"
                                >
                                    {createGroupMutation.isPending ? (
                                        <div className="flex items-center justify-center gap-2">
                                            <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
                                            <span>Creating Group...</span>
                                        </div>
                                    ) : (
                                        <div className="flex items-center justify-center gap-2">
                                            <Plus className="w-5 h-5 group-hover:rotate-90 transition-transform duration-300" />
                                            <span>Initialize Group</span>
                                        </div>
                                    )}
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        </div>
    )
}
