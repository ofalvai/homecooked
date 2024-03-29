import { type Message } from 'ai'
import { type } from 'os'

export interface AppConfig {
  llmApiBaseUrl: string
}

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

export interface Persona {
  id: string
  display_name: string
  prompt: string
  color: string
}

export interface Personas {
  default: string
  personas: Persona[]
}

export interface Spell {
  id: string
  label: string
  prompt: string
}

export interface Spells {
  templates: Spell[]
}

export interface YoutubeSummaryRequest {
  url: string
  prompt: string | undefined
}

export interface WebSummaryRequest {
  url: string
  prompt: string | undefined
}

export interface ReadwiseRequest {
  query: string
}

export interface WorkingEvent {
  type: "working"
  label: string
}

export interface ErrorEvent {
  type: "error"
  label: string
  error: string | undefined
}

export interface IntermediateOutputEvent {
  type: "intermediate_output"
  label: string
  content: string
}

export interface OutputEevent {
  type: "output"
  content: string
}

export interface FinishedEvent {
  type: "finished"
}

export type ToolEvent = WorkingEvent | ErrorEvent | IntermediateOutputEvent | OutputEevent | FinishedEvent
