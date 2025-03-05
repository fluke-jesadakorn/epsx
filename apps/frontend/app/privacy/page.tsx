"use client";

export default function PrivacyPage() {
  return (
    <div className="max-w-3xl mx-auto p-6">
      <h1 className="text-3xl font-bold mb-6">Privacy Policy</h1>
      
      <div className="prose prose-gray max-w-none">
        <p>Last updated: {new Date().toLocaleDateString()}</p>
        
        <h3>1. Information We Collect</h3>
        <p>
          When you use our services, we collect certain information about you:
        </p>
        <ul>
          <li>Basic profile information from Google Sign-in (name and email)</li>
          <li>Account preferences and settings</li>
          <li>Usage data and analytics</li>
        </ul>

        <h3>2. How We Use Your Information</h3>
        <p>We use the collected information for:</p>
        <ul>
          <li>Account creation and management</li>
          <li>Providing personalized services</li>
          <li>Communication about service updates</li>
          <li>Security and fraud prevention</li>
        </ul>

        <h3>3. Third-Party Services</h3>
        <p>
          We use Google Sign-in for authentication. When you choose to sign in with Google:
        </p>
        <ul>
          <li>We only request necessary permissions (email and basic profile)</li>
          <li>Your Google credentials are handled securely by Google</li>
          <li>We receive only basic profile information needed for account creation</li>
        </ul>

        <h3>4. Data Security</h3>
        <p>
          We take security seriously and implement industry-standard measures to protect your data:
        </p>
        <ul>
          <li>Secure authentication using OAuth 2.0</li>
          <li>Encrypted data storage and transfer</li>
          <li>Regular security audits and updates</li>
          <li>Secure session management</li>
        </ul>

        <h3>5. Your Rights</h3>
        <p>You have the right to:</p>
        <ul>
          <li>Access your personal data</li>
          <li>Request data correction or deletion</li>
          <li>Revoke access to third-party services like Google Sign-in</li>
          <li>Opt-out of communications</li>
        </ul>

        <h3>6. Changes to Privacy Policy</h3>
        <p>
          We may update this privacy policy from time to time. We will notify you of any changes by posting the new policy on this page and updating the "Last updated" date.
        </p>

        <h3>7. Contact Us</h3>
        <p>
          If you have questions about this Privacy Policy, please contact us at:
          [Your Contact Information]
        </p>
      </div>
    </div>
  );
}
