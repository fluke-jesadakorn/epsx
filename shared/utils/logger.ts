/**
 * EPSX Logger
 * 
 * Standardized logging utility for the EPSX platform.
 * Provides a way to log messages while satisfying strict ESLint rules (no-console).
 */

type LogLevel = 'debug' | 'info' | 'warn' | 'error';

const LOG_LEVEL_MAP: Record<LogLevel, number> = {
    debug: 0,
    info: 1,
    warn: 2,
    error: 3,
};

function isLogLevel(value: string | undefined): value is LogLevel {
    return value !== undefined && Object.prototype.hasOwnProperty.call(LOG_LEVEL_MAP, value);
}

interface LogEntry {
    level: LogLevel;
    timestamp: string;
    message: string;
    args: unknown[];
}

class Logger {
    private static instance?: Logger;
    private minLevel = 1; // Default to 'info'

    private constructor() {
        this.initializeLevel();
    }

    private initializeLevel() {
        const rawLevel = (typeof window === 'undefined'
            ? process.env.LOG_LEVEL
            : process.env.NEXT_PUBLIC_LOG_LEVEL
        )?.toLowerCase();
        const mapped = isLogLevel(rawLevel) ? LOG_LEVEL_MAP[rawLevel] : undefined;
        if (mapped !== undefined) {
            this.minLevel = mapped;
        } else if (process.env.NODE_ENV !== 'production') {
            this.minLevel = LOG_LEVEL_MAP.debug;
        }
    }

    public static getInstance(): Logger {
        Logger.instance ??= new Logger();
        return Logger.instance;
    }

    private log(level: LogLevel, message: string, ...args: unknown[]): void {
        if (LOG_LEVEL_MAP[level] < this.minLevel) { return; }

        const entry: LogEntry = { level, timestamp: new Date().toISOString(), message, args };
        const isProduction = process.env.NODE_ENV === 'production';

        if (typeof window === 'undefined' && isProduction) {
            this.logProduction(entry);
            return;
        }

        this.logDev(entry);
    }

    private logProduction(entry: LogEntry): void {
        const json = JSON.stringify({
            timestamp: entry.timestamp,
            level: entry.level.toUpperCase(),
            message: entry.message,
            args: entry.args.length > 0 ? entry.args : undefined,
        });
        switch (entry.level) {
            case 'warn':
                // eslint-disable-next-line no-console
                console.warn(json);
                break;
            case 'error':
                // eslint-disable-next-line no-console
                console.error(json);
                break;
            default:
                // eslint-disable-next-line no-console
                console.log(json);
        }
    }

    private logDev(entry: LogEntry): void {
        const prefix = `[${entry.timestamp}] [${entry.level.toUpperCase()}]`;
        switch (entry.level) {
            case 'info':
                // eslint-disable-next-line no-console
                console.info(prefix, entry.message, ...entry.args);
                break;
            case 'warn':
                // eslint-disable-next-line no-console
                console.warn(prefix, entry.message, ...entry.args);
                break;
            case 'error':
                // eslint-disable-next-line no-console
                console.error(prefix, entry.message, ...entry.args);
                break;
            case 'debug':
                // eslint-disable-next-line no-console
                console.debug(prefix, entry.message, ...entry.args);
                break;
        }
    }

    public info(message: string, ...args: unknown[]): void {
        this.log('info', message, ...args);
    }

    public warn(message: string, ...args: unknown[]): void {
        this.log('warn', message, ...args);
    }

    public error(message: string, ...args: unknown[]): void {
        this.log('error', message, ...args);
    }

    public debug(message: string, ...args: unknown[]): void {
        this.log('debug', message, ...args);
    }
}

export const logger = Logger.getInstance();
