local toml_edit = require("toml_edit")

local make = toml_edit.document

it("is userdata", function()
  assert.is_userdata(make())
end)

describe("Deref<Target = Table>", function()
  it("pub fn fmt(&mut self) #extended", function()
    local content = [[
[owner]
name              =            "Preston"
age = 42
bio = "Rustacean"
    ]]
    local expected = [[
[owner]
name = "Preston"
age = 42
bio = "Rustacean"
    ]]
    local doc = make(content)
    doc:format()
    assert.equal(expected, tostring(doc))
  end)
end)
