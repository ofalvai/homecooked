'use client'

import { useChat, type Message } from 'ai/react'

import { cn, nanoid } from '@/lib/utils'
import { ChatList } from '@/components/chat-list'
import { ChatPanel } from '@/components/chat-panel'
import { EmptyScreen } from '@/components/empty-screen'
import { ChatScrollAnchor } from '@/components/chat-scroll-anchor'
import { useState } from 'react'
import { Button } from './ui/button'
import { Input } from './ui/input'
import { toast } from 'react-hot-toast'
import { ChatSettings } from './chat-settings'
import { ChatParams } from '@/lib/types'
import { addChat } from '@/app/actions'
import { Chat as ChatType } from '@/lib/types'

export interface ChatProps extends React.ComponentProps<'div'> {
  initialMessages?: Message[]
  id?: string
}

export function Chat({ id, initialMessages, className }: ChatProps) {
  const defaultParams: ChatParams = {
    model: 'claude-instant',
    maxLength: 512,
    temp: 'medium',
    systemPrompt: "You are a helpful assistant."
  }
  const [chatParams, setChatParams] = useState(defaultParams)

  const { messages, append, reload, stop, isLoading, input, setInput } =
    useChat({
      api: '/api/chat',
      initialMessages,
      id,
      body: {
        params: chatParams
      },
      onResponse(response) {
        if (response.status === 401) {
          toast.error(response.statusText)
        }
      },
      onFinish(message) {
        const title = message.content?.substring(0, 100)
        const createdAt = new Date()
        const path = `/chat/${id}`
        const mappedMessages = messages.map(message => ({
          id: nanoid(),
          content: message.content ?? '',
          role: message.role
        }))
        const payload: ChatType = {
          id: id ?? nanoid(),
          title,
          createdAt,
          path,
          messages: [
            ...mappedMessages,
            message
          ]
        }
        addChat(payload)
      }
    })
  return (
    <>
      <div className={cn('flex flex-row', className)}>
        <div className={cn('flex-1 pb-[200px] pt-4 md:pt-10', className)}>
          {messages.length ? (
            <>
              <ChatList messages={messages} model={chatParams.model} />
              <ChatScrollAnchor trackVisibility={isLoading} />
            </>
          ) : (
            <EmptyScreen setInput={setInput} />
          )}
          <ChatPanel
            id={id}
            isLoading={isLoading}
            stop={stop}
            append={append}
            reload={reload}
            messages={messages}
            input={input}
            setInput={setInput}
          />
        </div>
        <div
          className={cn(
            'bg-background border-l w-64 fixed inset-y-0 right-0 top-16',
            className
          )}
        >
          <ChatSettings params={chatParams} setParams={setChatParams} />
        </div>
      </div>
    </>
  )
}
