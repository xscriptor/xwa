import type { Metadata } from "next";
import "./globals.css";
import ThemeWrapper from "@/components/ThemeWrapper";

export const metadata: Metadata = {
  title: "xwa - Web Analysis Dashboard",
  description: "Advanced SEO, Sitemap, Security & Accessibility Analysis Tool.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" data-theme="dark" suppressHydrationWarning>
      <body>
        <ThemeWrapper>
          {children}
        </ThemeWrapper>
      </body>
    </html>
  );
}
