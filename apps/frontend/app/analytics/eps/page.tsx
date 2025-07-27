'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { AlertTriangle, ArrowLeft } from 'lucide-react';
import Link from 'next/link';

export default function EPSAnalysisPage() {
  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto py-8 px-4">
        <div className="mb-8">
          <h1 className="text-4xl font-bold mb-2">EPS Growth Analysis</h1>
          <p className="text-muted-foreground text-lg">
            Advanced algorithmic analysis of earnings per share trends and patterns
          </p>
        </div>

        <Card className="max-w-2xl mx-auto">
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-orange-100 dark:bg-orange-900/20">
              <AlertTriangle className="h-8 w-8 text-orange-600 dark:text-orange-400" />
            </div>
            <CardTitle className="text-2xl">Feature Under Development</CardTitle>
            <CardDescription className="text-base">
              EPS Growth Analysis algorithms are currently being developed
            </CardDescription>
          </CardHeader>
          <CardContent className="text-center space-y-4">
            <p className="text-muted-foreground">
              Our team is working on implementing sophisticated EPS growth analysis algorithms. 
              This feature will provide advanced earnings per share trend analysis, pattern recognition, 
              and algorithmic insights when completed.
            </p>
            <div className="bg-muted/50 p-4 rounded-lg text-sm">
              <strong>Coming Soon:</strong>
              <ul className="mt-2 text-left list-disc list-inside space-y-1 text-muted-foreground">
                <li>Real-time EPS data integration</li>
                <li>Advanced pattern recognition algorithms</li>
                <li>Historical trend analysis</li>
                <li>Comparative company analysis</li>
                <li>Educational context and explanations</li>
              </ul>
            </div>
            <div className="pt-4">
              <Button asChild variant="outline">
                <Link href="/analytics" className="inline-flex items-center gap-2">
                  <ArrowLeft className="h-4 w-4" />
                  Back to Analytics Dashboard
                </Link>
              </Button>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}