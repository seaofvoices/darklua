const { process_code } = require("darklua-wasm/darklua_wasm")

test("process empty string", () => {
  expect(process_code("")).toEqual("")
})

test("process with default rules", () => {
  expect(process_code("return 1 + 1")).toEqual("return 2")
})

test("process with no rules", () => {
  expect(process_code("return 1 + 1", { rules: [] })).toEqual("return 1 + 1")
})

test("process with no rules (string config)", () => {
  expect(process_code("return 1 + 1", "{ rules: [] }")).toEqual("return 1 + 1")
})

test("process and inject global", () => {
  const options = {
    rules: [
      { rule: "inject_global_value", identifier: "CONSTANT", value: true },
    ],
  }
  expect(process_code("return CONSTANT", options)).toEqual("return true")
})
