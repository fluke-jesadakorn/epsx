'use client';

import React, { useState } from 'react';
import { Button } from '@/components/ui/button';
import { FormField, Input, Select, Textarea } from '@/components/ui/form-components';
import { toast } from 'react-hot-toast';
import { Send, CheckCircle, Building, Mail, Globe, Code } from 'lucide-react';

interface ApiKeyRequest {
  company_name: string;
  contact_name: string;
  email: string;
  phone?: string;
  website?: string;
  company_size: string;
  use_case: string;
  expected_volume: string;
  modules_interested: string[];
  integration_timeline: string;
  additional_info?: string;
}

const COMPANY_SIZES = [
  { value: 'startup', label: 'Startup (1-10 employees)' },
  { value: 'small', label: 'Small Business (11-50 employees)' },
  { value: 'medium', label: 'Medium Business (51-200 employees)' },
  { value: 'large', label: 'Large Enterprise (200+ employees)' },
];

const EXPECTED_VOLUMES = [
  { value: 'low', label: 'Low (< 1,000 requests/month)' },
  { value: 'medium', label: 'Medium (1,000 - 10,000 requests/month)' },
  { value: 'high', label: 'High (10,000 - 100,000 requests/month)' },
  { value: 'enterprise', label: 'Enterprise (100,000+ requests/month)' },
];

const INTEGRATION_TIMELINES = [
  { value: 'immediate', label: 'Immediate (within 1 week)' },
  { value: 'short', label: 'Short-term (1-4 weeks)' },
  { value: 'medium', label: 'Medium-term (1-3 months)' },
  { value: 'long', label: 'Long-term (3+ months)' },
];

const AVAILABLE_MODULES = [
  { id: 'stock-ranking', name: 'Stock Ranking', description: 'Advanced stock ranking and analysis' },
  { id: 'market-data', name: 'Market Data', description: 'Real-time and historical market data' },
  { id: 'portfolio-analysis', name: 'Portfolio Analysis', description: 'Portfolio management and risk analysis' },
  { id: 'trading-signals', name: 'Trading Signals', description: 'AI-powered trading signals' },
];

export const ApiKeyRequestForm: React.FC = () => {
  const [formData, setFormData] = useState<ApiKeyRequest>({
    company_name: '',
    contact_name: '',
    email: '',
    phone: '',
    website: '',
    company_size: '',
    use_case: '',
    expected_volume: '',
    modules_interested: [],
    integration_timeline: '',
    additional_info: '',
  });
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isSubmitted, setIsSubmitted] = useState(false);

  const handleInputChange = (field: keyof ApiKeyRequest, value: string | string[]) => {
    setFormData(prev => ({ ...prev, [field]: value }));
  };

  const handleModuleToggle = (moduleId: string) => {
    setFormData(prev => ({
      ...prev,
      modules_interested: prev.modules_interested.includes(moduleId)
        ? prev.modules_interested.filter(id => id !== moduleId)
        : [...prev.modules_interested, moduleId]
    }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    // Basic validation
    if (!formData.company_name || !formData.contact_name || !formData.email || 
        !formData.use_case || formData.modules_interested.length === 0) {
      toast.error('Please fill in all required fields');
      return;
    }

    setIsSubmitting(true);
    
    try {
      // Simulate API call - in real implementation, this would send to your backend
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      
      setIsSubmitted(true);
      toast.success('Your API key request has been submitted successfully!');
    } catch (error) {
      console.error('Failed to submit request:', error);
      toast.error('Failed to submit request. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  if (isSubmitted) {
    return (
      <div className="max-w-2xl mx-auto p-6">
        <div className="text-center">
          <CheckCircle className="w-16 h-16 text-green-500 mx-auto mb-4" />
          <h2 className="text-2xl font-bold text-gray-900 mb-2">Request Submitted!</h2>
          <p className="text-gray-600 mb-6">
            Thank you for your interest in the EPSX API. Our team will review your request and 
            get back to you within 2-3 business days.
          </p>
          
          <div className="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-6">
            <h3 className="font-semibold text-blue-900 mb-2">What happens next?</h3>
            <ul className="text-sm text-blue-800 text-left space-y-1">
              <li>• Our team will review your application</li>
              <li>• We&apos;ll contact you to discuss your requirements</li>
              <li>• Upon approval, you&apos;ll receive developer portal access</li>
              <li>• You can then create and manage your API keys</li>
            </ul>
          </div>
          
          <div className="space-y-3">
            <Button onClick={() => window.location.href = '/docs/api'} variant="outline">
              View API Documentation
            </Button>
            <Button onClick={() => {
              setIsSubmitted(false);
              setFormData({
                company_name: '',
                contact_name: '',
                email: '',
                phone: '',
                website: '',
                company_size: '',
                use_case: '',
                expected_volume: '',
                modules_interested: [],
                integration_timeline: '',
                additional_info: '',
              });
            }}>
              Submit Another Request
            </Button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">Request API Access</h1>
        <p className="text-lg text-gray-600">
          Get access to the EPSX API platform and start integrating financial data into your applications.
        </p>
      </div>

      <form onSubmit={handleSubmit} className="space-y-8">
        {/* Company Information */}
        <div className="bg-white border rounded-lg p-6">
          <h2 className="text-xl font-semibold text-gray-900 mb-4 flex items-center">
            <Building className="w-5 h-5 mr-2 text-blue-600" />
            Company Information
          </h2>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <FormField label="Company Name" required>
              <Input
                value={formData.company_name}
                onChange={(e) => handleInputChange('company_name', e.target.value)}
                placeholder="Your Company Inc."
              />
            </FormField>
            
            <FormField label="Company Size">
              <Select
                value={formData.company_size}
                onChange={(e) => handleInputChange('company_size', e.target.value)}
              >
                <option value="">Select company size</option>
                {COMPANY_SIZES.map(size => (
                  <option key={size.value} value={size.value}>{size.label}</option>
                ))}
              </Select>
            </FormField>
            
            <FormField label="Website">
              <Input
                type="url"
                value={formData.website}
                onChange={(e) => handleInputChange('website', e.target.value)}
                placeholder="https://yourcompany.com"
              />
            </FormField>
          </div>
        </div>

        {/* Contact Information */}
        <div className="bg-white border rounded-lg p-6">
          <h2 className="text-xl font-semibold text-gray-900 mb-4 flex items-center">
            <Mail className="w-5 h-5 mr-2 text-blue-600" />
            Contact Information
          </h2>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <FormField label="Contact Name" required>
              <Input
                value={formData.contact_name}
                onChange={(e) => handleInputChange('contact_name', e.target.value)}
                placeholder="John Doe"
              />
            </FormField>
            
            <FormField label="Email Address" required>
              <Input
                type="email"
                value={formData.email}
                onChange={(e) => handleInputChange('email', e.target.value)}
                placeholder="john@yourcompany.com"
              />
            </FormField>
            
            <FormField label="Phone Number">
              <Input
                type="tel"
                value={formData.phone}
                onChange={(e) => handleInputChange('phone', e.target.value)}
                placeholder="+1 (555) 123-4567"
              />
            </FormField>
          </div>
        </div>

        {/* Project Information */}
        <div className="bg-white border rounded-lg p-6">
          <h2 className="text-xl font-semibold text-gray-900 mb-4 flex items-center">
            <Code className="w-5 h-5 mr-2 text-blue-600" />
            Project Information
          </h2>
          
          <div className="space-y-4">
            <FormField label="Use Case Description" required>
              <Textarea
                value={formData.use_case}
                onChange={(e) => handleInputChange('use_case', e.target.value)}
                placeholder="Describe how you plan to use the EPSX API in your application..."
                rows={4}
              />
            </FormField>
            
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <FormField label="Expected API Volume">
                <Select
                  value={formData.expected_volume}
                  onChange={(e) => handleInputChange('expected_volume', e.target.value)}
                >
                  <option value="">Select expected volume</option>
                  {EXPECTED_VOLUMES.map(volume => (
                    <option key={volume.value} value={volume.value}>{volume.label}</option>
                  ))}
                </Select>
              </FormField>
              
              <FormField label="Integration Timeline">
                <Select
                  value={formData.integration_timeline}
                  onChange={(e) => handleInputChange('integration_timeline', e.target.value)}
                >
                  <option value="">Select timeline</option>
                  {INTEGRATION_TIMELINES.map(timeline => (
                    <option key={timeline.value} value={timeline.value}>{timeline.label}</option>
                  ))}
                </Select>
              </FormField>
            </div>
          </div>
        </div>

        {/* Module Selection */}
        <div className="bg-white border rounded-lg p-6">
          <h2 className="text-xl font-semibold text-gray-900 mb-4 flex items-center">
            <Globe className="w-5 h-5 mr-2 text-blue-600" />
            Modules of Interest
          </h2>
          <p className="text-gray-600 mb-4">Select the modules you&apos;re interested in accessing (select at least one):</p>
          
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {AVAILABLE_MODULES.map(module => (
              <div key={module.id} className="border rounded-lg p-4">
                <label className="flex items-start space-x-3 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.modules_interested.includes(module.id)}
                    onChange={() => handleModuleToggle(module.id)}
                    className="mt-1 h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                  />
                  <div>
                    <h3 className="font-medium text-gray-900">{module.name}</h3>
                    <p className="text-sm text-gray-600">{module.description}</p>
                  </div>
                </label>
              </div>
            ))}
          </div>
        </div>

        {/* Additional Information */}
        <div className="bg-white border rounded-lg p-6">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">Additional Information</h2>
          
          <FormField label="Additional Details">
            <Textarea
              value={formData.additional_info}
              onChange={(e) => handleInputChange('additional_info', e.target.value)}
              placeholder="Any additional information about your project, special requirements, or questions..."
              rows={4}
            />
          </FormField>
        </div>

        {/* Submit Button */}
        <div className="flex justify-center">
          <Button
            type="submit"
            disabled={isSubmitting}
            className="px-8 py-3 text-lg"
          >
            {isSubmitting ? (
              <>
                <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-white mr-2"></div>
                Submitting...
              </>
            ) : (
              <>
                <Send className="w-5 h-5 mr-2" />
                Submit Request
              </>
            )}
          </Button>
        </div>
      </form>

      {/* Help Section */}
      <div className="mt-12 bg-blue-50 border border-blue-200 rounded-lg p-6">
        <h3 className="font-semibold text-blue-900 mb-2">Need Help?</h3>
        <p className="text-blue-800 mb-3">
          Have questions about our API or need assistance with your request?
        </p>
        <div className="text-sm text-blue-700 space-y-1">
          <div>📧 Email: api-support@epsx.com</div>
          <div>📞 Phone: +1 (555) 123-4567</div>
          <div>💬 Live Chat: Available Monday-Friday, 9 AM - 5 PM EST</div>
        </div>
      </div>
    </div>
  );
};
