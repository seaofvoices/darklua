import * as React from "react"

export const DarkluaContext = React.createContext()

const DarkluaProvider = ({ children }) => {
  const [darklua, setDarklua] = React.useState(null)

  React.useEffect(() => {
    import("../../darklua-wasm/pkg/darklua_wasm")
      .then(darklua => {
        setDarklua(darklua)
      })
      .catch(error => {
        console.warn(`unable to load darklua: ${error}`)
      })
  }, [])

  return (
    <DarkluaContext.Provider value={darklua}>
      {children}
    </DarkluaContext.Provider>
  )
}

export default DarkluaProvider
