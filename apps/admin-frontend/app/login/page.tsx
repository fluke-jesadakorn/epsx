/**
 * Simplified Admin Login Page
 */
export default function AdminLoginPage() {
  return (
    <div style={{ padding: '40px', textAlign: 'center', minHeight: '100vh', backgroundColor: '#f5f5f5' }}>
      <div style={{ backgroundColor: 'white', padding: '40px', borderRadius: '8px', maxWidth: '400px', margin: '0 auto', boxShadow: '0 2px 8px rgba(0,0,0,0.1)' }}>
        <h1 style={{ marginBottom: '20px', color: '#333' }}>Admin Login</h1>
        <p style={{ marginBottom: '20px', color: '#666' }}>
          Administrative access for EPSX platform
        </p>
        
        <a 
          href="/api/auth/signin/epsx-backend"
          style={{
            display: 'inline-block',
            padding: '12px 24px',
            backgroundColor: '#0070f3',
            color: 'white',
            textDecoration: 'none',
            borderRadius: '6px',
            fontWeight: '500'
          }}
        >
          Sign in with EPSX Backend
        </a>
        
        <div style={{ marginTop: '20px', fontSize: '12px', color: '#999' }}>
          🔒 Administrative Access Only
        </div>
      </div>
    </div>
  );
}