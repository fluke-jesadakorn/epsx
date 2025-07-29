// Security: Password strength validation utility
export interface PasswordStrength {
  score: number; // 0-5 scale
  symbol: string;
  color: string;
  text: string;
  requirements: {
    length: boolean;
    lowercase: boolean;
    uppercase: boolean;
    numbers: boolean;
    symbols: boolean;
  };
}

export function calculatePasswordStrength(password: string): PasswordStrength {
  let score = 0;
  const requirements = {
    length: password.length >= 8,
    lowercase: /[a-z]/.test(password),
    uppercase: /[A-Z]/.test(password),
    numbers: /\d/.test(password),
    symbols: /[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/.test(password),
  };

  // Calculate score based on requirements (matching server-side logic)
  if (requirements.length) score++;
  if (requirements.lowercase) score++;
  if (requirements.uppercase) score++;
  if (requirements.numbers) score++;
  if (requirements.symbols) score++;

  // Check for common patterns (matching server-side logic)
  const commonPatterns = [
    /(.)\1{2,}/, // Repeated characters
    /123|abc|qwe/i, // Sequential patterns
    /password|admin|user/i // Common words
  ];
  
  if (commonPatterns.some(pattern => pattern.test(password))) {
    score = Math.max(0, score - 1);
  }

  // Get strength info based on score (updated for 0-5 scale)
  const getStrengthInfo = (score: number) => {
    switch (score) {
      case 0:
      case 1:
        return {
          symbol: '🔴',
          color: 'text-red-600 dark:text-red-400',
          text: 'Very Weak'
        };
      case 2:
        return {
          symbol: '🟠',
          color: 'text-orange-600 dark:text-orange-400',
          text: 'Weak'
        };
      case 3:
        return {
          symbol: '🟡',
          color: 'text-yellow-600 dark:text-yellow-400',
          text: 'Fair'
        };
      case 4:
        return {
          symbol: '🟢',
          color: 'text-green-600 dark:text-green-400',
          text: 'Good'
        };
      case 5:
        return {
          symbol: '🔵',
          color: 'text-blue-600 dark:text-blue-400',
          text: 'Strong'
        };
      default:
        return {
          symbol: '⚪',
          color: 'text-gray-400',
          text: 'Unknown'
        };
    }
  };

  const strengthInfo = getStrengthInfo(score);

  return {
    score,
    symbol: strengthInfo.symbol,
    color: strengthInfo.color,
    text: strengthInfo.text,
    requirements
  };
}

export function isPasswordWeak(password: string): boolean {
  const strength = calculatePasswordStrength(password);
  return strength.score < 3; // Score 0-2 is considered weak
}