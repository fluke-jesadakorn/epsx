import { COOKIES } from '@/shared/auth/cookies';
import { cookies } from 'next/headers';
import { NextRequest, NextResponse } from 'next/server';

// Payment confirmation request body
interface PaymentConfirmRequest {
    plan_id: string;
    transaction_hash: string;
    amount: number;
    currency: string;
    network?: string;
}

// Note: Blockchain validation is now handled by the backend API
// Frontend only handles authentication and API communication

export async function POST(req: NextRequest) {
    try {
        // 1. Authenticate user using Web3 cookies
        const cookieStore = await cookies();
        const userCookie = cookieStore.get(COOKIES.user);
        const accessCookie = cookieStore.get(COOKIES.access);

        if (!userCookie?.value) {
            return NextResponse.json(
                { success: false, message: 'Authentication required: no user cookie found' },
                { status: 401 }
            );
        }

        let user;
        try {
            user = JSON.parse(userCookie.value);
        } catch (error) {
            return NextResponse.json(
                { success: false, message: 'Invalid authentication data: malformed user cookie' },
                { status: 401 }
            );
        }

        if (!user?.wallet_address) {
            return NextResponse.json(
                { success: false, message: 'Invalid user authentication: missing wallet address' },
                { status: 401 }
            );
        }

        const body: PaymentConfirmRequest = await req.json();
        const { plan_id, transaction_hash, amount, currency, network = 'localhost' } = body;

        // Validate required fields
        if (!plan_id || !transaction_hash || !amount) {
            return NextResponse.json(
                { success: false, message: 'Missing required fields: plan_id, transaction_hash, amount' },
                { status: 400 }
            );
        }

        // 2. Call backend API for payment validation

        const requestBody = {
            plan_id,
            transaction_hash,
            amount: Math.round(amount * 100), // Convert to cents (integer)
            currency,
            network,
            wallet_address: user.wallet_address,
        };

        const backendResponse = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL}/api/payments/validate`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                // Use wallet address as primary auth since access token is missing
                'X-Wallet-Address': user.wallet_address || '',
                // Try access token if available (fallback)
                ...(accessCookie?.value ? { 'Authorization': `Bearer ${accessCookie.value}` } : {}),
            },
            body: JSON.stringify(requestBody),
        });

        if (!backendResponse.ok) {
            const errorText = await backendResponse.text();
            console.error('[Payment Confirmation] Backend validation failed:', {
                status: backendResponse.status,
                statusText: backendResponse.statusText,
                errorText: errorText,
            });

            let errorData;
            try {
                errorData = JSON.parse(errorText);
            } catch {
                errorData = { message: errorText || 'Backend validation failed' };
            }

            return NextResponse.json(
                {
                    success: false,
                    message: errorData.message || 'Backend validation failed',
                    details: errorData.details || errorText,
                    status: backendResponse.status
                },
                { status: backendResponse.status }
            );
        }

        const validationResult = await backendResponse.json();

        if (!validationResult.success) {
            return NextResponse.json(validationResult, { status: 400 });
        }

        // 3. Return success response with validation data
        // Note: Validation handler creates the payment record, so no separate activation is needed
        return NextResponse.json({
            success: true,
            message: 'Payment confirmed and subscription activated successfully',
            data: {
                payment: validationResult.data,
                subscription: validationResult.data, // Using same data for now as validation returns payment info
            },
        });

    } catch (error) {
        console.error('[Payment Confirmation] Error:', error);

        return NextResponse.json(
            {
                success: false,
                message: 'Payment confirmation failed',
                error: error instanceof Error ? error.message : 'Unknown error',
            },
            { status: 500 }
        );
    }
}
