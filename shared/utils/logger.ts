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

class Logger {
    private static instance?: Logger;
    private minLevel: number = 1; // Default to 'info'

    private constructor() {
        this.initializeLevel();
    }

    private initializeLevel() {
        if (typeof window === 'undefined') {
            // Server side
            const envLevel = process.env.LOG_LEVEL?.toLowerCase() as LogLevel | undefined;
            if (envLevel && LOG_LEVEL_MAP[envLevel] !== undefined) {
                this.minLevel = LOG_LEVEL_MAP[envLevel];
            } else if (process.env.NODE_ENV !== 'production') {
                this.minLevel = LOG_LEVEL_MAP.debug;
            }
        } else {
            // Client side
            const envLevel = process.env.NEXT_PUBLIC_LOG_LEVEL?.toLowerCase() as LogLevel | undefined;
            if (envLevel && LOG_LEVEL_MAP[envLevel] !== undefined) {
                this.minLevel = LOG_LEVEL_MAP[envLevel];
            } else if (process.env.NODE_ENV !== 'production') {
                this.minLevel = LOG_LEVEL_MAP.debug;
            }
        }
    }

    public static getInstance(): Logger {
        Logger.instance ??= new Logger();
        return Logger.instance;
    }

    private log(level: LogLevel, message: string, ...args: unknown[]): void {
        const currentLevelWeight = LOG_LEVEL_MAP[level];
        if (currentLevelWeight < this.minLevel) {
            return; // Skip logs below the minimum threshold
        }

        const timestamp = new Date().toISOString();
        const isServer = typeof window === 'undefined';
        const isProduction = process.env.NODE_ENV === 'production';

        // PRODUCTION SERVER logs as JSON for structured aggregation (e.g. ELK, Datadog)
        if (isServer && isProduction) {
            const logEntry = {
                timestamp,
                level: level.toUpperCase(),
                message,
                args: args.length > 0 ? args : undefined,
            };

            switch (level) {
                case 'info':
                case 'debug': // Just in case debug is forced in production
                    // eslint-disable-next-line no-console
                    console.log(JSON.stringify(logEntry));
                    break;
                case 'warn':
                    // eslint-disable-next-line no-console
                    console.warn(JSON.stringify(logEntry));
                    break;
                case 'error':
                    // eslint-disable-next-line no-console
                    console.error(JSON.stringify(logEntry));
                    break;
            }
            return;
        }

        // DEV OR CLIENT: Use rich standard console output
        const prefix = `[${timestamp}] [${level.toUpperCase()}]`;

        switch (level) {
            case 'info':
                // eslint-disable-next-line no-console
                console.info(prefix, message, ...args);
                break;
            case 'warn':
                // eslint-disable-next-line no-console
                console.warn(prefix, message, ...args);
                break;
            case 'error':
                // eslint-disable-next-line no-console
                console.error(prefix, message, ...args);
                break;
            case 'debug':
                // eslint-disable-next-line no-console
                console.debug(prefix, message, ...args);
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
