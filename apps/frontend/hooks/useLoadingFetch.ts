import { useLoading } from "@/context/loading-context";

export function useLoadingFetch() {
  const { startLoading, stopLoading } = useLoading();

  const fetchWithLoading = async <T>(
    asyncFn: () => Promise<T>,
    options: {
      showLoading?: boolean;
    } = { showLoading: true }
  ): Promise<T> => {
    try {
      if (options.showLoading) {
        startLoading();
      }
      const result = await asyncFn();
      return result;
    } finally {
      if (options.showLoading) {
        stopLoading();
      }
    }
  };

  return { fetchWithLoading };
}
