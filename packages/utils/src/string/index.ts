export const cap = (str: string): string => {
  return str.charAt(0).toUpperCase() + str.slice(1).toLowerCase();
};

export const slug = (str: string): string => {
  return str
    .toLowerCase()
    .trim()
    .replace(/[^\w\s-]/g, "")
    .replace(/[\s_-]+/g, "-")
    .replace(/^-+|-+$/g, "");
};

export const truncate = (str: string, len: number): string => {
  if (str.length <= len) return str;
  return str.slice(0, len) + "...";
};

export const rmSpecial = (str: string): string => {
  return str.replace(/[^\w\s]/gi, "");
};

export const initials = (name: string): string => {
  return name
    .split(" ")
    .map((n) => n[0])
    .join("")
    .toUpperCase();
};
