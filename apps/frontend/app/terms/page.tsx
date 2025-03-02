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
          to be bound by these terms and conditions.
        </p>

        <h3>2. Data Collection</h3>
        <p>
          We may collect and process your email address for communication
          purposes. By providing your email, you consent to this collection.
        </p>

        <h3>3. User Responsibilities</h3>
        <p>
          You are responsible for maintaining the confidentiality of your
          account information and for all activities that occur under your
          account.
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
