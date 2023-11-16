import React from 'react'

import { cn } from '@/lib/utils'

export function FooterText({ className, ...props }: React.ComponentProps<'p'>) {
  return (
    <p
      className={cn(
        'text-muted-foreground px-2 text-center text-xs leading-normal',
        className
      )}
      {...props}
    >
      {/* Open source AI chatbot built with{' '}
      <ExternalLink href="https://nextjs.org">Next.js</ExternalLink> and{' '}
      <ExternalLink href="https://vercel.com/storage/kv">
        Vercel KV
      </ExternalLink>
      . */}
    </p>
  )
}
