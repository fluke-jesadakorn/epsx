
export interface AccessLevelConfig {
    value: string;
    label: string;
    color: string;
    description: string;
}

// Helper to get color based on access level name/slug
// Apps can pass dynamic groups, but we keep some UI defaults for standard tier names
export const getAccessLevelColor = (level: string): string => {
    const normalizedLevel = level.toLowerCase();

    if (normalizedLevel.includes('bronze')) {return 'text-amber-600';}
    if (normalizedLevel.includes('silver')) {return 'text-gray-500';}
    if (normalizedLevel.includes('gold')) {return 'text-yellow-500';}
    if (normalizedLevel.includes('platinum')) {return 'text-purple-600';}
    if (normalizedLevel.includes('enterprise')) {return 'text-blue-600';}

    // Default fallback
    return 'text-gray-600';
};

export const maskKeyPrefix = (prefix: string): string => {
    if (prefix.length <= 8) {
        return `${prefix}...`;
    }
    const start = prefix.slice(0, 4);
    const end = prefix.slice(-3);
    return `${start}...${end}`;
};

export const truncateWallet = (address: string): string => {
    if (!address || address.length < 12) { return address || 'Unknown'; }
    return `${address.slice(0, 6)}...${address.slice(-4)}`;
};

export const getStatusColor = (status: string) => {
    switch (status) {
        case 'active':
            return 'bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-200';
        case 'revoked':
            return 'bg-red-100 dark:bg-red-900/50 text-red-800 dark:text-red-200';
        case 'expired':
            return 'bg-yellow-100 dark:bg-yellow-900/50 text-yellow-800 dark:text-yellow-200';
        default:
            return 'bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-300';
    }
};
