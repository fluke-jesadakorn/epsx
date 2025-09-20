import { Card, CardContent, CardHeader, CardTitle, Badge } from '@/components/ui';
import { CheckCircle, XCircle } from 'lucide-react';

interface FirebaseConfigValue {
  key: string;
  value: string | undefined;
  label: string;
  isRequired: boolean;
}

export function FirebaseServerConfig() {
  // Minimal Firebase config for analytics only
  const firebaseConfigs: FirebaseConfigValue[] = [
    {
      key: 'NEXT_PUBLIC_FIREBASE_API_KEY',
      value: process.env.NEXT_PUBLIC_FIREBASE_API_KEY,
      label: 'API Key',
      isRequired: true
    },
    {
      key: 'NEXT_PUBLIC_FIREBASE_PROJECT_ID',
      value: process.env.NEXT_PUBLIC_FIREBASE_PROJECT_ID,
      label: 'Project ID',
      isRequired: true
    },
    {
      key: 'NEXT_PUBLIC_FIREBASE_APP_ID',
      value: process.env.NEXT_PUBLIC_FIREBASE_APP_ID,
      label: 'App ID',
      isRequired: true
    },
    {
      key: 'NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID',
      value: process.env.NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID,
      label: 'Measurement ID (Analytics)',
      isRequired: false
    }
  ];

  const configuredCount = firebaseConfigs.filter(config => config.value).length;
  const requiredCount = firebaseConfigs.filter(config => config.isRequired).length;
  const requiredConfigured = firebaseConfigs.filter(config => config.isRequired && config.value).length;

  const isFullyConfigured = requiredConfigured === requiredCount;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          📊 Firebase Analytics Configuration
          {isFullyConfigured ? (
            <Badge variant="default" className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
              <CheckCircle className="h-3 w-3 mr-1" />
              Analytics Ready
            </Badge>
          ) : (
            <Badge variant="destructive">
              <XCircle className="h-3 w-3 mr-1" />
              Incomplete
            </Badge>
          )}
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="text-sm text-muted-foreground">
          Configuration status: {configuredCount} of {firebaseConfigs.length} environment variables set
          ({requiredConfigured} of {requiredCount} required variables configured)
        </div>

        <div className="grid gap-3">
          {firebaseConfigs.map((config) => {
            const isConfigured = !!config.value;
            const displayValue = config.value 
              ? `${config.value.substring(0, 8)}...${config.value.slice(-4)}`
              : 'Not configured';

            return (
              <div key={config.key} className="flex items-center justify-between p-3 border rounded-lg">
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="font-medium text-sm">{config.label}</span>
                    {config.isRequired && (
                      <Badge variant="outline" className="text-xs px-1 py-0">
                        Required
                      </Badge>
                    )}
                  </div>
                  <div className="text-xs text-muted-foreground font-mono">
                    {config.key}
                  </div>
                </div>
                <div className="flex items-center gap-2">
                  <span className={`text-xs font-mono px-2 py-1 rounded ${
                    isConfigured 
                      ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200' 
                      : 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200'
                  }`}>
                    {displayValue}
                  </span>
                  {isConfigured ? (
                    <CheckCircle className="h-4 w-4 text-green-600" />
                  ) : (
                    <XCircle className="h-4 w-4 text-red-600" />
                  )}
                </div>
              </div>
            );
          })}
        </div>

        <div className="bg-blue-50 dark:bg-blue-950 p-3 rounded-lg">
          <div className="text-sm text-blue-700 dark:text-blue-300">
            <strong>📊 Analytics Only:</strong> Firebase is configured for analytics only. 
            FCM messaging has been removed for Web3 migration. Authentication is handled via OIDC.
          </div>
        </div>

        {!isFullyConfigured && (
          <div className="bg-yellow-50 dark:bg-yellow-950 p-3 rounded-lg">
            <div className="text-sm text-yellow-700 dark:text-yellow-300">
              <strong>⚠️ Configuration Required:</strong> Some required Firebase environment variables are missing. 
              Please check your .env file and ensure all required values are set.
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}