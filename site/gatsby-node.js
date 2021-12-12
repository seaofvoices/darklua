const path = require(`path`)
const { createFilePath } = require(`gatsby-source-filesystem`)

exports.onCreateWebpackConfig = ({ actions }) => {
  actions.setWebpackConfig({
    experiments: {
      syncWebAssembly: true,
    },
  })
}

exports.createPages = async ({ graphql, actions, reporter }) => {
  const { createPage } = actions

  // Define a template for blog post
  const docsPage = path.resolve(`./src/templates/docs-page.js`)

  // Get all markdown documentation posts sorted by date
  const result = await graphql(
    `
      {
        allMarkdownRemark(limit: 1000) {
          nodes {
            id
            fields {
              slug
            }
          }
        }
      }
    `
  )

  if (result.errors) {
    reporter.panicOnBuild(
      `There was an error loading the documentation content`,
      result.errors
    )
    return
  }

  const documentionPages = result.data.allMarkdownRemark.nodes

  if (documentionPages.length == 0) {
    reporter.panicOnBuild(`No documentation page was found`)
    return
  }

  documentionPages.forEach((page, index) => {
    const previousPostId = index === 0 ? null : documentionPages[index - 1].id
    const nextPostId =
      index === documentionPages.length - 1
        ? null
        : documentionPages[index + 1].id

    const path = `/docs${page.fields.slug}`

    createPage({
      path,
      component: docsPage,
      context: {
        id: page.id,
        previousPostId,
        nextPostId,
      },
    })
  })
}

exports.onCreateNode = ({ node, actions, getNode }) => {
  const { createNodeField } = actions

  if (node.internal.type === `MarkdownRemark`) {
    const value = createFilePath({ node, getNode })

    createNodeField({
      name: `slug`,
      node,
      value,
    })
  }
}

exports.createSchemaCustomization = ({ actions }) => {
  const { createTypes } = actions

  // Explicitly define the siteMetadata {} object
  // This way those will always be defined even if removed from gatsby-config.js

  // Also explicitly define the Markdown frontmatter
  // This way the "MarkdownRemark" queries will return `null` even when no
  // blog posts are stored inside "content/blog" instead of returning an error
  createTypes(`
    type SiteSiteMetadata {
      siteUrl: String
      social: Social
    }

    type Author {
      name: String
      summary: String
    }

    type Social {
      twitter: String
    }

    type MarkdownRemark implements Node {
      frontmatter: Frontmatter
      fields: Fields
    }

    type Frontmatter {
      title: String
      description: String
      date: Date @dateformat
    }

    type Fields {
      slug: String
    }
  `)
}
