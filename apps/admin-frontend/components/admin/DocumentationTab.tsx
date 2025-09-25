'use client';

import { memo, useState, useCallback } from 'react';
import { BookOpen, Code, Download, Globe, Copy } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

interface Module {
  id: string;
  name: string;
  description: string;
  endpoints: Array<{
    path: string;
    method: string;
    description: string;
    parameters?: Array<{
      name: string;
      type: string;
      required: boolean;
      description: string;
    }>;
    example?: {
      request?: string;
      response?: string;
    };
  }>;
}

interface DocumentationTabProps {
  modules: Module[];
  onCopyToClipboard: (text: string, label: string) => void;
}

function DocumentationTab({ modules, onCopyToClipboard }: DocumentationTabProps) {
  const [selectedModule, setSelectedModule] = useState<string>(modules[0]?.id || '');
  const [selectedEndpoint, setSelectedEndpoint] = useState<number>(0);

  const currentModule = modules.find(m => m.id === selectedModule);
  const currentEndpoint = currentModule?.endpoints[selectedEndpoint];

  const getMethodColor = useCallback((method: string) => {
    switch (method.toUpperCase()) {
      case 'GET':
        return 'bg-green-100 text-green-800';
      case 'POST':
        return 'bg-blue-100 text-blue-800';
      case 'PUT':
        return 'bg-yellow-100 text-yellow-800';
      case 'DELETE':
        return 'bg-red-100 text-red-800';
      case 'PATCH':
        return 'bg-orange-100 text-orange-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  }, []);

  const generateCurlExample = useCallback((endpoint: any) => {
    const method = endpoint.method.toUpperCase();
    const baseUrl = 'https://api.epsx.io';
    const url = `${baseUrl}${endpoint.path}`;
    
    let curl = `curl -X ${method} "${url}" \\\n`;
    curl += `  -H "Authorization: Bearer YOUR_API_KEY" \\\n`;
    curl += `  -H "Content-Type: application/json"`;
    
    if (method === 'POST' || method === 'PUT' || method === 'PATCH') {
      curl += ` \\\n  -d '{"example": "data"}'`;
    }
    
    return curl;
  }, []);

  const downloadDocs = useCallback(() => {
    const content = modules.map(module => {
      let doc = `# ${module.name}\n\n${module.description}\n\n`;
      module.endpoints.forEach(endpoint => {
        doc += `## ${endpoint.method.toUpperCase()} ${endpoint.path}\n\n`;
        doc += `${endpoint.description}\n\n`;
        if (endpoint.parameters) {
          doc += `### Parameters\n\n`;
          endpoint.parameters.forEach(param => {
            doc += `- **${param.name}** (${param.type}${param.required ? ', required' : ', optional'}): ${param.description}\n`;
          });
          doc += '\n';
        }
        if (endpoint.example) {
          doc += `### Example\n\n`;
          if (endpoint.example.request) {
            doc += `Request:\n\`\`\`json\n${endpoint.example.request}\n\`\`\`\n\n`;
          }
          if (endpoint.example.response) {
            doc += `Response:\n\`\`\`json\n${endpoint.example.response}\n\`\`\`\n\n`;
          }
        }
        doc += '\n---\n\n';
      });
      return doc;
    }).join('\n');

    const blob = new Blob([content], { type: 'text/markdown' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = 'epsx-api-documentation.md';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }, [modules]);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            API Documentation
          </h2>
          <p className="text-sm text-gray-600 dark:text-gray-300">
            Explore available endpoints and integration examples
          </p>
        </div>
        <Button variant="outline" onClick={downloadDocs}>
          <Download className="w-4 h-4 mr-2" />
          Download Docs
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
        {/* Module Sidebar */}
        <div className="lg:col-span-1">
          <Card>
            <CardHeader>
              <CardTitle className="text-sm">Modules</CardTitle>
            </CardHeader>
            <CardContent className="p-0">
              <div className="space-y-1">
                {modules.map(module => (
                  <button
                    key={module.id}
                    onClick={() => {
                      setSelectedModule(module.id);
                      setSelectedEndpoint(0);
                    }}
                    className={`w-full text-left px-4 py-3 text-sm border-l-2 hover:bg-gray-50 dark:hover:bg-gray-800 ${
                      selectedModule === module.id
                        ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-300'
                        : 'border-transparent text-gray-700 dark:text-gray-300'
                    }`}
                  >
                    <div className="font-medium">{module.name}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                      {module.endpoints.length} endpoints
                    </div>
                  </button>
                ))}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Main Documentation */}
        <div className="lg:col-span-3">
          {currentModule ? (
            <div className="space-y-6">
              {/* Module Overview */}
              <Card>
                <CardHeader>
                  <CardTitle className="flex items-center">
                    <Globe className="w-5 h-5 mr-2" />
                    {currentModule.name}
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <p className="text-gray-600 dark:text-gray-300">
                    {currentModule.description}
                  </p>
                </CardContent>
              </Card>

              {/* Endpoints List */}
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm">Endpoints</CardTitle>
                </CardHeader>
                <CardContent className="p-0">
                  <div className="space-y-1">
                    {currentModule.endpoints.map((endpoint, index) => (
                      <button
                        key={index}
                        onClick={() => setSelectedEndpoint(index)}
                        className={`w-full text-left px-4 py-3 border-l-2 hover:bg-gray-50 dark:hover:bg-gray-800 ${
                          selectedEndpoint === index
                            ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
                            : 'border-transparent'
                        }`}
                      >
                        <div className="flex items-center space-x-3">
                          <Badge className={getMethodColor(endpoint.method)}>
                            {endpoint.method.toUpperCase()}
                          </Badge>
                          <code className="text-sm font-mono">{endpoint.path}</code>
                        </div>
                        <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                          {endpoint.description}
                        </p>
                      </button>
                    ))}
                  </div>
                </CardContent>
              </Card>

              {/* Endpoint Details */}
              {currentEndpoint && (
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center justify-between">
                      <div className="flex items-center space-x-3">
                        <Badge className={getMethodColor(currentEndpoint.method)}>
                          {currentEndpoint.method.toUpperCase()}
                        </Badge>
                        <code className="text-lg">{currentEndpoint.path}</code>
                      </div>
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-6">
                    <p className="text-gray-600 dark:text-gray-300">
                      {currentEndpoint.description}
                    </p>

                    {/* Parameters */}
                    {currentEndpoint.parameters && currentEndpoint.parameters.length > 0 && (
                      <div>
                        <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-3">
                          Parameters
                        </h4>
                        <div className="space-y-3">
                          {currentEndpoint.parameters.map((param, index) => (
                            <div key={index} className="border border-gray-200 dark:border-gray-700 rounded-lg p-3">
                              <div className="flex items-center space-x-2 mb-1">
                                <code className="text-sm font-semibold text-blue-600 dark:text-blue-400">
                                  {param.name}
                                </code>
                                <Badge variant="outline" className="text-xs">
                                  {param.type}
                                </Badge>
                                {param.required && (
                                  <Badge variant="destructive" className="text-xs">
                                    required
                                  </Badge>
                                )}
                              </div>
                              <p className="text-sm text-gray-600 dark:text-gray-300">
                                {param.description}
                              </p>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}

                    {/* cURL Example */}
                    <div>
                      <div className="flex items-center justify-between mb-3">
                        <h4 className="font-medium text-gray-900 dark:text-gray-100">
                          cURL Example
                        </h4>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => onCopyToClipboard(generateCurlExample(currentEndpoint), 'cURL command')}
                        >
                          <Copy className="w-4 h-4 mr-1" />
                          Copy
                        </Button>
                      </div>
                      <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                        <code>{generateCurlExample(currentEndpoint)}</code>
                      </pre>
                    </div>

                    {/* Example Request/Response */}
                    {currentEndpoint.example && (
                      <div className="space-y-4">
                        {currentEndpoint.example.request && (
                          <div>
                            <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-3">
                              Example Request
                            </h4>
                            <pre className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg overflow-x-auto text-sm">
                              <code>{currentEndpoint.example.request}</code>
                            </pre>
                          </div>
                        )}
                        {currentEndpoint.example.response && (
                          <div>
                            <h4 className="font-medium text-gray-900 dark:text-gray-100 mb-3">
                              Example Response
                            </h4>
                            <pre className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg overflow-x-auto text-sm">
                              <code>{currentEndpoint.example.response}</code>
                            </pre>
                          </div>
                        )}
                      </div>
                    )}
                  </CardContent>
                </Card>
              )}
            </div>
          ) : (
            <Card>
              <CardContent className="p-12 text-center">
                <BookOpen className="w-12 h-12 text-gray-400 mx-auto mb-4" />
                <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
                  No Modules Available
                </h3>
                <p className="text-gray-600 dark:text-gray-300">
                  There are no API modules available for documentation.
                </p>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}

export default memo(DocumentationTab);