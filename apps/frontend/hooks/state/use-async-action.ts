import { useCallback, useState, useRef } from 'react';
import { useToasts } from '@/context/ui-context';

interface AsyncActionOptions {
  successMessage?: string;
  errorMessage?: string;
  loadingKey?: string;
  optimistic?: boolean;
  onSuccess?: (result: unknown) => void;
  onError?: (error: Error) => void;
  transform?: (result: unknown) => unknown;
}

interface AsyncActionState {
  loading: boolean;
  error: string | null;
  data: unknown;
}

export function useAsyncAction<T extends (...args: unknown[]) => Promise<unknown>>(
  asyncFn: T,
  options: AsyncActionOptions = {}
) {
  const [state, setState] = useState<AsyncActionState>({
    loading: false,
    error: null,
    data: null
  });
  
  const { success, error: showError } = useToasts();
  const abortControllerRef = useRef<AbortController | null>(null);

  const execute = useCallback(async (...args: Parameters<T>) => {
    // Cancel previous request if still running
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    // Create new abort controller
    abortControllerRef.current = new AbortController();
    const signal = abortControllerRef.current.signal;

    setState(prev => ({ ...prev, loading: true, error: null }));

    try {
      // Add signal to args if function accepts it
      const finalArgs = asyncFn.length > args.length ? [...args, { signal }] : args;
      const result = await asyncFn(...finalArgs);
      
      if (signal.aborted) {
        return;
      }

      const transformedResult = options.transform ? options.transform(result) : result;
      
      setState(prev => ({ 
        ...prev, 
        loading: false, 
        data: transformedResult 
      }));

      if (options.successMessage) {
        success(options.successMessage);
      }

      if (options.onSuccess) {
        options.onSuccess(transformedResult);
      }

      return transformedResult;
    } catch (err) {
      if (signal.aborted) {
        return;
      }

      const error = err instanceof Error ? err : new Error('An error occurred');
      
      setState(prev => ({ 
        ...prev, 
        loading: false, 
        error: error.message 
      }));

      if (options.errorMessage) {
        showError(options.errorMessage, error.message);
      }

      if (options.onError) {
        options.onError(error);
      }

      throw error;
    }
  }, [asyncFn, options, success, showError]);

  const reset = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    setState({ loading: false, error: null, data: null });
  }, []);

  const cancel = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
  }, []);

  return {
    ...state,
    execute,
    reset,
    cancel
  };
}