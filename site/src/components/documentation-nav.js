import * as React from "react"
import { graphql, useStaticQuery } from "gatsby"

import {
  Collapse,
  List,
  ListItemButton,
  ListItemIcon,
  ListItemText,
} from "@mui/material"
import ExpandLess from "@mui/icons-material/ExpandLess"
import ExpandMore from "@mui/icons-material/ExpandMore"
import MenuBook from "@mui/icons-material/MenuBook"
import ViewStateLink from "./ViewStateLink"
import { useLocation } from "./location-context"

const query = graphql`
  query allDocumentation {
    site {
      siteMetadata {
        title
        groupsOrder
      }
    }
    allMarkdownRemark {
      nodes {
        fields {
          slug
        }
        frontmatter {
          title
          group
          order
        }
      }
    }
  }
`

const DocumentationLink = ({ title, slug, isSelected, drawerOpened }) => {
  return (
    <ListItemButton
      selected={isSelected}
      sx={{ pl: 4 }}
      component={ViewStateLink}
      to={slug}
      state={{ drawerOpened }}
    >
      <ListItemText primary={title} />
    </ListItemButton>
  )
}

const DocumentationGroup = ({ name, content, drawerOpened, openDrawer }) => {
  const [open, setOpen] = React.useState(true)

  return (
    <>
      <ListItemButton
        onClick={() => {
          if (drawerOpened) {
            setOpen(!open)
          } else {
            openDrawer()
            setOpen(true)
          }
        }}
      >
        <ListItemIcon>
          <MenuBook />
        </ListItemIcon>
        {drawerOpened && (
          <>
            <ListItemText primary={name} />
            {open ? <ExpandLess /> : <ExpandMore />}
          </>
        )}
      </ListItemButton>
      {drawerOpened && (
        <Collapse in={open} timeout="auto" unmountOnExit>
          <List component="div" disablePadding>
            {content.map(documentation => (
              <DocumentationLink
                key={documentation.slug}
                drawerOpened={drawerOpened}
                {...documentation}
              />
            ))}
          </List>
        </Collapse>
      )}
    </>
  )
}

const sortGroupContent = (a, b) => a.order - b.order

export const DocumentationNavigation = ({ drawerOpened, openDrawer }) => {
  const data = useStaticQuery(query)
  const location = useLocation()

  const groupNameToIndex = new Map()
  const groups = data.site.siteMetadata.groupsOrder.map((groupName, index) => {
    groupNameToIndex.set(groupName, index)
    return { name: groupName, content: [] }
  })

  const documents = data.allMarkdownRemark.nodes.map(node => ({
    ...node.frontmatter,
    slug: `/docs${node.fields.slug}`,
    isSelected: location.pathname === `/docs${node.fields.slug}`,
  }))

  documents.forEach(document => {
    const { group: groupName } = document
    const groupIndex = groupNameToIndex.get(groupName)

    if (typeof groupIndex !== "number") {
      const { title, slug } = document
      throw new Error(
        `Unknown group '${groupName}' associated with '${title}' (at ${slug})`
      )
    }
    groups[groupIndex].content.push(document)
  })

  groups.forEach(group => group.content.sort(sortGroupContent))

  return (
    <List>
      {groups.map(({ name, content }) => (
        <DocumentationGroup
          location={location}
          key={name}
          name={name}
          content={content}
          drawerOpened={drawerOpened}
          openDrawer={openDrawer}
        />
      ))}
    </List>
  )
}
