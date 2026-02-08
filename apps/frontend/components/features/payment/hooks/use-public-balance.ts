
import { useEffect, useState } from 'react';
import { createPublicClient, formatUnits, http } from 'viem';
import { foundry } from 'viem/chains';
import { useChainId } from 'wagmi';

/**
 * Hook to fetch token balance using a public client, bypassing the connected wallet's provider.
 * This is useful for verifying on-chain state when the wallet's RPC might be misconfigured (e.g. localhost vs tailscale).
 */
export function usePublicBalance(
    tokenAddress: string | undefined,
    userAddress: string | undefined
) {
    const chainId = useChainId();
    const [balance, setBalance] = useState<string | null>(null);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!tokenAddress || !userAddress || chainId !== 31337) {
            setBalance(null);
            return;
        }

        const fetchBalance = async () => {
            setIsLoading(true);
            setError(null);
            try {
                // Determine the correct RPC URL for Anvil based on browser window
                const hostname = window.location.hostname;
                const rpcUrl = (hostname === 'localhost' || hostname === '127.0.0.1')
                    ? 'http://127.0.0.1:8545'
                    : `http://${hostname}:8545`;

                console.log('🔍 [usePublicBalance] Using RPC URL:', rpcUrl);

                // Create a temporary public client
                const publicClient = createPublicClient({
                    chain: foundry,
                    transport: http(rpcUrl)
                });

                // Mock Token ABI - balanceOf
                const abi = [{
                    name: 'balanceOf',
                    type: 'function',
                    inputs: [{ name: 'account', type: 'address' }],
                    outputs: [{ name: '', type: 'uint256' }],
                    stateMutability: 'view',
                }] as const;

                const rawBalance = await publicClient.readContract({
                    address: tokenAddress as `0x${string}`,
                    abi,
                    functionName: 'balanceOf',
                    args: [userAddress as `0x${string}`]
                });

                // Assume 18 decimals for our mock tokens
                const formatted = formatUnits(rawBalance, 18);
                setBalance(formatted);
            } catch (err) {
                console.error('Failed to fetch public balance:', err);
                setError(err instanceof Error ? err.message : 'Failed to fetch');
            } finally {
                setIsLoading(false);
            }
        };

        fetchBalance();
        // Poll every 5 seconds
        const interval = setInterval(fetchBalance, 5000);
        return () => clearInterval(interval);
    }, [tokenAddress, userAddress, chainId]);

    return { balance, isLoading, error };
}
