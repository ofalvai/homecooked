"use client"
import { Button } from "@/components/ui/button"
import { useAppConfig } from "@/lib/swr-utils"
import { ToolUseEvent, YoutubeSummaryRequest } from "@/lib/types"
import { useState } from "react"
import { EventSourceParserStream } from "eventsource-parser/stream"
import { ParsedEvent } from "eventsource-parser"
import { Input } from "@/components/ui/input"
import { Collapsible, CollapsibleContent } from "@/components/ui/collapsible"
import { CollapsibleTrigger } from "@radix-ui/react-collapsible"
import { AlertCircle, CheckCircle2, ChevronsUpDown, CircleDotDashed } from "lucide-react"

export default function YoutubePage() {
  const [url, setUrl] = useState("")
  const [events, setEvents] = useState<ToolUseEvent[]>([])
  const { data: appConfig } = useAppConfig()

  const onEvent = (event: ToolUseEvent) => {
    setEvents(events => [...events, event])
  }

  const onSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    setEvents([])
    summarize(appConfig!.llmApiBaseUrl, url, onEvent)
    e.preventDefault()
  }

  return (
    <form onSubmit={onSubmit} className="container mx-auto max-w-2xl space-y-4 p-4">
      <Input
        type="text"
        className=""
        value={url}
        onChange={e => setUrl(e.target.value)}
        placeholder="Youtube URL"
      />
      <Button
        type="submit"
        disabled={url.trim() === ""}
      >
        Summarize
      </Button>
      <div className="space-y-4">
        {events.map((event, i) => (
          <div key={i} className="bg-background rounded-lg border p-4 text-sm">
            <ToolEventComponent
              event={event}
              isActive={i == events.length - 1}
            />
          </div>
        ))}
      </div>
    </form>
  )
}

function ToolEventComponent(props: { event: ToolUseEvent; isActive: boolean }) {
  switch (props.event.type) {
    case "working":
      return (
        <div className={props.isActive ? "animate-pulse" : ""}>
          <CircleDotDashed
            className={
              "mr-2 inline-block h-4 w-4 " +
              (props.isActive ? "animate-spin" : "")
            }
          />
          <span className="align-middle">{props.event.label}</span>
        </div>
      )
    case "error":
      return (
        <div>
          <AlertCircle className="mr-2 inline-block h-4 w-4" />
          <span>{props.event.label}</span>
          {props.event.error && <div>{props.event.error}</div>}
        </div>
      )
    case "output":
      return (
        <div className="m-4 whitespace-pre-wrap leading-relaxed">
          {props.event.content}
        </div>
      )
    case "intermediate_output":
      return (
        <Collapsible>
          <CollapsibleTrigger className="w-full text-left">
            <ChevronsUpDown className="mr-2 inline-block h-4 w-4" />
            <span className="sr-only">Toggle</span>
            <span className="align-middle">{props.event.label}</span>
          </CollapsibleTrigger>
          <CollapsibleContent className="mx-4 whitespace-pre-wrap py-2 leading-normal">
            {props.event.content}
          </CollapsibleContent>
        </Collapsible>
      )
    case "finished":
      return (
        <div>
          <CheckCircle2 className="mr-2 inline-block h-4 w-4" />
          Finished
        </div>
      )
  }
}

async function summarize(
  baseUrl: string,
  url: string,
  onEvent: (event: ToolUseEvent) => void
) {
  const req: YoutubeSummaryRequest = {
    url: url,
    prompt: undefined
  }

  const resp = await fetch(`${baseUrl}/v1/tools/youtube`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify(req)
  })

  try {
    const reader = parseStreamResponse(resp).getReader()
  
    while (true) {
      const { value, done } = await reader.read()
      if (done) break
      const toolEvent: ToolUseEvent = JSON.parse(value.data)
      onEvent(toolEvent)
    }
  } catch (e: any) {
    onEvent({
      type: "error",
      label: "Unexpected error",
      error: e.message
    })
  }
}

function parseStreamResponse(resp: Response): ReadableStream<ParsedEvent> {
  if (!resp.ok) {
    throw new Error(`Response is not ok: ${resp.status}`)
  }

  if (!resp.body) {
    throw new Error("Response body is empty")
  }

  return resp.body
    .pipeThrough(new TextDecoderStream())
    .pipeThrough(new EventSourceParserStream())
}
