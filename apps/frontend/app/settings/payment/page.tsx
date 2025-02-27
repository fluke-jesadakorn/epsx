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
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Loader2 } from "lucide-react";

const stripePromise = loadStripe(
  process.env.NEXT_PUBLIC_STRIPE_PUBLISHABLE_KEY!
);

const PaymentForm = () => {
  const stripe = useStripe();
  const elements = useElements();
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const router = useRouter();

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!stripe || !elements) return;

    setLoading(true);
    setError(null);

    try {
      const { error: stripeError } = await stripe.createPaymentMethod({
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
      console.error(err);
      setError("An unexpected error occurred");
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4 max-w-md mx-auto">
      <div className="space-y-2">
        <label className="text-sm font-medium">
          Card Details <span className="text-red-500">*</span>
        </label>
        <div className="p-3 border rounded-md">
          <CardElement options={{ style: { base: { fontSize: "16px" } } }} />
        </div>
      </div>
      
      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      
      <Button
        type="submit"
        className="w-full"
        disabled={!stripe || loading}
      >
        {loading ? (
          <div className="flex items-center">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            <span>Processing...</span>
          </div>
        ) : (
          "Pay"
        )}
      </Button>
    </form>
  );
};

const PaymentSettings = () => {
  return (
    <div className="p-6 max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold mb-2">Payment Settings</h1>
      <p className="text-muted-foreground mb-6">
        Securely manage your payments and subscriptions here.
      </p>

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
