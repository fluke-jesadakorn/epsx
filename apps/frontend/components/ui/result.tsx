import type { ReactNode } from "react"
import { AlertCircle, AlertTriangle, CheckCircle } from "lucide-react"
import { cn } from "@/lib/utils"

interface ResultProps {
  status?: 'success' | 'error' | 'warning' | 'info'
  title: string
  subTitle?: string
  extra?: ReactNode
  className?: string
}

export function Result({
  status = 'info',
  title,
  subTitle,
  extra,
  className,
}: ResultProps) {
  const Icon = {
    success: CheckCircle,
    error: AlertCircle,
    warning: AlertTriangle,
    info: AlertCircle,
  }[status]

  const colors = {
    success: 'text-green-500',
    error: 'text-red-500',
    warning: 'text-yellow-500',
    info: 'text-blue-500',
  }[status]

  return (
    <div className={cn("flex flex-col items-center justify-center text-center p-8", className)}>
      {Icon && <Icon className={cn("w-16 h-16 mb-4", colors)} />}
      <h2 className="text-2xl font-semibold mb-2">{title}</h2>
      {subTitle && <p className="text-gray-600 mb-6">{subTitle}</p>}
      {extra && <div className="flex gap-2">{extra}</div>}
    </div>
  )
}
