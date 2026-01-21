/**
 * Group List Page
 * Overview of all permission groups
 */
'use client';

import { useQuery } from '@tanstack/react-query';
import { ArrowLeft, Package, Plus, Shield, Users } from 'lucide-react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { groupMgmt, PermissionGroup } from '@/lib/api/group-management-client';

export default function GroupListPage() {
    const router = useRouter();

    const { data: groups, isLoading } = useQuery({
        queryKey: ['permission-groups'],
        queryFn: async () => await groupMgmt.getPermissionGroups()
    });

    if (isLoading) {
        return (
            <div className="p-6 max-w-6xl mx-auto space-y-6">
                <Skeleton className="h-10 w-1/4" />
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                    <Skeleton className="h-48 rounded-xl" />
                    <Skeleton className="h-48 rounded-xl" />
                    <Skeleton className="h-48 rounded-xl" />
                </div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-gray-50 dark:bg-gray-900 p-4 sm:p-6">
            <div className="max-w-6xl mx-auto space-y-6">

                {/* Header */}
                <div className="flex flex-col sm:flex-row items-center justify-between gap-4">
                    <div className="flex items-center gap-4 w-full sm:w-auto">
                        <Link href="/wallet-management" className="p-2 border rounded-xl hover:bg-white transition-colors">
                            <ArrowLeft className="h-5 w-5" />
                        </Link>
                        <div>
                            <h1 className="text-2xl font-bold flex items-center gap-2 text-gray-900 dark:text-white">
                                <Users className="h-6 w-6 text-purple-600" />
                                Permission Groups
                            </h1>
                            <p className="text-gray-500 text-sm">Manage access tiers and bundles</p>
                        </div>
                    </div>

                    <Link href="/wallet-management/groups/new">
                        <Button className="w-full sm:w-auto gap-2 bg-purple-600 hover:bg-purple-700 text-white">
                            <Plus className="h-4 w-4" />
                            Create Group
                        </Button>
                    </Link>
                </div>

                {/* Groups Grid */}
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {groups?.map((group: PermissionGroup) => (
                        <Card
                            key={group.id}
                            className="hover:shadow-lg transition-all cursor-pointer border-transparent hover:border-purple-200 dark:hover:border-purple-800"
                            onClick={() => router.push(`/wallet-management/groups/${group.id}`)}
                        >
                            <CardContent className="p-5 space-y-4">
                                <div className="flex items-start justify-between">
                                    <div className="h-10 w-10 rounded-lg bg-purple-100 dark:bg-purple-900/30 flex items-center justify-center text-purple-600">
                                        <Package className="h-5 w-5" />
                                    </div>
                                    <Badge variant="outline" className={(group.priority_level || 0) > 0 ? 'bg-amber-50 text-amber-600' : ''}>
                                        Priority: {group.priority_level}
                                    </Badge>
                                </div>

                                <div>
                                    <h3 className="font-semibold text-lg text-gray-900 dark:text-white">{group.name}</h3>
                                    <p className="text-sm text-gray-500 dark:text-gray-400 line-clamp-2 mt-1">
                                        {group.description || 'No description provided'}
                                    </p>
                                </div>

                                <div className="pt-4 border-t flex items-center justify-between text-sm text-gray-500">
                                    <div className="flex items-center gap-1">
                                        <Shield className="h-3.5 w-3.5" />
                                        {group.permissions?.length || 0} permissions
                                    </div>
                                    <span className="text-purple-600 font-medium group-hover:underline">Edit &rarr;</span>
                                </div>
                            </CardContent>
                        </Card>
                    ))}
                </div>
            </div>
        </div>
    );
}
