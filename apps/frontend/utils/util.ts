// General utilities
export const deb = (fn: Function, ms: number) => {
  let timeout: NodeJS.Timeout
  return (...args: any[]) => {
    clearTimeout(timeout)
    timeout = setTimeout(() => fn(...args), ms)
  }
}

export const thr = (fn: Function, ms: number) => {
  let last = 0
  return (...args: any[]) => {
    const now = Date.now()
    if (now - last >= ms) {
      last = now
      fn(...args)
    }
  }
}

export const clone = <T>(obj: T): T => JSON.parse(JSON.stringify(obj))

export const id = (pre = 'id') => `${pre}_${Math.random().toString(36).substr(2, 9)}`

export const mail = (email: string): boolean => /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)

export const phone = (num: string): boolean => /^\+?[\d\s-()]{10,}$/.test(num)

export const trunc = (text: string, len = 50): string => 
  text.length > len ? text.substring(0, len) + '...' : text

export const sleep = (ms: number) => new Promise(r => setTimeout(r, ms))

// Local storage
export const ls = {
  get: <T>(key: string): T | null => {
    try {
      const item = localStorage.getItem(key)
      return item ? JSON.parse(item) : null
    } catch {
      return null
    }
  },
  set: (key: string, val: any) => localStorage.setItem(key, JSON.stringify(val)),
  del: (key: string) => localStorage.removeItem(key)
}

// Array helpers
export const arr = {
  uniq: <T>(arr: T[]): T[] => [...new Set(arr)],
  chunk: <T>(arr: T[], n: number): T[][] => 
    Array.from({ length: Math.ceil(arr.length / n) }, (_, i) => arr.slice(i * n, i * n + n)),
  group: <T>(arr: T[], key: keyof T): Record<string, T[]> => 
    arr.reduce((acc, item) => {
      const k = String(item[key])
      acc[k] = acc[k] || []
      acc[k].push(item)
      return acc
    }, {} as Record<string, T[]>)
}

// Object helpers
export const obj = {
  pick: <T, K extends keyof T>(obj: T, keys: K[]): Pick<T, K> => 
    keys.reduce((acc, key) => ({ ...acc, [key]: obj[key] }), {} as Pick<T, K>),
  omit: <T, K extends keyof T>(obj: T, keys: K[]): Omit<T, K> => {
    const result = { ...obj }
    keys.forEach(key => delete result[key])
    return result
  },
  isEmpty: (obj: any): boolean => !obj || Object.keys(obj).length === 0
}

// URL helpers
export const url = {
  build: (base: string, params: Record<string, any>) => {
    const u = new URL(base)
    Object.entries(params).forEach(([k, v]) => v && u.searchParams.set(k, String(v)))
    return u.toString()
  },
  parse: (url: string) => Object.fromEntries(new URL(url).searchParams.entries())
}

// Backward compatibility
export const debounce = deb
export const throttle = thr
export const deepClone = clone
export const generateId = id
export const isValidEmail = mail
export const isValidPhone = phone
export const truncateText = trunc
export const storage = ls
export const array = arr
export const object = obj
