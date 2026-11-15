--- @meta

local toml_edit = {}

--- Parse a document from a string, returns editable content
--- @param str string
--- @return any
function toml_edit.parse(str)
end

--- Parse a document from a string as a table
--- @param str string
--- @return any
function toml_edit.parse_as_tbl(str)
end

--- Full feature api of toml_edit, parse or create one from scratch
--- @param str? string
--- @return toml_edit.Document
function toml_edit.document(str)
end

--- @alias toml_edit.Value nil | string | number | boolean | toml_edit.Value[] | table<string, toml_edit.Value> | toml_edit.Datetime

--- @class toml_edit.Datetime

--- @class toml_edit.Item
--- @field value toml_edit.Value The lua representation of the TOML

--- @class toml_edit.Collection
--- @field value toml_edit.Value The lua representation of the TOML
--- @field trailing string Content after the last key-vaue pair

--- @class toml_edit.Document : toml_edit.Item, toml_edit.Collection
local Document = {}

--- Auto formats the table.
--- Note: toml_edit's `Table::fmt()` is not recursive — it only formats the root table.
--- This method formats the root table and then iterates over all items to format
--- each sub-table explicitly, so that sections like `[owner]` also get normalized whitespace.
function Document:format()
end

return toml_edit
