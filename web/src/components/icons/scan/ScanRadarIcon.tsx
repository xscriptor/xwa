import { ICON_PATHS } from "@/components/icons/catalog";

type ScanRadarIconProps = {
  className?: string;
};

export default function ScanRadarIcon({ className }: ScanRadarIconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="16"
      height="16"
      aria-hidden="true"
      focusable="false"
      className={className}
    >
      <path fill="currentColor" d={ICON_PATHS.scan.radar} />
    </svg>
  );
}
