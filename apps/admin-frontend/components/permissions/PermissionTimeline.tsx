'use client'

import { useState, useMemo, useEffect } from 'react'
import { 
  Clock, 
  Calendar, 
  Filter,
  ZoomIn,
  ZoomOut,
  Play,
  Pause,
  RotateCcw,
  ChevronLeft,
  ChevronRight,
  AlertTriangle,
  CheckCircle,
  XCircle,
  User,
  Shield,
  Activity,
  TrendingUp,
  Eye,
  Settings
} from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Slider } from '@/components/ui/slider'
import { Switch } from '@/components/ui/switch'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import { format, addHours, addDays, addWeeks, isAfter, isBefore, startOfDay, endOfDay } from 'date-fns'

// ============================================================================
// TYPES AND INTERFACES
// ============================================================================

interface TimelineEvent {
  id: string
  type: 'granted' | 'expired' | 'revoked' | 'extended' | 'warning'
  timestamp: Date
  userId: string
  userName: string
  userEmail: string
  permission: string
  basePermission: string
  expiryTimestamp?: number
  duration?: number // minutes
  reason?: string
  details?: string
  metadata?: Record<string, any>
}

interface TimelineRange {
  start: Date
  end: Date
  label: string
}

interface PermissionTimelineProps {
  userId?: string // If provided, show timeline for specific user
  className?: string
}

// ============================================================================
// MOCK DATA
// ============================================================================

const generateMockTimelineEvents = (): TimelineEvent[] => {
  const now = new Date()
  const events: TimelineEvent[] = []

  // Generate events over the past 7 days and future 3 days
  for (let i = -7; i <= 3; i++) {
    const date = addDays(now, i)
    
    // Add some random events for demonstration
    if (Math.random() > 0.3) {
      const eventTypes: TimelineEvent['type'][] = ['granted', 'expired', 'revoked', 'extended', 'warning']
      const permissions = [
        'epsx:analytics:view',
        'epsx:rankings:view:100',
        'admin:users:manage',
        'epsx:analytics:export',
        'epsx:realtime:access'
      ]
      
      events.push({
        id: `event-${i}-${Math.random()}`,
        type: eventTypes[Math.floor(Math.random() * eventTypes.length)],
        timestamp: addHours(date, Math.floor(Math.random() * 24)),
        userId: `user-${Math.floor(Math.random() * 5) + 1}`,
        userName: ['John Doe', 'Jane Smith', 'Bob Wilson', 'Alice Johnson', 'Charlie Brown'][Math.floor(Math.random() * 5)],
        userEmail: `user${Math.floor(Math.random() * 5) + 1}@example.com`,
        permission: permissions[Math.floor(Math.random() * permissions.length)],
        basePermission: permissions[Math.floor(Math.random() * permissions.length)],
        expiryTimestamp: Math.floor((addHours(date, Math.floor(Math.random() * 48) + 1).getTime()) / 1000),
        duration: [60, 240, 480, 1440, 4320][Math.floor(Math.random() * 5)],
        reason: ['Emergency access', 'Temporary project work', 'Client demo', 'System maintenance', 'Training session'][Math.floor(Math.random() * 5)]
      })
    }
  }

  return events.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime())
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export function PermissionTimeline({ userId, className = '' }: PermissionTimelineProps) {
  // State
  const [events] = useState<TimelineEvent[]>(generateMockTimelineEvents())
  const [selectedRange, setSelectedRange] = useState<'24h' | '7d' | '30d' | 'custom'>('7d')
  const [eventTypeFilter, setEventTypeFilter] = useState<'all' | TimelineEvent['type']>('all')
  const [zoomLevel, setZoomLevel] = useState<number>(1)
  const [autoRefresh, setAutoRefresh] = useState<boolean>(true)
  const [showFutureEvents, setShowFutureEvents] = useState<boolean>(true)
  const [selectedEvent, setSelectedEvent] = useState<TimelineEvent | null>(null)
  const [currentTime, setCurrentTime] = useState<Date>(new Date())

  // Auto refresh current time
  useEffect(() => {
    if (!autoRefresh) return
    
    const interval = setInterval(() => {
      setCurrentTime(new Date())
    }, 60000) // Update every minute
    
    return () => clearInterval(interval)
  }, [autoRefresh])

  // Calculate timeline range
  const timelineRange = useMemo((): TimelineRange => {
    const now = new Date()
    
    switch (selectedRange) {
      case '24h':
        return {
          start: addHours(now, -12),
          end: addHours(now, 12),
          label: 'Last 12h - Next 12h'
        }
      case '7d':
        return {
          start: addDays(now, -4),
          end: addDays(now, 3),
          label: 'Last 4 days - Next 3 days'
        }
      case '30d':
        return {
          start: addDays(now, -15),
          end: addDays(now, 15),
          label: 'Last 15 days - Next 15 days'
        }
      default:
        return {
          start: addDays(now, -7),
          end: addDays(now, 3),
          label: 'Custom Range'
        }
    }
  }, [selectedRange])

  // Filter events
  const filteredEvents = useMemo(() => {
    let filtered = events.filter(event => {
      // Time range filter
      const inRange = isAfter(event.timestamp, timelineRange.start) && 
                      isBefore(event.timestamp, timelineRange.end)
      
      if (!inRange) return false

      // Future events filter
      if (!showFutureEvents && isAfter(event.timestamp, currentTime)) {
        return false
      }

      // Event type filter
      if (eventTypeFilter !== 'all' && event.type !== eventTypeFilter) {
        return false
      }

      // User filter
      if (userId && event.userId !== userId) {
        return false
      }

      return true
    })

    return filtered
  }, [events, timelineRange, eventTypeFilter, showFutureEvents, userId, currentTime])

  // Group events by date
  const eventsByDate = useMemo(() => {
    const groups: Record<string, TimelineEvent[]> = {}
    
    filteredEvents.forEach(event => {
      const dateKey = format(event.timestamp, 'yyyy-MM-dd')
      if (!groups[dateKey]) {
        groups[dateKey] = []
      }
      groups[dateKey].push(event)
    })
    
    return groups
  }, [filteredEvents])

  // Timeline statistics
  const timelineStats = useMemo(() => {
    const stats = {
      totalEvents: filteredEvents.length,
      grantedCount: filteredEvents.filter(e => e.type === 'granted').length,
      expiredCount: filteredEvents.filter(e => e.type === 'expired').length,
      revokedCount: filteredEvents.filter(e => e.type === 'revoked').length,
      warningCount: filteredEvents.filter(e => e.type === 'warning').length,
      upcomingExpirations: filteredEvents.filter(e => 
        e.type === 'granted' && 
        e.expiryTimestamp && 
        e.expiryTimestamp > Date.now() / 1000 && 
        e.expiryTimestamp < (Date.now() / 1000) + 86400 // Next 24 hours
      ).length
    }
    
    return stats
  }, [filteredEvents])

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-xl font-semibold">Permission Timeline</h3>
          <p className="text-muted-foreground">
            {userId ? 'User-specific permission history and forecasts' : 'System-wide permission activity and projections'}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Switch 
            id="auto-refresh" 
            checked={autoRefresh}
            onCheckedChange={setAutoRefresh}
          />
          <Label htmlFor="auto-refresh" className="text-sm">Auto refresh</Label>
        </div>
      </div>

      {/* Timeline Stats */}
      <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
        <Card>
          <CardContent className="p-3">
            <div className="text-center">
              <Activity className="h-5 w-5 mx-auto mb-1 text-blue-500" />
              <p className="text-lg font-bold">{timelineStats.totalEvents}</p>
              <p className="text-xs text-muted-foreground">Total Events</p>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-3">
            <div className="text-center">
              <CheckCircle className="h-5 w-5 mx-auto mb-1 text-green-500" />
              <p className="text-lg font-bold">{timelineStats.grantedCount}</p>
              <p className="text-xs text-muted-foreground">Granted</p>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-3">
            <div className="text-center">
              <Clock className="h-5 w-5 mx-auto mb-1 text-gray-500" />
              <p className="text-lg font-bold">{timelineStats.expiredCount}</p>
              <p className="text-xs text-muted-foreground">Expired</p>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-3">
            <div className="text-center">
              <XCircle className="h-5 w-5 mx-auto mb-1 text-red-500" />
              <p className="text-lg font-bold">{timelineStats.revokedCount}</p>
              <p className="text-xs text-muted-foreground">Revoked</p>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-3">
            <div className="text-center">
              <AlertTriangle className="h-5 w-5 mx-auto mb-1 text-yellow-500" />
              <p className="text-lg font-bold">{timelineStats.warningCount}</p>
              <p className="text-xs text-muted-foreground">Warnings</p>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardContent className="p-3">
            <div className="text-center">
              <TrendingUp className="h-5 w-5 mx-auto mb-1 text-orange-500" />
              <p className="text-lg font-bold">{timelineStats.upcomingExpirations}</p>
              <p className="text-xs text-muted-foreground">Expiring 24h</p>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Timeline Controls */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            Timeline Controls
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            {/* Range Selection */}
            <div className="space-y-2">
              <Label>Time Range</Label>
              <Select value={selectedRange} onValueChange={(value: any) => setSelectedRange(value)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="24h">24 Hours</SelectItem>
                  <SelectItem value="7d">7 Days</SelectItem>
                  <SelectItem value="30d">30 Days</SelectItem>
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">{timelineRange.label}</p>
            </div>

            {/* Event Type Filter */}
            <div className="space-y-2">
              <Label>Event Type</Label>
              <Select value={eventTypeFilter} onValueChange={(value: any) => setEventTypeFilter(value)}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All Events</SelectItem>
                  <SelectItem value="granted">Granted</SelectItem>
                  <SelectItem value="expired">Expired</SelectItem>
                  <SelectItem value="revoked">Revoked</SelectItem>
                  <SelectItem value="extended">Extended</SelectItem>
                  <SelectItem value="warning">Warnings</SelectItem>
                </SelectContent>
              </Select>
            </div>

            {/* Zoom Level */}
            <div className="space-y-2">
              <Label>Zoom Level: {zoomLevel.toFixed(1)}x</Label>
              <Slider
                value={[zoomLevel]}
                onValueChange={(value) => setZoomLevel(value[0])}
                min={0.5}
                max={3}
                step={0.1}
                className="w-full"
              />
            </div>

            {/* Toggle Options */}
            <div className="space-y-3">
              <Label>Options</Label>
              <div className="flex items-center space-x-2">
                <Switch 
                  id="future-events" 
                  checked={showFutureEvents}
                  onCheckedChange={setShowFutureEvents}
                />
                <Label htmlFor="future-events" className="text-sm">Show future events</Label>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Timeline Visualization */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Calendar className="h-5 w-5" />
            Timeline View
          </CardTitle>
          <CardDescription>
            Permission events over time with embedded timestamp tracking
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {/* Current Time Indicator */}
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Clock className="h-4 w-4" />
              Current time: {format(currentTime, 'PPp')}
            </div>

            {/* Timeline */}
            <div className="relative">
              {Object.keys(eventsByDate).length === 0 ? (
                <div className="text-center py-8">
                  <Calendar className="h-12 w-12 mx-auto text-muted-foreground mb-4" />
                  <h3 className="text-lg font-medium mb-2">No events in selected timeframe</h3>
                  <p className="text-muted-foreground">
                    Try adjusting your filters or time range
                  </p>
                </div>
              ) : (
                <div className="space-y-6" style={{ transform: `scale(${zoomLevel})`, transformOrigin: 'top left' }}>
                  {Object.entries(eventsByDate).map(([dateKey, dayEvents]) => (
                    <TimelineDayGroup
                      key={dateKey}
                      date={new Date(dateKey)}
                      events={dayEvents}
                      currentTime={currentTime}
                      onEventSelect={setSelectedEvent}
                      selectedEvent={selectedEvent}
                    />
                  ))}
                </div>
              )}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Selected Event Details */}
      {selectedEvent && (
        <Card className="border-blue-200 bg-blue-50 dark:bg-blue-900/20">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Eye className="h-5 w-5" />
              Event Details
            </CardTitle>
          </CardHeader>
          <CardContent>
            <TimelineEventDetails
              event={selectedEvent}
              onClose={() => setSelectedEvent(null)}
            />
          </CardContent>
        </Card>
      )}
    </div>
  )
}

// ============================================================================
// TIMELINE DAY GROUP COMPONENT
// ============================================================================

interface TimelineDayGroupProps {
  date: Date
  events: TimelineEvent[]
  currentTime: Date
  onEventSelect: (event: TimelineEvent) => void
  selectedEvent: TimelineEvent | null
}

function TimelineDayGroup({ 
  date, 
  events, 
  currentTime, 
  onEventSelect, 
  selectedEvent 
}: TimelineDayGroupProps) {
  const isToday = format(date, 'yyyy-MM-dd') === format(currentTime, 'yyyy-MM-dd')
  const isPast = isBefore(endOfDay(date), currentTime)
  const isFuture = isAfter(startOfDay(date), currentTime)

  return (
    <div className="relative">
      {/* Date Header */}
      <div className={`flex items-center gap-2 mb-4 pb-2 border-b ${
        isToday ? 'border-blue-500' : 'border-gray-200'
      }`}>
        <div className={`w-3 h-3 rounded-full ${
          isToday ? 'bg-blue-500' : isPast ? 'bg-gray-400' : 'bg-green-500'
        }`} />
        <h4 className={`text-lg font-medium ${
          isToday ? 'text-blue-600' : isPast ? 'text-gray-600' : 'text-green-600'
        }`}>
          {format(date, 'EEEE, MMM d')}
          {isToday && <span className="ml-2 text-sm">(Today)</span>}
          {isFuture && <span className="ml-2 text-sm">(Future)</span>}
        </h4>
        <Badge variant="secondary" className="ml-auto">
          {events.length} events
        </Badge>
      </div>

      {/* Events */}
      <div className="space-y-2 pl-5">
        {events.map((event, index) => (
          <TimelineEventCard
            key={event.id}
            event={event}
            isSelected={selectedEvent?.id === event.id}
            onSelect={() => onEventSelect(event)}
          />
        ))}
      </div>
    </div>
  )
}

// ============================================================================
// TIMELINE EVENT CARD COMPONENT
// ============================================================================

interface TimelineEventCardProps {
  event: TimelineEvent
  isSelected: boolean
  onSelect: () => void
}

function TimelineEventCard({ event, isSelected, onSelect }: TimelineEventCardProps) {
  const getEventIcon = (type: TimelineEvent['type']) => {
    const icons = {
      granted: CheckCircle,
      expired: Clock,
      revoked: XCircle,
      extended: TrendingUp,
      warning: AlertTriangle
    }
    return icons[type] || Shield
  }

  const getEventColor = (type: TimelineEvent['type']) => {
    const colors = {
      granted: 'text-green-600 bg-green-50 border-green-200',
      expired: 'text-gray-600 bg-gray-50 border-gray-200',
      revoked: 'text-red-600 bg-red-50 border-red-200',
      extended: 'text-blue-600 bg-blue-50 border-blue-200',
      warning: 'text-yellow-600 bg-yellow-50 border-yellow-200'
    }
    return colors[type] || 'text-gray-600 bg-gray-50 border-gray-200'
  }

  const Icon = getEventIcon(event.type)
  
  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <div
            className={`
              flex items-center gap-3 p-3 rounded-lg border cursor-pointer transition-all
              ${getEventColor(event.type)}
              ${isSelected ? 'ring-2 ring-blue-500' : 'hover:shadow-sm'}
            `}
            onClick={onSelect}
          >
            <Icon className="h-4 w-4 flex-shrink-0" />
            
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between">
                <p className="font-medium truncate">
                  {event.type.charAt(0).toUpperCase() + event.type.slice(1)} - {event.userName}
                </p>
                <span className="text-xs text-muted-foreground">
                  {format(event.timestamp, 'HH:mm')}
                </span>
              </div>
              
              <p className="text-sm text-muted-foreground truncate">
                {event.basePermission}
              </p>
              
              {event.reason && (
                <p className="text-xs text-muted-foreground truncate">
                  {event.reason}
                </p>
              )}
            </div>
            
            {event.expiryTimestamp && (
              <div className="text-xs text-muted-foreground">
                Expires: {format(new Date(event.expiryTimestamp * 1000), 'MMM d, HH:mm')}
              </div>
            )}
          </div>
        </TooltipTrigger>
        <TooltipContent>
          <div className="space-y-1 text-sm">
            <div><strong>Type:</strong> {event.type}</div>
            <div><strong>User:</strong> {event.userName}</div>
            <div><strong>Permission:</strong> {event.basePermission}</div>
            <div><strong>Time:</strong> {format(event.timestamp, 'PPp')}</div>
            {event.reason && <div><strong>Reason:</strong> {event.reason}</div>}
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}

// ============================================================================
// EVENT DETAILS COMPONENT
// ============================================================================

interface TimelineEventDetailsProps {
  event: TimelineEvent
  onClose: () => void
}

function TimelineEventDetails({ event, onClose }: TimelineEventDetailsProps) {
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h4 className="text-lg font-medium">
          {event.type.charAt(0).toUpperCase() + event.type.slice(1)} Event
        </h4>
        <Button variant="ghost" size="sm" onClick={onClose}>
          ×
        </Button>
      </div>
      
      <Separator />
      
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="space-y-3">
          <div>
            <Label className="text-sm font-medium">User</Label>
            <p className="text-sm">{event.userName}</p>
            <p className="text-xs text-muted-foreground">{event.userEmail}</p>
          </div>
          
          <div>
            <Label className="text-sm font-medium">Permission</Label>
            <p className="text-sm font-mono">{event.basePermission}</p>
          </div>
          
          <div>
            <Label className="text-sm font-medium">Event Time</Label>
            <p className="text-sm">{format(event.timestamp, 'PPp')}</p>
          </div>
        </div>
        
        <div className="space-y-3">
          {event.expiryTimestamp && (
            <div>
              <Label className="text-sm font-medium">Expires At</Label>
              <p className="text-sm">{format(new Date(event.expiryTimestamp * 1000), 'PPp')}</p>
            </div>
          )}
          
          {event.duration && (
            <div>
              <Label className="text-sm font-medium">Duration</Label>
              <p className="text-sm">
                {event.duration < 60 ? `${event.duration} minutes` :
                 event.duration < 1440 ? `${Math.round(event.duration / 60)} hours` :
                 `${Math.round(event.duration / 1440)} days`}
              </p>
            </div>
          )}
          
          {event.reason && (
            <div>
              <Label className="text-sm font-medium">Reason</Label>
              <p className="text-sm">{event.reason}</p>
            </div>
          )}
        </div>
      </div>
      
      {event.details && (
        <>
          <Separator />
          <div>
            <Label className="text-sm font-medium">Additional Details</Label>
            <p className="text-sm text-muted-foreground">{event.details}</p>
          </div>
        </>
      )}
    </div>
  )
}