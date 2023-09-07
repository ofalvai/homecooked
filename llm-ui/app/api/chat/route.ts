import { OpenAIStream, StreamingTextResponse } from 'ai'
import { OpenAI } from 'openai'

import { ChatCompletionMessageParam } from 'openai/resources/chat/completions'
import { ChatParams, Temp } from '@/lib/types'

const openai = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY!,
  baseURL: 'http://localhost:8080/v1'
})

interface ChatRequest {
  id: string
  messages: [ChatCompletionMessageParam]
  params: ChatParams
}

export async function POST(req: Request) {
  const json: ChatRequest = await req.json()
  const { id, messages, params } = json

  if (messages.length === 1 && messages[0].role !== 'system') {
    messages.unshift({
      role: 'system',
      content: params.systemPrompt
    })
  }

  const res = await openai.chat.completions.create({
    model: params.model,
    messages,
    temperature: mapTemp(params.temp),
    max_tokens: params.maxLength,
    stream: true
  })

  const stream = OpenAIStream(res)

  return new StreamingTextResponse(stream)
}

function mapTemp(temp: Temp) {
  switch (temp) {
    case 'low':
      return 0.5
    case 'medium':
      return 0.7
    case 'high':
      return 0.9
  }
}
