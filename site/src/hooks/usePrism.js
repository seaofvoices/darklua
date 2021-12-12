import * as React from "react"

import ReactDOM from "react-dom"
import Prism from "prismjs"
import ViewStateLink from "../components/ViewStateLink"
import { Button } from "@mui/material"
import { ThemeProvider as MuiThemeProvider } from "@mui/system"

const TryItButtonLink = React.forwardRef(({ theme, code }, ref) => {
  return (
    <MuiThemeProvider theme={theme}>
      <Button
        ref={ref}
        variant="contained"
        size="small"
        component={ViewStateLink}
        to="/try-it"
        target="_blank"
        state={{ code }}
      >
        Try it
      </Button>
    </MuiThemeProvider>
  )
})

export const usePrism = theme => {
  React.useEffect(() => {
    Prism.plugins.toolbar.registerButton("try-it", env => {
      if (env.language !== "lua") {
        return null
      }

      const container = document.createElement("div")

      ReactDOM.render(
        <TryItButtonLink theme={theme} code={env.code} />,
        container
      )

      return container
    })
    Prism.highlightAll()
  }, [theme])
}
