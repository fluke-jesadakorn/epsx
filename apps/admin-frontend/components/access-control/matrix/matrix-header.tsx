import { Plus, Search, Shield } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { toast } from 'sonner';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';

interface MatrixHeaderProps {
    className?: string;
    search: string;
    setSearch: (value: string) => void;
}

export function MatrixHeader({
    className,
    search,
    setSearch,
}: MatrixHeaderProps) {
    const router = useRouter();

    return (
        <div
            className={cn(
                'flex flex-col sm:flex-row gap-4 justify-between items-start sm:items-center bg-card/50 p-1 rounded-xl',
                className
            )}
        >
            <div className="relative w-full sm:max-w-md">
                <Search className="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input
                    placeholder="Search permissions..."
                    className="pl-9 h-10 bg-background/50 border-border/50 focus:bg-background transition-colors"
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                />
            </div>

            <div className="flex gap-2 w-full sm:w-auto">
                <Button
                    variant="outline"
                    size="sm"
                    onClick={() => router.push('/wallet-management/access/plans')}
                >
                    <Plus className="h-4 w-4 mr-2" />
                    New Policy
                </Button>
                <Button
                    size="sm"
                    onClick={() => {
                        // Ideally open a modal, but for now specific route or placeholder
                        toast.info('Standard permission creation modal coming next.');
                    }}
                >
                    <Shield className="h-4 w-4 mr-2" />
                    New Permission
                </Button>
            </div>
        </div>
    );
}
