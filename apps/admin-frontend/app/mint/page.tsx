'use client';

import { useEffect, useState } from 'react';
import { parseUnits } from 'viem';
import { useAccount, useWaitForTransactionReceipt, useWriteContract } from 'wagmi';

// Deterministic addresses for Anvil Local
const MOCK_USDT_ADDRESS = '0x5FbDB2315678afecb367f032d93F642f64180aa3';
const MOCK_USDC_ADDRESS = '0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512';

const MINT_ABI = [
    {
        "inputs": [
            { "internalType": "address", "name": "to", "type": "address" },
            { "internalType": "uint256", "name": "amount", "type": "uint256" }
        ],
        "name": "mint",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    }
] as const;

export default function MintPage() {
    const { address, isConnected, chainId } = useAccount();
    const [amount, setAmount] = useState('1000');
    const [recipient, setRecipient] = useState('');

    // Set default recipient to connected address
    useEffect(() => {
        if (address && !recipient) {
            setRecipient(address);
        }
    }, [address, recipient]);

    const { writeContract, data: hash, isPending, error } = useWriteContract();

    const { isLoading: isConfirming, isSuccess: isConfirmed } =
        useWaitForTransactionReceipt({
            hash,
        });

    const handleMint = (tokenAddress: `0x${string}`) => {
        if (!recipient) return;
        try {
            writeContract({
                address: tokenAddress,
                abi: MINT_ABI,
                functionName: 'mint',
                args: [recipient as `0x${string}`, parseUnits(amount, 18)],
            });
        } catch (err) {
            console.error("Minting failed", err);
        }
    }

    return (
        <div className="p-8 max-w-4xl mx-auto">
            <div className="mb-8">
                <h1 className="text-3xl font-bold text-gray-900 dark:text-white mb-2">Mint Test Tokens</h1>
                <p className="text-gray-600 dark:text-gray-400">
                    Mint mock USDT/USDC tokens to your wallet for testing existing payment flows.
                    <br />
                    <span className="text-sm font-mono text-purple-600 dark:text-purple-400">Only works on Anvil Local (Chain ID: 31337)</span>
                </p>
            </div>

            {chainId !== 31337 && (
                <div className="bg-yellow-50 dark:bg-yellow-900/30 border-l-4 border-yellow-500 text-yellow-700 dark:text-yellow-200 p-4 mb-8 rounded-r">
                    <div className="flex">
                        <div className="flex-shrink-0">⚠️</div>
                        <div className="ml-3">
                            <p className="font-bold">Wrong Network Detected</p>
                            <p className="text-sm">You are connected to Chain ID {chainId}. Please switch to Anvil Local (31337) to mint tokens.</p>
                        </div>
                    </div>
                </div>
            )}

            <div className="grid gap-8 md:grid-cols-2">
                {/* Configuration Card */}
                <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6 border border-gray-200 dark:border-gray-700">
                    <h2 className="text-xl font-semibold mb-6 text-gray-900 dark:text-white flex items-center gap-2">
                        ⚙️ Configuration
                    </h2>

                    <div className="space-y-4">
                        <div>
                            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                Recipient Address
                            </label>
                            <input
                                type="text"
                                value={recipient}
                                onChange={(e) => setRecipient(e.target.value)}
                                placeholder="0x..."
                                className="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500 outline-none font-mono text-sm"
                            />
                        </div>

                        <div>
                            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                Amount to Mint
                            </label>
                            <input
                                type="number"
                                value={amount}
                                onChange={(e) => setAmount(e.target.value)}
                                className="w-full px-4 py-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-purple-500 outline-none"
                            />
                        </div>
                    </div>
                </div>

                {/* Actions Card */}
                <div className="bg-white dark:bg-gray-800 rounded-xl shadow-lg p-6 border border-gray-200 dark:border-gray-700">
                    <h2 className="text-xl font-semibold mb-6 text-gray-900 dark:text-white flex items-center gap-2">
                        🚀 Actions
                    </h2>

                    {!isConnected ? (
                        <div className="text-center py-8">
                            <p className="text-gray-500 dark:text-gray-400 mb-4">Connect wallet to mint tokens</p>
                            {/* Wallet connection is handled in header/sidebar usually, but maybe add button here if needed */}
                        </div>
                    ) : (
                        <div className="space-y-4">
                            <button
                                onClick={() => handleMint(MOCK_USDT_ADDRESS)}
                                disabled={isPending || chainId !== 31337}
                                className="w-full py-3 px-4 bg-green-600 hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
                            >
                                {isPending ? 'Minting...' : 'Mint USDT'}
                            </button>

                            <button
                                onClick={() => handleMint(MOCK_USDC_ADDRESS)}
                                disabled={isPending || chainId !== 31337}
                                className="w-full py-3 px-4 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
                            >
                                {isPending ? 'Minting...' : 'Mint USDC'}
                            </button>
                        </div>
                    )}

                    {/* Status Feedback */}
                    {isConfirming && (
                        <div className="mt-6 p-4 bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300 rounded-lg text-sm text-center animate-pulse">
                            Waiting for confirmation...
                        </div>
                    )}

                    {isConfirmed && (
                        <div className="mt-6 p-4 bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300 rounded-lg text-sm text-center">
                            ✅ Tokens minted successfully!
                            {hash && (
                                <div className="mt-1 text-xs font-mono opacity-80 break-all">
                                    Tx: {hash}
                                </div>
                            )}
                        </div>
                    )}

                    {error && (
                        <div className="mt-6 p-4 bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-300 rounded-lg text-sm break-words">
                            Error: {error.message.split('\n')[0]}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
