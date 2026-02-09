// Adapted from Shadcn UI

import { useEffect, useState } from "react"

const TOAST_LIMIT = 1
const TOAST_REMOVE_DELAY = 1000000

interface ToasterToast {
    id: string
    title?: string
    description?: string
    action?: React.ReactNode
    variant?: "default" | "destructive" | "success" | "warning" | "info"
    duration?: number
    open?: boolean
    onOpenChange?: (open: boolean) => void
}

const actionTypes = {
    ADD_TOAST: "ADD_TOAST",
    UPDATE_TOAST: "UPDATE_TOAST",
    DISMISS_TOAST: "DISMISS_TOAST",
    REMOVE_TOAST: "REMOVE_TOAST",
} as const

let count = 0

function genId() {
    count = (count + 1) % Number.MAX_VALUE
    return count.toString()
}

type ActionType = typeof actionTypes

type Action =
    | {
        type: ActionType["ADD_TOAST"]
        toast: ToasterToast
    }
    | {
        type: ActionType["UPDATE_TOAST"]
        toast: Partial<ToasterToast>
    }
    | {
        type: ActionType["DISMISS_TOAST"]
        toastId?: ToasterToast["id"]
    }
    | {
        type: ActionType["REMOVE_TOAST"]
        toastId?: ToasterToast["id"]
    }

interface State {
    toasts: ToasterToast[]
}

const toastTimeouts = new Map<string, ReturnType<typeof setTimeout>>()

const addToRemoveQueue = (toastId: string) => {
    if (toastTimeouts.has(toastId)) {
        return
    }

    const timeout = setTimeout(() => {
        toastTimeouts.delete(toastId)
        dispatch({
            type: actionTypes.REMOVE_TOAST,
            toastId,
        })
    }, TOAST_REMOVE_DELAY)

    toastTimeouts.set(toastId, timeout)
}

export const reducer = (state: State, action: Action): State => {
    switch (action.type) {
        case actionTypes.ADD_TOAST:
            return {
                ...state,
                toasts: [action.toast, ...state.toasts].slice(0, TOAST_LIMIT),
            }

        case actionTypes.UPDATE_TOAST:
            return {
                ...state,
                toasts: state.toasts.map((t) =>
                    t.id === action.toast.id ? { ...t, ...action.toast } : t
                ),
            }

        case actionTypes.DISMISS_TOAST: {
            const { toastId } = action

            if (toastId !== undefined) {
                addToRemoveQueue(toastId)
            } else {
                state.toasts.forEach((t) => {
                    addToRemoveQueue(t.id)
                })
            }

            return {
                ...state,
                toasts: state.toasts.map((t) =>
                    t.id === toastId || toastId === undefined
                        ? {
                            ...t,
                            open: false,
                        }
                        : t
                ),
            }
        }

        case actionTypes.REMOVE_TOAST:
            if (action.toastId === undefined) {
                return {
                    ...state,
                    toasts: [],
                }
            }
            return {
                ...state,
                toasts: state.toasts.filter((t) => t.id !== action.toastId),
            }
    }
}

const listeners: Array<(state: State) => void> = []

let memoryState: State = { toasts: [] }

function dispatch(action: Action) {
    memoryState = reducer(memoryState, action)
    listeners.forEach((listener) => {
        listener(memoryState)
    })
}

export type Toast = Omit<ToasterToast, "id">

function toast({ ...props }: Toast) {
    const id = genId()

    const update = (p: ToasterToast) =>
        dispatch({
            type: actionTypes.UPDATE_TOAST,
            toast: { ...p, id },
        })
    const dismiss = () => dispatch({ type: actionTypes.DISMISS_TOAST, toastId: id })

    dispatch({
        type: actionTypes.ADD_TOAST,
        toast: {
            ...props,
            id,
            open: true,
            onOpenChange: (open: boolean) => {
                if (!open) { dismiss() }
            },
        },
    })

    return {
        id,
        dismiss,
        update,
    }
}

function useToast() {
    const [state, setState] = useState<State>(memoryState)

    useEffect(() => {
        listeners.push(setState)
        return () => {
            const index = listeners.indexOf(setState)
            if (index > -1) {
                listeners.splice(index, 1)
            }
        }
    }, [state])

    return {
        ...state,
        toast,
        dismiss: (toastId?: string) => dispatch({ type: actionTypes.DISMISS_TOAST, toastId }),
    }
}

export { toast, useToast }

