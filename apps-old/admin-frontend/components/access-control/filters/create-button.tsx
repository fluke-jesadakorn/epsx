import { ChevronDown, Plus } from 'lucide-react';

import { Button } from '@/components/ui/button';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

interface PolicyCreateButtonProps {
    onCreatePlan?: () => void;
    onCreateGroup?: () => void;
}

export function PolicyCreateButton({
    onCreatePlan,
    onCreateGroup,
}: PolicyCreateButtonProps) {
    if (!onCreatePlan && !onCreateGroup) {
        return null;
    }

    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button className="gap-2 h-10">
                    <Plus className="h-4 w-4" />
                    <span>New Policy</span>
                    <ChevronDown className="h-4 w-4" />
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-56 rounded-xl p-1">
                {onCreatePlan && (
                    <DropdownMenuItem onClick={onCreatePlan} className="rounded-lg">
                        <span className="mr-2">💳</span>
                        <span>Subscription Plan</span>
                    </DropdownMenuItem>
                )}
                {onCreateGroup && (
                    <DropdownMenuItem onClick={onCreateGroup} className="rounded-lg">
                        <span className="mr-2">👥</span>
                        <span>Manual Group</span>
                    </DropdownMenuItem>
                )}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}
