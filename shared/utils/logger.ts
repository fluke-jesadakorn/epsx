/**
 * EPSX Logger
 * 
 * Standardized logging utility for the EPSX platform.
 * Provides a way to log messages while satisfying strict ESLint rules (no-console).
 */

type LogLevel = 'info' | 'warn' | 'error' | 'debug';

class Logger {
    private static instance?: Logger;

    private constructor() { }

    public static getInstance(): Logger {
        Logger.instance ??= new Logger();
        return Logger.instance;
    }

    private log(level: LogLevel, message: string, ...args: unknown[]): void {
        // In production, we might want to send these to a logging service
        const timestamp = new Date().toISOString();
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
                if (process.env.NODE_ENV !== 'production') {
                    // eslint-disable-next-line no-console
                    console.debug(prefix, message, ...args);
                }
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
