import { ICON_PATHS } from "@/components/icons/catalog";

type AsideSymbolIconProps = {
  symbol: string;
  size?: number;
  className?: string;
};

export default function AsideSymbolIcon({ symbol, size = 14, className }: AsideSymbolIconProps) {
  const path = ICON_PATHS.aside[symbol as keyof typeof ICON_PATHS.aside] || ICON_PATHS.aside.fallback;

  return (
    <svg
      viewBox="0 0 24 24"
      width={size}
      height={size}
      aria-hidden="true"
      focusable="false"
      className={className}
    >
      <path d={path} fill="currentColor" />
    </svg>
  );
}
