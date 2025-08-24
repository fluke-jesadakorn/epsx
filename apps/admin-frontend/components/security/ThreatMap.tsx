'use client';

import { useState, useEffect, useMemo } from 'react';
import { cn } from '@/lib/utils';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent } from '@/components/ui/card';
import {
  Target,
  MapPin,
  Zap,
  Shield,
  AlertTriangle,
  ZoomIn,
  ZoomOut,
  RotateCcw,
  Info
} from 'lucide-react';

interface ThreatData {
  id: string;
  ip: string;
  country: string;
  city: string;
  latitude: number;
  longitude: number;
  threatType: 'brute_force' | 'sql_injection' | 'ddos' | 'malware' | 'phishing';
  severity: 'low' | 'medium' | 'high' | 'critical';
  firstSeen: Date;
  lastSeen: Date;
  attackCount: number;
  blocked: boolean;
  reputation: number;
}

interface ThreatMapProps {
  threats: ThreatData[];
  onThreatSelect?: (threat: ThreatData) => void;
}

export function ThreatMap({ threats, onThreatSelect }: ThreatMapProps) {
  const [selectedThreat, setSelectedThreat] = useState<ThreatData | null>(null);
  const [zoomLevel, setZoomLevel] = useState(1);
  const [panX, setPanX] = useState(0);
  const [panY, setPanY] = useState(0);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

  // World map SVG path (simplified)
  const worldMapPath = "M158.1,139.9 L171.6,137.2 L178.6,133.6 L184.3,130.8 L188.6,128.6 L191.5,127 L193.1,125.9 L194.1,125.4 L195.4,125.3 L197.1,125.6 L199.3,126.4 L202.1,127.8 L205.6,129.9 L210,132.7 L215.4,136.4 L221.9,141 L229.6,146.7 L238.7,153.5 L249.2,161.5 L261.2,170.8 L274.8,181.4 L290,193.4 L306.9,206.8 L325.6,221.7 L346.2,238.1 L368.6,256.1 L392.9,275.7 L419.1,296.9 L447.2,319.8 L477.3,344.4 L509.4,370.7 L543.5,398.8 L579.7,428.7 L618,460.4 L658.4,493.9 L700.9,529.3 L745.6,566.6 L792.4,605.8 L841.4,646.9 L892.6,690 L946,735.1 L1001.6,782.2 L1059.4,831.3 L1119.4,882.5 L1181.6,935.8 L1246,991.2 L1312.6,1048.7 L1381.4,1108.3 L1452.4,1170.1 L1525.6,1234 L1601,1300.1 L1678.6,1368.4 L1758.4,1438.9 L1840.4,1511.6 L1924.6,1586.5 L2011,1663.6 L2099.6,1742.9 L2190.4,1824.4 L2283.4,1908.1 L2378.6,1994 L2476,2082.1 L2575.6,2172.4 L2677.4,2264.9 L2781.4,2359.6 L2887.6,2456.5 L2996,2555.6 L3106.6,2656.9 L3219.4,2760.4 L3334.4,2866.1 L3451.6,2974 L3571,3084.1 L3692.6,3196.4 L3816.4,3310.9 L3942.4,3427.6 L4070.6,3546.5 L4201,3667.6 L4333.6,3790.9 L4468.4,3916.4 L4605.4,4044.1 L4744.6,4174 L4886,4306.1 L5029.6,4440.4 L5175.4,4576.9 L5323.4,4715.6 L5473.6,4856.5 L5626,4999.6";

  // Convert lat/lng to SVG coordinates
  const latLngToSvg = (lat: number, lng: number) => {
    // Simple mercator projection
    const x = ((lng + 180) / 360) * 800;
    const y = ((90 - lat) / 180) * 400;
    return { x, y };
  };

  // Get threat marker color based on severity
  const getThreatColor = (severity: string, blocked: boolean) => {
    if (blocked) return 'fill-gray-500';
    switch (severity) {
      case 'critical': return 'fill-red-500';
      case 'high': return 'fill-orange-500';
      case 'medium': return 'fill-yellow-500';
      case 'low': return 'fill-blue-500';
      default: return 'fill-gray-500';
    }
  };

  // Get threat icon based on type
  const getThreatIcon = (type: string) => {
    switch (type) {
      case 'brute_force': return '🔐';
      case 'sql_injection': return '💉';
      case 'ddos': return '⚡';
      case 'malware': return '🦠';
      case 'phishing': return '🎣';
      default: return '⚠️';
    }
  };

  // Handle threat marker click
  const handleThreatClick = (threat: ThreatData) => {
    setSelectedThreat(threat);
    onThreatSelect?.(threat);
  };

  // Handle map controls
  const handleZoomIn = () => setZoomLevel(prev => Math.min(prev * 1.5, 5));
  const handleZoomOut = () => setZoomLevel(prev => Math.max(prev / 1.5, 0.5));
  const handleReset = () => {
    setZoomLevel(1);
    setPanX(0);
    setPanY(0);
  };

  // Handle mouse events for panning
  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    setDragStart({ x: e.clientX - panX, y: e.clientY - panY });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDragging) return;
    setPanX(e.clientX - dragStart.x);
    setPanY(e.clientY - dragStart.y);
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  // Group threats by location for clustering
  const clusteredThreats = useMemo(() => {
    const clusters = new Map<string, ThreatData[]>();
    
    threats.forEach(threat => {
      const key = `${threat.latitude.toFixed(1)},${threat.longitude.toFixed(1)}`;
      if (!clusters.has(key)) {
        clusters.set(key, []);
      }
      clusters.get(key)!.push(threat);
    });
    
    return Array.from(clusters.entries()).map(([key, threats]) => ({
      key,
      threats,
      position: latLngToSvg(threats[0].latitude, threats[0].longitude),
      count: threats.length,
      maxSeverity: threats.reduce((max, t) => 
        ['low', 'medium', 'high', 'critical'].indexOf(t.severity) > 
        ['low', 'medium', 'high', 'critical'].indexOf(max) ? t.severity : max
      , 'low'),
      hasBlocked: threats.some(t => t.blocked)
    }));
  }, [threats]);

  return (
    <div className="space-y-4">
      {/* Map Controls */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Button size="sm" variant="outline" onClick={handleZoomIn}>
            <ZoomIn className="w-4 h-4" />
          </Button>
          <Button size="sm" variant="outline" onClick={handleZoomOut}>
            <ZoomOut className="w-4 h-4" />
          </Button>
          <Button size="sm" variant="outline" onClick={handleReset}>
            <RotateCcw className="w-4 h-4" />
          </Button>
          <span className="text-sm text-muted-foreground ml-2">
            Zoom: {Math.round(zoomLevel * 100)}%
          </span>
        </div>

        {/* Legend */}
        <div className="flex items-center gap-4 text-sm">
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded-full bg-red-500"></div>
            <span>Critical</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded-full bg-orange-500"></div>
            <span>High</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded-full bg-yellow-500"></div>
            <span>Medium</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded-full bg-blue-500"></div>
            <span>Low</span>
          </div>
          <div className="flex items-center gap-1">
            <div className="w-3 h-3 rounded-full bg-gray-500"></div>
            <span>Blocked</span>
          </div>
        </div>
      </div>

      {/* Map Container */}
      <div className="relative bg-muted/10 rounded-lg overflow-hidden border" style={{ height: '500px' }}>
        <svg
          width="100%"
          height="100%"
          viewBox="0 0 800 400"
          className="cursor-move"
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
        >
          <g
            transform={`translate(${panX}, ${panY}) scale(${zoomLevel})`}
          >
            {/* World Map Background */}
            <rect width="800" height="400" fill="hsl(var(--muted))" opacity="0.3" />
            
            {/* Simplified world map outlines */}
            <g fill="none" stroke="hsl(var(--border))" strokeWidth="1" opacity="0.6">
              {/* Continents (simplified rectangles) */}
              <rect x="50" y="80" width="150" height="120" rx="10" /> {/* North America */}
              <rect x="220" y="100" width="100" height="100" rx="8" /> {/* Europe */}
              <rect x="340" y="90" width="120" height="140" rx="10" /> {/* Asia */}
              <rect x="480" y="120" width="80" height="100" rx="8" /> {/* Middle East */}
              <rect x="200" y="220" width="120" height="100" rx="10" /> {/* Africa */}
              <rect x="550" y="180" width="100" height="80" rx="8" /> {/* Southeast Asia */}
              <rect x="650" y="250" width="80" height="60" rx="8" /> {/* Australia */}
              <rect x="80" y="240" width="100" height="120" rx="10" /> {/* South America */}
            </g>

            {/* Grid lines */}
            <defs>
              <pattern id="grid" width="40" height="40" patternUnits="userSpaceOnUse">
                <path d="M 40 0 L 0 0 0 40" fill="none" stroke="hsl(var(--border))" strokeWidth="0.5" opacity="0.3"/>
              </pattern>
            </defs>
            <rect width="800" height="400" fill="url(#grid)" />

            {/* Threat Markers */}
            {clusteredThreats.map(cluster => (
              <g key={cluster.key}>
                {/* Cluster marker */}
                <circle
                  cx={cluster.position.x}
                  cy={cluster.position.y}
                  r={Math.min(8 + cluster.count * 2, 20)}
                  className={cn(
                    "stroke-2 stroke-white transition-all cursor-pointer hover:scale-110",
                    getThreatColor(cluster.maxSeverity, cluster.hasBlocked)
                  )}
                  onClick={() => handleThreatClick(cluster.threats[0])}
                />
                
                {/* Cluster count */}
                <text
                  x={cluster.position.x}
                  y={cluster.position.y + 4}
                  textAnchor="middle"
                  className="fill-white text-xs font-bold pointer-events-none"
                >
                  {cluster.count}
                </text>

                {/* Pulse effect for critical threats */}
                {cluster.maxSeverity === 'critical' && !cluster.hasBlocked && (
                  <circle
                    cx={cluster.position.x}
                    cy={cluster.position.y}
                    r={Math.min(8 + cluster.count * 2, 20)}
                    className="fill-red-500 opacity-50 animate-ping"
                  />
                )}
              </g>
            ))}
          </g>
        </svg>

        {/* Real-time indicator */}
        <div className="absolute top-4 left-4 flex items-center gap-2 bg-card/95 backdrop-blur-sm rounded-lg px-3 py-2 border">
          <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
          <span className="text-xs font-medium">Live Tracking</span>
        </div>

        {/* Stats overlay */}
        <div className="absolute top-4 right-4 bg-card/95 backdrop-blur-sm rounded-lg p-3 border">
          <div className="text-xs text-muted-foreground mb-1">Active Threats</div>
          <div className="text-2xl font-bold">{threats.filter(t => !t.blocked).length}</div>
        </div>
      </div>

      {/* Selected Threat Details */}
      {selectedThreat && (
        <Card className="bg-muted/50">
          <CardContent className="p-4">
            <div className="flex items-start justify-between mb-3">
              <div>
                <h4 className="font-semibold flex items-center gap-2">
                  <MapPin className="w-4 h-4" />
                  {selectedThreat.city}, {selectedThreat.country}
                </h4>
                <p className="text-sm text-muted-foreground font-mono">{selectedThreat.ip}</p>
              </div>
              <div className="flex gap-2">
                <Badge className={cn("text-xs", getThreatColor(selectedThreat.severity, false).replace('fill-', 'bg-'))}>
                  {selectedThreat.severity}
                </Badge>
                {selectedThreat.blocked && (
                  <Badge variant="destructive" className="text-xs">Blocked</Badge>
                )}
              </div>
            </div>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
              <div>
                <div className="text-muted-foreground">Threat Type</div>
                <div className="font-medium flex items-center gap-1">
                  <span>{getThreatIcon(selectedThreat.threatType)}</span>
                  {selectedThreat.threatType.replace('_', ' ')}
                </div>
              </div>
              <div>
                <div className="text-muted-foreground">Attack Count</div>
                <div className="font-medium">{selectedThreat.attackCount}</div>
              </div>
              <div>
                <div className="text-muted-foreground">Reputation</div>
                <div className={cn(
                  "font-medium",
                  selectedThreat.reputation < 25 ? "text-red-500" :
                  selectedThreat.reputation < 50 ? "text-orange-500" :
                  selectedThreat.reputation < 75 ? "text-yellow-500" : "text-green-500"
                )}>
                  {selectedThreat.reputation}/100
                </div>
              </div>
              <div>
                <div className="text-muted-foreground">Last Seen</div>
                <div className="font-medium">{selectedThreat.lastSeen.toLocaleTimeString()}</div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Help Text */}
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <Info className="w-4 h-4" />
        <span>Click and drag to pan, use zoom controls to navigate. Click threat markers for details.</span>
      </div>
    </div>
  );
}