import { ArrowLeft, Plus, Shield } from 'lucide-react'
import Link from 'next/link'
import { redirect } from 'next/navigation'
import { Suspense } from 'react'

import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Textarea } from '@/components/ui/textarea'
import { getServerSession } from '@/lib/server/auth'
import { createPlansClient } from '@/shared/api/plans'
import { createAdminApiClient } from '@/shared/utils/api-client'

// Force dynamic rendering since we use cookies for auth
export const dynamic = 'force-dynamic'

interface CreateApiKeyFormData {
  client_name: string
  client_description?: string
  client_contact_email?: string
  ip_restrictions: string[]
  expires_at?: string
}

interface FormFieldProps {
  id: string
  label: string
  required?: boolean
  children: React.ReactNode
}

function FormField({ id, label, required, children }: FormFieldProps) {
  return (
    <div className="space-y-2">
      <Label htmlFor={id} className="text-sm font-medium">
        {label}
        {required && <span className="text-red-500 ml-1">*</span>}
      </Label>
      {children}
    </div>
  )
}

async function CreateApiKeyForm() {
  const handleCreateApiKey = async (formData: FormData) => {
    'use server'

    const clientName = formData.get('client_name') as string
    const clientDescription = formData.get('client_description') as string
    const clientContactEmail = formData.get('client_contact_email') as string
    const ipRestrictionsRaw = formData.get('ip_restrictions') as string
    const expiresAt = formData.get('expires_at') as string

    // Basic validation
    if (!clientName.trim()) {
      redirect('/developer-portal/api-keys/create?error=client-name-required')
    }

    const ipRestrictions = ipRestrictionsRaw
      .split('\n')
      .map(ip => ip.trim())
      .filter(ip => ip.length > 0)

    const createRequest = {
      client_name: clientName.trim(),
      client_description: clientDescription?.trim(),
      client_contact_email: clientContactEmail?.trim(),
      allowed_modules: [], // Will need to be configured separately
      ip_restrictions: ipRestrictions,
      expires_at: expiresAt || undefined
    }

    try {
      // Call the actual API to create the API key
      const apiClient = createAdminApiClient()
      const plansClient = createPlansClient(apiClient)
      const response = await plansClient.createApiKey({
        client_name: createRequest.client_name,
        client_description: createRequest.client_description,
        client_contact_email: createRequest.client_contact_email,
        allowed_modules: createRequest.allowed_modules,
        ip_restrictions: createRequest.ip_restrictions || [],
        expires_at: createRequest.expires_at,
      })

      if (response.success) {
        const searchParams = new URLSearchParams({
          success: 'true',
          client_name: clientName,
          new_key: response.data?.full_key || 'key-created'
        })
        redirect(`/developer-portal?${searchParams.toString()}`)
      } else {
        redirect('/developer-portal/api-keys/create?error=api-creation-failed')
      }
    } catch (_error) {
       
      console.error('Failed to create API key:', _error)
      redirect('/developer-portal/api-keys/create?error=creation-failed')
    }
  }

  return (
    <div className="max-w-4xl mx-auto">
      <Card className="p-8">
        <div className="flex items-center gap-3 mb-8">
          <div className="p-2 bg-blue-100 rounded-lg">
            <Shield className="w-6 h-6 text-blue-600" />
          </div>
          <div>
            <h1 className="text-2xl font-bold">Create API Key</h1>
            <p className="text-gray-600">Generate a new API key for third-party integration</p>
          </div>
        </div>

        <form action={handleCreateApiKey} className="space-y-6">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <FormField id="client_name" label="Client Name" required>
              <Input
                id="client_name"
                name="client_name"
                placeholder="My Application"
                required
              />
            </FormField>

            <FormField id="client_contact_email" label="Contact Email">
              <Input
                id="client_contact_email"
                name="client_contact_email"
                type="email"
                placeholder="contact@example.com"
              />
            </FormField>
          </div>

          <FormField id="client_description" label="Description">
            <Textarea
              id="client_description"
              name="client_description"
              placeholder="Brief description of your application and use case"
              rows={3}
            />
          </FormField>

          <FormField id="expires_at" label="Expiration Date (Optional)">
            <Input
              id="expires_at"
              name="expires_at"
              type="datetime-local"
            />
          </FormField>

          <FormField id="ip_restrictions" label="IP Restrictions (Optional)">
            <Textarea
              id="ip_restrictions"
              name="ip_restrictions"
              placeholder="192.168.1.0/24&#10;203.0.113.0/24&#10;One IP address or CIDR block per line"
              rows={4}
            />
          </FormField>

          <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-4">
            <div className="flex items-start gap-3">
              <div className="w-5 h-5 text-yellow-600 mt-0.5">⚠️</div>
              <div>
                <h3 className="font-medium text-yellow-800 mb-1">Module Configuration Required</h3>
                <p className="text-sm text-yellow-700">
                  After creating the API key, you'll need to configure module permissions and access levels
                  in the main developer portal.
                </p>
              </div>
            </div>
          </div>

          <div className="flex gap-3 pt-6 border-t">
            <Link href="/developer-portal">
              <Button type="button" variant="outline" className="flex items-center gap-2">
                <ArrowLeft className="w-4 h-4" />
                Cancel
              </Button>
            </Link>
            <Button type="submit" className="flex items-center gap-2">
              <Plus className="w-4 h-4" />
              Create API Key
            </Button>
          </div>
        </form>
      </Card>
    </div>
  )
}

/**
 *
 */
export default async function CreateApiKeyPage() {
  // Verify admin session
  const session = await getServerSession()
  if (!session?.user?.permissions?.some(p => p.startsWith('admin:'))) {
    redirect('/unauthorized')
  }

  return (
    <div className="min-h-screen bg-gray-50 py-12">
      <div className="container mx-auto px-4">
        <Suspense fallback={<div>Loading form...</div>}>
          <CreateApiKeyForm />
        </Suspense>
      </div>
    </div>
  )
}