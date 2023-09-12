import { Metadata } from "next"

import { Toaster } from "react-hot-toast"

import "@/app/globals.css"
import { fontMono, fontSans } from "@/lib/fonts"
import { cn } from "@/lib/utils"
import { TailwindIndicator } from "@/components/tailwind-indicator"
import { Providers } from "@/components/providers"
import { Header } from "@/components/header"

export const metadata: Metadata = {
  title: {
    default: "LLM UI",
    template: `%s - LLM UI`
  },
  description: "LLM UI is a chat UI for various LLM tools",
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "white" },
    { media: "(prefers-color-scheme: dark)", color: "black" }
  ],
  icons: {
    icon: "/favicon.ico",
    shortcut: "/favicon-16x16.png",
    apple: "/apple-touch-icon.png"
  },
  twitter: {
    title: "LLM UI",
    card: "summary",
  },
}

interface RootLayoutProps {
  children: React.ReactNode
}

export default function RootLayout({ children }: RootLayoutProps) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head />
      <body
        className={cn(
          "font-sans antialiased",
          fontSans.variable,
          fontMono.variable
        )}
      >
        <Toaster position="bottom-right" />
        <Providers attribute="class" defaultTheme="system" enableSystem>
          <div className="flex min-h-screen flex-col">
            {/* @ts-ignore */}
            <Header />
            <main className="bg-muted/50 flex flex-1 flex-col">{children}</main>
          </div>
          <TailwindIndicator />
        </Providers>
      </body>
    </html>
  )
}
