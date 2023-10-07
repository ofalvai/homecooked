import { AppConfig } from "@/lib/types"

export async function GET() {
  const baseUrl = process.env.LLM_API_BASE_URL
  if (!baseUrl) throw new Error("Missing env var LLM_API_BASE_URL")

  const appConfig: AppConfig = {
    llmApiBaseUrl: baseUrl
  }
  return Response.json(appConfig)
}
