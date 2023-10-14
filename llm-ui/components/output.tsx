import remarkMath from "remark-math"
import remarkGfm from "remark-gfm"
import remarkBreaks from "remark-breaks"
import { MemoizedReactMarkdown } from "./markdown"
import { CodeBlock } from "./ui/codeblock"

export function FormattedOutput(props: { content: string }) {
  return (
    <MemoizedReactMarkdown
      className="prose dark:prose-invert prose-p:leading-relaxed prose-pre:p-0 break-words"
      remarkPlugins={[remarkMath, remarkGfm, remarkBreaks]}
      components={{
        a: ({ node, children, href, title, ...props }) => {
          return (
            <a href={href} title={title} {...props} target="_blank">
              {children}
            </a>
          )
        },
        p({ children }) {
          return <p className="mb-2 last:mb-0">{children}</p>
        },
        code({ node, inline, className, children, ...props }) {
          if (children.length) {
            if (children[0] == "▍") {
              return (
                <span className="mt-1 animate-pulse cursor-default">▍</span>
              )
            }

            children[0] = (children[0] as string).replace("`▍`", "▍")
          }

          const match = /language-(\w+)/.exec(className || "")

          if (inline) {
            return (
              <code className={className} {...props}>
                {children}
              </code>
            )
          }

          return (
            <CodeBlock
              key={Math.random()}
              language={(match && match[1]) || ""}
              value={String(children).replace(/\n$/, "")}
              {...props}
            />
          )
        }
      }}
    >
      {props.content}
    </MemoizedReactMarkdown>
  )
}
