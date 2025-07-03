import { NextResponse } from 'next/server';
import { db } from '../../../../../lib/firebase';
import { doc, setDoc, getDoc } from 'firebase/firestore';
import crypto from 'crypto';

// Mock environment variable before importing the route
process.env.MUSEPAY_PUBLIC_KEY = 'mock_public_key';

// Import the route after setting up mocks
import { POST } from '../route';

// Mock dependencies
jest.mock('firebase/firestore', () => ({
  doc: jest.fn(),
  setDoc: jest.fn(() => Promise.resolve()),
  getDoc: jest.fn(() => Promise.resolve({
    data: () => ({ userId: 'testUser' })
  })),
}));
jest.mock('crypto', () => ({
  createVerify: jest.fn(() => ({
    update: jest.fn(),
    verify: jest.fn(() => true), // Default to valid signature
  })),
}));
jest.mock('../../../../../lib/firebase', () => ({
  db: jest.fn(),
}));

describe('MusePay Webhook Endpoint', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    // Reset the crypto verify mock to return true by default for valid signature tests
    (crypto.createVerify as jest.Mock).mockReturnValue({
      update: jest.fn(),
      verify: jest.fn(() => true),
    });
  });

  it('should return 200 for valid request with valid signature', async () => {
    const request = new Request('http://localhost/api/webhook/musepay', {
      method: 'POST',
      body: JSON.stringify({
        order_no: 'TEST_ORDER_123',
        status: 'success',
        actual_amount: '100.00',
        currency: 'USD',
        finish_time: '2025-02-07T17:00:00Z',
        request_id: 'TEST_REQUEST_456',
        order_type: 'payment',
        product_code: 'PREMIUM_SUB',
        reason: '',
        extra_info: '{}',
        partner_id: 'TEST_PARTNER',
        order_amount: '100.00',
        fee_amount: '0.00',
        sign: 'valid_signature',
      }),
    });

    const response = await POST(request);
    expect(response.status).toBe(200);
    expect(setDoc).toHaveBeenCalledWith(
      expect.anything(), // transactionRef
      expect.objectContaining({
        orderNo: 'TEST_ORDER_123',
        status: 'completed',
        actualAmount: 100,
      }),
      { merge: true }
    );
  });

  it('should return 403 for request with missing signature', async () => {
    const request = new Request('http://localhost/api/webhook/musepay', {
      method: 'POST',
      body: JSON.stringify({
        order_no: 'TEST_ORDER_123',
        status: 'success',
        actual_amount: '100.00',
      }),
    });

    const response = await POST(request);
    expect(response.status).toBe(403);
    const responseBody = await response.json();
    expect(responseBody).toEqual({ error: 'Invalid request: No signature provided' });
    expect(setDoc).not.toHaveBeenCalled();
  });

  it('should return 403 for request with invalid signature', async () => {
    // Override the mock to return false for invalid signature
    (crypto.createVerify as jest.Mock).mockReturnValue({
      update: jest.fn(),
      verify: jest.fn(() => false),
    });

    const request = new Request('http://localhost/api/webhook/musepay', {
      method: 'POST',
      body: JSON.stringify({
        order_no: 'TEST_ORDER_123',
        status: 'success',
        actual_amount: '100.00',
        currency: 'USD',
        finish_time: '2025-02-07T17:00:00Z',
        request_id: 'TEST_REQUEST_456',
        order_type: 'payment',
        product_code: 'PREMIUM_SUB',
        sign: 'invalid_signature',
      }),
    });

    const response = await POST(request);
    expect(response.status).toBe(403);
    const responseBody = await response.json();
    expect(responseBody).toEqual({ error: 'Invalid signature' });
    expect(setDoc).not.toHaveBeenCalled();
  });

  it('should update user payment status when transaction status is completed', async () => {
    const request = new Request('http://localhost/api/webhook/musepay', {
      method: 'POST',
      body: JSON.stringify({
        order_no: 'TEST_ORDER_123',
        status: 'success',
        actual_amount: '100.00',
        currency: 'USD',
        finish_time: '2025-02-07T17:00:00Z',
        request_id: 'TEST_REQUEST_456',
        order_type: 'payment',
        product_code: 'PREMIUM_SUB',
        sign: 'valid_signature',
      }),
    });

    const response = await POST(request);
    expect(response.status).toBe(200);
    expect(setDoc).toHaveBeenCalledTimes(2); // Once for transaction, once for user
    expect(setDoc).toHaveBeenCalledWith(
      expect.anything(), // userRef
      expect.objectContaining({
        paymentStatus: expect.objectContaining({
          hasPaid: true,
        }),
        userLevel: 'Premium',
      }),
      { merge: true }
    );
  });

  it('should return 500 for internal server error', async () => {
    // Simulate a database error
    (setDoc as jest.Mock).mockRejectedValue(new Error('Database error'));

    const request = new Request('http://localhost/api/webhook/musepay', {
      method: 'POST',
      body: JSON.stringify({
        order_no: 'TEST_ORDER_123',
        status: 'success',
        actual_amount: '100.00',
        currency: 'USD',
        finish_time: '2025-02-07T17:00:00Z',
        request_id: 'TEST_REQUEST_456',
        order_type: 'payment',
        product_code: 'PREMIUM_SUB',
        sign: 'valid_signature',
      }),
    });

    const response = await POST(request);
    expect(response.status).toBe(500);
    const responseBody = await response.json();
    expect(responseBody).toEqual({ error: 'Internal server error' });
  });
});
