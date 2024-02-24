import * as React from "react"
import { Link } from "gatsby"
import rehypeReact from "rehype-react"

import { styled, useTheme } from "@mui/system"
import { Typography, Link as MuiLink } from "@mui/material"
import { RuleReference } from "./rule-reference"
import { toHast } from "mdast-util-to-hast"
import { fromMarkdown } from "mdast-util-from-markdown"
import ViewStateLink from "./ViewStateLink"
import { CompareCode } from "./compare-code"

const AnchorOffsetByToolbar = styled("div")(({ theme }) => {
  const newStyle = {
    display: "block",
    position: "relative",
    visibility: "hidden",
    bottom: `${theme.mixins.toolbar.minHeight}px`,
  }

  Object.entries(theme.mixins.toolbar).forEach(([key, value]) => {
    if (key.startsWith("@media")) {
      newStyle[key] = { bottom: `${value.minHeight}px` }
    }
  })

  return newStyle
})

const createHeaderComponent = (level, linkable = false) => {
  const LinkableHeader = ({ children, id = null }) => {
    if (!linkable || !id) {
      return <Typography variant={`h${level}`}>{children}</Typography>
    }

    return (
      <Typography variant={`h${level}`}>
        <AnchorOffsetByToolbar id={id} />

        {children}

        <MuiLink
          component={Link}
          to={`#${id}`}
          underline="none"
          sx={{
            ml: "0.5rem",
            opacity: 0.3,
          }}
        >
          ¶
        </MuiLink>
      </Typography>
    )
  }

  return LinkableHeader
}

const visitNodes = (node, callback) => {
  callback(node)
  if (!!node.children) {
    node.children.forEach((child, i) => {
      visitNodes(child, callback)
    })
  }
}

const generateId = ({ children, ...node }) => {
  const text = children[0].value
  return text.toLowerCase().replaceAll(" ", "-")
}

const TextLink = React.forwardRef((props, ref) => {
  const theme = useTheme()

  const linkColor =
    theme.palette.mode === "dark"
      ? theme.palette.primary.light
      : theme.palette.primary.dark

  return <ViewStateLink ref={ref} style={{ color: linkColor }} {...props} />
})

const renderAst = new rehypeReact({
  createElement: React.createElement,
  components: {
    h1: createHeaderComponent(1),
    h2: createHeaderComponent(2, true),
    h3: createHeaderComponent(3, true),
    h4: createHeaderComponent(4),
    h5: createHeaderComponent(5),
    h6: createHeaderComponent(6),
    a: TextLink,
    "rule-reference": RuleReference,
    "compare-code": CompareCode,
  },
}).Compiler

export const MarkdownRenderer = ({ htmlAst, ...props }) => {
  React.useEffect(() => {
    const generatedIds = new Set()

    visitNodes(htmlAst, node => {
      if (node.type === "element") {
        const { tagName, properties } = node

        if (tagName === "h2" || tagName === "h3") {
          const baseId = generateId(node)

          if (generatedIds.has(baseId)) {
            let i = 0
            let newId = ""
            do {
              i += 1
              newId = `${baseId}-${i}`
            } while (generatedIds.has(newId))
            generatedIds.add(newId)

            properties.id = newId
          } else {
            generatedIds.add(baseId)

            properties.id = baseId
          }
        }
      }
    })
  }, [htmlAst])

  return renderAst(htmlAst)
}

const EMPTY_OBJECT = JSON.stringify({})

export const RenderMarkdown = ({ markdown, ...context }) => {
  const serializedContext = JSON.stringify(context)

  const htmlAst = React.useMemo(() => {
    const html = toHast(fromMarkdown(markdown))
    if (serializedContext !== EMPTY_OBJECT) {
      html.children[0].properties.__darkluacontext = serializedContext
    }
    return html
  }, [markdown, serializedContext])

  return <MarkdownRenderer htmlAst={htmlAst} />
}

export const RenderCode = ({ code, language = "lua", ...context }) => (
  <RenderMarkdown
    markdown={`\`\`\`${language}\n${code}${
      code.endsWith("\n") ? "" : "\n"
    }\`\`\``}
    {...context}
  />
)
