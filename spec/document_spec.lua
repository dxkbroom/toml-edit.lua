local toml_edit = require("toml_edit")

local make = toml_edit.document

it("is userdata", function()
  assert.is_userdata(make())
end)

describe("DocumentMut", function()
  it("pub fn trailing/set_trailing", function()
    local content = [[title = "test"
]]
    local doc = make(content)
    assert.equal("", doc.trailing)
    doc.trailing = "# a comment"
    assert.equal("# a comment", doc.trailing)
  end)
end)

describe("Table", function()
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

describe("Item.value", function()
  it("document is table", function()
    local doc = make()
    assert.is_table(doc.value)
  end)

  it("document supports all value variants", function()
    local content = [==[
    # meta.wip = false
    meta.active = true
    meta.release_date = 1979-05-27T07:32:00
    [workspace.members]
    root = ["/boot/"]
    [[schema]]
    name = "John"
    age = 1
    [[schema]]
    name = "Mary"
    age = -2
    salary = 1_000
    ]==]
    local doc = make(content)
    assert.equal(nil, doc.value.meta.wip)
    assert.equal("1979-05-27T07:32:00", doc.value.meta.release_date)
    assert.equal("/boot/", doc.value.workspace.members.root[1])
    assert.equal(2, #doc.value.schema)
    assert.equal("John", doc.value.schema[1].name)
    assert.equal(1, doc.value.schema[1].age)
    assert.equal("Mary", doc.value.schema[2].name)
    assert.equal(-2, doc.value.schema[2].age)
    assert.equal(1000, doc.value.schema[2].salary)
  end)
end)
