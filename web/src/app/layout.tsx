import type { Metadata } from "next";
import { Anonymous_Pro, Fira_Code, Inter, Space_Grotesk } from "next/font/google";
import "./globals.css";
import ThemeWrapper from "@/components/ThemeWrapper";

const headlineFont = Space_Grotesk({
  subsets: ["latin"],
  weight: ["300", "400", "500", "600", "700"],
  variable: "--font-headline",
});

const bodyFont = Inter({
  subsets: ["latin"],
  weight: ["300", "400", "500", "600", "700"],
  variable: "--font-body",
});

const monoFont = Fira_Code({
  subsets: ["latin"],
  weight: ["300", "400", "500"],
  variable: "--font-mono",
});

const anonymousProFont = Anonymous_Pro({
  subsets: ["latin"],
  weight: ["400", "700"],
  style: ["normal", "italic"],
  variable: "--font-anonymous-pro",
});

export const metadata: Metadata = {
  title: "Only XWA",
  description: "Advanced SEO, Sitemap, Security & Accessibility Analysis Tool.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" data-theme="dark" suppressHydrationWarning>
      <body className={`${headlineFont.variable} ${bodyFont.variable} ${monoFont.variable} ${anonymousProFont.variable}`}>
        <ThemeWrapper>
          {children}
        </ThemeWrapper>
      </body>
    </html>
  );
}
