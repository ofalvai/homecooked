"use client"

import { useState } from "react"
import { Label } from "./ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue
} from "./ui/select"
import { Slider } from "./ui/sliders"
import { ChatParams, Persona, Personas, Temp } from "@/lib/types"
import { Separator } from "@radix-ui/react-separator"
import { SelectGroup } from "@radix-ui/react-select"
import { Textarea } from "./ui/textarea"
import { parse } from "yaml"
import useSWRImmutable from "swr/immutable"
import { useAppConfig } from "@/lib/swr-utils"

export interface ChatSettingsProps {
  params: ChatParams
  setParams: (p: ChatParams) => void
}

async function personaFetcher(url: string): Promise<Personas> {
  return fetch(url).then(async res => {
    const text = await res.text()
    return parse(text) as Personas
  })
}

export function ChatSettings({
  params: params,
  setParams: setParams
}: ChatSettingsProps) {
  const { data: appConfig } = useAppConfig()
  const { data: personas } = useSWRImmutable(
    () => {
      // Throw error on purpose to let SWR know of the dependency
      return appConfig!.llmApiBaseUrl + "/config/personas.yml"
    },
    personaFetcher,
    {
      onSuccess: (personas: Personas) => {
        const persona = personas.personas.find(p => p.id === personas.default)
        if (persona) {
          setPersonaId(personas.default)
          setParams({ ...params, systemPrompt: persona.prompt })
        }
      },
      onError: err => console.log(err)
    }
  )
  const [personaId, setPersonaId] = useState<string | "manual">("manual")

  return (
    <div className="p-4">
      <Label>Model</Label>
      <Select
        value={params.model}
        onValueChange={(v: string) => setParams({ ...params, model: v })}
      >
        <SelectTrigger className="w-full">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectGroup>
            <SelectLabel>OpenAI</SelectLabel>
            <SelectItem value="gpt-3.5-turbo">GPT 3.5 Turbo</SelectItem>
            <SelectItem value="gpt-3.5-turbo-16k">GPT 3.5 Turbo 16K</SelectItem>
            <SelectItem value="gpt-4">GPT 4</SelectItem>
          </SelectGroup>
          <SelectGroup>
            <SelectLabel>Anthropic</SelectLabel>
            <SelectItem value="claude-instant">Claude Instant</SelectItem>
            <SelectItem value="claude-2">Claude 2</SelectItem>
          </SelectGroup>
        </SelectContent>
      </Select>

      <Separator className="my-8" />

      <Label>Temperature</Label>
      <Select
        value={params.temp}
        onValueChange={(v: Temp) => setParams({ ...params, temp: v })}
      >
        <SelectTrigger className="w-full">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="low">Low</SelectItem>
          <SelectItem value="medium">Medium</SelectItem>
          <SelectItem value="high">High</SelectItem>
        </SelectContent>
      </Select>

      <Separator className="my-8" />

      <div className="flex flex-row justify-between">
        <Label>Max length</Label>
        <Label className="self-end">{params.maxLength}</Label>
      </div>
      <Slider
        value={[params.maxLength]}
        onValueChange={(v: number[]) =>
          setParams({ ...params, maxLength: v[0] })
        }
        max={1024}
        step={64}
        className="my-4"
      />

      <Separator className="my-8" />

      <div className="grid w-full gap-1.5">
        <Label>Persona</Label>
        <Select
          value={personaId ?? personas?.default ?? "manual"}
          onValueChange={(v: string) => {
            setPersonaId(v)
            const persona = personas?.personas.find(p => p.id === v)
            if (persona) {
              setParams({ ...params, systemPrompt: persona.prompt })
            }
          }}
        >
          <SelectTrigger className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {personas?.personas.map((persona: Persona) => (
              <SelectItem value={persona.id} key={persona.id}>
                <div className="flex flex-row items-center">
                  <div
                    className="border-foreground/50 mr-2 h-3 w-3 rounded-full border"
                    style={{ backgroundColor: persona.color }}
                  ></div>
                  {persona.display_name}
                </div>
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Textarea
          id="system-prompt"
          value={params.systemPrompt}
          onChange={e => setParams({ ...params, systemPrompt: e.target.value })}
          className="mt-2 h-64 text-xs"
        />
      </div>
    </div>
  )
}
