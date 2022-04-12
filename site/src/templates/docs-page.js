import * as React from "react"
import { graphql } from "gatsby"

import Layout from "../components/layout"
import { DocumentationNavigation } from "../components/documentation-nav"
import {
  Divider,
  Drawer as MuiDrawer,
  IconButton,
  Toolbar,
  Typography,
  Paper,
  useMediaQuery,
} from "@mui/material"
import { styled, useTheme } from "@mui/material/styles"
import MenuIcon from "@mui/icons-material/Menu"
import ChevronLeftIcon from "@mui/icons-material/ChevronLeft"
import ChevronRightIcon from "@mui/icons-material/ChevronRight"
import { Box } from "@mui/system"
import { LocationContext, useLocation } from "../components/location-context"
import ThemeProvider from "../components/theme-provider"
import { MarkdownRenderer } from "../components/markdown-renderer"

const drawerWidth = 240
const MARGIN = 4

const openedMixin = theme => ({
  width: drawerWidth,
  // transition: theme.transitions.create("width", {
  //   easing: theme.transitions.easing.sharp,
  //   duration: theme.transitions.duration.enteringScreen,
  // }),
  overflowX: "hidden",
})

const openedMixinSide = (theme, fullWidth) => ({
  ...theme.breakpoints.keys.reduce((props, breakpoint) => {
    props[theme.breakpoints.only(breakpoint)] = {
      width: `calc(${
        fullWidth ? "100vw" : theme.breakpoints.values[breakpoint] + "px"
      } - ${drawerWidth}px - 1px - 2*${theme.spacing(MARGIN)})`,
    }
    return props
  }, {}),
  // transition: theme.transitions.create("width", {
  //   easing: theme.transitions.easing.sharp,
  //   duration: theme.transitions.duration.enteringScreen,
  // }),
})

const CLOSED_SPACING = 9

const closedMixin = theme => ({
  // transition: theme.transitions.create("width", {
  //   easing: theme.transitions.easing.sharp,
  //   duration: theme.transitions.duration.leavingScreen,
  // }),
  overflowX: "hidden",
  width: `calc(${theme.spacing(CLOSED_SPACING)} + 1px)`,
})

const closedMixinSide = (theme, fullWidth) => ({
  // transition: theme.transitions.create("width", {
  //   easing: theme.transitions.easing.sharp,
  //   duration: theme.transitions.duration.leavingScreen,
  // }),
  overflowX: "hidden",
  ...theme.breakpoints.keys.reduce((props, breakpoint) => {
    props[theme.breakpoints.only(breakpoint)] = {
      width: `calc(${
        fullWidth ? "100vw" : theme.breakpoints.values[breakpoint] + "px"
      } - ${theme.spacing(CLOSED_SPACING)} - 1px - 2*${theme.spacing(MARGIN)})`,
    }
    return props
  }, {}),
})

const DrawerHeader = styled("div")(({ theme }) => ({
  display: "flex",
  alignItems: "center",
  justifyContent: "flex-end",
  padding: theme.spacing(0, 1),
  // necessary for content to be below app bar
  ...theme.mixins.toolbar,
}))

const Drawer = styled(MuiDrawer, {
  shouldForwardProp: prop => prop !== "open",
})(({ theme, open }) => ({
  width: drawerWidth,
  flexShrink: 0,
  whiteSpace: "nowrap",
  boxSizing: "border-box",
  ...(open && {
    ...openedMixin(theme),
    "& .MuiDrawer-paper": openedMixin(theme),
  }),
  ...(!open && {
    ...closedMixin(theme),
    "& .MuiDrawer-paper": closedMixin(theme),
  }),
}))

const Side = styled(Box, {
  shouldForwardProp: prop => prop !== "open",
})(({ theme, open, fullWidth }) => ({
  boxSizing: "border-box",
  ...(open && {
    ...openedMixinSide(theme, fullWidth),
  }),
  ...(!open && {
    ...closedMixinSide(theme, fullWidth),
  }),
}))

const DocsTemplate = ({ data }) => {
  const location = useLocation()
  const theme = useTheme()

  const [opened, setOpen] = React.useState(
    location.state ? location.state.drawerOpened || false : false
  )
  const forceOpenedDrawer = useMediaQuery(theme.breakpoints.up("lg"))
  const fullWidth = useMediaQuery(theme.breakpoints.down("md"))

  const handleDrawerOpen = () => setOpen(true)
  const handleDrawerClose = () => setOpen(false)

  // const { previous, next } = data

  const drawerOpened = opened || forceOpenedDrawer

  React.useEffect(() => {
    if (forceOpenedDrawer && !opened) {
      setOpen(true)
    }
  }, [forceOpenedDrawer, opened])

  const drawerHeaderContent = forceOpenedDrawer ? (
    <></>
  ) : drawerOpened ? (
    <IconButton onClick={handleDrawerClose}>
      {theme.direction === "rtl" ? <ChevronRightIcon /> : <ChevronLeftIcon />}
    </IconButton>
  ) : (
    <IconButton
      color="inherit"
      aria-label="open drawer"
      onClick={handleDrawerOpen}
      edge="start"
      sx={{
        margin: "auto",
      }}
    >
      <MenuIcon />
    </IconButton>
  )

  const title = data.markdownRemark.frontmatter.title
  const htmlAst = data.markdownRemark.htmlAst

  return (
    <Layout title={title} margin={MARGIN}>
      <Paper
        elevation={0}
        square={false}
        sx={{
          display: "flex",
          border: 0,
          justifyContent: fullWidth ? "center" : "center",
        }}
      >
        <Drawer variant="permanent" anchor="left" open={drawerOpened}>
          <Toolbar />
          <DrawerHeader>{drawerHeaderContent}</DrawerHeader>
          <Divider />

          <DocumentationNavigation
            drawerOpened={drawerOpened}
            openDrawer={handleDrawerOpen}
            pages={data.allMarkdownRemark.nodes.map(node => ({
              slug: node.fields.slug,
              title: node.frontmatter.title,
            }))}
          />
          <Toolbar />
        </Drawer>
        <Side
          open={drawerOpened}
          fullWidth={fullWidth}
          id="MAIN_WITH_MARGIN"
          component="main"
          sx={{ width: "100vw", margin: 0 }}
        >
          <header>
            <Typography
              variant="h1"
              itemProp="headline"
              sx={{
                marginTop: 3,
              }}
            >
              {title}
            </Typography>
          </header>
          <hr />
          <MarkdownRenderer htmlAst={htmlAst} />
          <hr />
        </Side>
      </Paper>
    </Layout>
  )
}

const DocsPageTemplate = ({ data, location }) => (
  <LocationContext.Provider value={location}>
    <ThemeProvider>
      <DocsTemplate data={data} />
    </ThemeProvider>
  </LocationContext.Provider>
)

export default DocsPageTemplate

export const pageQuery = graphql`
  query DocsPageBySlug(
    $id: String!
    $previousPostId: String
    $nextPostId: String
  ) {
    site {
      siteMetadata {
        title
      }
    }
    allMarkdownRemark {
      nodes {
        fields {
          slug
        }
        frontmatter {
          title
        }
      }
    }
    markdownRemark(id: { eq: $id }) {
      id
      excerpt(pruneLength: 160)
      htmlAst
      frontmatter {
        title
        description
      }
      headings {
        depth
        value
      }
    }
    previous: markdownRemark(id: { eq: $previousPostId }) {
      fields {
        slug
      }
      frontmatter {
        title
      }
    }
    next: markdownRemark(id: { eq: $nextPostId }) {
      fields {
        slug
      }
      frontmatter {
        title
      }
    }
  }
`
