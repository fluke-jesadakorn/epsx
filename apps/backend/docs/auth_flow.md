# Authentication Flow with Firebase

```mermaid
sequenceDiagram
    participant Client
    participant API as API Server
    participant Firebase

    %% Protected Route Flow with Auth Middleware
    Client->>API: Request with Bearer Token
    activate API
    
    API->>Firebase: Verify Token <br/> (using Firebase Admin SDK)
    
    Firebase-->>API: Token Claims <br/> (uid, email, roles)
    
    Note over API: Middleware Checks:
    Note over API: 1. Extract Bearer Token
    Note over API: 2. Initialize Firebase Admin
    Note over API: 3. Verify Token
    Note over API: 4. Check User Roles
    Note over API: 5. Add User to Context
    
    alt Valid Token & Has Required Role
        API-->>Client: Access Granted to Protected Resource
    else Invalid Token or Missing Role
        API-->>Client: 401 Unauthorized / 403 Forbidden
    end
    deactivate API
