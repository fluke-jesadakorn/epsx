// Decorator configuration interface
export interface RetryConfig {
  maxAttempts: number;
  initialDelay: number;
  maxDelay: number;
  retryableErrors: (string | RegExp)[];
}

// Retry decorator
export function Retry(config: Partial<RetryConfig>) {
  return function (
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor,
  ) {
    const originalMethod = descriptor.value;

    descriptor.value = async function (...args: any[]) {
      const finalConfig: RetryConfig = {
        maxAttempts: config.maxAttempts || 3,
        initialDelay: config.initialDelay || 1000,
        maxDelay: config.maxDelay || 10000,
        retryableErrors: config.retryableErrors || [],
      };

      let lastError: Error = new Error('Initial error');
      let delay = finalConfig.initialDelay;

      for (let attempt = 0; attempt < finalConfig.maxAttempts; attempt++) {
        try {
          return await originalMethod.apply(this, args);
        } catch (e) {
          const error = e as Error;
          lastError = error;

          // Check if error matches any of the retryable patterns
          const shouldRetry = finalConfig.retryableErrors.some((pattern) => {
            if (pattern instanceof RegExp) {
              return pattern.test(error.message);
            }
            return error.message?.includes(pattern);
          });

          if (!shouldRetry || attempt === finalConfig.maxAttempts - 1) {
            throw lastError;
          }

          // Exponential backoff
          await new Promise((resolve) => setTimeout(resolve, delay));
          delay = Math.min(delay * 2, finalConfig.maxDelay);
        }
      }

      throw lastError;
    };

    return descriptor;
  };
}
