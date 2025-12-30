/**
 * Hook to add ERC20 tokens to MetaMask using wallet_watchAsset
 * 
 * This helps MetaMask recognize the token and display proper amounts
 * in approval and transfer dialogs.
 */

import { getTokenAddress } from '@/lib/contracts/addresses';
import { devLog } from '@/shared/utils';
import { useCallback, useState } from 'react';
import { useAccount, useChainId } from 'wagmi';

interface TokenInfo {
    symbol: 'USDT' | 'USDC';
    name: string;
    decimals: number;
    image?: string;
}

const TOKEN_INFO: Record<'USDT' | 'USDC', TokenInfo> = {
    USDT: {
        symbol: 'USDT',
        name: 'Tether USD',
        decimals: 18, // BSC uses 18 decimals for USDT
        image: 'https://cryptologos.cc/logos/tether-usdt-logo.png',
    },
    USDC: {
        symbol: 'USDC',
        name: 'USD Coin',
        decimals: 18, // BSC uses 18 decimals for USDC
        image: 'https://cryptologos.cc/logos/usd-coin-usdc-logo.png',
    },
};

export function useAddTokenToWallet() {
    const { connector } = useAccount();
    const chainId = useChainId();
    const [isAdding, setIsAdding] = useState(false);
    const [addedTokens, setAddedTokens] = useState<Set<string>>(new Set());

    const addToken = useCallback(async (symbol: 'USDT' | 'USDC'): Promise<boolean> => {
        if (!connector || isAdding) return false;

        const tokenKey = `${chainId}-${symbol}`;
        if (addedTokens.has(tokenKey)) {
            devLog(`Token ${symbol} already added to wallet`);
            return true;
        }

        setIsAdding(true);

        try {
            const tokenAddress = getTokenAddress(symbol, chainId);
            const tokenInfo = TOKEN_INFO[symbol];

            const provider = await connector.getProvider();

            const wasAdded = await (provider as any).request({
                method: 'wallet_watchAsset',
                params: {
                    type: 'ERC20',
                    options: {
                        address: tokenAddress,
                        symbol: tokenInfo.symbol,
                        decimals: tokenInfo.decimals,
                        image: tokenInfo.image,
                    },
                },
            });

            if (wasAdded) {
                devLog(`✅ Token ${symbol} added to wallet`);
                setAddedTokens(prev => new Set(prev).add(tokenKey));
                return true;
            } else {
                devLog(`❌ User rejected adding ${symbol} token`);
                return false;
            }
        } catch (error) {
            console.error('Failed to add token to wallet:', error);
            return false;
        } finally {
            setIsAdding(false);
        }
    }, [connector, chainId, isAdding, addedTokens]);

    const isTokenAdded = useCallback((symbol: 'USDT' | 'USDC'): boolean => {
        return addedTokens.has(`${chainId}-${symbol}`);
    }, [chainId, addedTokens]);

    return {
        addToken,
        isAdding,
        isTokenAdded,
    };
}
