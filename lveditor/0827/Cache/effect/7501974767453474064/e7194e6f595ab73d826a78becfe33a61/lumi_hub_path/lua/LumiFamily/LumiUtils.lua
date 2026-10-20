LUMI_VERSION = "0.0.1"

LUMI_SUCCESS = 0
LUMI_ERROR = -201
LUMI_FILE_NOT_FOUND = -202
LUMI_EFFECT_ID_NOT_FOUND = -203
LUMI_INVALID_PARA = -204
LUMI_TARGET_NOT_FOUND = -205
LUMI_INVALID_TARGET_TYPE = -206
LUMI_LUMI_OBJECT_ERROR = -207
LUMI_INVALID_PREFAB = -208

LUMI_TARGET_TYPE_INPUT_RT = 0
LUMI_TARGET_TYPE_OUTPUT_RT = 1
LUMI_TARGET_TYPE_CAMERA_RT = 2
LUMI_TARGET_TYPE_RENDERER = 3

LUMI_NODE_EFFECT = 1
LUMI_NODE_CAMERA = 2
LUMI_NODE_EXTRA = 3

LUMI_TARGET_INPUT_RT_IDENTIFIER = "__LUMI__INPUT"
LUMI_TARGET_OUTPUT_RT_IDENTIFIER = "__LUMI__OUTPUT"

LUMI_START_TIME = "startTime"
LUMI_END_TIME = "endTime"
LUMI_CURRENT_TIME = "curTime"

LUMI_USE_ORIGINAL_RT_SIZE = "useOriginalRTSize"
LUMI_WILL_CHANGE_ORIGINAL_RT_SIZE = "willChangeOriginalRTSize"

LUMI_EXCLUDE_ENTITY_NAMES = {"Painter2DLine", "Painter3DLine", "PainterBlock", "Painter2DLine", "Painter3DLine", "PainterBlock", "Painter2DLine", "Painter3DLine", "PainterBlock"}

local function dump(o)
    if type(o) == 'table' then
       local s = '{ '
       for k,v in pairs(o) do
          if type(k) ~= 'number' then k = '"'..tostring(k)..'"' end
          if type(v) == string then
            s = s .. '['..k..'] = "' .. dump(v) .. '",'
          else
            s = s .. '['..k..'] = ' .. dump(v) .. ','
          end
        --   s = s .. dump(v) .. ', '
       end
       return s .. '} '
    else
       return tostring(o)
    end
end

local function getMessage(...)
    local arg = { ... }
    local msg = ""
    for k, v in pairs(arg) do
        msg = msg .. dump(v) .. " "
    end
    return msg
end

function print(...)
    Amaz.LOGI("lumi", getMessage(...))
end

function printe(...)  -- print error
    Amaz.LOGE("lumi", getMessage(...))
end

function fileExists(path)
    local file = io.open(path, "r")
    if file then
        io.close(file)
        return true
    else
        return false
    end
end

function table.getIndex(t, element)
    for k, v in pairs(t) do
        if v == element then
            return k
        end
    end
    return -1
end

function table.exists(t, element)
    for k, v in pairs(t) do
        if v == element then
            return true
        end
    end
    return false
end

function table.removeElement(t, element)
    local index = table.getIndex(t, element)
    if index > 0 then
        table.remove(t, index)
    end
end

-- function table.getOne(t, element)
--     local index = table.getIndex(t, element)
--     if index > 0 then
--         local value = t[index]
--         table.remove(t, index)
--         return value
--     end
--     return nil
-- end

-- function table.keys(t)
--     local ret = {}
--     for k, v in pairs(t) do
--         table.insert(ret, k)
--     end
--     return ret
-- end

-- function table.values(t)
--     local ret = {}
--     for k, v in pairs(t) do
--         table.insert(ret, v)
--     end
--     return ret
-- end

LumiTable = LumiTable or {}
LumiTable.__index = LumiTable
function LumiTable.new()
    local self = setmetatable({}, LumiTable)
    self.lumiTable = {}
    self.unusedLumiEffects = {}
    self.effectIDTolumiTarget = {}
    return self
end

-- LumiTable: from lumiTarget to effectIDList
function LumiTable:insert(lumiTarget, effectID)
    if self.lumiTable[lumiTarget] == nil then
        self.lumiTable[lumiTarget] = {}
    end
    table.insert(self.lumiTable[lumiTarget], effectID)
    -- print("enableLumiEffect 15 self.lumiTable insert", self.lumiTable)
end

function LumiTable:reuse(lumiTarget, effectID)
    if self.lumiTable[lumiTarget] == nil then
        self.lumiTable[lumiTarget] = {}
    end
    table.insert(self.lumiTable[lumiTarget], effectID)
    table.removeElement(self.unusedLumiEffects, effectID)
    if self.effectIDTolumiTarget[effectID] ~= nil then
        self.effectIDTolumiTarget[effectID] = nil
        -- table.remove(self.effectIDTolumiTarget[effectID], lumiTarget)
    end
end

function LumiTable:remove(effectID, enableLumiEffectCache)
    if self.effectIDTolumiTarget[effectID] == nil then
        self.effectIDTolumiTarget[effectID] = {}
    end
    for k1,v1 in pairs(self.lumiTable) do
        for k2,v2 in pairs(v1) do
            if v2 == effectID then
                table.remove(v1, k2)
                if enableLumiEffectCache then
                    table.insert(self.unusedLumiEffects, effectID)
                    table.insert(self.effectIDTolumiTarget[effectID], k1)
                end
                return LUMI_SUCCESS
            end
        end
    end
    return LUMI_EFFECT_ID_NOT_FOUND
end

function LumiTable:clearForTarget(lumiTarget, enableLumiEffectCache)
    -- target = getTarget(targetType, target)
    for k, v in pairs(self.lumiTable[lumiTarget]) do
        if self.effectIDTolumiTarget[v] == nil then
            self.effectIDTolumiTarget[v] = {}
        end
        if enableLumiEffectCache then
            table.insert(self.unusedLumiEffects, v)
            table.insert(self.effectIDTolumiTarget[v], lumiTarget)
        end
    end
    self.lumiTable[lumiTarget] = nil
    return LUMI_SUCCESS
end

-- function LumiTable:get(targetType, target)
--     -- target = getTarget(targetType, target)
--     return self.lumiTable[target]
-- end

function LumiTable:clear(enableLumiEffectCache)
    for k1, v1 in pairs(self.lumiTable) do
        for _, v2 in pairs(v1) do
            if self.effectIDTolumiTarget[v2] == nil then
                self.effectIDTolumiTarget[v2] = {}
            end
            if enableLumiEffectCache then
                table.insert(self.unusedLumiEffects, v2)
                table.insert(self.effectIDTolumiTarget[v2], k1)
            end
        end
    end
    self.lumiTable = {}
    return LUMI_SUCCESS
end

function LumiTable:getIndex(effectID)
    for k1,v1 in pairs(self.lumiTable) do
        local index = table.getIndex(v1, effectID)
        if index > 0 then
            return k1, index
        end
    end
    return -1, -1
end

-- index: start from 1
function LumiTable:changeOrder(effectID, index)
    local lumiTarget, srcIndex = self:getIndex(effectID)
    if srcIndex < 0 then
        return LUMI_EFFECT_ID_NOT_FOUND
    elseif srcIndex == index then
        return LUMI_SUCCESS
    else
        local effects = self.lumiTable[lumiTarget]
        local count = #effects
        table.remove(effects, srcIndex)
        table.insert(effects, index, effectID)
        return LUMI_SUCCESS
    end
    return LUMI_EFFECT_ID_NOT_FOUND
end

function createRenderTexture(name, width, height)
    local rt = Amaz.RenderTexture()
    rt.width = width
    rt.height = height
    rt.depth = 1
    rt.filterMag = Amaz.FilterMode.FilterMode_LINEAR
    rt.filterMin = Amaz.FilterMode.FilterMode_LINEAR
    rt.filterMipmap = Amaz.FilterMipmapMode.FilterMode_NONE
    rt.attachment = Amaz.RenderTextureAttachment.NONE
    rt.name = name
    return rt
end

function getPingPongTextures(_input_tex, _output_tex, _pingpong_tex, count)
    local pingpong = { _output_tex, _pingpong_tex }
    if count%2 == 0 then
        pingpong = { _pingpong_tex, _output_tex }
    end
    local ret = {}
    for i = 1, count do
        local idx1 = (i-1)%2+1
        local idx2 = (i)%2+1
        if i == 1 then
            table.insert(ret, {input=_input_tex, output=pingpong[idx1]})
        else
            table.insert(ret, {input=pingpong[idx2], output=pingpong[idx1]})
        end
    end
    return ret
end

function setLayer(_cam, _layer)
    local str = "1"
    for i = 1, _layer do
        str = str.."0"
    end
    local dynamic_bitset = Amaz.DynamicBitset.new(str)

    _cam.layerVisibleMask = dynamic_bitset
end

function createLumiCamera(camera)
    local lumiCamera = {camera=camera, entity=camera.entity, inputTexture=camera.inputTexture, renderTexture=camera.renderTexture, order=camera.renderOrder, layerVisibleMask=camera.layerVisibleMask}
    return lumiCamera
end

function createLumiRenderer(renderer)
    local entity = renderer.entity
    local lumiRenderer = {renderer=renderer, entity=entity, layer=entity.layer, order=renderer.sortingOrder, autoSortingOrder=renderer.autoSortingOrder}
    return lumiRenderer
end

function createLumiExtraNode(entity, camera, renderer, lumiTarget)
    return {entity=entity, camera=camera, renderer=renderer, lumiTarget=lumiTarget}
end
