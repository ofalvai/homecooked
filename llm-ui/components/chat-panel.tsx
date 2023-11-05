import { type UseChatHelpers } from "ai/react"

import { Button } from "@/components/ui/button"
import { PromptForm } from "@/components/prompt-form"
import { ButtonScrollToBottom } from "@/components/button-scroll-to-bottom"
import { IconPlus, IconRefresh, IconStop } from "@/components/ui/icons"
import { useRouter } from "next/navigation"

export interface ChatPanelProps
  extends Pick<
    UseChatHelpers,
    | "append"
    | "isLoading"
    | "reload"
    | "messages"
    | "stop"
    | "input"
    | "setInput"
  > {
  id?: string
}

export function ChatPanel({
  id,
  isLoading,
  stop,
  append,
  reload,
  input,
  setInput,
  messages
}: ChatPanelProps) {
  const router = useRouter()
  return (
    <div className="from-muted/10 to-muted/30 fixed inset-x-0 bottom-0 bg-gradient-to-b from-10% to-50% pr-80">
      <ButtonScrollToBottom />
      <div className="mx-auto sm:max-w-2xl sm:px-4">
        <div className="mb-2 flex h-10 items-center justify-center">
          {isLoading ? (
            <Button
              variant="outline"
              onClick={() => stop()}
              className="bg-background"
            >
              <IconStop className="mr-2" />
              Stop generating
            </Button>
          ) : (
            messages?.length > 0 && (
              <>
                <Button
                  variant="outline"
                  onClick={() => reload()}
                  className="bg-background mx-1"
                >
                  <IconRefresh className="mr-2" />
                  Regenerate response
                </Button>
                <Button
                  variant="outline"
                  onClick={e => {
                    e.preventDefault()
                    router.refresh()
                    router.push("/")
                  }}
                  className="bg-background mx-1"
                >
                  <IconPlus className="mr-2" />
                  New chat
                </Button>
              </>
            )
          )}
        </div>
        <div className="bg-background space-y-4 border-t px-4 py-2 shadow-lg sm:rounded-t-xl sm:border md:py-4">
          <PromptForm
            onSubmit={async value => {
              await append({
                id,
                content: value,
                role: "user"
              })
            }}
            input={input}
            setInput={setInput}
            isLoading={isLoading}
          />
          {/* <FooterText className="hidden sm:block" /> */}
        </div>
      </div>
    </div>
  )
}
