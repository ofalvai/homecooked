"use client"
import { Button } from "@/components/ui/button"
import { useAppConfig } from "@/lib/swr-utils"
import { ReadwiseRequest, ToolEvent } from "@/lib/types"
import { useState } from "react"
import { ToolEventList } from "@/components/tool"
import { readToolEventStream } from "@/lib/tools"
import { Textarea } from "@/components/ui/textarea"

export default function ReadwisePage() {
  const [query, setQuery] = useState("")
  const [events, setEvents] = useState<ToolEvent[]>([])
  const { data: appConfig } = useAppConfig()

  const onEvent = (event: ToolEvent) => {
    setEvents(events => [...events, event])
  }

  const onSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    setEvents([])
    invokeTool(appConfig!.llmApiBaseUrl, query, onEvent)
    e.preventDefault()
  }

  return (
    <form
      onSubmit={onSubmit}
      className="container mx-auto max-w-2xl space-y-4 p-4"
    >
      <p className="text-sm">Article selection criteria:</p>
      <Textarea
        value={query}
        onChange={e => setQuery(e.target.value)}
        placeholder="Something that makes me smarter"
      />
      <Button type="submit" disabled={query.trim() === ""}>
        Create curated list
      </Button>
      <ToolEventList events={events} />
    </form>
  )
}

async function invokeTool(
  baseUrl: string,
  query: string,
  onEvent: (event: ToolEvent) => void
) {
  const req: ReadwiseRequest = { query }

  const resp = await fetch(`${baseUrl}/v1/tools/readwise`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(req)
  })

  readToolEventStream(resp, onEvent)
}
