import { Edit, Eye, MoreHorizontal, Trash2, Users } from 'lucide-react';
import Link from 'next/link';

import { Button } from '@/components/ui/button';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { cn } from '@/lib/utils';

export interface PolicyActionsProps {
    editUrl: string;
    membersUrl: string;
    isSubscription: boolean;
    isSystemGroup?: boolean;
    onDelete?: () => void;
}

export function PolicyActions({
    editUrl,
    membersUrl,
    isSubscription,
    isSystemGroup,
    onDelete,
}: PolicyActionsProps) {
    return (
        <div className="flex items-center gap-1.5">
            <Link href={editUrl}>
                <Button
                    variant="ghost"
                    size="sm"
                    className={cn(
                        'hidden sm:flex h-9 px-4 gap-2 text-sm font-medium rounded-xl',
                        'bg-gradient-to-r from-blue-50 to-indigo-50 text-blue-700',
                        'hover:from-blue-100 hover:to-indigo-100 hover:text-blue-800',
                        'dark:from-blue-900/30 dark:to-indigo-900/30 dark:text-blue-400',
                        'dark:hover:from-blue-900/50 dark:hover:to-indigo-900/50',
                        'transition-all duration-200 hover:scale-105 hover:shadow-md'
                    )}
                >
                    <Eye className="h-4 w-4" />
                    View
                </Button>
            </Link>

            <DropdownMenu>
                <DropdownMenuTrigger asChild>
                    <Button
                        variant="ghost"
                        size="sm"
                        className="h-9 w-9 p-0 rounded-xl hover:bg-muted transition-all duration-200 hover:scale-110"
                    >
                        <MoreHorizontal className="h-4 w-4" />
                    </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-48 rounded-xl p-1">
                    <Link href={editUrl}>
                        <DropdownMenuItem className="rounded-lg">
                            <Edit className="h-4 w-4 mr-2" />
                            <span className="text-sm">Edit Policy</span>
                        </DropdownMenuItem>
                    </Link>
                    <Link href={membersUrl}>
                        <DropdownMenuItem className="rounded-lg">
                            <Users className="h-4 w-4 mr-2" />
                            <span className="text-sm">
                                {isSubscription ? 'View Subscribers' : 'Manage Members'}
                            </span>
                        </DropdownMenuItem>
                    </Link>
                    <DropdownMenuSeparator />
                    {isSystemGroup !== true && onDelete && (
                        <DropdownMenuItem
                            onClick={onDelete}
                            className="text-red-700 dark:text-red-400 rounded-lg"
                        >
                            <Trash2 className="h-4 w-4 mr-2" />
                            <span className="text-sm">Delete Policy</span>
                        </DropdownMenuItem>
                    )}
                </DropdownMenuContent>
            </DropdownMenu>
        </div>
    );
}
