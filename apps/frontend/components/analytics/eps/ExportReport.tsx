'use client';

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { 
  Download, 
  FileText, 
  Share2, 
  Plus, 
  Settings,
  Calendar,
  BarChart3
} from 'lucide-react';

interface ExportReportProps {
  data: any;
  symbol: string;
  onExport: (format: string) => void;
  onAddComparison: (symbol: string) => void;
}

const EXPORT_FORMATS = [
  { value: 'pdf', label: 'PDF Report', icon: FileText },
  { value: 'excel', label: 'Excel Spreadsheet', icon: BarChart3 },
  { value: 'csv', label: 'CSV Data', icon: FileText },
  { value: 'json', label: 'JSON Data', icon: Settings }
];

const REPORT_SECTIONS = [
  { id: 'summary', label: 'Executive Summary', default: true },
  { id: 'analysis', label: 'Detailed Analysis', default: true },
  { id: 'charts', label: 'Charts & Visualizations', default: true },
  { id: 'patterns', label: 'Pattern Recognition', default: true },
  { id: 'historical', label: 'Historical Comparison', default: false },
  { id: 'predictions', label: 'AI Predictions', default: false },
  { id: 'methodology', label: 'Methodology & Disclaimers', default: true }
];

export function ExportReport({ data, symbol, onExport, onAddComparison }: ExportReportProps) {
  const [selectedFormat, setSelectedFormat] = useState('pdf');
  const [comparisonSymbol, setComparisonSymbol] = useState('');
  const [selectedSections, setSelectedSections] = useState(
    REPORT_SECTIONS.filter(section => section.default).map(section => section.id)
  );
  const [customTitle, setCustomTitle] = useState(`${symbol} EPS Analysis Report`);
  const [includeTimestamp, setIncludeTimestamp] = useState(true);

  const handleSectionToggle = (sectionId: string, checked: boolean) => {
    if (checked) {
      setSelectedSections([...selectedSections, sectionId]);
    } else {
      setSelectedSections(selectedSections.filter(id => id !== sectionId));
    }
  };

  const handleExport = () => {
    const exportData = {
      format: selectedFormat,
      sections: selectedSections,
      title: customTitle,
      timestamp: includeTimestamp,
      symbol: symbol
    };
    
    onExport(selectedFormat);
    
    // In real implementation, this would trigger the export process
    console.log('Exporting with config:', exportData);
  };

  const handleAddComparison = () => {
    if (comparisonSymbol.trim()) {
      onAddComparison(comparisonSymbol.toUpperCase());
      setComparisonSymbol('');
    }
  };

  const generatePreview = () => {
    const sections = selectedSections.map(id => 
      REPORT_SECTIONS.find(section => section.id === id)?.label
    ).filter(Boolean);
    
    return {
      title: customTitle,
      sections: sections,
      estimatedPages: Math.max(sections.length * 2, 3),
      dataPoints: 45 + (selectedSections.includes('historical') ? 30 : 0),
      charts: selectedSections.includes('charts') ? 4 : 0
    };
  };

  const preview = generatePreview();

  return (
    <div className="space-y-6">
      <Tabs defaultValue="export" className="w-full">
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="export">Export Report</TabsTrigger>
          <TabsTrigger value="share">Share Analysis</TabsTrigger>
          <TabsTrigger value="compare">Add Comparison</TabsTrigger>
        </TabsList>

        <TabsContent value="export" className="space-y-4">
          {/* Format Selection */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Download className="h-5 w-5" />
                Export Format
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
                {EXPORT_FORMATS.map((format) => (
                  <Card 
                    key={format.value}
                    className={`cursor-pointer transition-all ${
                      selectedFormat === format.value 
                        ? 'ring-2 ring-primary bg-primary/5' 
                        : 'hover:shadow-md'
                    }`}
                    onClick={() => setSelectedFormat(format.value)}
                  >
                    <CardContent className="p-4 text-center">
                      <format.icon className="h-6 w-6 mx-auto mb-2" />
                      <div className="font-medium text-sm">{format.label}</div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Report Configuration */}
          <Card>
            <CardHeader>
              <CardTitle>Report Configuration</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {/* Custom Title */}
              <div className="space-y-2">
                <Label htmlFor="title">Report Title</Label>
                <Input
                  id="title"
                  value={customTitle}
                  onChange={(e) => setCustomTitle(e.target.value)}
                  placeholder="Enter custom report title"
                />
              </div>

              {/* Section Selection */}
              <div className="space-y-3">
                <Label>Include Sections</Label>
                <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                  {REPORT_SECTIONS.map((section) => (
                    <div key={section.id} className="flex items-center space-x-2">
                      <Checkbox
                        id={section.id}
                        checked={selectedSections.includes(section.id)}
                        onCheckedChange={(checked) => 
                          handleSectionToggle(section.id, checked as boolean)
                        }
                      />
                      <Label 
                        htmlFor={section.id} 
                        className="text-sm font-normal cursor-pointer"
                      >
                        {section.label}
                      </Label>
                    </div>
                  ))}
                </div>
              </div>

              {/* Options */}
              <div className="flex items-center space-x-2">
                <Checkbox
                  id="timestamp"
                  checked={includeTimestamp}
                  onCheckedChange={setIncludeTimestamp}
                />
                <Label htmlFor="timestamp" className="text-sm font-normal cursor-pointer">
                  Include generation timestamp
                </Label>
              </div>
            </CardContent>
          </Card>

          {/* Report Preview */}
          <Card>
            <CardHeader>
              <CardTitle>Report Preview</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Title:</span>
                  <span className="text-sm">{preview.title}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Sections:</span>
                  <span className="text-sm">{preview.sections.length} selected</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Estimated Pages:</span>
                  <span className="text-sm">{preview.estimatedPages}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Data Points:</span>
                  <span className="text-sm">{preview.dataPoints}</span>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm font-medium">Charts:</span>
                  <span className="text-sm">{preview.charts}</span>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Export Button */}
          <Button 
            onClick={handleExport} 
            className="w-full" 
            size="lg"
            disabled={selectedSections.length === 0}
          >
            <Download className="h-4 w-4 mr-2" />
            Generate {selectedFormat.toUpperCase()} Report
          </Button>
        </TabsContent>

        <TabsContent value="share" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Share2 className="h-5 w-5" />
                Share Analysis
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Button variant="outline" className="h-20 flex-col">
                  <Share2 className="h-6 w-6 mb-2" />
                  Share Link
                </Button>
                <Button variant="outline" className="h-20 flex-col">
                  <FileText className="h-6 w-6 mb-2" />
                  Email Report
                </Button>
              </div>

              <div className="p-4 bg-muted rounded-lg">
                <h4 className="font-medium mb-2">Shareable Link</h4>
                <div className="flex items-center gap-2">
                  <Input 
                    value={`https://epsx.ai/analysis/${symbol.toLowerCase()}/eps/${Date.now()}`}
                    readOnly 
                    className="flex-1"
                  />
                  <Button size="sm">Copy</Button>
                </div>
                <p className="text-xs text-muted-foreground mt-2">
                  Link expires in 30 days • View-only access
                </p>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="compare" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <Plus className="h-5 w-5" />
                Add Company for Comparison
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center gap-2">
                <Input
                  placeholder="Enter symbol (e.g., MSFT, GOOGL)"
                  value={comparisonSymbol}
                  onChange={(e) => setComparisonSymbol(e.target.value.toUpperCase())}
                  onKeyPress={(e) => e.key === 'Enter' && handleAddComparison()}
                  className="flex-1"
                />
                <Button onClick={handleAddComparison} disabled={!comparisonSymbol.trim()}>
                  <Plus className="h-4 w-4 mr-2" />
                  Add
                </Button>
              </div>

              <div className="space-y-2">
                <Label>Quick Add Popular Comparisons</Label>
                <div className="flex flex-wrap gap-2">
                  {['MSFT', 'GOOGL', 'AMZN', 'META', 'NFLX', 'NVDA'].map((symbol) => (
                    <Badge 
                      key={symbol}
                      variant="outline" 
                      className="cursor-pointer hover:bg-accent"
                      onClick={() => onAddComparison(symbol)}
                    >
                      {symbol}
                    </Badge>
                  ))}
                </div>
              </div>

              <div className="p-4 bg-blue-50 border border-blue-200 rounded-lg">
                <h4 className="font-medium text-blue-900 mb-2">Comparison Benefits</h4>
                <ul className="text-sm text-blue-700 space-y-1">
                  <li>• Benchmark against industry peers</li>
                  <li>• Identify relative performance patterns</li>
                  <li>• Compare growth consistency and volatility</li>
                  <li>• Generate comprehensive sector analysis</li>
                </ul>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}