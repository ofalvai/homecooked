"use client"
import { Button } from "@/components/ui/button"
import { useAppConfig } from "@/lib/swr-utils"
import { ToolEvent, WebSummaryRequest } from "@/lib/types"
import { useState } from "react"
import { Input } from "@/components/ui/input"
import { ToolEventList } from "@/components/tool"
import { readToolEventStream } from "@/lib/tools"

export default function WebPage() {
  const [url, setUrl] = useState("")
  const [events, setEvents] = useState<ToolEvent[]>([])
  const { data: appConfig } = useAppConfig()

  const onEvent = (event: ToolEvent) => {
    setEvents(events => [...events, event])
  }

  const onSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    setEvents([])
    invokeTool(appConfig!.llmApiBaseUrl, url, onEvent)
    e.preventDefault()
  }

  return (
    <form
      onSubmit={onSubmit}
      className="container mx-auto max-w-2xl space-y-4 p-4"
    >
      <Input
        type="text"
        value={url}
        onChange={e => setUrl(e.target.value)}
        placeholder="URL"
      />
      <Button type="submit" disabled={url.trim() === ""}>
        Summarize
      </Button>
      <ToolEventList events={events} />
    </form>
  )
}

async function invokeTool(
  baseUrl: string,
  url: string,
  onEvent: (event: ToolEvent) => void
) {
  const req: WebSummaryRequest = {
    url: url,
    prompt: undefined
  }

  const resp = await fetch(`${baseUrl}/v1/tools/web`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(req)
  })

  readToolEventStream(resp, onEvent)
}
