import remarkMath from "remark-math"
import remarkGfm from "remark-gfm"
import remarkBreaks from "remark-breaks"
import { MemoizedReactMarkdown } from "./markdown"
import { CodeBlock } from "./ui/codeblock"
import React from "react"
import { onlyText } from "react-children-utilities"

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
        code({ children, className, ...props }) {
          return (
            <code className={className} {...props}>
              {children}
            </code>
          )
        },
        pre({ node, className, children, ...props }) {
          const _children = React.Children.toArray(children)
          if (_children.length) {
            if (_children[0] == "▍") {
              return (
                <span className="mt-1 animate-pulse cursor-default">▍</span>
              )
            }

            _children[0] = onlyText(_children[0]).replace("`▍`", "▍")
          }

          const match = /language-(\w+)/.exec(className || "")

          if (children) {
            return (
              <CodeBlock
                key={Math.random()}
                language={(match && match[1]) || ""}
                value={onlyText(children).replace(/\n$/, "")}
                {...props}
              />
            )
          }
        }
      }}
    >
      {props.content}
    </MemoizedReactMarkdown>
  )
}
