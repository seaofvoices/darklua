import * as React from "react"

import { useThemeMode } from "../hooks/useThemeMode"

export const MonacoContext = React.createContext()
export const DarkluaContext = React.createContext()

const THEME_MAPPING = {
  light: "vs",
  dark: "vs-dark",
}

const EditorProviders = ({ children }) => {
  const [monaco, setMonaco] = React.useState(null)
  const [darklua, setDarklua] = React.useState(null)

  const monacoTheme = useThemeMode(THEME_MAPPING)

  React.useEffect(() => {
    if (monaco) {
      monaco.editor.setTheme(monacoTheme)
    }
  }, [monaco, monacoTheme])

  React.useEffect(() => {
    import("monaco-editor")
      .then(monaco => {
        setMonaco(monaco)
      })
      .catch(error => {
        console.warn(`unable to load monaco-editor: ${error}`)
      })

    import("../../darklua-wasm/pkg/darklua_wasm")
      .then(darklua => {
        setDarklua(darklua)
      })
      .catch(error => {
        console.warn(`unable to load darklua: ${error}`)
      })
  }, [])

  if (!!monaco && !!darklua) {
    return (
      <MonacoContext.Provider value={monaco}>
        <DarkluaContext.Provider value={darklua}>
          {children}
        </DarkluaContext.Provider>
      </MonacoContext.Provider>
    )
  }

  return (
    <MonacoContext.Provider value={monaco}>
      <DarkluaContext.Provider value={darklua}>
        {children}
      </DarkluaContext.Provider>
    </MonacoContext.Provider>
  )
}

export default EditorProviders
