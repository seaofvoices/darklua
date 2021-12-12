import * as React from "react"

import Layout from "../components/layout"
import { Button, CardContent, Paper, Typography, useTheme } from "@mui/material"
import ThemeProvider from "../components/theme-provider"
import { LocationContext } from "../components/location-context"
import { Box } from "@mui/system"
import RocketLaunchIcon from "@mui/icons-material/RocketLaunch"
import ViewStateLink from "../components/ViewStateLink"

const InfoCard = ({ title, content, children }) => (
  <Paper elevation={2} sx={{ width: "280px", mx: 2, my: 0.5 }}>
    <CardContent>
      <Typography variant="h5" component="div"></Typography>
      {title}
      <Typography sx={{ mb: 1.5 }} color="text.secondary"></Typography>
      <Typography variant="body2">{content}</Typography>
    </CardContent>
  </Paper>
)

const cards = [
  {
    title: "Preserve stack traces",
    content:
      "When generating code, darklua keeps the line numbers aligned with " +
      "the original code to keep stack traces easily actionable.",
  },
  {
    title: "Inject global variables",
    content:
      "Inline a global directly in the code, like an environment configuration " +
      "value. Use the same source but produce artifacts with debug information " +
      "or optimization enabled.",
  },
  {
    title: "Process Roblox Luau",
    content: "darklua can parse and process Roblox Luau.",
  },
]

const Home = () => {
  const theme = useTheme()

  return (
    <Layout title="Home">
      <Paper
        square
        elevation={1}
        sx={{ maxWidth: "lg", margin: "auto", mt: 0, height: "100%" }}
      >
        <Paper
          square
          sx={{
            pt: 6,
            pb: 6,
            color: theme.palette.primary.contrastText,
            backgroundColor: theme.palette.primary.main,
          }}
        >
          <Typography variant="h1" align="center">
            darklua
          </Typography>
        </Paper>
        <Box sx={{ mt: 4, pb: 2 }}>
          <Typography variant="h4" align="center">
            Transform Lua code.
          </Typography>
        </Box>
        <Box
          sx={{
            display: "flex",
            flexDirection: "row",
            flexWrap: "wrap",
            justifyContent: "center",
            alignItems: "center",
            pb: 3,
          }}
        >
          {cards.map((card, index) => (
            <InfoCard key={`${index}`} {...card} />
          ))}
        </Box>
        <Paper
          square
          elevation={2}
          sx={{
            pt: 6,
            pb: 6,
          }}
        >
          <Typography variant="h4" align="center" sx={{ pb: 2 }}>
            Not convinced?
          </Typography>

          <Typography variant="h5" align="center">
            You can try it directly in your browser
          </Typography>
          <Box
            sx={{
              display: "flex",
              flexDirection: "row",
              flexWrap: "wrap",
              justifyContent: "center",
              alignItems: "center",
              py: 3,
            }}
          >
            <Button
              variant="contained"
              size="large"
              endIcon={<RocketLaunchIcon />}
              component={ViewStateLink}
              to="/try-it"
            >
              Try it
            </Button>
          </Box>
        </Paper>
      </Paper>
    </Layout>
  )
}

const HomePage = ({ location }) => (
  <LocationContext.Provider value={location}>
    <ThemeProvider>
      <Home />
    </ThemeProvider>
  </LocationContext.Provider>
)

export default HomePage
