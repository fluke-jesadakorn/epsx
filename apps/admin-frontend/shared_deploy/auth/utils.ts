/**
 * Auth-related utilities
 */

/**
 * Format wallet address for display (0x1234...5678)
 * @param address The full wallet address
 * @returns Formatted address string
 */
export function formatAddress(address: string | undefined): string {
    if (!address) return '';
    if (address.length < 10) return address;
    return `${address.substring(0, 6)}...${address.substring(address.length - 4)}`;
}
