import { type Message } from 'ai'

export interface Chat extends Record<string, any> {
  id: string
  title: string
  createdAt: Date
  path: string
  messages: Message[]
  sharePath?: string
}

export type ServerActionResult<Result> = Promise<
  | Result
  | {
      error: string
    }
>

export type Temp = 'low' | 'medium' | 'high'

export interface ChatParams {
  model: string
  maxLength: number
  temp: Temp
  systemPrompt: string
}
