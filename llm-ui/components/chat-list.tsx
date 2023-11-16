import { type Message } from "ai"

import { Separator } from "@/components/ui/separator"
import { ChatMessage } from "@/components/chat-message"

export interface ChatList {
  messages: Message[]
  setMessages: (messages: Message[]) => void
  model: string
}

export function ChatList({ messages, setMessages, model }: ChatList) {
  if (!messages.length) {
    return null
  }

  const onMessageEdit = (index: number, newContent: string) => {
    messages[index].content = newContent
    setMessages(messages)
  }

  return (
    <div className="relative mx-auto max-w-2xl px-4">
      {messages.map((message, index) => (
        <div key={index}>
          <ChatMessage
            message={message}
            onMessageEdit={newContent => onMessageEdit(index, newContent)}
            model={model}
          />
          {index < messages.length - 1 && (
            <Separator className="my-4 md:my-8" />
          )}
        </div>
      ))}
    </div>
  )
}
