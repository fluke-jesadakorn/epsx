'use client';

import { memo } from 'react';
import { Clock, AlertTriangle } from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { format } from 'date-fns';

interface BulkOperationResult {
  operation: string;
  timestamp: string;
  totalRequested: number;
  successful: number;
  failed: number;
  executionTimeMs: number;
  errors: Array<{ id?: string; error: string; details?: string }>;
}

interface BulkHistoryTabProps {
  operationResults: BulkOperationResult[];
}

function BulkHistoryTab({ operationResults }: BulkHistoryTabProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Clock className="h-5 w-5" />
          Operation History
        </CardTitle>
        <CardDescription>
          Recent bulk operations and their results
        </CardDescription>
      </CardHeader>
      <CardContent>
        {operationResults.length === 0 ? (
          <div className="text-center py-8 text-muted-foreground">
            <Clock className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>No bulk operations performed yet</p>
          </div>
        ) : (
          <div className="space-y-4">
            {operationResults.map((result, index) => (
              <Card key={index} className="border">
                <CardContent className="p-4">
                  <div className="flex items-start justify-between mb-2">
                    <div>
                      <h4 className="font-medium">{result.operation}</h4>
                      <p className="text-sm text-muted-foreground">
                        {format(new Date(result.timestamp), 'PPp')}
                      </p>
                    </div>
                    <div className={`px-3 py-2 rounded-xl font-light uppercase tracking-wide text-xs min-h-[32px] flex items-center justify-center ${
                        result.failed > 0 
                          ? 'bg-gradient-to-r from-red-400 to-orange-400 text-white shadow-lg' 
                          : 'bg-gradient-to-r from-green-400 to-yellow-400 text-black shadow-lg'
                      }`}>
                      {result.successful}/{result.totalRequested} successful
                    </div>
                  </div>
                  
                  <div className="grid grid-cols-4 gap-4 text-sm">
                    <div>
                      <p className="text-muted-foreground">Total</p>
                      <p className="font-medium">{result.totalRequested}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">Successful</p>
                      <p className="font-medium text-green-600">{result.successful}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">Failed</p>
                      <p className="font-medium text-red-600">{result.failed}</p>
                    </div>
                    <div>
                      <p className="text-muted-foreground">Duration</p>
                      <p className="font-medium">{result.executionTimeMs}ms</p>
                    </div>
                  </div>

                  {result.errors.length > 0 && (
                    <div className="mt-4">
                      <details className="text-sm">
                        <summary className="cursor-pointer text-red-600 hover:text-red-700">
                          View {result.errors.length} errors
                        </summary>
                        <div className="mt-2 space-y-2">
                          {result.errors.slice(0, 5).map((error, errorIndex) => (
                            <div key={errorIndex} className="p-2 bg-red-50 rounded text-red-700">
                              <p className="font-medium">{error.error}</p>
                              {error.details && (
                                <p className="text-xs mt-1">{error.details}</p>
                              )}
                            </div>
                          ))}
                          {result.errors.length > 5 && (
                            <p className="text-xs text-muted-foreground">
                              ... and {result.errors.length - 5} more errors
                            </p>
                          )}
                        </div>
                      </details>
                    </div>
                  )}
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

export default memo(BulkHistoryTab);