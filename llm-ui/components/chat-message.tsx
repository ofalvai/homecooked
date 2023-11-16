// Inspired by Chatbot-UI and modified to fit the needs of this project
// @see https://github.com/mckaywrigley/chatbot-ui/blob/main/components/Chat/ChatMessage.tsx

import { Message } from 'ai'
import { cn } from '@/lib/utils'
import { IconAnthropic, IconOpenAI, IconUser } from '@/components/ui/icons'
import { ChatMessageActions } from '@/components/chat-message-actions'
import { FormattedOutput } from './output'

export interface ChatMessageProps {
  message: Message
  model: string
}

export function ChatMessage({ message, ...props }: ChatMessageProps) {
  return (
    <div
      className={cn('group relative mb-4 flex items-start md:-ml-12')}
      {...props}
    >
      <div
        className={cn(
          'bg-background flex h-8 w-8 shrink-0 select-none items-center justify-center rounded-md border shadow',
        )}
      >
        {message.role === 'user' ? <IconUser /> : iconForModel(props.model)}
      </div>
      <div className="ml-4 flex-1 space-y-2 overflow-hidden px-1">
        <FormattedOutput content={message.content} />
        <ChatMessageActions message={message} />
      </div>
    </div>
  )
}

function iconForModel(model: string): React.JSX.Element {
  switch (model) {
    case 'claude-instant':
    case 'claude-2':
      return <IconAnthropic />
    case 'gpt-3.5-turbo':
    case 'gpt-3.5-turbo-16k':
    case 'gpt-4':
      return <IconOpenAI />
    default:
      return <IconUser />
  }
}
