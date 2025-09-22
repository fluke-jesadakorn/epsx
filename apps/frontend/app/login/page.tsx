'use client'

import { WalletConnectAuth } from '@/components/auth/WalletConnectAuth'
import Link from 'next/link'
import { useRouter, useSearchParams } from 'next/navigation'

export const dynamic = 'force-dynamic'

export default function LoginPage() {
  const router = useRouter()
  const searchParams = useSearchParams()
  const redirectTo = searchParams.get('redirectTo') || '/dashboard'

  return (
    <div className="min-h-[70vh] flex items-center justify-center px-4 py-16">
      <div className="w-full max-w-2xl">
        <div className="rounded-2xl border bg-white/80 p-8 shadow-sm backdrop-blur dark:bg-slate-900/80">
          <div className="mb-6 text-center">
            <h1 className="text-2xl font-semibold tracking-tight">Sign in with your Web3 wallet</h1>
            <p className="mt-2 text-sm text-muted-foreground">
              Connect your wallet and sign a message to authenticate. No password required.
            </p>
          </div>

          <div className="my-6">
            <WalletConnectAuth
              onAuthSuccess={() => {
                // After successful auth, redirect to the original destination
                router.replace(redirectTo)
              }}
            />
          </div>

          <div className="mt-8 text-center text-xs text-muted-foreground">
            <p>By connecting a wallet, you agree to our terms and acknowledge our privacy policy.</p>
            <div className="mt-2 flex items-center justify-center gap-4">
              <Link href="/terms" className="underline-offset-2 hover:underline">Terms</Link>
              <Link href="/privacy" className="underline-offset-2 hover:underline">Privacy</Link>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
