import { logger } from '@/lib/utils/logging';

/**
 * Copy text to clipboard with fallback for older browsers
 */
export async function copyToClipboard(text: string): Promise<boolean> {
  if (typeof window === 'undefined') { return false; }

  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (_error) {
    try {
      const textArea = document.createElement('textarea');
      textArea.value = text;
      document.body.appendChild(textArea);
      textArea.select();
      document.execCommand('copy');
      document.body.removeChild(textArea);
      return true;
    } catch (fallbackError) {
      logger.error('Failed to copy text to clipboard', fallbackError);
      return false;
    }
  }
}
