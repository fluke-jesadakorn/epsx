import { LucideIcon } from 'lucide-react'

interface PageHeaderProps {
    title: string
    description?: string
    icon?: LucideIcon
}

/**
 *
 * @param root0
 * @param root0.title
 * @param root0.description
 * @param root0.icon
 */
export function PageHeader({ title, description, icon: Icon }: PageHeaderProps) {
    return (
        <div className="flex flex-col gap-2 mb-8 animate-in fade-in slide-in-from-top-4 duration-500">
            <div className="flex items-center gap-3">
                {Icon && (
                    <div className="p-2.5 bg-white dark:bg-gray-800 rounded-xl shadow-sm border border-gray-200 dark:border-gray-700">
                        <Icon className="w-6 h-6 text-gray-700 dark:text-gray-300" />
                    </div>
                )}
                <h1 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-gray-100">
                    {title}
                </h1>
            </div>
            {description && (
                <p className="text-gray-500 dark:text-gray-400 ml-1">
                    {description}
                </p>
            )}
        </div>
    )
}
