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
      textArea.style.cssText = 'position:fixed;top:0;left:0;opacity:0';
      document.body.appendChild(textArea);
      textArea.focus();
      textArea.select();
      const ok = document.execCommand('copy');
      document.body.removeChild(textArea);
      return ok;
    } catch (fallbackError) {
      logger.error('Failed to copy text to clipboard', fallbackError);
      return false;
    }
  }
}
