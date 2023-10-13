import { ParsedEvent } from "eventsource-parser"
import { EventSourceParserStream } from "eventsource-parser/stream"
import { ToolEvent } from "./types"

export async function readToolEventStream(
  resp: Response,
  onEvent: (event: ToolEvent) => void
) {
  try {
    const reader = parseStreamResponse(resp).getReader()

    while (true) {
      const { value, done } = await reader.read()
      if (done) break
      const toolEvent: ToolEvent = JSON.parse(value.data)
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
