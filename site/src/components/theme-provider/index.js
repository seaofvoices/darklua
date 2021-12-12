import * as React from "react"

import {
  ThemeProvider as MuiThemeProvider,
  useMediaQuery,
  createTheme,
} from "@mui/material"

import LightThemeOptions from "./light"
import DarkThemeOptions from "./dark"
import { useLocation } from "../location-context"
import { useThemeLocalStorage } from "../../hooks/useLocalStorage"
import { usePrism } from "../../hooks/usePrism"

const LIGHT_THEME = createTheme(LightThemeOptions)
const DARK_THEME = createTheme(DarkThemeOptions)

export const ThemeContext = React.createContext(() => {})

const THEMES = {
  light: LIGHT_THEME,
  dark: DARK_THEME,
}

const ThemeProvider = ({ children }) => {
  const prefersDarkMode = useMediaQuery("(prefers-color-scheme: dark)")
  const [themeName, setThemeName] = React.useState(null)
  const { state } = useLocation()

  const [themeNameCookie, setThemeNameCookie] = useThemeLocalStorage(null)

  let finalThemeName = themeName
  if (!finalThemeName) {
    if (state && state.currentThemeName && THEMES[state.currentThemeName]) {
      finalThemeName = state.currentThemeName
    } else if (themeNameCookie && THEMES[themeNameCookie]) {
      finalThemeName = themeNameCookie
    } else {
      finalThemeName = prefersDarkMode ? "dark" : "light"
    }
  }

  const theme = THEMES[finalThemeName]

  const setTheme = themeName => {
    setThemeName(themeName)
    setThemeNameCookie(themeName)
  }

  usePrism(theme)

  return (
    <MuiThemeProvider theme={theme}>
      <ThemeContext.Provider
        value={{
          current: finalThemeName,
          setTheme,
        }}
      >
        {children}
      </ThemeContext.Provider>
    </MuiThemeProvider>
  )
}

export default ThemeProvider
