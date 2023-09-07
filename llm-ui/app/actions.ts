'use client'

import { type Chat } from '@/lib/types'

export async function getChats() {
  try {
    const item = localStorage.getItem('chats')
    if (item) {
      return JSON.parse(item) as Chat[]
    } else {
      return []
    }
  } catch (error) {
    return []
  }
}

export function addChat(chat: Chat) {
  const item = localStorage.getItem('chats')
  if (item) {
    const chats = JSON.parse(item) as Chat[]
    const existing = chats.find(c => c.id === chat.id)
    if (existing) {
      chats.splice(chats.indexOf(existing), 1, chat)
    } else {
      chats.push(chat)
    }
    localStorage.setItem('chats', JSON.stringify(chats))
  } else {
    localStorage.setItem('chats', JSON.stringify([chat]))
  }
}

export function getChat(id: string): Chat | undefined {
  const item = localStorage.getItem('chats')
  if (item) {
    const chats = JSON.parse(item) as Chat[]
    return chats.find(c => c.id === id)

  } else {
    return undefined
  }
}

export async function removeChat({ id, path }: { id: string; path: string }) {
  const item = localStorage.getItem('chats')
  if (item) {
    const chats = JSON.parse(item) as Chat[]
    const existing = chats.find(c => c.id === id)
    if (existing) {
      chats.splice(chats.indexOf(existing), 1)
      localStorage.setItem('chats', JSON.stringify(chats))
    }
  }
}

export async function clearChats() {
  localStorage.removeItem('chats')
}
