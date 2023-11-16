import * as React from "react"
import { CornerRightDown, TextCursorInput, Wand2 } from "lucide-react"

import { cn } from "@/lib/utils"
import { Button, buttonVariants } from "@/components/ui/button"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem
} from "@/components/ui/command"
import {
  Popover,
  PopoverContent,
  PopoverTrigger
} from "@/components/ui/popover"
import { useRef, useState } from "react"
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger
} from "@/components/ui/hover-card"
import { useMutationObserver } from "@/lib/hooks/use-mutation-observer"
import { Spell } from "@/lib/types"
import { Tooltip, TooltipContent, TooltipTrigger } from "./ui/tooltip"

interface SpellbookProps {
  spells: Spell[]
  onSelect: (spell: Spell) => void
  onInsert: (spell: Spell) => void
}

export function Spellbook({ spells, onSelect, onInsert }: SpellbookProps) {
  const [open, setOpen] = useState(false)
  const [peekedSpell, setPeekedSpell] = useState<Spell | undefined>()

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className={cn(
            buttonVariants({ size: "sm", variant: "outline" }),
            "bg-background absolute left-0 top-4 h-8 w-8 rounded-full p-0 sm:left-4"
          )}
        >
          <span className="sr-only">Spellbook</span>
          <Wand2 className="h-4 w-4 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent sideOffset={24} className="h-[300px] w-[300px] p-0">
        <HoverCard openDelay={0} closeDelay={0}>
          <HoverCardContent
            side="right"
            className="w-80"
            forceMount
            align="start"
          >
            <div className="">
              <div className="whitespace-pre-wrap text-sm">
                {peekedSpell?.prompt}
              </div>
            </div>
          </HoverCardContent>
          <Command>
            <HoverCardTrigger />
            <CommandInput placeholder="Select from spellbook..." />
            <CommandEmpty>No results.</CommandEmpty>
            <CommandGroup className="overflow-auto">
              {spells.map(spell => (
                <SpellItem
                  key={spell.id}
                  spell={spell}
                  onSelect={spell => {
                    setOpen(false)
                    onSelect(spell)
                  }}
                  onPeek={spell => {
                    setPeekedSpell(spell)
                  }}
                  onInsert={spell => onInsert(spell)}
                />
              ))}
            </CommandGroup>
          </Command>
        </HoverCard>
      </PopoverContent>
    </Popover>
  )
}

interface SpellItemProps {
  spell: Spell
  onSelect: (spell: Spell) => void
  onInsert: (spell: Spell) => void
  onPeek: (spell: Spell) => void
}
function SpellItem({ spell, onPeek, onSelect, onInsert }: SpellItemProps) {
  const ref = useRef<HTMLDivElement>(null)

  useMutationObserver(
    ref,
    mutations => {
      for (const mutation of mutations) {
        if (mutation.type === "attributes") {
          if (
            mutation.attributeName === "aria-selected" &&
            mutation.oldValue != "true"
          ) {
            onPeek(spell)
          }
        }
      }
    },
    {
      attributes: true,
      attributeOldValue: true,
      characterData: false,
      childList: false,
      subtree: false
    }
  )

  return (
    <CommandItem ref={ref} onSelect={() => {}}>
      <div className="flex w-full flex-row items-center">
        <span className="grow text-ellipsis">{spell.label}</span>
        <Tooltip>
          <TooltipTrigger>
            <Button
              variant="outline"
              className={cn(
                buttonVariants({ size: "icon", variant: "ghost" }),
                "mx-2"
              )}
              onClick={e => {
                e.preventDefault()
                onSelect(spell)
              }}
            >
              <CornerRightDown className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Replace</TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger>
            <Button
              variant="outline"
              className={cn(buttonVariants({ size: "icon", variant: "ghost" }))}
              onClick={e => {
                e.preventDefault()
                onInsert(spell)
              }}
            >
              <TextCursorInput className="h-4 w-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Insert at cursor</TooltipContent>
        </Tooltip>
      </div>
    </CommandItem>
  )
}
