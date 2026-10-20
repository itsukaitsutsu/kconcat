--[[
    Script Name: SceneMgrModule.lua
    Version: 1.1
    Update Data: 2024.07.19
--]]

--[[  ****************************************
    storge the loaded prefab entity info
--    **************************************** ]]  
local PrefabEntity = {}
PrefabEntity.__index = PrefabEntity
function PrefabEntity.new(_sticker, _prefab, _parent, _name)
    self.sticker = _sticker
    if _parent then
        self.entity = _prefab:instantiateToEntity(self.entity.scene, _parent.entity)
        self.parent = _parent
    else
        self.entity = _prefab:instantiateToEntity(self.entity.scene, _sticker.entity)
        self.parent = _sticker.trans
    end
    self.trans = self.entity:getComponent("Transform")
    if _name then
        self.entity.name = _name
    end
    return self
end

function PrefabEntity:getSubEntities()
    if self.subEntities == nil then
        local subEntities = Amaz.Vector()
        for i = 0, self.trans.children:size() - 1 do
            local subEntity = self.trans.children:get(i).entity
            subEntities:pushBack(subEntity)
        end
        self.subEntities = subEntities
    end
    return self.subEntities
end

function PrefabEntity:destroy()
    self.parent.children:erase(self.trans)
    self.entity.scene:removeEntity(self.entity)
    self.entity = nil
    self = {}
end


--[[  ****************************************
    storge the CloneText info
--    **************************************** ]]  
local CloneText = {}
CloneText.__index = CloneText

function CloneText.new(_sticker, _name, _parent, _layer)
    local self = setmetatable({}, CloneText)

    local parentEntity = _sticker.entity
    self.entity = parentEntity.scene:createEntity("cloneText_".._name)

    if _layer ~= nil then
        self.entity.layer = _layer
    else
        self.entity.layer = _sticker.entity.layer
    end
    
    self.trans = self.entity:addComponent("Transform")
    if _sticker.richText then
        self.richText = self.entity:cloneComponentOf(_sticker.richText)
    end
    self.renderer = self.entity:addComponent("MeshRenderer")
    self.renderer.sortingOrder = _sticker.renderer.sortingOrder
    self.richText.richStr = _sticker.richText.richStr
    self.richText.letters = self.richText.letters   --trick
    self.richText:forceTypeSetting()
    self.richText.bloomEnable = false

    if _parent then
        self.parent = _parent
        self.trans.parent = _parent
        _parent.children:pushBack(self.trans)
    else
        self.parent = _sticker.parent
        self.trans.parent = _sticker.parent
        _sticker.parent.children:pushBack(self.trans)
    end

    self.richText.canvas.canvasEnabled = false
    for i = 0, self.richText.letters:size() - 1 do
        local letter = self.richText.letters:get(i)
        letter.letterStyle.fontDecoration = 0
    end
    if _sticker.text then
        self.text = self.entity:cloneComponentOf(_sticker.text)
        if self.richText then
            self.text:setTextWrapper(self.richText)
            self.richText:forceTypeSetting()
        end
        if _sticker.text.effectTextParam == nil then
            return self
        end
    end

    ---- clone effectLayers (flower text)-----
    local effectLayers = _sticker.text.effectTextParam.effectLayers
	local cloneEffectTextParam = _sticker.text.effectTextParam

	if cloneEffectTextParam ~= nil and effectLayers ~= nil then
		local cloneEffectLayers = cloneEffectTextParam.effectLayers
		for i = 0, cloneEffectLayers:size() - 1 do 
			if i < effectLayers:size() then
				cloneEffectLayers:get(i).mat = effectLayers:get(i).mat
				cloneEffectLayers:get(i).texture = effectLayers:get(i).texture
			end
		end
	end
    return self
end

function CloneText:destroy()
    self.parent.children:erase(self.trans)
    self.entity.scene:removeEntity(self.entity)
    self.entity = nil
    self = {}
end


--[[  ****************************************
    SceneMgrModule: manage the custom entity
--    **************************************** ]]  
local SceneMgrModule = {}
SceneMgrModule.__index = SceneMgrModule
SceneMgrModule.TEXT_STICKER = "TEXT_STICKER"

--- new func of module
--- receives the 'self' variable of the original script as an argument
function SceneMgrModule.new(_sticker)
    local self = setmetatable({}, SceneMgrModule)
    self.sticker = _sticker
    self.cloneTextList = {}
    self.prefabEntityList = {}
    self.prefabLoader = PrefabEntity
    return self
end

function SceneMgrModule:addCloneText(_parent, _layer)
    local cloneText = CloneText.new(self.sticker, #self.cloneTextList, _parent, _layer)
    table.insert(self.cloneTextList, cloneText)
    return cloneText
end

function SceneMgrModule:loadPrefab(_prefab, _parent)
    local prefabEntity = PrefabEntity.new(self.sticker, _prefab, _parent, "prefab_"..#self.prefabEntityList)
    table.insert(self.prefabEntityList, prefabEntity)
    return prefabEntity
end

--- init the script data (optional)
function SceneMgrModule:init()
end

--- seek per frame (optional)
function SceneMgrModule:seek()
end

--- reset the script data (optional)
function SceneMgrModule:reset()
    if self.cloneTextList then
        for i = 1, #self.cloneTextList do
            local ct = self.cloneTextList[i]
            ct:destroy()
        end
        self.cloneTextList = {}
    end
    if self.prefabEntityList then
        for i = 1, #self.prefabEntityList do
            local pe = self.prefabEntityList[i]
            pe:destroy()
        end
        self.prefabEntityList = {}
    end

    collectgarbage("collect")
end

return SceneMgrModule