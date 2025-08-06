/**
 * Input sanitization utilities that consolidate sanitization patterns
 * Replaces scattered sanitization implementations across the codebase
 */

// Sanitization options
export interface SanitizationOptions {
  /** Remove HTML tags */
  stripHtml?: boolean;
  /** Allow specific HTML tags */
  allowedTags?: string[];
  /** Remove JavaScript */
  stripScripts?: boolean;
  /** Normalize whitespace */
  normalizeWhitespace?: boolean;
  /** Maximum length */
  maxLength?: number;
  /** Convert to lowercase */
  toLowerCase?: boolean;
  /** Remove special characters */
  removeSpecialChars?: boolean;
  /** Keep only alphanumeric characters */
  alphanumericOnly?: boolean;
  /** Custom character whitelist */
  allowedChars?: string;
}

export interface EmailSanitizationOptions {
  /** Remove dots before @ symbol */
  removeDots?: boolean;
  /** Remove plus addressing (everything after +) */
  removePlusAddressing?: boolean;
  /** Convert to lowercase */
  toLowerCase?: boolean;
}

export interface PhoneSanitizationOptions {
  /** Remove all non-digit characters except + */
  digitsOnly?: boolean;
  /** Add country code if missing */
  defaultCountryCode?: string;
  /** Format output style */
  format?: 'e164' | 'national' | 'international' | 'raw';
}

/**
 * Comprehensive sanitization utility class
 * Consolidates input sanitization from multiple implementations
 */
export class InputSanitizer {
  /**
   * General text sanitization (consolidates basic sanitization patterns)
   */
  static sanitizeText(
    input: string | null | undefined,
    options: SanitizationOptions = {}
  ): string {
    if (!input || typeof input !== 'string') return '';

    const {
      stripHtml = false,
      allowedTags = [],
      stripScripts = true,
      normalizeWhitespace = true,
      maxLength,
      toLowerCase = false,
      removeSpecialChars = false,
      alphanumericOnly = false,
      allowedChars,
    } = options;

    let sanitized = input;

    // Strip HTML tags (with allowlist support)
    if (stripHtml) {
      if (allowedTags.length > 0) {
        // Complex HTML sanitization with allowed tags
        sanitized = this.sanitizeHtml(sanitized, allowedTags);
      } else {
        // Simple HTML tag removal
        sanitized = sanitized.replace(/<[^>]*>/g, '');
      }
    }

    // Strip JavaScript
    if (stripScripts) {
      sanitized = sanitized
        .replace(/<script[^>]*>.*?<\/script>/gi, '')
        .replace(/javascript:/gi, '')
        .replace(/on\w+\s*=\s*["'][^"']*["']/gi, '')
        .replace(/on\w+\s*=\s*[^>\s]+/gi, '');
    }

    // Normalize whitespace
    if (normalizeWhitespace) {
      sanitized = sanitized
        .replace(/\s+/g, ' ') // Multiple spaces to single space
        .replace(/[\r\n\t]/g, ' ') // Line breaks and tabs to spaces
        .trim();
    }

    // Character filtering
    if (alphanumericOnly) {
      sanitized = sanitized.replace(/[^a-zA-Z0-9\s]/g, '');
    } else if (removeSpecialChars) {
      sanitized = sanitized.replace(/[^\w\s-_.]/g, '');
    } else if (allowedChars) {
      const regex = new RegExp(`[^${allowedChars.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}]`, 'g');
      sanitized = sanitized.replace(regex, '');
    }

    // Case conversion
    if (toLowerCase) {
      sanitized = sanitized.toLowerCase();
    }

    // Length limiting
    if (maxLength && sanitized.length > maxLength) {
      sanitized = sanitized.substring(0, maxLength);
    }

    return sanitized.trim();
  }

  /**
   * HTML sanitization with tag allowlist
   * Safer than complete HTML stripping for rich text content
   */
  static sanitizeHtml(input: string, allowedTags: string[] = []): string {
    if (!input || typeof input !== 'string') return '';

    const allowedTagsSet = new Set(allowedTags.map(tag => tag.toLowerCase()));
    
    // Basic HTML sanitization - in production, use a proper library like DOMPurify
    return input.replace(/<(\/?)([\w-]+)([^>]*)>/gi, (_match, slash, tagName, attributes) => {
      const tag = (tagName as string).toLowerCase();
      
      if (!allowedTagsSet.has(tag)) {
        return ''; // Remove disallowed tags
      }

      // For allowed tags, sanitize attributes
      const cleanAttributes = (attributes as string)
        .replace(/on\w+\s*=\s*["'][^"']*["']/gi, '') // Remove event handlers
        .replace(/javascript:/gi, '') // Remove javascript: URLs
        .replace(/data:/gi, '') // Remove data: URLs
        .replace(/vbscript:/gi, ''); // Remove vbscript: URLs

      return `<${slash}${tagName}${cleanAttributes}>`;
    });
  }

  /**
   * Email sanitization (consolidates email sanitization patterns)
   */
  static sanitizeEmail(
    email: string | null | undefined,
    options: EmailSanitizationOptions = {}
  ): string {
    if (!email || typeof email !== 'string') return '';

    const {
      removeDots = false,
      removePlusAddressing = false,
      toLowerCase = true,
    } = options;

    let sanitized = email.trim();

    // Convert to lowercase
    if (toLowerCase) {
      sanitized = sanitized.toLowerCase();
    }

    // Split into local and domain parts
    const atIndex = sanitized.lastIndexOf('@');
    if (atIndex === -1) return sanitized; // Invalid email format

    let localPart = sanitized.substring(0, atIndex);
    const domainPart = sanitized.substring(atIndex + 1);

    // Remove dots from local part (Gmail-style)
    if (removeDots) {
      localPart = localPart.replace(/\./g, '');
    }

    // Remove plus addressing (everything after +)
    if (removePlusAddressing) {
      const plusIndex = localPart.indexOf('+');
      if (plusIndex !== -1) {
        localPart = localPart.substring(0, plusIndex);
      }
    }

    return `${localPart}@${domainPart}`;
  }

  /**
   * Phone number sanitization (consolidates phone sanitization patterns)
   */
  static sanitizePhone(
    phone: string | null | undefined,
    options: PhoneSanitizationOptions = {}
  ): string {
    if (!phone || typeof phone !== 'string') return '';

    const {
      digitsOnly = true,
      defaultCountryCode,
      format = 'e164',
    } = options;

    let sanitized = phone.trim();

    if (digitsOnly) {
      // Keep only digits and + symbol
      sanitized = sanitized.replace(/[^\d+]/g, '');
    }

    // Add default country code if missing
    if (defaultCountryCode && !sanitized.startsWith('+')) {
      sanitized = `+${defaultCountryCode}${sanitized}`;
    }

    // Format based on requested format
    switch (format) {
      case 'e164':
        // E.164 format: +1234567890
        if (!sanitized.startsWith('+')) {
          sanitized = '+' + sanitized;
        }
        break;
      
      case 'national':
        // National format: remove country code
        if (sanitized.startsWith('+')) {
          // Simple heuristic: remove +1 for US/Canada, +44 for UK, etc.
          sanitized = sanitized.replace(/^\+\d{1,3}/, '');
        }
        break;
      
      case 'international':
        // International format: ensure + prefix
        if (!sanitized.startsWith('+')) {
          sanitized = '+' + sanitized;
        }
        break;
      
      case 'raw':
        // Raw format: no formatting
        break;
    }

    return sanitized;
  }

  /**
   * URL sanitization (removes dangerous protocols and validates structure)
   */
  static sanitizeUrl(url: string | null | undefined): string {
    if (!url || typeof url !== 'string') return '';

    let sanitized = url.trim();

    // Remove dangerous protocols
    const dangerousProtocols = [
      'javascript:',
      'data:',
      'vbscript:',
      'file:',
      'ftp:',
    ];

    for (const protocol of dangerousProtocols) {
      if (sanitized.toLowerCase().startsWith(protocol)) {
        return '';
      }
    }

    // Ensure HTTP/HTTPS protocol if missing
    if (!/^https?:\/\//i.test(sanitized)) {
      // Don't add protocol to relative URLs or anchors
      if (!sanitized.startsWith('/') && !sanitized.startsWith('#')) {
        sanitized = 'https://' + sanitized;
      }
    }

    return sanitized;
  }

  /**
   * Filename sanitization (safe for file system)
   */
  static sanitizeFilename(filename: string | null | undefined): string {
    if (!filename || typeof filename !== 'string') return '';

    let sanitized = filename.trim();

    // Remove path separators and dangerous characters
    sanitized = sanitized
      .replace(/[/\\:*?"<>|]/g, '') // Forbidden characters
      .replace(/\.\./g, '') // Directory traversal
      .replace(/^\.+/, '') // Leading dots
      .replace(/\s+/g, '_') // Spaces to underscores
      .replace(/_+/g, '_'); // Multiple underscores to single

    // Limit length
    if (sanitized.length > 255) {
      const extension = sanitized.split('.').pop() || '';
      const nameWithoutExt = sanitized.substring(0, sanitized.lastIndexOf('.'));
      const maxNameLength = 255 - extension.length - 1;
      sanitized = nameWithoutExt.substring(0, maxNameLength) + '.' + extension;
    }

    return sanitized || 'untitled';
  }

  /**
   * Credit card number sanitization
   */
  static sanitizeCreditCard(cardNumber: string | null | undefined): string {
    if (!cardNumber || typeof cardNumber !== 'string') return '';

    // Remove all non-digit characters
    return cardNumber.replace(/\D/g, '');
  }

  /**
   * Slug sanitization (URL-friendly strings)
   */
  static sanitizeSlug(text: string | null | undefined): string {
    if (!text || typeof text !== 'string') return '';

    return text
      .toLowerCase()
      .trim()
      .replace(/[^\w\s-]/g, '') // Remove special characters
      .replace(/[\s_-]+/g, '-') // Replace spaces and underscores with hyphens
      .replace(/^-+|-+$/g, '') // Remove leading/trailing hyphens
      .substring(0, 100); // Limit length
  }

  /**
   * JSON sanitization (safe parsing with error handling)
   */
  static sanitizeJson<T = unknown>(
    jsonString: string | null | undefined,
    defaultValue: T | null = null
  ): T | null {
    if (!jsonString || typeof jsonString !== 'string') return defaultValue;

    try {
      return JSON.parse(jsonString) as T;
    } catch {
      return defaultValue;
    }
  }

  /**
   * SQL injection prevention (basic sanitization)
   * Note: Use parameterized queries instead of sanitization for SQL security
   */
  static sanitizeSql(input: string | null | undefined): string {
    if (!input || typeof input !== 'string') return '';

    return input
      .replace(/'/g, "''") // Escape single quotes
      .replace(/"/g, '""') // Escape double quotes
      .replace(/;/g, '') // Remove semicolons
      .replace(/--/g, '') // Remove SQL comments
      .replace(/\/\*/g, '') // Remove comment start
      .replace(/\*\//g, ''); // Remove comment end
  }

  /**
   * XSS prevention for display in HTML context
   */
  static sanitizeForDisplay(input: string | null | undefined): string {
    if (!input || typeof input !== 'string') return '';

    return input
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#x27;')
      .replace(/\//g, '&#x2F;');
  }

  /**
   * Batch sanitization for arrays
   */
  static sanitizeBatch<T>(
    items: (string | null | undefined)[],
    sanitizer: (item: string | null | undefined) => T
  ): T[] {
    return items.map(sanitizer);
  }

  /**
   * Deep object sanitization
   */
  static sanitizeObject(
    obj: Record<string, unknown>,
    sanitizers: Record<string, (value: unknown) => unknown>
  ): Record<string, unknown> {
    const result: Record<string, unknown> = {};

    for (const [key, value] of Object.entries(obj)) {
      if (sanitizers[key]) {
        result[key] = sanitizers[key](value);
      } else if (typeof value === 'string') {
        result[key] = this.sanitizeText(value);
      } else {
        result[key] = value;
      }
    }

    return result;
  }
}

// Convenience exports for common sanitization operations
export const sanitizeText = InputSanitizer.sanitizeText;
export const sanitizeHtml = InputSanitizer.sanitizeHtml;
export const sanitizeEmail = InputSanitizer.sanitizeEmail;
export const sanitizePhone = InputSanitizer.sanitizePhone;
export const sanitizeUrl = InputSanitizer.sanitizeUrl;
export const sanitizeFilename = InputSanitizer.sanitizeFilename;
export const sanitizeCreditCard = InputSanitizer.sanitizeCreditCard;
export const sanitizeSlug = InputSanitizer.sanitizeSlug;
export const sanitizeJson = InputSanitizer.sanitizeJson;
export const sanitizeForDisplay = InputSanitizer.sanitizeForDisplay;

// Preset sanitization configurations
export const SanitizationPresets = {
  /** Basic text sanitization for user input */
  userInput: {
    stripHtml: true,
    stripScripts: true,
    normalizeWhitespace: true,
    maxLength: 1000,
  },

  /** Rich text with allowed HTML tags */
  richText: {
    stripHtml: true,
    allowedTags: ['p', 'br', 'strong', 'em', 'u', 'ol', 'ul', 'li', 'a'],
    stripScripts: true,
    normalizeWhitespace: true,
  },

  /** Search query sanitization */
  searchQuery: {
    stripHtml: true,
    stripScripts: true,
    normalizeWhitespace: true,
    maxLength: 200,
    removeSpecialChars: true,
  },

  /** Username sanitization */
  username: {
    alphanumericOnly: true,
    toLowerCase: true,
    maxLength: 30,
  },

  /** Comment/review sanitization */
  comment: {
    stripHtml: true,
    stripScripts: true,
    normalizeWhitespace: true,
    maxLength: 2000,
  },
} as const;