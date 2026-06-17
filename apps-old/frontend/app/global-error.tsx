'use client'

export default function GlobalError({
    error: _error,
    reset,
}: {
    error: Error & { digest?: string }
    reset: () => void
}) {
    return (
        <html>
            <body>
                <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100vh', gap: '20px', fontFamily: 'system-ui, sans-serif' }}>
                    <h2>Something went wrong!</h2>
                    <button
                        onClick={() => reset()}
                        style={{ padding: '10px 20px', backgroundColor: '#0070f3', color: 'white', border: 'none', borderRadius: '5px', cursor: 'pointer' }}
                    >
                        Try again
                    </button>
                </div>
            </body>
        </html>
    )
}
