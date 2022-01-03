import * as React from "react"

class RulesStack {
  constructor(rules, updateRules) {
    this.rules = rules
    this._updateRules = updateRules
  }

  replaceRule(index, rule) {
    const newRules = [...this.rules]
    newRules[index] = rule
    this._updateRules(newRules)
  }

  pushRule(ruleName, ruleProperties) {
    const newRules = [...this.rules]
    newRules.push({
      name: ruleName,
      properties: ruleProperties,
    })
    this._updateRules(newRules)
  }

  deleteRule(index) {
    const newRules = [...this.rules]
    newRules.splice(index, 1)
    this._updateRules(newRules)
  }

  getRules() {
    return this.rules
  }

  replaceWithDarkluaConfig(config) {
    this._updateRules(
      config.rules.map(rule => {
        if (typeof rule == "string") {
          return { name: rule }
        }
        if (Object.keys(rule).length === 1) {
          return {
            name: rule.rule,
          }
        } else {
          const properties = { ...rule }
          delete properties.rule
          return {
            name: rule.rule,
            properties,
          }
        }
      })
    )
  }

  getDarkluaConfig() {
    return {
      rules: this.rules
        .filter(rule => !rule.disabled)
        .map(rule => {
          if (!rule.properties || Object.keys(rule.properties).length === 0) {
            return rule.name
          } else {
            return {
              rule: rule.name,
              ...rule.properties,
            }
          }
        }),
    }
  }
}

export const RulesStackContext = React.createContext()

const DEFAULT_RULES = [
  { name: "remove_comments" },
  { name: "remove_spaces" },
  {
    name: "inject_global_value",
    properties: {
      identifier: "__DEV__",
      value: false,
    },
  },
  { name: "compute_expression" },
  { name: "remove_unused_if_branch" },
  { name: "remove_unused_while" },
  { name: "remove_empty_do" },
  { name: "remove_method_definition" },
  { name: "remove_function_call_parens" },
]

export const RulesStackProvider = ({ children }) => {
  const [rules, setRules] = React.useState(DEFAULT_RULES)

  const rulesStack = new RulesStack(rules, setRules)

  return (
    <RulesStackContext.Provider value={rulesStack}>
      {children}
    </RulesStackContext.Provider>
  )
}
