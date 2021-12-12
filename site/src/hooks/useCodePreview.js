import * as React from "react"

import * as json5 from "json5"
import useMonacoEditor from "./useMonacoEditor"
import { DarkluaContext } from "../components/editor-providers"
import { RulesStackContext } from "../components/rules-stack"
import { useLocation } from "../components/location-context"

const DEFAULT_TEXT = "-- paste code here to preview the darklua transform"

const useCodePreview = () => {
  const darklua = React.useContext(DarkluaContext)
  const rulesStack = React.useContext(RulesStackContext)
  const location = useLocation()

  const {
    model: previewModel,
    editor: previewEditor,
    ref: previewRef,
  } = useMonacoEditor({ readOnly: true })

  const locationCode = location.state && location.state.code
  const defaultText = locationCode || DEFAULT_TEXT
  const { model, editor, ref } = useMonacoEditor({ defaultText })

  React.useEffect(() => {
    if (!model || !previewModel) {
      return
    }
    const processCode = code => {
      try {
        return darklua.process_code(code, rulesStack.getDarkluaConfig())
      } catch (error) {
        return (
          `--[[\n\tan error happened while trying to process the code:\n${error}\n\n` +
          `Configuration: ${json5.stringify(
            rulesStack.getDarkluaConfig(),
            null,
            2
          )}]]`
        )
      }
    }
    const onChange = _event => {
      const newCode = processCode(model.getValue())
      previewModel.setValue(newCode)
    }
    const connection = model.onDidChangeContent(onChange)
    onChange()

    return () => connection.dispose()
  }, [darklua, model, previewModel, rulesStack])

  return { previewEditor, editor, previewRef, ref }
}

export default useCodePreview
