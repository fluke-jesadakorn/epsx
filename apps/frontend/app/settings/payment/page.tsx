"use client";

import { useState } from "react";
import { loadStripe } from "@stripe/stripe-js";
import {
  Elements,
  CardElement,
  useStripe,
  useElements,
} from "@stripe/react-stripe-js";
import { useRouter } from "next/navigation";
// TODO: Implement payment record storage
import { Button, Form, Typography, Alert, Skeleton } from "antd";
// TODO: Add skeleton loading for payment form
// TODO: Implement progress indicators for payment processing
// TODO: Add loading states for payment history

const stripePromise = loadStripe(
  process.env.NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY!
);

const { Title, Text } = Typography;

const PaymentForm = () => {
  const stripe = useStripe();
  const elements = useElements();
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const router = useRouter();

  const handleSubmit = async () => {
    if (!stripe || !elements) return;

    setLoading(true);
    setError(null);

    try {
      const { error: stripeError, paymentMethod } =
        await stripe.createPaymentMethod({
          type: "card",
          card: elements.getElement(CardElement)!,
        });

      if (stripeError) {
        setError(stripeError.message || "Payment failed");
        return;
      }

      // TODO: Save payment record to database
      // Example implementation:
      // await savePaymentRecord({
      //   paymentMethodId: paymentMethod.id,
      //   amount: 1000, // Example amount in cents ($10.00)
      //   currency: "thb",
      //   status: "succeeded"
      // });

      router.refresh();
      alert("Payment successful!");
    } catch (err) {
      console.error(err)
      setError("An unexpected error occurred");
    } finally {
      setLoading(false);
    }
  };

  return (
    <Form
      onFinish={handleSubmit}
      className="payment-form"
      layout="vertical"
      style={{ maxWidth: 400, margin: "auto" }}
    >
      <Form.Item label="Card Details" required>
        <div
          style={{
            padding: "8px",
            border: "1px solid #d9d9d9",
            borderRadius: "4px",
          }}
        >
          <CardElement options={{ style: { base: { fontSize: "16px" } } }} />
        </div>
      </Form.Item>
      {error && <Alert type="error" message={error} showIcon />}
      <Form.Item>
        <Button
          type="primary"
          htmlType="submit"
          loading={loading}
          disabled={!stripe}
          block
        >
          {loading ? (
            <Skeleton.Avatar 
              active 
              size="small" 
              shape="circle" 
              style={{ marginRight: 8 }}
            />
          ) : "Pay"}
        </Button>
      </Form.Item>
    </Form>
  );
};

const PaymentSettings = () => {
  return (
    <div style={{ padding: "24px", maxWidth: "600px", margin: "auto" }}>
      <Title level={2}>Payment Settings</Title>
      <Text type="secondary">
        Securely manage your payments and subscriptions here.
      </Text>

      <Elements stripe={stripePromise}>
        <PaymentForm />
      </Elements>

      {/* Future Features */}
      {/* TODO: Add payment history section */}
      {/* TODO: Implement subscription management */}
      {/* TODO: Add currency selection */}
    </div>
  );
};

export default PaymentSettings;
