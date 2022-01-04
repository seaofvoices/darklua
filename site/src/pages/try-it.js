import * as React from "react"

import { RulesStackProvider } from "../components/rules-stack"
import EditorProviders from "../components/editor-providers"
import Seo from "../components/seo"
import NavBar, { NavBarFiller } from "../components/nav-bar"
import {
  AppBar,
  Button,
  Drawer,
  Paper,
  Stack,
  useMediaQuery,
  useTheme,
} from "@mui/material"
import { Box, styled } from "@mui/system"
import RuleConfiguration from "../components/rule-configuration"
import useCodePreview from "../hooks/useCodePreview"
import SettingsIcon from "@mui/icons-material/Settings"
import MonacoContainer from "../components/monaco-container"
import ThemeProvider from "../components/theme-provider"
import LoadingEditorProviders from "../components/loading-editor-providers"
import { LocationContext } from "../components/location-context"
import DarkluaProvider from "../components/darklua-provider"

const drawerWidth = 500

const ToolbarButton = styled(Button)(({ theme }) => ({
  color: theme.palette.primary.contrastText,
}))

const TryItToolbar = ({ theme, openConfiguration }) => {
  return (
    <Paper
      square
      sx={{ p: 1, backgroundColor: theme.palette.primary.dark, height: "48px" }}
      elevation={0}
    >
      <Stack direction="row" spacing={2}>
        <ToolbarButton
          variant="outlined"
          startIcon={<SettingsIcon />}
          onClick={openConfiguration}
          sx={{ alignSelf: "center" }}
        >
          Configure
        </ToolbarButton>
      </Stack>
    </Paper>
  )
}

const TryItLayout = () => {
  const theme = useTheme()
  const isPermanent = useMediaQuery(theme.breakpoints.up("lg"))

  const { editor, previewEditor, ref, previewRef } = useCodePreview()

  const [drawerOpened, setDrawerOpened] = React.useState(false)

  const openConfiguration = () => {
    if (isPermanent) {
      setDrawerOpened(!drawerOpened)
    } else {
      setDrawerOpened(true)
    }
  }
  const closeDrawer = () => {
    setDrawerOpened(false)
  }

  return (
    <Box sx={{ display: "flex", flexGrow: 1, overflow: "hidden" }}>
      <Seo title={"Try it"} />
      <AppBar
        position="fixed"
        sx={{ zIndex: theme => theme.zIndex.drawer + 1 }}
      >
        <NavBar />
      </AppBar>

      <Drawer
        variant={isPermanent ? "permanent" : "temporary"}
        open={drawerOpened}
        onClose={closeDrawer}
        ModalProps={{
          keepMounted: true,
        }}
        sx={{
          display: drawerOpened || !isPermanent ? null : "none",
          width: drawerWidth,
          overflowX: "hidden",
          flexShrink: 0,
          [`& .MuiDrawer-paper`]: {
            width: drawerWidth,
            boxSizing: "border-box",
          },
        }}
      >
        <NavBarFiller />
        <RuleConfiguration />
      </Drawer>

      <Paper
        component="main"
        sx={{ display: "flex", flexDirection: "column", flexGrow: 1, p: 0 }}
      >
        <NavBarFiller />
        <TryItToolbar theme={theme} openConfiguration={openConfiguration} />

        <LoadingEditorProviders>
          <Stack direction="row" spacing={0} sx={{ flexGrow: 1 }}>
            <MonacoContainer
              ref={ref}
              editor={editor}
              sx={{ width: "0.5vw" }}
            />
            <MonacoContainer
              ref={previewRef}
              editor={previewEditor}
              sx={{ width: "0.5vw" }}
            />
          </Stack>
        </LoadingEditorProviders>
      </Paper>
    </Box>
  )
}

const TryIt = ({ location }) => {
  return (
    <LocationContext.Provider value={location}>
      <ThemeProvider>
        <DarkluaProvider>
          <RulesStackProvider>
            <EditorProviders>
              <TryItLayout />
            </EditorProviders>
          </RulesStackProvider>
        </DarkluaProvider>
      </ThemeProvider>
    </LocationContext.Provider>
  )
}

export default TryIt
