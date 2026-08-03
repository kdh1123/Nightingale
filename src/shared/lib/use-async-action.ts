import { useCallback, useState } from "react";

export function useAsyncAction<T = undefined>(fallbackMessage: string) {
  const [error, setError] = useState<string | null>(null);
  const [activeTask, setActiveTask] = useState<T | null>(null);

  const run = useCallback(
    async <R>(task: T, action: () => Promise<R>, onSuccess?: (result: R) => unknown) => {
      try {
        setError(null);
        setActiveTask(task);
        const result = await action();
        await onSuccess?.(result);
        return result;
      } catch (cause) {
        setError(cause instanceof Error ? cause.message : fallbackMessage);
        return undefined;
      } finally {
        setActiveTask(null);
      }
    },
    [fallbackMessage],
  );

  return { activeTask, error, run };
}
