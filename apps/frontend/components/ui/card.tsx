/**
 * FRONTEND CARD COMPONENT
 * Migrated to use unified shared UI component
 */

import {
    CardContent,
    CardDescription,
    CardFooter,
    CardHeader,
    CardTitle,
    Card as SharedCard,
    type CardProps,
} from "../../../../shared/components/ui/card"

const Card = SharedCard
Card.displayName = "Card"

export { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle }
export type { CardProps }

