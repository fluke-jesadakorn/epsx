import { cn } from '@/lib/utils';

// Generate a deterministic gradient based on policy name
function getAvatarGradient(name: string): string {
    let hash = 0;
    for (let i = 0; i < name.length; i++) {
        hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    const hue1 = Math.abs(hash % 360);
    const hue2 = (hue1 + 40) % 360;
    return `linear-gradient(135deg, hsl(${hue1}, 70%, 60%) 0%, hsl(${hue2}, 80%, 50%) 100%)`;
}

// Get initials from policy name
function getInitials(name: string): string {
    return name
        .split(' ')
        .map((word) => word[0])
        .join('')
        .slice(0, 2)
        .toUpperCase();
}

interface PolicyAvatarProps {
    name: string;
    isSystemGroup?: boolean;
    className?: string;
}

export function PolicyAvatar({
    name,
    isSystemGroup,
    className,
}: PolicyAvatarProps) {
    return (
        <div className={cn('relative group/avatar', className)}>
            <div
                className={cn(
                    'relative flex h-12 w-12 shrink-0 items-center justify-center rounded-xl text-sm font-bold shadow-lg',
                    'text-white transition-all duration-300',
                    'group-hover/avatar:scale-105 group-hover/avatar:shadow-xl'
                )}
                style={{ background: getAvatarGradient(name) }}
            >
                {getInitials(name)}

                {/* System/Protected indicator */}
                {isSystemGroup === true && (
                    <div
                        className={cn(
                            'absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-white dark:border-gray-900',
                            'bg-purple-500'
                        )}
                    >
                        <span
                            className={cn(
                                'absolute inset-0 rounded-full bg-purple-500',
                                'animate-ping opacity-40'
                            )}
                        />
                    </div>
                )}
            </div>
        </div>
    );
}
