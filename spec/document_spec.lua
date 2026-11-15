local toml_edit = require("toml_edit")

local make = toml_edit.document

local Examples = {}

function Examples.full()
    --- @type string
    local content = [==[
    # meta.wip = false
    meta.active = true
    meta.release_date = 1979-05-27T07:32:00
    contact = {
      email = "support@email.com",
      height = 1.78
    }
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
    return doc, content
  
end

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

describe("Item", function()
  it("document value is lua table", function()
    local doc = make()
    local value = doc:deserialize()
    assert.is_table(value)
  end)

  it("document value supports all value variants", function()
    local doc = Examples.full()
    local value = doc:deserialize()
    assert.equal(nil, value.meta.wip)
    assert.equal("support@email.com", value.contact.email)
    assert.equal(1.78, value.contact.height)
    assert.equal("1979-05-27T07:32:00", value.meta.release_date)
    assert.equal(true, value.meta.active)
    assert.equal("/boot/", value.workspace.members.root[1])
    assert.equal(2, #value.schema)
    assert.equal("John", value.schema[1].name)
    assert.equal(1, value.schema[1].age)
    assert.equal("Mary", value.schema[2].name)
    assert.equal(-2, value.schema[2].age)
    assert.equal(1000, value.schema[2].salary)
  end)

  it("Document.get is Lua 1 based index", function()
    local doc = make([[
    tags = ["admin", 1]
    ]])
    assert.equal("string", doc:get({"tags", 1}).type_name)
    assert.equal("integer", doc:get({"tags", 2}).type_name)
  end)

  it("Item type_name", function()
    local doc = Examples.full()
    assert.equal("none", doc:get({"meta", "wip"}).type_name)
    assert.equal("string", doc:get({"workspace", "members", "root", 1}).type_name)
    assert.equal("integer", doc:get({"schema", 2, "age"}).type_name)
    assert.equal("float", doc:get({"contact", "height"}).type_name)
    assert.equal("boolean", doc:get({"meta", "active"}).type_name)
    assert.equal("datetime", doc:get({"meta", "release_date"}).type_name)
    assert.equal("inline table", doc:get("contact").type_name)
    assert.equal("datetime", doc:get({"meta", "release_date"}).type_name)
    assert.equal("array of tables", doc:get("schema").type_name)
  end)

  it("Item deserialize", function()
    local doc = Examples.full()
    assert.equal("/boot/", doc:get({"workspace", "members", "root", 1}):deserialize())
  end)
end)

