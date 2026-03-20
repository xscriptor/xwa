import { ICON_PATHS } from "@/components/icons/catalog";

type ScanErrorIconProps = {
  className?: string;
};

export default function ScanErrorIcon({ className }: ScanErrorIconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="14"
      height="14"
      aria-hidden="true"
      focusable="false"
      className={className}
    >
      <path fill="currentColor" d={ICON_PATHS.scan.error} />
    </svg>
  );
}
