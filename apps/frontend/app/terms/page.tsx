"use client";

import { zodResolver } from "@hookform/resolvers/zod";
import { Suspense } from "react";
import { useForm } from "react-hook-form";
import { z } from "zod";

import { SkeletonLoader } from "@/components/common/skeleton";
import { Button, Card, Form, FormControl, FormField, FormItem, FormMessage, InputWithIcon } from "@/components/ui";
import { Send } from "lucide-react";

const formSchema = z.object({
  email: z.string().email("Please enter a valid email address"),
});

type SubscribeFormData = z.infer<typeof formSchema>;

function SubscribeForm() {
  const form = useForm<SubscribeFormData>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      email: "",
    },
  });

  const handleSubmit = async (values: SubscribeFormData) => {
    try {
      const response = await fetch('/api/public/subscribe', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email: values.email }),
      });

      if (response.ok) {
        form.reset();
        alert('Successfully subscribed to updates!');
      } else {
        alert('Failed to subscribe. Please try again.');
      }
    } catch (_error) {
      alert('An error occurred. Please try again.');
    }
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="email"
          render={({ field }) => (
            <FormItem>
              <FormControl asChild>
                <InputWithIcon
                  placeholder="Enter your email"
                  icon={
                    <svg
                      className="h-4 w-4 text-purple-400"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    >
                      <rect width="20" height="16" x="2" y="4" rx="2" />
                      <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" />
                    </svg>
                  }
                  className="bg-[#27262c] border-[#383241] focus:border-purple-500 focus-visible:ring-purple-500/20"
                  {...field}
                />
              </FormControl>
              <FormMessage className="text-red-400" />
            </FormItem>
          )}
        />

        <Button
          type="submit"
          className="bg-gradient-to-r from-purple-500 to-pink-500 hover:from-purple-600 hover:to-pink-600 text-white font-bold flex items-center gap-2"
        >
          <Send className="h-4 w-4" />
          Subscribe
        </Button>
      </form>
    </Form>
  );
}

export default function TermsPage() {
  return (
    <div className="min-h-screen bg-[#08060B] text-white">
      <div className="max-w-4xl mx-auto p-6">
        <div className="space-y-6">
          <div className="text-center mb-12">
            <h1 className="text-4xl font-bold mb-4 bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
              Terms and Conditions
            </h1>
            <p className="text-gray-400">Last updated: {new Date().toLocaleDateString()}</p>
          </div>

          <Card className="p-8 bg-[#27262c] border-[#383241] rounded-[24px] shadow-xl">
            <div className="prose prose-invert prose-purple max-w-none">
              <h3 className="text-2xl font-bold text-purple-400 mb-4">1. Introduction</h3>
              <p className="text-gray-300">
                Welcome to our platform. By accessing or using our services, you agree
                to be bound by these terms and conditions, including our use of Google Sign-in
                for authentication.
              </p>

              <h3 className="text-2xl font-bold text-purple-400 mt-8 mb-4">2. Authentication & Account Security</h3>
              <p className="text-gray-300">
                We use OpenID Connect authentication to provide secure authentication. By using this service:
              </p>
              <ul className="list-disc pl-6 text-gray-300 space-y-2">
                <li>You agree to provide accurate information during the sign-in process</li>
                <li>You acknowledge that we only request necessary permissions (email and basic profile)</li>
                <li>You understand that token revocation may occur for security purposes</li>
                <li>You are responsible for maintaining the security of your account</li>
              </ul>

              <h3 className="text-2xl font-bold text-purple-400 mt-8 mb-4">3. Data Collection & Usage</h3>
              <p className="text-gray-300">
                We collect and process certain data as outlined in our Privacy Policy, including:
              </p>
              <ul className="list-disc pl-6 text-gray-300 space-y-2">
                <li>Basic profile information from Google (name and email)</li>
                <li>Account preferences and settings</li>
                <li>Authentication tokens and session data</li>
              </ul>

              <h3 className="text-2xl font-bold text-purple-400 mt-8 mb-4">4. User Responsibilities</h3>
              <p className="text-gray-300">
                As a user of our platform, you are responsible for:
              </p>
              <ul className="list-disc pl-6 text-gray-300 space-y-2">
                <li>Maintaining the confidentiality of your account</li>
                <li>All activities that occur under your account</li>
                <li>Notifying us of any unauthorized access</li>
                <li>Keeping your Google account secure</li>
              </ul>

              <h3 className="text-2xl font-bold text-purple-400 mt-8 mb-4">5. Service Changes & Termination</h3>
              <p className="text-gray-300">
                We reserve the right to:
              </p>
              <ul className="list-disc pl-6 text-gray-300 space-y-2">
                <li>Modify or discontinue services at any time</li>
                <li>Revoke access tokens for security purposes</li>
                <li>Update authentication methods and requirements</li>
                <li>Terminate accounts that violate these terms</li>
              </ul>

              <h3 className="text-2xl font-bold text-purple-400 mt-8 mb-4">6. Authentication Standards</h3>
              <p className="text-gray-300">
                Our authentication system follows OpenID Connect standards and OAuth 2.0 specifications.
                We implement industry-standard security protocols to protect your account and data.
              </p>
            </div>
          </Card>

          <Card className="p-8 bg-[#27262c] border-[#383241] rounded-[24px] shadow-xl mt-8">
            <h2 className="text-2xl font-bold text-purple-400 mb-6">Subscribe for Updates</h2>
            <Suspense fallback={<SkeletonLoader />}>
              <SubscribeForm />
            </Suspense>
          </Card>
        </div>
      </div>
    </div>
  );
}
