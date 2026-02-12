 
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
    symbol: string;
    name: string;
    decimals: number;
    image?: string;
}

const TOKEN_INFO: Record<string, TokenInfo> = {
    USDT: {
        symbol: 'USDT',
        name: 'Tether USD',
        decimals: 18,
        image: 'https://cryptologos.cc/logos/tether-usdt-logo.png',
    },
    USDC: {
        symbol: 'USDC',
        name: 'USD Coin',
        decimals: 18,
        image: 'https://cryptologos.cc/logos/usd-coin-usdc-logo.png',
    },
    DAI: {
        symbol: 'DAI',
        name: 'Dai Stablecoin',
        decimals: 18,
        image: 'https://cryptologos.cc/logos/multi-collateral-dai-dai-logo.png',
    },
};

export function useAddTokenToWallet() {
    const { connector } = useAccount();
    const chainId = useChainId();
    const [isAdding, setIsAdding] = useState(false);
    const [addedTokens, setAddedTokens] = useState<Set<string>>(new Set());

    const addToken = useCallback(async (symbol: string): Promise<boolean> => {
        if (!connector || isAdding) {return false;}

        const tokenKey = `${chainId}-${symbol}`;
        if (addedTokens.has(tokenKey)) {
            devLog(`Token ${symbol} already added to wallet`);
            return true;
        }

        setIsAdding(true);

        try {
            const tokenAddress = getTokenAddress(symbol, chainId);
            const tokenInfo = TOKEN_INFO[symbol];

            devLog(`📝 Adding token ${symbol} to wallet:`, {
                address: tokenAddress,
                chainId,
                decimals: tokenInfo.decimals,
            });

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
                devLog(`✅ Token ${symbol} added to wallet successfully`);
                setAddedTokens(prev => new Set(prev).add(tokenKey));
                // Small delay to allow MetaMask to refresh balances
                await new Promise(resolve => setTimeout(resolve, 500));
                return true;
            } else {
                devLog(`❌ User rejected adding ${symbol} token`);
                return false;
            }
        } catch (error) {
      // Error logged silently
            devLog(`❌ Error adding token: ${error instanceof Error ? error.message : String(error)}`);
            return false;
        } finally {
            setIsAdding(false);
        }
    }, [connector, chainId, isAdding, addedTokens]);

    const isTokenAdded = useCallback((symbol: string): boolean => {
        return addedTokens.has(`${chainId}-${symbol}`);
    }, [chainId, addedTokens]);

    return {
        addToken,
        isAdding,
        isTokenAdded,
    };
}
