"use client"

import { notFound } from "next/navigation"

import { getChat } from "@/app/actions"
import { Chat } from "@/components/chat"

export interface ChatPageProps {
  params: {
    id: string
  }
}

export default function ChatPage({ params }: ChatPageProps) {
  const chat = getChat(params.id)

  if (!chat) {
    notFound()
  }

  return <Chat id={chat.id} initialMessages={chat.messages} />
}
