import * as React from "react"

import {
  ThemeProvider as MuiThemeProvider,
  useMediaQuery,
  createTheme,
  responsiveFontSizes,
} from "@mui/material"

import LightThemeOptions from "./light"
import DarkThemeOptions from "./dark"
import { useLocation } from "../location-context"
import { useThemeLocalStorage } from "../../hooks/useLocalStorage"
import { usePrism } from "../../hooks/usePrism"

const COMMON_OPTIONS = {
  typography: {
    h1: { fontSize: "3.8rem", marginTop: "3rem" },
    h2: { fontSize: "2.7rem", marginTop: "2.3rem" },
    h3: { fontSize: "2rem", marginTop: "1.8rem" },
    h4: { fontSize: "1.5rem", marginTop: "1.5rem" },
    h5: { fontSize: "1.2rem", marginTop: "1.3rem" },
    h6: { fontSize: "1.1rem", marginTop: "1.2rem" },
  },
}
const LIGHT_THEME = responsiveFontSizes(
  createTheme({ ...COMMON_OPTIONS, ...LightThemeOptions })
)
const DARK_THEME = responsiveFontSizes(
  createTheme({ ...COMMON_OPTIONS, ...DarkThemeOptions })
)

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
