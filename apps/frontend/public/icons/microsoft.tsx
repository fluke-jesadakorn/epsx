interface MicrosoftIconProps {
  width?: number;
  height?: number;
  className?: string;
}

export default function MicrosoftIcon({ width = 32, height = 32, className }: MicrosoftIconProps) {
  return (
    <svg 
      viewBox="0 0 21 21" 
      width={width} 
      height={height}
      className={className}
    >
      <path fill="#F25022" d="M1 1h9v9H1z" />
      <path fill="#00A4EF" d="M1 11h9v9H1z" />
      <path fill="#7FBA00" d="M11 1h9v9h-9z" />
      <path fill="#FFB900" d="M11 11h9v9h-9z" />
    </svg>
  );
}
