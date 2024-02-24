import * as React from "react"

import { createRoot } from "react-dom/client"
import Prism from "prismjs"
import ViewStateLink from "../components/ViewStateLink"
import { Button } from "@mui/material"
import { ThemeProvider as MuiThemeProvider } from "@mui/system"

const TryItButtonLink = React.forwardRef(
  ({ theme, code, configuration }, ref) => {
    return (
      <MuiThemeProvider theme={theme}>
        <Button
          ref={ref}
          variant="contained"
          size="small"
          component={ViewStateLink}
          to="/try-it"
          target="_blank"
          state={{ code, configuration }}
        >
          Try it
        </Button>
      </MuiThemeProvider>
    )
  }
)

let registerTryItOnce = false

export const usePrism = theme => {
  React.useEffect(() => {
    if (registerTryItOnce) {
      return
    }
    registerTryItOnce = true
    Prism.plugins.toolbar.registerButton("try-it", env => {
      if (env.language !== "lua") {
        return null
      }

      let configuration

      try {
        const context = JSON.parse(
          env.element.parentNode.attributes.__darkluacontext.nodeValue
        )
        configuration = { rules: context?.rules }
      } catch (_) {}

      const container = document.createElement("div")

      const root = createRoot(container)

      root.render(
        <TryItButtonLink
          theme={theme}
          code={env.code}
          configuration={configuration}
        />
      )

      return container
    })
  }, [theme])

  React.useEffect(() => {
    Prism.highlightAll()
  }, [])
}
