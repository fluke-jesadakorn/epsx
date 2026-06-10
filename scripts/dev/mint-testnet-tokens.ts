#!/usr/bin/env bun
/**
 * Mint Test Tokens on BSC Testnet
 * 
 * This script mints USDT/USDC test tokens on BSC Testnet.
 * These are community test tokens that have public mint functions.
 * 
 * USDT Testnet: 0x66E972502A34A625828C544a1914E8D8cc2A9dE5 (PandaTool)
 * USDC Testnet: 0x64544969ed7EBf5f083679233325356EbE738930
 * 
 * Usage:
 *   bun scripts/dev/mint-testnet-tokens.ts <recipient_address> [amount] [token]
 * 
 * Example:
 *   bun scripts/dev/mint-testnet-tokens.ts 0x9Dd4Db1aA7826A94E479f3387A464772f1E2C4B7 1000 USDT
 */

import { createPublicClient, createWalletClient, formatUnits, http, parseUnits } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'
import { bscTestnet } from 'viem/chains'

// BSC Testnet Token Addresses
const TOKENS = {
    USDT: '0x66E972502A34A625828C544a1914E8D8cc2A9dE5' as const,
    USDC: '0x64544969ed7EBf5f083679233325356EbE738930' as const,
}

// PandaTool Test Token ABI (includes mint function)
const TEST_TOKEN_ABI = [
    {
        type: 'function',
        name: 'mint',
        inputs: [
            { name: 'to', type: 'address' },
            { name: 'amount', type: 'uint256' }
        ],
        outputs: [],
        stateMutability: 'nonpayable'
    },
    {
        type: 'function',
        name: 'balanceOf',
        inputs: [{ name: 'account', type: 'address' }],
        outputs: [{ type: 'uint256' }],
        stateMutability: 'view'
    },
    {
        type: 'function',
        name: 'decimals',
        inputs: [],
        outputs: [{ type: 'uint8' }],
        stateMutability: 'view'
    },
    {
        type: 'function',
        name: 'symbol',
        inputs: [],
        outputs: [{ type: 'string' }],
        stateMutability: 'view'
    }
] as const

async function main() {
    const args = process.argv.slice(2)

    if (args.length < 1) {
        console.log(`
╔═══════════════════════════════════════════════════════════════════╗
║          🪙 BSC Testnet Token Minter                              ║
╠═══════════════════════════════════════════════════════════════════╣
║ Usage:                                                            ║
║   bun scripts/dev/mint-testnet-tokens.ts <address> [amount] [token]║
║                                                                   ║
║ Arguments:                                                        ║
║   address  - Recipient wallet address                             ║
║   amount   - Amount to mint (default: 1000)                       ║
║   token    - USDT or USDC (default: USDT)                        ║
║                                                                   ║
║ Example:                                                          ║
║   bun scripts/dev/mint-testnet-tokens.ts 0x9Dd4Db... 1000 USDT   ║
╚═══════════════════════════════════════════════════════════════════╝
        `)
        process.exit(1)
    }

    const recipient = args[0] as `0x${string}`
    const amount = args[1] ? parseFloat(args[1]) : 1000
    const tokenSymbol = (args[2]?.toUpperCase() || 'USDT') as keyof typeof TOKENS

    if (!TOKENS[tokenSymbol]) {
        console.error(`❌ Unknown token: ${tokenSymbol}. Use USDT or USDC.`)
        process.exit(1)
    }

    // Check for private key
    const privateKey = process.env.BSC_TESTNET_PRIVATE_KEY || process.env.PRIVATE_KEY
    if (!privateKey) {
        console.log(`
⚠️  No private key found!

Since BSC Testnet test tokens require a transaction to mint, you need a private key.

Option 1: Use the web faucet directly
   USDT: https://testnet.bscscan.com/token/0x66E972502A34A625828C544a1914E8D8cc2A9dE5#writeContract
   USDC: https://testnet.bscscan.com/token/0x64544969ed7EBf5f083679233325356EbE738930#writeContract
   
   1. Connect wallet to BSC Testnet
   2. Go to "Write Contract" tab
   3. Call "mint" with your address and amount (e.g., 1000000000000000000000 for 1000 tokens)

Option 2: Set environment variable and run again
   export BSC_TESTNET_PRIVATE_KEY=0x...your_private_key
   bun scripts/dev/mint-testnet-tokens.ts ${recipient} ${amount} ${tokenSymbol}
        `)
        process.exit(1)
    }

    const tokenAddress = TOKENS[tokenSymbol]
    const amountWei = parseUnits(amount.toString(), 18)

    console.log(`
╔═══════════════════════════════════════════════════════════════════╗
║          🪙 Minting Test Tokens on BSC Testnet                    ║
╠═══════════════════════════════════════════════════════════════════╣
║ Token:     ${tokenSymbol.padEnd(52)}║
║ Contract:  ${tokenAddress}                   ║
║ Recipient: ${recipient}                   ║
║ Amount:    ${amount.toLocaleString().padEnd(52)}║
╚═══════════════════════════════════════════════════════════════════╝
    `)

    // Normalize private key (add 0x prefix if missing)
    const normalizedPrivateKey = privateKey.startsWith('0x') ? privateKey : `0x${privateKey}`

    // BSC Testnet RPC URL (use publicnode for better reliability)
    const RPC_URL = 'https://bsc-testnet-rpc.publicnode.com'

    // Create clients
    const account = privateKeyToAccount(normalizedPrivateKey as `0x${string}`)

    const publicClient = createPublicClient({
        chain: bscTestnet,
        transport: http(RPC_URL, { timeout: 30000 })
    })

    const walletClient = createWalletClient({
        account,
        chain: bscTestnet,
        transport: http(RPC_URL, { timeout: 30000 })
    })

    try {
        // Check current balance
        console.log('📊 Checking current balance...')
        const balanceBefore = await publicClient.readContract({
            address: tokenAddress,
            abi: TEST_TOKEN_ABI,
            functionName: 'balanceOf',
            args: [recipient]
        })
        console.log(`   Current balance: ${formatUnits(balanceBefore, 18)} ${tokenSymbol}`)

        // Mint tokens
        console.log('\n🔄 Sending mint transaction...')
        const hash = await walletClient.writeContract({
            address: tokenAddress,
            abi: TEST_TOKEN_ABI,
            functionName: 'mint',
            args: [recipient, amountWei]
        })
        console.log(`   Tx Hash: ${hash}`)
        console.log(`   Explorer: https://testnet.bscscan.com/tx/${hash}`)

        // Wait for confirmation
        console.log('\n⏳ Waiting for confirmation...')
        const receipt = await publicClient.waitForTransactionReceipt({ hash })
        console.log(`   Status: ${receipt.status === 'success' ? '✅ Success' : '❌ Failed'}`)
        console.log(`   Block: ${receipt.blockNumber}`)

        // Check new balance
        console.log('\n📊 Checking new balance...')
        const balanceAfter = await publicClient.readContract({
            address: tokenAddress,
            abi: TEST_TOKEN_ABI,
            functionName: 'balanceOf',
            args: [recipient]
        })
        console.log(`   New balance: ${formatUnits(balanceAfter, 18)} ${tokenSymbol}`)

        console.log(`
╔═══════════════════════════════════════════════════════════════════╗
║ ✅ Minting Complete!                                              ║
╚═══════════════════════════════════════════════════════════════════╝
        `)
    } catch (error: any) {
        console.error('\n❌ Error:', error.message)

        if (error.message.includes('insufficient funds')) {
            console.log(`
💡 You need testnet BNB for gas fees!
   Get free tBNB from: https://testnet.bnbchain.org/faucet-smart
            `)
        }

        if (error.message.includes('execution reverted')) {
            console.log(`
💡 The mint function might not be publicly callable on this token.
   Try using the web interface instead:
   https://testnet.bscscan.com/token/${tokenAddress}#writeContract
            `)
        }

        process.exit(1)
    }
}

main()
