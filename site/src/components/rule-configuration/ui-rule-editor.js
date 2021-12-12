import * as React from "react"
import {
  IconButton,
  List,
  ListItem,
  MenuItem,
  Select,
  Switch,
  Typography,
} from "@mui/material"
import DeleteIcon from "@mui/icons-material/Delete"
import { RulesStackContext } from "../rules-stack"
import { Box } from "@mui/system"

const ALL_RULES = [
  "remove_comments",
  "remove_spaces",
  "inject_global_value",
  "compute_expression",
  "remove_unused_if_branch",
  "remove_unused_while",
  "remove_empty_do",
  "remove_method_definition",
  "remove_function_call_parens",
]

const RuleSelector = ({ selectedName, onChange }) => {
  return (
    <Select
      value={selectedName}
      label="Rule"
      onChange={e => onChange(e.target.value)}
      autoWidth={true}
    >
      {ALL_RULES.map(ruleName => (
        <MenuItem key={ruleName} value={ruleName} dense={true}>
          {ruleName}
        </MenuItem>
      ))}
    </Select>
  )
}

const RuleItem = ({ rule, index, rulesStack }) => {
  return (
    <ListItem
      dense={true}
      primary={rule.name}
      secondaryAction={
        <IconButton
          size="small"
          edge="end"
          aria-label="delete"
          onClick={() => rulesStack.deleteRule(index)}
        >
          <DeleteIcon />
        </IconButton>
      }
    >
      <Switch
        size="small"
        checked={!rule.disabled}
        onChange={event => {
          rulesStack.replaceRule(index, {
            ...rule,
            disabled: !event.target.checked,
          })
        }}
      />
      <RuleSelector
        selectedName={rule.name}
        onChange={newRuleName => {
          rulesStack.replaceRule(index, {
            name: newRuleName,
            disabled: rule.disabled,
          })
        }}
      />
    </ListItem>
  )
}

const UiRuleEditor = () => {
  const rulesStack = React.useContext(RulesStackContext)
  const generateRules = (rule, index) => (
    <RuleItem key={index} rule={rule} index={index} rulesStack={rulesStack} />
  )

  return (
    <Box>
      <Typography variant="h6">Rules</Typography>
      <List dense={true} disablePadding={true}>
        {rulesStack.getRules().map(generateRules)}
      </List>
    </Box>
  )
}

export default UiRuleEditor
