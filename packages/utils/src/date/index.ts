import { format, parseISO } from 'date-fns';

export const formatDate = (
  date: string | Date,
  formatString: string = 'PPP',
): string => {
  const parsedDate = typeof date === 'string' ? parseISO(date) : date;
  return format(parsedDate, formatString);
};

export const isValidDate = (date: string | Date): boolean => {
  const parsedDate = typeof date === 'string' ? parseISO(date) : date;
  return !isNaN(parsedDate.getTime());
};

export const toISOString = (date: string | Date): string => {
  const parsedDate = typeof date === 'string' ? parseISO(date) : date;
  return parsedDate.toISOString();
};
