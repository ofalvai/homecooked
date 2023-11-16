import { ToolEvent } from "@/lib/types"
import {
  Collapsible,
  CollapsibleTrigger,
  CollapsibleContent
} from "@radix-ui/react-collapsible"
import {
  CircleDotDashed,
  AlertCircle,
  ChevronsUpDown,
  CheckCircle2
} from "lucide-react"
import { FormattedOutput } from "./output"

export function ToolEventList(props: { events: ToolEvent[] }) {
  return (
    <div className="space-y-4">
      {props.events.map((event, i) => (
        <div key={i} className="bg-background rounded-lg border p-4 text-sm">
          <ToolEventComponent
            event={event}
            isActive={i == props.events.length - 1}
          />
        </div>
      ))}
    </div>
  )
}

export function ToolEventComponent(props: {
  event: ToolEvent
  isActive: boolean
}) {
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
        <FormattedOutput content={props.event.content} />
      )
    case "intermediate_output":
      return (
        <Collapsible>
          <CollapsibleTrigger className="w-full text-left">
            <ChevronsUpDown className="mr-2 inline-block h-4 w-4" />
            <span className="sr-only">Toggle</span>
            <span className="align-middle">{props.event.label}</span>
          </CollapsibleTrigger>
          <CollapsibleContent className="prose prose-sm dark:prose-invert prose-p:leading-relaxed prose-pre:p-0 mx-4 whitespace-pre-wrap break-words py-2">
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
