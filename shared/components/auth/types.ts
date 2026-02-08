export interface AuthResult {
    wallet_address: string;
    permissions: string[];
    is_new_user: boolean;
    access_token: string;
    error?: string;
    success?: boolean;
}

export type AuthStep = 'connect' | 'switch-chain' | 'sign' | 'authenticating' | 'success' | 'error';
