import { useLoading } from "@/context/loading-context";

export function useLoadingFetch() {
  const { startLoading, stopLoading } = useLoading();

  const fetchWithLoading = async <T>(
    asyncFn: () => Promise<T>,
    opts: {
      showLoading?: boolean;
    } = { showLoading: true }
  ): Promise<T> => {
    try {
      if (opts.showLoading) {
        startLoading();
      }
      const result = await asyncFn();
      return result;
    } finally {
      if (opts.showLoading) {
        stopLoading();
      }
    }
  };

  return { fetchWithLoading };
}
