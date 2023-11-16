"use client"

import * as React from "react"
import Link from "next/link"

import { clearChats } from "@/app/actions"
import { Sidebar } from "@/components/sidebar"
import { SidebarList } from "@/components/sidebar-list"
import { SidebarFooter } from "@/components/sidebar-footer"
import { ThemeToggle } from "@/components/theme-toggle"
import { ClearHistory } from "@/components/clear-history"
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuIndicator,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
  navigationMenuTriggerStyle
} from "./ui/navigation-menu"
import { usePathname } from "next/navigation"
import { cn } from "@/lib/utils"

const tools: { title: string; href: string; description: string }[] = [
  {
    title: "Youtube",
    href: "/tools/youtube",
    description:
      "Summarize Youtube videos via transcripts."
  },
  {
    title: "Webpage",
    href: "/tools/web",
    description: "Summarize or ask questions about webpages."
  },
  {
    title: "Readwise",
    href: "/tools/readwise",
    description:
      "Create a curated reading list from your Readwise inbox by telling the LLM what you want to read about."
  },
]

export async function Header() {
  const pathname = usePathname()
  return (
    <header className="bg-background sticky top-0 z-50 flex h-16 w-full shrink-0 items-center justify-between border-b px-4">
      <div className="flex items-center">
        <Sidebar>
          <React.Suspense fallback={<div className="flex-1 overflow-auto" />}>
            {/* @ts-ignore */}
            <SidebarList />
          </React.Suspense>
          <SidebarFooter>
            <ThemeToggle />
            <ClearHistory clearChats={clearChats} />
          </SidebarFooter>
        </Sidebar>
        <NavigationMenu orientation="horizontal">
          <NavigationMenuList>
          <NavigationMenuIndicator />
            <NavigationMenuItem>
              <Link href="/" legacyBehavior passHref>
                <NavigationMenuLink
                  className={navigationMenuTriggerStyle()}
                  active={pathname == "/"}
                >
                  Chat
                </NavigationMenuLink>
              </Link>
            </NavigationMenuItem>
            <NavigationMenuItem>
              <NavigationMenuTrigger>Tools</NavigationMenuTrigger>
              <NavigationMenuContent>
                <ul className="grid w-[400px] gap-3 p-4 md:w-[500px] md:grid-cols-2 lg:w-[600px] ">
                  {tools.map(component => (
                    <ListItem
                      key={component.title}
                      title={component.title}
                      href={component.href}
                      
                    >
                      {component.description}
                    </ListItem>
                  ))}
                </ul>
              </NavigationMenuContent>
            </NavigationMenuItem>
          </NavigationMenuList>
        </NavigationMenu>
      </div>
    </header>
  )
}

const ListItem = React.forwardRef<
  React.ElementRef<"a">,
  React.ComponentPropsWithoutRef<"a">
>(({ className, title, children, ...props }, ref) => {
  return (
    <li>
      <NavigationMenuLink asChild>
        <a
          ref={ref}
          className={cn(
            "hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors",
            className
          )}
          {...props}
        >
          <div className="text-sm font-medium leading-none">{title}</div>
          <p className="text-muted-foreground line-clamp-2 text-sm leading-snug">
            {children}
          </p>
        </a>
      </NavigationMenuLink>
    </li>
  )
})
ListItem.displayName = "ListItem"
