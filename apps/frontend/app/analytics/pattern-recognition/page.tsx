'use client';

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { AlertTriangle, ArrowLeft } from 'lucide-react';
import Link from 'next/link';

export default function PatternRecognitionPage() {
  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto py-8 px-4">
        <div className="mb-8">
          <h1 className="text-4xl font-bold mb-2">Pattern Recognition</h1>
          <p className="text-muted-foreground text-lg">
            AI-powered market pattern detection and analysis
          </p>
        </div>

        <Card className="max-w-2xl mx-auto">
          <CardHeader className="text-center">
            <div className="mx-auto mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-orange-100 dark:bg-orange-900/20">
              <AlertTriangle className="h-8 w-8 text-orange-600 dark:text-orange-400" />
            </div>
            <CardTitle className="text-2xl">Feature Under Development</CardTitle>
            <CardDescription className="text-base">
              Pattern recognition algorithms are currently being developed
            </CardDescription>
          </CardHeader>
          <CardContent className="text-center space-y-4">
            <p className="text-muted-foreground">
              Our team is working on implementing advanced AI-powered pattern recognition algorithms. 
              This feature will provide automated detection of market patterns, technical analysis patterns, 
              and algorithmic trading signals when completed.
            </p>
            <div className="bg-muted/50 p-4 rounded-lg text-sm">
              <strong>Coming Soon:</strong>
              <ul className="mt-2 text-left list-disc list-inside space-y-1 text-muted-foreground">
                <li>Technical pattern detection (Head & Shoulders, Triangles, etc.)</li>
                <li>Machine learning-based pattern recognition</li>
                <li>Real-time pattern alerts and notifications</li>
                <li>Pattern success rate analytics</li>
                <li>Educational pattern explanations</li>
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