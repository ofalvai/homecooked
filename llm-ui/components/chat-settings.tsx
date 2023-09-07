'use client'

import { useState } from 'react'
import { Label } from './ui/label'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue
} from './ui/select'
import { Slider } from './ui/sliders'
import { ChatParams, Temp } from '@/lib/types'
import { Separator } from '@radix-ui/react-separator'
import { SelectGroup, SelectSeparator } from '@radix-ui/react-select'
import { Textarea } from './ui/textarea'

export interface ChatSettingsProps {
  params: ChatParams
  setParams: (p: ChatParams) => void
}

export function ChatSettings({
  params: params,
  setParams: setParams
}: ChatSettingsProps) {
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
        <Label htmlFor="system-prompt">System prompt</Label>
        <Textarea
          id="system-prompt"
          value={params.systemPrompt}
          onChange={e => setParams({ ...params, systemPrompt: e.target.value })}
          className="text-xs"
        />
      </div>
    </div>
  )
}
