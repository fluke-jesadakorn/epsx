import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { API_ROUTES } from '../../../shared/config/route-constants';
import { getBackendUrl } from '../../../shared/utils/url-resolver';
import { PureWeb3AuthStore, SignedRequestHeaders } from './types';

export interface PureWeb3StoreConfig {
    persistenceName: string;
    requireAdmin?: boolean;
    verifyMethod?: 'POST' | 'GET'; // Defaults to POST (Frontend style). Admin uses GET.
}

const getBaseUrl = () => getBackendUrl('client');

export const createPureWeb3AuthStore = (config: PureWeb3StoreConfig) => {
    return create<PureWeb3AuthStore>()(
        persist(
            (set, get) => ({
                // Initial state
                isConnected: false,
                isAuthenticating: false,
                isLoading: true,
                hasInitialized: false,
                chainId: 1,
                permissions: [],
                groups: [],
                bearerToken: undefined,
                tokenExpiresAt: undefined,
                currentNonce: undefined,
                nonceExpiry: undefined,
                error: undefined,

                // State setters
                setConnected: (connected, address, chainId) => set({
                    isConnected: connected,
                    walletAddress: address,
                    chainId: chainId || 1
                }),
                setAuthenticating: (authenticating) => set({ isAuthenticating: authenticating }),
                setLoading: (loading) => set({ isLoading: loading }),
                setInitialized: (initialized) => set({ hasInitialized: initialized }),
                setPermissions: (permissions) => set({ permissions }),
                setGroups: (groups) => set({ groups }),
                setBearerToken: (token, expiresAt) => set({ bearerToken: token, tokenExpiresAt: expiresAt }),
                clearBearerToken: () => set({ bearerToken: undefined, tokenExpiresAt: undefined }),
                setNonce: (nonce, expiry) => set({ currentNonce: nonce, nonceExpiry: expiry }),
                clearNonce: () => set({ currentNonce: undefined, nonceExpiry: undefined }),
                setError: (error) => set({ error }),

                // Generate authentication challenge from backend
                generateChallenge: async (endpoint = API_ROUTES.AUTH.WEB3_VERIFY) => {
                    const state = get();

                    if (!state.walletAddress) {
                        throw new Error('Wallet not connected');
                    }

                    try {
                        const response = await fetch(`${getBaseUrl()}${API_ROUTES.AUTH.WEB3_CHALLENGE}`, {
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json',
                            },
                            body: JSON.stringify({
                                wallet_address: state.walletAddress,
                                chain_id: state.chainId,
                                endpoint
                            }),
                        });

                        if (!response.ok) {
                            throw new Error(`Failed to get challenge: ${response.status}`);
                        }

                        const challenge = await response.json();

                        // Store nonce with expiry
                        const expiryTime = new Date(challenge.expires_at).getTime();
                        set({
                            currentNonce: challenge.nonce,
                            nonceExpiry: expiryTime
                        });

                        return {
                            nonce: challenge.nonce,
                            message: challenge.message,
                            chainId: challenge.chain_id
                        };
                    } catch (error) {
                        const errorMsg = error instanceof Error ? error.message : 'Challenge generation failed';
                        set({ error: errorMsg });
                        throw new Error(errorMsg);
                    }
                },

                // Verify wallet connection by testing signature
                verifyConnection: async () => {
                    const state = get();

                    if (!state.walletAddress || !window.__pureWeb3_signMessage) {
                        return false;
                    }

                    try {
                        set({ isAuthenticating: true, error: undefined });

                        // Generate challenge
                        const challenge = await get().generateChallenge(API_ROUTES.AUTH.WEB3_VERIFY);
                        const message = challenge.message;

                        if (!message) {
                            throw new Error('Invalid challenge received from backend');
                        }

                        // Sign the message
                        const messageToSign = typeof message === 'string' ? message : JSON.stringify(message);

                        if (!messageToSign.trim()) {
                            throw new Error('Empty message cannot be signed');
                        }

                        const signature = await window.__pureWeb3_signMessage(messageToSign);

                        // Performa verification
                        let response: Response;

                        if (config.verifyMethod === 'GET') {
                            // Admin style: Send headers
                            const headers: any = {
                                'X-Wallet-Address': state.walletAddress,
                                'X-Chain-Id': state.chainId.toString(),
                                'X-Web3-Signature': signature,
                                'X-Signed-Message': message,
                                'X-Timestamp': Math.floor(Date.now() / 1000).toString(),
                                'X-Nonce': challenge.nonce
                            };

                            response = await fetch(`${getBaseUrl()}${API_ROUTES.AUTH.WEB3_VERIFY}`, {
                                method: 'GET',
                                headers
                            });

                        } else {
                            // Frontend style: Send body
                            response = await fetch(`${getBaseUrl()}${API_ROUTES.AUTH.WEB3_VERIFY}`, {
                                method: 'POST',
                                headers: {
                                    'Content-Type': 'application/json',
                                },
                                body: JSON.stringify({
                                    wallet_address: state.walletAddress,
                                    message: message,
                                    signature,
                                    nonce: challenge.nonce,
                                }),
                            });
                        }

                        if (response.ok) {
                            const verifyData = await response.json();

                            // Admin check if required
                            if (config.requireAdmin) {
                                const adminPermissions = (verifyData.permissions || []).filter((p: string) => p.startsWith('admin:'));
                                if (adminPermissions.length === 0) {
                                    set({
                                        isAuthenticating: false,
                                        error: 'No admin permissions found for this wallet'
                                    });
                                    return false;
                                }
                            }

                            // Store Bearer token if provided
                            if (verifyData.bearer_token && verifyData.token_expires_at) {
                                set({
                                    bearerToken: verifyData.bearer_token,
                                    tokenExpiresAt: verifyData.token_expires_at
                                });
                            }

                            set({
                                isAuthenticating: false,
                                permissions: verifyData.permissions || [],
                                groups: verifyData.groups || []
                            });
                            return true;
                        } else {
                            set({ isAuthenticating: false, error: 'Signature verification failed' });
                            return false;
                        }
                    } catch (error) {
                        const errorMsg = error instanceof Error ? error.message : 'Connection verification failed';
                        set({ isAuthenticating: false, error: errorMsg });
                        return false;
                    }
                },

                // Refresh permissions from backend
                refreshPermissions: async () => {
                    const state = get();

                    if (!state.walletAddress) {
                        throw new Error('Wallet not connected');
                    }

                    try {
                        // Sign request for permissions endpoint
                        const endpoint = API_ROUTES.AUTH.PERMISSIONS; // '/api/v1/auth/users/permissions'
                        // Or '/api/users/permissions' as seen in original file?
                        // "Naming Convention: RESTful /api/v1/{resource}/{action}"
                        // Original: '/api/users/permissions' (Frontend), '/user/permissions' (Admin)
                        // route-constants: PERMISSIONS: '/api/v1/auth/users/permissions'
                        // This suggests I should use the constant but might need to verify URL.
                        // Using `API_ROUTES.AUTH.PERMISSIONS` which is '/api/v1/auth/users/permissions'.

                        const signedHeaders = await get().signRequest(endpoint, 'GET');

                        const response = await fetch(`${getBaseUrl()}${endpoint}`, {
                            method: 'GET',
                            headers: signedHeaders as any,
                        });

                        if (response.ok) {
                            const data = await response.json();
                            set({
                                permissions: data.unique_permissions || [],
                                groups: data.group_permissions?.map((g: any) => ({
                                    group_id: g.group_name,
                                    name: g.group_name,
                                    permissions: [g.permission]
                                })) || []
                            });
                        }
                    } catch (error) {
                        console.warn('Failed to refresh permissions:', error);
                    }
                },

                // Sign out
                signOut: async () => {
                    const state = get();

                    try {
                        if (state.walletAddress && state.currentNonce) {
                            const endpoint = API_ROUTES.AUTH.WEB3_LOGOUT;
                            const signedHeaders = await get().signRequest(endpoint, 'DELETE');

                            await fetch(`${getBaseUrl()}${endpoint}`, {
                                method: 'DELETE',
                                headers: signedHeaders as any,
                                body: JSON.stringify({ clear_all_sessions: true }),
                            });
                        }
                    } catch (error) {
                        console.warn('Logout request failed:', error);
                    } finally {
                        get().resetState();
                    }
                },

                // Sign API request
                signRequest: async (endpoint: string, method: string, body?: any): Promise<SignedRequestHeaders> => {
                    const state = get();

                    if (!state.walletAddress) {
                        throw new Error('Wallet not connected');
                    }

                    if (!window.__pureWeb3_signMessage) {
                        throw new Error('Wallet signing not available');
                    }

                    try {
                        const now = Date.now();
                        let nonce = state.currentNonce;
                        let needNewNonce = !nonce || !state.nonceExpiry || state.nonceExpiry <= now;

                        if (needNewNonce) {
                            const challenge = await get().generateChallenge(endpoint);
                            nonce = challenge.nonce;
                        }

                        if (!nonce) {
                            throw new Error('Failed to get nonce for request signing');
                        }

                        const timestamp = Math.floor(Date.now() / 1000);
                        const bodyHash = body ? JSON.stringify(body) : '';

                        const message = [
                            config.requireAdmin ? `EPSX Admin API Request` : `EPSX API Request`,
                            `Wallet: ${state.walletAddress}`,
                            `Method: ${method}`,
                            `Endpoint: ${endpoint}`,
                            `Chain ID: ${state.chainId}`,
                            `Timestamp: ${timestamp}`,
                            `Nonce: ${nonce}`,
                            `Body: ${bodyHash}`
                        ].join('\n');

                        const messageToSign = typeof message === 'string' ? message : JSON.stringify(message);
                        const signature = await window.__pureWeb3_signMessage(messageToSign);

                        return {
                            'X-Wallet-Address': state.walletAddress,
                            'X-Chain-Id': state.chainId.toString(),
                            'X-Web3-Signature': signature,
                            'X-Signed-Message': message,
                            'X-Timestamp': timestamp.toString(),
                            'X-Nonce': nonce
                        };
                    } catch (error) {
                        const errorMsg = error instanceof Error ? error.message : 'Request signing failed';
                        set({ error: errorMsg });
                        throw new Error(errorMsg);
                    }
                },

                // Reset state
                resetState: () => set({
                    isConnected: false,
                    isAuthenticating: false,
                    isLoading: false,
                    hasInitialized: false,
                    walletAddress: undefined,
                    chainId: 1,
                    permissions: [],
                    groups: [],
                    bearerToken: undefined,
                    tokenExpiresAt: undefined,
                    currentNonce: undefined,
                    nonceExpiry: undefined,
                    error: undefined,
                }),
            }),
            {
                name: config.persistenceName,
                partialize: (state) => ({
                    chainId: state.chainId,
                }),
            }
        )
    );
};
