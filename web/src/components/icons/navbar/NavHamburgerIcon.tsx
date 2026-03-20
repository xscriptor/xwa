import { ICON_PATHS } from "@/components/icons/catalog";

type NavHamburgerIconProps = {
  open: boolean;
  className?: string;
};

export default function NavHamburgerIcon({ open, className }: NavHamburgerIconProps) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="18"
      height="18"
      aria-hidden="true"
      focusable="false"
      className={className}
    >
      <path d={open ? ICON_PATHS.navbar.close : ICON_PATHS.navbar.menu} fill="currentColor" />
    </svg>
  );
}
