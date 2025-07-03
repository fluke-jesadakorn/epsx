'use client';

import { useEffect, useState } from 'react';
import { PAYMENT_DURATION } from '@/app/constants/packages';

interface ExpirationCountdownProps {
  expirationDate: string | Date;
  className?: string;
}

export function ExpirationCountdown({ expirationDate, className = '' }: ExpirationCountdownProps) {
  const [timeLeft, setTimeLeft] = useState<{
    days: number;
    hours: number;
    minutes: number;
  }>({ days: 0, hours: 0, minutes: 0 });

  useEffect(() => {
    const calculateTimeLeft = () => {
      const expiration = new Date(expirationDate).getTime();
      const now = new Date().getTime();
      const difference = expiration - now;

      if (difference > 0) {
        const days = Math.floor(difference / (1000 * 60 * 60 * 24));
        const hours = Math.floor((difference % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
        const minutes = Math.floor((difference % (1000 * 60 * 60)) / (1000 * 60));

        setTimeLeft({ days, hours, minutes });
      } else {
        setTimeLeft({ days: 0, hours: 0, minutes: 0 });
      }
    };

    calculateTimeLeft();
    const timer = setInterval(calculateTimeLeft, 60000); // Update every minute

    return () => clearInterval(timer);
  }, [expirationDate]);

  const getStatusColor = () => {
    if (timeLeft.days > 7) {
      return 'text-green-600 dark:text-green-400';
    } else if (timeLeft.days > 2) {
      return 'text-yellow-600 dark:text-yellow-400';
    }
    return 'text-red-600 dark:text-red-400';
  };

  if (!timeLeft.days && !timeLeft.hours && !timeLeft.minutes) {
    return (
      <div className={`${className} text-red-600 dark:text-red-400 font-medium`}>
        Expired
      </div>
    );
  }

  return (
    <div className={`${className} font-medium ${getStatusColor()}`}>
      {timeLeft.days > 0 && `${timeLeft.days}d `}
      {timeLeft.hours > 0 && `${timeLeft.hours}h `}
      {timeLeft.minutes > 0 && `${timeLeft.minutes}m `}
      remaining
    </div>
  );
}
