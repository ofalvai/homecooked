import { UseChatHelpers } from "ai/react"
import * as React from "react"
import Textarea from "react-textarea-autosize"

import { Button } from "@/components/ui/button"
import { IconArrowElbow } from "@/components/ui/icons"
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger
} from "@/components/ui/tooltip"
import { useEnterSubmit } from "@/lib/hooks/use-enter-submit"
import { Spellbook } from "./spellbook"
import { useAppConfig } from "@/lib/swr-utils"
import useSWRImmutable from "swr/immutable"
import { Spells } from "@/lib/types"
import { parse } from "yaml"

export interface PromptProps
  extends Pick<UseChatHelpers, "input" | "setInput"> {
  onSubmit: (value: string) => Promise<void>
  isLoading: boolean
}

async function spellFetcher(url: string): Promise<Spells> {
  return fetch(url).then(async res => {
    const text = await res.text()
    return parse(text) as Spells
  })
}

export function PromptForm({
  onSubmit,
  input,
  setInput,
  isLoading
}: PromptProps) {
  const { formRef, onKeyDown } = useEnterSubmit()
  const inputRef = React.useRef<HTMLTextAreaElement>(null)

  React.useEffect(() => {
    if (inputRef.current) {
      inputRef.current.focus()
    }
  }, [])

  const { data: appConfig } = useAppConfig()
  const { data: spells } = useSWRImmutable(
    () => {
      // Throw error on purpose to let SWR know of the dependency
      return appConfig!.llmApiBaseUrl + "/config/templates.yml"
    },
    spellFetcher
  )

  return (
    <form
      onSubmit={async e => {
        e.preventDefault()
        if (!input?.trim()) {
          return
        }
        setInput("")
        await onSubmit(input)
      }}
      ref={formRef}
    >
      <div className="bg-background relative flex max-h-60 w-full grow flex-col overflow-hidden px-8 sm:rounded-md sm:border sm:px-12">
        <Spellbook
          spells={spells?.templates ?? []}
          onSelect={spell => {
            setInput(spell.prompt)
            inputRef.current?.focus()
          }}
          onInsert={spell => {
            if (inputRef.current) {
              const [start, end] = [
                inputRef.current.selectionStart,
                inputRef.current.selectionEnd
              ]
              inputRef.current.setRangeText(spell.prompt, start, end, "select")
              inputRef.current.focus()
            }
          }}
        />
        <Textarea
          ref={inputRef}
          tabIndex={0}
          onKeyDown={onKeyDown}
          rows={1}
          value={input}
          onChange={e => setInput(e.target.value)}
          placeholder="Send a message."
          spellCheck={false}
          className="min-h-[60px] w-full resize-none bg-transparent px-4 py-[1.3rem] focus-within:outline-none sm:text-sm"
        />
        <div className="absolute right-0 top-4 sm:right-4">
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                type="submit"
                size="icon"
                disabled={isLoading || input === ""}
              >
                <IconArrowElbow />
                <span className="sr-only">Send message</span>
              </Button>
            </TooltipTrigger>
            <TooltipContent>Send message</TooltipContent>
          </Tooltip>
        </div>
      </div>
    </form>
  )
}
