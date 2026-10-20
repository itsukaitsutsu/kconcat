--[[
    Script Name: TextSticker.lua
    Version: 1.2
    Update Data: 2024.04.18
--]]
local isEditor = (Amaz.Macros and Amaz.Macros.EditorSDK) and true or false
local exports = exports or {}
local TextSticker = TextSticker or {}
---@class TextSticker : ScriptComponent
---@field duration number
---@field progress number [UI(Range={0.0, 1.0}, Slider)]
---@field autoPlay boolean
TextSticker.__index = TextSticker

TextSticker.CustomScriptName = "TextAnim"

--- get the root path of sticker
function TextSticker:getRootDir()
    return self.comp.assetMgr.rootDir
end

--- load asset to sticker, Param[path: relative path]
function TextSticker:loadAsset(path)
    local asset = nil
    asset = self.comp.assetMgr:SyncLoad(path)
    return asset
end

function TextSticker:setTextBloomState(state)
    if self.richText then
        self.oriBloomState = self.richText.bloomEnable
        self.richText.bloomEnable = state
    end
end

-- func related to modules
function TextSticker:_moduleCall(callbackName, ...)
    if self.modules then
        for _, moduleObj in pairs(self.modules) do
            local cn = callbackName
            if moduleObj[cn] then
                moduleObj[cn](moduleObj, ...)
            end
        end
    end
end

function TextSticker:getModule(name)
    if self.modules and self.modules[name] then
        return self.modules[name]
    end
    return nil
end

function TextSticker:stickerRegistingModules(moduleNameList)
    if self["_registerModule"] and moduleNameList then
        for i = 1, #moduleNameList do
            local mn = moduleNameList[i]
            self.modules[mn] = self:_registerModule("modules/"..mn)
        end
    end
end

--[[
    register other modules
]]
function TextSticker:_registerModule(_module_path)
    local module = includeRelativePath(_module_path)
    if module and module.TEXT_STICKER then
        module = module.new(self)
    end
    return module
end

function TextSticker:registingModule(key, moduleName)
    self.modules[key] = self:_registerModule("modules/"..moduleName)
end

--[[
    register AETools
]]
function TextSticker.getAEData()
    local aeTools = includeRelativePath("modules/AETools")
    local aeData = includeRelativePath("modules/AEData")
    local attrs = aeTools.new(aeData.ae_attribute)
    return attrs
end

function TextSticker:_call(_name, ...)
    if self[_name] then
        self[_name](self, ...)
    end
end
-----]]

--[[
    storage the ori letters
--]]
function TextSticker:getOriLetters()
    if self.oriLettersClone == nil and self.oriLetters then
        self.oriLettersClone = self.oriLetters:clone()
    end
    return self.oriLettersClone
end

--[[
    CALL when load the caption data
--]]
function TextSticker:onSetProperty(key, value)
    self:_moduleCall("onSetProperty", key, value)
end

function TextSticker:beforeTextSystemUpdate(comp)
    self:_call("textBeforeUpdate", comp)
    self:_moduleCall("beforeUpdate", comp)
end

function TextSticker:afterTextSystemUpdate(comp)
    self:_call("textAfterUpdate", comp)
    self:_moduleCall("afterUpdate", comp)
end


function TextSticker.new()
    local self = {}
    local textAnim = includeRelativePath(TextSticker.CustomScriptName)
    if textAnim[TextSticker.CustomScriptName] then
        setmetatable(TextSticker, textAnim[TextSticker.CustomScriptName])
    else
        setmetatable(TextSticker, textAnim)
    end
    setmetatable(self, TextSticker)
    self.first = true
    self.duration = 1.0
    self.curTime = 0.0
    self.progress = 0
    self.autoPlay = false
    return self
end

--[[ ******************************
    CALL when load the sticker
     ******************************]]
function TextSticker:onStart(comp) 
    self:_call("textNew", comp)
    --[[
        attribute for sticker
    --]] 
    -- business attr
    self.curTime = 0.0
    -- self.progress = 0

    -- Runtime ---
    self.frame = 0
    self.first = true
    
    --[[
        normal engine attribute 
    --]] 
    self.comp = comp
    self.entity = self.comp.entity
	self.text = self.comp.entity:getComponent("SDFText")
    if self.text == nil then
        local text = self.comp.entity:getComponent('Text')
        if text ~= nil then
			self.text = self.comp.entity:addComponent('SDFText')
            self.text:setTextWrapper(text)
        end
    end
	self.richText = self.comp.entity:getComponent("Text")
    self.trans = self.comp.entity:getComponent("Transform")
    self.renderer = self.comp.entity:getComponent("MeshRenderer")
    self.parent = self.trans.parent
	if self.text ~= nil then
		self.renderer = self.comp.entity:getComponent("MeshRenderer")
	else
		self.renderer = self.comp.entity:getComponent("Sprite2DRenderer")
	end
    self.rootDir = self:getRootDir()
    
    --[[
        register part
    --]] 
    if self.registerModules then
        self.modules = {}
        self:stickerRegistingModules(self.registerModules)
    end

    self:_call("textStart", comp)
    
    if isEditor then
        self.curTime = 0
        self:onEnter()
    end
end

--[[ ******************************
    CALL when enter the anim or after clear function
     ******************************]]
function TextSticker:onEnter()
    self:_moduleCall("init")
    self.first = true
    self.oriStr = self.richText.str
    if self.richText then
        self.oriLetters = self.richText.letters:clone()
        self.oriLettersClone = nil
    end
    if self.trans then
        self.oriTextLocalPosition = self.trans.localPosition
    end
    self.width = Amaz.BuiltinObject:getOutputTextureWidth()
    self.height = Amaz.BuiltinObject:getOutputTextureHeight()

    self:_call("textInit")
end

function TextSticker:onUpdate(comp, time)
    if isEditor then
        if self.autoPlay then
            self.curTime = self.curTime + time
            self.progress = self.curTime % self.duration / self.duration
        else
            self.curTime = 0
        end
        self:seek(self.curTime)
    end
end

function TextSticker:seek(time)
    if isEditor then
    else
        self.progress = time % (self.duration+0.0001) / (self.duration+0.0001)
    end

    self.width = Amaz.BuiltinObject:getOutputTextureWidth()
    self.height = Amaz.BuiltinObject:getOutputTextureHeight()

    self:_moduleCall("seek", time)
    self:_call("textSeek", time)
end

--[[ ******************************
    reset the text data
     ******************************]]
function TextSticker:_reset()
    self.first = true
    if self.richText then
        self.richText.letters = self.oriLetters
    end

	if self.richText then
        self.richText.canvas.renderToRT = false
        self.richText.canvas.targetRTExtraSize = Amaz.Vector2f(0.0, 0.0)
	end
    if self.trans and self.oriTextLocalPosition then
        self.trans.localScale = Amaz.Vector3f(1, 1, 1)
        self.trans.localPosition = self.oriTextLocalPosition
        self.trans.localEulerAngle = Amaz.Vector3f(0, 0, 0)
    end

    if self.oriBloomState ~= nil then
        self.richText.bloomEnable = self.oriBloomState
    end

    self:_moduleCall("reset")
    self:_call("textReset")
end

function TextSticker:setDuration(duration)
    self.duration = duration
end

--[[ ******************************
    only CALL when change the style and trans
     ****************************** ]]
function TextSticker:clear()
    self:_reset()
end

--[[ ******************************
    only CALL when leave the text anim
     ****************************** ]]
function TextSticker:onLeave()
    self:_reset()
end

exports.TextSticker = TextSticker
return exports

