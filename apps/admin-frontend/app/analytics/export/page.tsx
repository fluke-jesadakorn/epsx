import { Suspense } from 'react'
import { redirect } from 'next/navigation'
import { FileText, Download, ArrowLeft } from 'lucide-react'
import Link from 'next/link'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { getServerSession } from '@/lib/server/auth'
import { exportAnalyticsReport } from '@/lib/actions/analytics-actions'

interface ExportPageProps {
  searchParams: {
    dateRange?: string
    selectedModule?: string
  }
}

async function ExportForm({ searchParams }: ExportPageProps) {
  const { dateRange = '7d', selectedModule = 'all' } = searchParams
  
  const handleExport = async (formData: FormData) => {
    'use server'
    
    const format = formData.get('format') as 'csv' | 'pdf' | 'xlsx'
    const dateRange = formData.get('dateRange') as string
    const selectedModule = formData.get('selectedModule') as string
    
    try {
      const result = await exportAnalyticsReport(dateRange, selectedModule, format)
      
      if (result.success && result.data?.downloadUrl) {
        redirect(result.data.downloadUrl)
      } else {
        // Handle error - could use cookies or query params to show error
        redirect('/analytics/export?error=export-failed')
      }
    } catch (error) {
      redirect('/analytics/export?error=unexpected-error')
    }
  }

  const getDateRangeLabel = (range: string) => {
    switch (range) {
      case '7d': return 'last 7 days'
      case '30d': return 'last 30 days'
      default: return range
    }
  }

  const getModuleLabel = (module: string) => {
    return module === 'all' ? 'all modules' : module
  }

  return (
    <div className="max-w-md mx-auto">
      <Card className="p-6">
        <div className="flex items-center gap-2 mb-6">
          <FileText className="w-6 h-6 text-blue-600" />
          <h1 className="text-xl font-semibold">Export Analytics Report</h1>
        </div>
        
        <div className="mb-6">
          <p className="text-gray-600 text-sm">
            Export analytics data for {getModuleLabel(selectedModule)} ({getDateRangeLabel(dateRange)})
          </p>
        </div>

        <form action={handleExport} className="space-y-3">
          <input type="hidden" name="dateRange" value={dateRange} />
          <input type="hidden" name="selectedModule" value={selectedModule} />
          
          <Button
            type="submit"
            name="format"
            value="pdf"
            variant="outline"
            className="w-full justify-start"
          >
            <FileText className="w-4 h-4 mr-2" />
            PDF Report
          </Button>
          
          <Button
            type="submit"
            name="format"
            value="xlsx"
            variant="outline"
            className="w-full justify-start"
          >
            <FileText className="w-4 h-4 mr-2" />
            Excel Spreadsheet (.xlsx)
          </Button>
          
          <Button
            type="submit"
            name="format"
            value="csv"
            variant="outline"
            className="w-full justify-start"
          >
            <FileText className="w-4 h-4 mr-2" />
            CSV Data (.csv)
          </Button>
        </form>

        <div className="pt-4 border-t mt-6">
          <Link href="/analytics">
            <Button variant="outline" className="w-full">
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back to Analytics
            </Button>
          </Link>
        </div>
      </Card>
    </div>
  )
}

export default async function ExportPage(props: ExportPageProps) {
  // Verify admin session
  const session = await getServerSession()
  if (!session?.user?.permissions?.some(p => p.startsWith('admin:'))) {
    redirect('/unauthorized')
  }

  return (
    <div className="min-h-screen bg-gray-50 py-12">
      <div className="container max-w-2xl mx-auto px-4">
        <Suspense fallback={<div>Loading export options...</div>}>
          <ExportForm searchParams={props.searchParams} />
        </Suspense>
      </div>
    </div>
  )
}