import { ICON_PATHS } from "@/components/icons/catalog";

type NavStatusIconProps = {
  className?: string;
};

export default function NavStatusIcon({ className }: NavStatusIconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="18"
      height="18"
      aria-hidden="true"
      focusable="false"
      className={className}
    >
      <path fill="currentColor" d={ICON_PATHS.navbar.status} />
    </svg>
  );
}
