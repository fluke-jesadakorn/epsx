'use client';

import { Card } from '@epsx/ui';

export default function PrivacyPage() {
  return (
    <div className="min-h-screen bg-[#08060B] text-white">
      <div className="max-w-4xl mx-auto p-6">
        <div className="space-y-6">
          <div className="text-center mb-12">
            <h1 className="text-4xl font-bold mb-4 bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
              Privacy Policy
            </h1>
            <p className="text-gray-400">
              Last updated: {new Date().toLocaleDateString()}
            </p>
          </div>

          <Card className="p-8 bg-[#27262c] border-[#383241] rounded-[24px] shadow-xl">
            <div className="prose prose-invert prose-purple max-w-none">
              <h3 className="text-2xl font-bold text-purple-400 mb-4">
                1. Information We Collect
              </h3>
              <p className="text-gray-300">
                When you use our services, we collect certain information about
                you:
              </p>
              <ul className="list-disc pl-6 text-gray-300 space-y-2">
                <li>
                  Basic profile information from Google Sign-in (name and email)
import { Card } from '@epsx/ui';
                </li>
                <li>Account preferences and settings</li>
                <li>Usage data and analytics</li>
              </ul>

              <h3 className="text-2xl font-bold text-purple-400 mt-8 mb-4">
                2. How We Use Your Information
              </h3>
              <p className="text-gray-300">
                We use the collected information for:
              </p>
              <ul className="list-disc pl-6 text-gray-300 space-y-2">
                <li>Account creation and management</li>
                <li>Providing personalized services</li>
                <li>Communication about service updates</li>
                <li>Security and fraud prevention</li>
              </ul>

              <h3 className="text-2xl font-bold text-purple-400 mt-8 mb-4">
                3. Third-Party Services
              </h3>
              <p className="text-gray-300">
                We use OpenID Connect authentication for secure sign-in. When you authenticate:
              </p>
              <ul className="list-disc pl-6 text-gray-300 space-y-2">
                <li>
                  We only request necessary permissions (email and basic
                  profile)
                </li>
                <li>Your credentials are handled securely by our authentication system</li>
                <li>
                  We receive only basic profile information needed for account
                  creation
                </li>
              </ul>

              <h3 className="text-2xl font-bold text-purple-400 mt-8 mb-4">
                4. Data Security
              </h3>
              <p className="text-gray-300">
                We take security seriously and implement industry-standard
                measures to protect your data:
              </p>
              <ul className="list-disc pl-6 text-gray-300 space-y-2">
                <li>Secure authentication using OAuth 2.0</li>
                <li>Encrypted data storage and transfer</li>
                <li>Regular security audits and updates</li>
                <li>Secure session management</li>
              </ul>

              <h3 className="text-2xl font-bold text-purple-400 mt-8 mb-4">
                5. Your Rights
              </h3>
              <p className="text-gray-300">You have the right to:</p>
              <ul className="list-disc pl-6 text-gray-300 space-y-2">
                <li>Access your personal data</li>
                <li>Request data correction or deletion</li>
                <li>
                  Revoke access to third-party services like Google Sign-in
                </li>
                <li>Opt-out of communications</li>
              </ul>

              <h3 className="text-2xl font-bold text-purple-400 mt-8 mb-4">
                6. Changes to Privacy Policy
              </h3>
              <p className="text-gray-300">
                We may update this privacy policy from time to time. We will
                notify you of any changes by posting the new policy on this page
                and updating the &ldquo;Last updated&rdquo; date.
              </p>

              <h3 className="text-2xl font-bold text-purple-400 mt-8 mb-4">
                7. Contact Us
              </h3>
              <p className="text-gray-300">
                If you have questions about this Privacy Policy, please contact
                us at: &ldquo;Your Contact Information&rdquo;
              </p>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
}
