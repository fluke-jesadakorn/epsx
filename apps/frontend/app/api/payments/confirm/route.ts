 
import { createPaymentsClient, type PaymentValidateRequest } from '@/shared/api/payments';
import { COOKIES } from '@/shared/auth/cookies';
import { cookies } from 'next/headers';
import type { NextRequest} from 'next/server';
import { NextResponse } from 'next/server';

// Payment confirmation request body (client format)
interface PaymentConfirmRequest {
    plan_id: string;
    transaction_hash: string;
    amount: number;
    currency: string;
    network?: string;
}

// Note: Blockchain validation is now handled by the backend API
// Frontend only handles authentication and API communication

// eslint-disable-next-line complexity
export async function POST(req: NextRequest) {
    try {
        // 1. Authenticate user using Web3 cookies
        const cookieStore = await cookies();
        const userCookie = cookieStore.get(COOKIES.user);
        const accessCookie = cookieStore.get(COOKIES.access_token);

        if ((userCookie?.value === null || userCookie?.value === undefined) || (accessCookie?.value === null || accessCookie?.value === undefined)) {
            return NextResponse.json(
                { success: false, message: 'Authentication required' },
                { status: 401 }
            );
        }

        let user;
        try {
            user = JSON.parse(userCookie.value);
        } catch (_error) {
            return NextResponse.json(
                { success: false, message: 'Invalid user session' },
                { status: 401 }
            );
        }

        if (user?.wallet_address === null || user?.wallet_address === undefined || typeof user?.wallet_address !== 'string') {
            return NextResponse.json(
                { success: false, message: 'Invalid user session: missing wallet' },
                { status: 401 }
            );
        }

        const body: PaymentConfirmRequest = await req.json();
        const { plan_id, transaction_hash, amount, currency, network = process.env.NEXT_PUBLIC_DEFAULT_NETWORK ?? 'bsc-mainnet' } = body;

        // Validate required fields
        if (!plan_id || !transaction_hash || !amount) {
            return NextResponse.json(
                { success: false, message: 'Missing required fields: plan_id, transaction_hash, amount' },
                { status: 400 }
            );
        }

        // 2. Call backend API for payment validation using shared client
        const paymentsApi = createPaymentsClient({
            serverSide: true,
            token: accessCookie?.value,
        });

        const validateRequest: PaymentValidateRequest = {
            plan_id,
            transaction_hash,
            amount: Math.round(amount * 100), // Convert to cents (integer)
            currency,
            network,
            wallet_address: user.wallet_address,
        };

        const response = await paymentsApi.validatePayment(validateRequest);

        if (!response.success || !response.data) {
            return NextResponse.json(
                {
                    success: false,
                    message: response.message ?? response.error ?? 'Backend validation failed',
                    details: response.data,
                    status: response.status
                },
                { status: response.status ?? 400 }
            );
        }

        const validationResult = response.data;

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
