'use client'

/**
 * Global Error Boundary
 * 
 * This special Next.js component handles uncaught errors at the root level.
 * It must:
 * - Be a Client Component ('use client')
 * - Define its own <html> and <body> tags
 * - Be self-contained with no external dependencies
 * @param root0
 * @param root0.error
 * @param root0.reset
 */
export default function GlobalError({
    error,
    reset,
}: {
    error: Error & { digest?: string }
    reset: () => void
}) {
    return (
        <html lang="en">
            <body style={{
                margin: 0,
                padding: 0,
                fontFamily: 'system-ui, -apple-system, sans-serif',
                backgroundColor: '#0f0f0f',
                color: '#ffffff',
                minHeight: '100vh',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center'
            }}>
                <div style={{
                    textAlign: 'center',
                    padding: '40px'
                }}>
                    <h2 style={{ fontSize: '24px', marginBottom: '16px' }}>
                        Something went wrong!
                    </h2>
                    <p style={{ color: '#9ca3af', marginBottom: '24px' }}>
                        An unexpected error occurred.
                    </p>
                    <button
                        onClick={() => reset()}
                        style={{
                            padding: '12px 24px',
                            backgroundColor: '#f97316',
                            color: 'white',
                            border: 'none',
                            borderRadius: '8px',
                            cursor: 'pointer',
                            fontSize: '14px',
                            fontWeight: 500
                        }}
                    >
                        Try again
                    </button>
                </div>
            </body>
        </html>
    )
}
