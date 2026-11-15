---@meta

---Group related tests together
---@param name string
---@param block fun()
function describe(name, block) end

---Alias for describe
---@param name string
---@param block fun()
function context(name, block) end

---Define an individual test case
---@param name string
---@param block fun()
function it(name, block) end

---Alias for it
---@param name string
---@param block fun()
function spec(name, block) end

---Placeholder for an unimplemented test
---@param name string
---@param block? fun()
function pending(name, block) end

---Run once before all tests in a describe block
---@param block fun()
function setup(block) end

---Run once after all tests in a describe block
---@param block fun()
function teardown(block) end

---Run before each test in a describe block
---@param block fun()
function before_each(block) end

---Run after each test in a describe block
---@param block fun()
function after_each(block) end

---Run immediately after each test completes
---@param block fun()
function finally(block) end

---Create a spy wrapping an existing function or empty function
---@param target? function
---@return table
function spy(target) end

---Create a stub on a table method
---@param object table
---@param method string
---@param ... any
---@return table
function stub(object, method, ...) end

---Create a mock object wrapping a table
---@param object table
---@param stub_methods? boolean
---@return table
function mock(object, stub_methods) end

---@class luassert
---@field is luassert
---@field are luassert
---@field has luassert
---@field does luassert
---@field is_not luassert
---@field are_not luassert
---@field has_not luassert
---@field does_not luassert
---@field equal fun(expected: any, actual: any, message?: string)
---@field equals fun(expected: any, actual: any, message?: string)
---@field same fun(expected: any, actual: any, message?: string)
---@field truthy fun(value: any, message?: string)
---@field falsy fun(value: any, message?: string)
---@field is_true fun(value: any, message?: string)
---@field is_false fun(value: any, message?: string)
---@field is_function fun(value: any, message?: string)
---@field is_table fun(value: any, message?: string)
---@field is_userdata fun(value: any, message?: string)
---@field is_nil fun(value: any, message?: string)
---@field is_not_nil fun(value: any, message?: string)
---@field error fun(fn: function, err?: string|function)
---@field has_error fun(fn: function, err?: string|function)
---@field no_error fun(fn: function)
---@field matches fun(pattern: string, actual: string, message?: string)
---@field spy fun(fn: function): table
---@field stub fun(tbl: table, key: string): table
assert = {} ---@type luassert
