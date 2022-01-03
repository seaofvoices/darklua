import * as React from "react"

import * as json5 from "json5"
import { RulesStackContext } from "../rules-stack"
import { Alert, Chip, Snackbar, Stack, Tooltip } from "@mui/material"
import useMonacoEditor from "../../hooks/useMonacoEditor"
import ThumbUp from "@mui/icons-material/ThumbUp"
import ThumbDown from "@mui/icons-material/ThumbDown"
import MonacoContainer from "../monaco-container"
import useDarkluaConfigSchema from "../../hooks/useDarkluaConfigSchema"

const WORD_PER_MINUTE = 85

const OK_PROPS = {
  label: "Ok!",
  title: "This configuration file looks good!",
  color: "success",
  icon: () => <ThumbUp fontSize="14" />,
}
const ERR_PROPS = {
  label: "Err!",
  title: "Oops, this configuration file is not valid.",
  color: "error",
  icon: () => <ThumbDown fontSize="14" />,
}

const ChipIsConfigOk = ({ isConfigOk }) => {
  const okProps = isConfigOk ? OK_PROPS : ERR_PROPS

  return (
    <Tooltip title={okProps.title}>
      <Chip
        label={okProps.label}
        icon={okProps.icon()}
        color={okProps.color}
        onClick={
          isConfigOk
            ? null
            : () => {
                window.open("/docs/config", "_blank").focus()
              }
        }
      />
    </Tooltip>
  )
}

const EditorBar = ({ formatCode, isConfigOk }) => {
  return (
    <Stack
      direction="row"
      spacing={0.5}
      sx={{ padding: 0.5, justifyContent: "flex-end" }}
    >
      <Chip
        key="format"
        label="format"
        onClick={formatCode}
        color="primary"
        disabled={!formatCode}
      />
      <ChipIsConfigOk key="ok" isConfigOk={isConfigOk} />
    </Stack>
  )
}

const TextRuleEditor = () => {
  const rulesStack = React.useContext(RulesStackContext)

  const configSchema = useDarkluaConfigSchema()

  const [isConfigOk, setIsConfigOk] = React.useState(true)
  const [defaultConfig, setDefaultConfig] = React.useState(null)
  const [alertMessage, setAlertMessage] = React.useState(null)

  const {
    model,
    editor,
    ref: parentRef,
  } = useMonacoEditor({
    language: "",
    tabSize: 2,
    insertSpaces: true,
    minimap: false,
  })

  React.useEffect(() => {
    if (!defaultConfig) {
      setDefaultConfig(rulesStack.getDarkluaConfig())
    }
  }, [defaultConfig, rulesStack])

  React.useEffect(() => {
    if (!model) {
      return
    }
    model.setValue(json5.stringify(defaultConfig, null, 2))
  }, [defaultConfig, model])

  React.useEffect(() => {
    if (!model) {
      return
    }
    const connection = model.onDidChangeContent(_event => {
      let config = null
      try {
        config = json5.parse(model.getValue())
      } catch (error) {
        setIsConfigOk(false)
      }

      if (!!config) {
        const { value, error } = configSchema.validate(config)
        if (error) {
          setIsConfigOk(false)
          setAlertMessage(`invalid darklua configuration: ${error.message}`)
        } else {
          rulesStack.replaceWithDarkluaConfig(value)
          setIsConfigOk(true)
        }
      }
    })
    return () => connection.dispose()
  }, [model, rulesStack, configSchema])

  const formatCode = () => {
    if (!model) {
      return
    }
    try {
      const config = json5.parse(model.getValue())
      model.setValue(json5.stringify(config, null, 2))
    } catch (error) {
      setAlertMessage(
        `unable to format: ${error.message.replace("JSON5: ", "")}`
      )
    }
  }

  const handleClose = (_event, reason) => {
    if (reason === "clickaway") {
      return
    }
    setAlertMessage(null)
  }

  let hideDuration = 3
  if (!!alertMessage) {
    hideDuration = (alertMessage.split(" ").length * 60) / WORD_PER_MINUTE
  }

  return (
    <>
      <Stack direction="column" spacing={0} flexGrow={1} height="100%">
        <EditorBar formatCode={model && formatCode} isConfigOk={isConfigOk} />
        <MonacoContainer ref={parentRef} editor={editor} sx={{ flexGrow: 1 }} />
      </Stack>
      <Snackbar
        open={alertMessage !== null}
        autoHideDuration={hideDuration * 1000}
        onClose={handleClose}
      >
        <Alert onClose={handleClose} severity="error" sx={{ width: "100%" }}>
          {alertMessage}
        </Alert>
      </Snackbar>
    </>
  )
}

export default TextRuleEditor
