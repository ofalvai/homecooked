import { AppConfig } from "@/lib/types"

// Force Next.js to treat this as a dynamic route because of reading runtime-defined env vars
export const dynamic = 'force-dynamic'

export async function GET() {
  const baseUrl = process.env.LLM_API_BASE_URL
  if (!baseUrl) throw new Error("Missing env var LLM_API_BASE_URL")

  const appConfig: AppConfig = {
    llmApiBaseUrl: baseUrl
  }
  return Response.json(appConfig)
}
