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
    error: _error,
    reset,
}: {
    error: Error & { digest?: string }
    reset: () => void
}) {
    return (
        <html lang="en">
            <head>
                <style dangerouslySetInnerHTML={{
                    __html: `
                        body {
                            margin: 0;
                            padding: 0;
                            font-family: system-ui, -apple-system, sans-serif;
                            background-color: hsl(var(--background));
                            color: hsl(var(--foreground));
                            min-height: 100vh;
                            display: flex;
                            align-items: center;
                            justify-content: center;
                        }
                        :root {
                            --background: 252 100% 99%;
                            --foreground: 223 84% 10%;
                        }
                        @media (prefers-color-scheme: dark) {
                            :root {
                                --background: 240 10% 4%;
                                --foreground: 0 0% 95%;
                            }
                        }
                    `
                }} />
            </head>
            <body>
                <div style={{
                    textAlign: 'center',
                    padding: '40px'
                }}>
                    <h2 style={{ fontSize: '24px', marginBottom: '16px', color: 'hsl(var(--foreground))' }}>
                        Something went wrong!
                    </h2>
                    <p style={{ color: 'hsl(var(--foreground) / 0.7)', marginBottom: '24px' }}>
                        An unexpected error occurred.
                    </p>
                    <button
                        onClick={() => reset()}
                        style={{
                            padding: '12px 24px',
                            backgroundColor: 'hsl(217 91% 60%)',
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
