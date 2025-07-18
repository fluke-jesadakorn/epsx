import * as React from 'react';
import type { Metadata } from 'next';
import { PaginatedStockGrid } from '@/components/shared/PaginatedStockGrid';
import { fetchPaginatedStockData } from '@/app/actions/stockRankingPaginated';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Server, Globe, Zap, Shield } from 'lucide-react';

export const metadata: Metadata = {
  title: 'Pagination Demo - EPSX',
  description: 'Demonstration of server-side and API-based pagination implementations',
};

export default async function PaginationDemoPage() {
  // Fetch initial data for server-side pagination
  const initialData = await fetchPaginatedStockData(1, 10);

  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold tracking-tight">Pagination Demo</h1>
          <p className="text-muted-foreground mt-2">
            Compare server-side and API-based pagination implementations
          </p>
        </div>

        {/* Feature Overview */}
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4 mb-8">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Server-Side</CardTitle>
              <Server className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">SSR</div>
              <p className="text-xs text-muted-foreground">
                Direct service calls
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">API-Based</CardTitle>
              <Globe className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">REST</div>
              <p className="text-xs text-muted-foreground">
                HTTP API endpoints
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Performance</CardTitle>
              <Zap className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">5m</div>
              <p className="text-xs text-muted-foreground">
                Cache TTL
              </p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">Security</CardTitle>
              <Shield className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">RBAC</div>
              <p className="text-xs text-muted-foreground">
                Role-based access
              </p>
            </CardContent>
          </Card>
        </div>
        
        <Tabs defaultValue="server" className="w-full">
          <TabsList className="grid w-full grid-cols-2">
            <TabsTrigger value="server" className="flex items-center gap-2">
              <Server className="h-4 w-4" />
              Server-Side Pagination
            </TabsTrigger>
            <TabsTrigger value="api" className="flex items-center gap-2">
              <Globe className="h-4 w-4" />
              API-Based Pagination
            </TabsTrigger>
          </TabsList>
          
          <TabsContent value="server" className="space-y-6">
            <div className="flex items-center gap-2 mb-4">
              <Badge variant="outline">Server Actions</Badge>
              <Badge variant="secondary">Direct Service Calls</Badge>
              <Badge variant="outline">SSR Optimized</Badge>
            </div>
            <PaginatedStockGrid 
              initialData={initialData} 
              useApi={false}
              className="border rounded-lg p-4" 
            />
          </TabsContent>
          
          <TabsContent value="api" className="space-y-6">
            <div className="flex items-center gap-2 mb-4">
              <Badge variant="outline">REST API</Badge>
              <Badge variant="secondary">HTTP Endpoints</Badge>
              <Badge variant="outline">Client-Side</Badge>
            </div>
            <PaginatedStockGrid 
              initialData={initialData} 
              useApi={true}
              className="border rounded-lg p-4" 
            />
          </TabsContent>
        </Tabs>

        {/* API Endpoints Documentation */}
        <div className="mt-12">
          <h2 className="text-2xl font-bold mb-4">Available API Endpoints</h2>
          <div className="grid gap-4 md:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Paginated Data</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <code className="text-sm bg-muted p-2 rounded block">
                  GET /api/stock/paginated
                </code>
                <p className="text-sm text-muted-foreground">
                  Returns paginated stock data with metadata
                </p>
                <div className="text-xs space-y-1">
                  <div><strong>Params:</strong> page, limit, country, quarters</div>
                  <div><strong>Response:</strong> {`{data: [], pagination: {}}`}</div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Stock Count</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <code className="text-sm bg-muted p-2 rounded block">
                  GET /api/stock/count
                </code>
                <p className="text-sm text-muted-foreground">
                  Returns total count of available stocks
                </p>
                <div className="text-xs space-y-1">
                  <div><strong>Params:</strong> country, quarters</div>
                  <div><strong>Response:</strong> {`{count: number}`}</div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Legacy Stock API</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <code className="text-sm bg-muted p-2 rounded block">
                  GET /api/stock
                </code>
                <p className="text-sm text-muted-foreground">
                  Backward compatible with pagination support
                </p>
                <div className="text-xs space-y-1">
                  <div><strong>Params:</strong> skip, limit, page, paginated</div>
                  <div><strong>Response:</strong> Array or paginated object</div>
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="text-lg">Performance Features</CardTitle>
              </CardHeader>
              <CardContent className="space-y-2">
                <div className="flex items-center gap-2">
                  <Badge variant="outline">5min Cache</Badge>
                  <Badge variant="outline">Parallel Queries</Badge>
                </div>
                <p className="text-sm text-muted-foreground">
                  Optimized for performance with caching and parallel data fetching
                </p>
                <div className="text-xs space-y-1">
                  <div><strong>Cache-Control:</strong> public, max-age=300</div>
                  <div><strong>Queries:</strong> Data + Count in parallel</div>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  );
}

// Revalidate page every 5 minutes
export const revalidate = 300;
