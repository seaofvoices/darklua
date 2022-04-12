import * as React from "react"

import { Button, Toolbar } from "@mui/material"
import NewTabLink from "./new-tab-link"
import { Box, styled } from "@mui/system"
import { ThemeContext } from "./theme-provider"
import ThemeSwitch from "./theme-switch"
import ViewStateLink from "./ViewStateLink"

const NavigationLink = ({ label, link }) => {
  if (link.startsWith("https://")) {
    return (
      <NewTabLink color="inherit" href={link} target="_blank" rel="noreferrer">
        {label}
      </NewTabLink>
    )
  } else {
    return (
      <Button color="inherit" component={ViewStateLink} to={link}>
        {label}
      </Button>
    )
  }
}

const NavBar = () => {
  const theme = React.useContext(ThemeContext)

  const handleChange = () => {
    theme.setTheme(theme.current === "light" ? "dark" : "light")
  }
  return (
    <Toolbar>
      <Box sx={{ flexGrow: 1 }}>
        <NavigationLink label="darklua" link="/" />
      </Box>
      <NavigationLink label="Documentation" link="/docs" />
      <NavigationLink label="Try it" link="/try-it" />
      <NavigationLink
        label="Gitlab"
        link="https://gitlab.com/seaofvoices/darklua"
      />
      <ThemeSwitch checked={theme.current === "dark"} onChange={handleChange} />
    </Toolbar>
  )
}

export const NavBarFiller = styled("div")(({ theme }) => ({
  // necessary for content to be below app bar
  ...theme.mixins.toolbar,
}))

export default NavBar
