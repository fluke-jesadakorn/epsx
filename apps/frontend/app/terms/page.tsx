"use client";

import { Suspense } from "react";
import { z } from "zod";
import { zodResolver } from "@hookform/resolvers/zod";
import { useForm } from "react-hook-form";
import { Mail } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";

const formSchema = z.object({
  email: z.string().email("Please enter a valid email address"),
});

type SubscribeForm = z.infer<typeof formSchema>;

function SubscribeForm() {
  const form = useForm<SubscribeForm>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      email: "",
    },
  });

  const handleSubmit = (values: SubscribeForm) => {
    console.log("Email to store:", values.email);
    // TODO: Implement actual email storage
  };

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="email"
          render={({ field }) => (
            <FormItem>
              <FormControl>
                <div className="relative">
                  <Mail className="absolute left-3 top-3 h-4 w-4 text-gray-400" />
                  <Input 
                    placeholder="Enter your email" 
                    className="pl-10" 
                    {...field} 
                  />
                </div>
              </FormControl>
              <FormMessage />
            </FormItem>
          )}
        />

        <Button type="submit">
          Subscribe
        </Button>
      </form>
    </Form>
  );
}

export default function TermsPage() {
  return (
    <div className="max-w-3xl mx-auto p-6">
      <h1 className="text-3xl font-bold mb-6">Terms and Conditions</h1>
      
      <div className="prose prose-gray max-w-none">
        <p>Last updated: {new Date().toLocaleDateString()}</p>
        
        <h3>1. Introduction</h3>
        <p>
          Welcome to our platform. By accessing or using our services, you agree
          to be bound by these terms and conditions, including our use of Google Sign-in
          for authentication.
        </p>

        <h3>2. Authentication & Account Security</h3>
        <p>
          We use Google Sign-in to provide secure authentication. By using this service:
        </p>
        <ul>
          <li>You agree to provide accurate information during the sign-in process</li>
          <li>You acknowledge that we only request necessary permissions (email and basic profile)</li>
          <li>You understand that token revocation may occur for security purposes</li>
          <li>You are responsible for maintaining the security of your Google account</li>
        </ul>

        <h3>3. Data Collection & Usage</h3>
        <p>
          We collect and process certain data as outlined in our Privacy Policy, including:
        </p>
        <ul>
          <li>Basic profile information from Google (name and email)</li>
          <li>Account preferences and settings</li>
          <li>Authentication tokens and session data</li>
        </ul>

        <h3>4. User Responsibilities</h3>
        <p>
          As a user of our platform, you are responsible for:
        </p>
        <ul>
          <li>Maintaining the confidentiality of your account</li>
          <li>All activities that occur under your account</li>
          <li>Notifying us of any unauthorized access</li>
          <li>Keeping your Google account secure</li>
        </ul>

        <h3>5. Service Changes & Termination</h3>
        <p>
          We reserve the right to:
        </p>
        <ul>
          <li>Modify or discontinue services at any time</li>
          <li>Revoke access tokens for security purposes</li>
          <li>Update authentication methods and requirements</li>
          <li>Terminate accounts that violate these terms</li>
        </ul>

        <h3>6. Compliance with Google's Terms</h3>
        <p>
          Our use of Google Sign-in complies with Google's OAuth 2.0 policies and terms of service.
          You acknowledge that your use of Google Sign-in is also subject to Google's terms and policies.
        </p>
      </div>

      <div className="mt-8">
        <h2 className="text-xl font-semibold mb-4">Subscribe for Updates</h2>
        <Suspense fallback={<div>Loading subscription form...</div>}>
          <SubscribeForm />
        </Suspense>
      </div>
    </div>
  );
}
