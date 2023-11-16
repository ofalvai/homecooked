"use client"

import { SWRResponse } from "swr"
import { AppConfig } from "./types"
import useSWRImmutable from "swr/immutable"
import { fetcher } from "./utils"

export function useAppConfig(): SWRResponse<AppConfig> {
  return useSWRImmutable<AppConfig>("/api/config", fetcher, {
    onError: err => console.log(err)
  })
}
