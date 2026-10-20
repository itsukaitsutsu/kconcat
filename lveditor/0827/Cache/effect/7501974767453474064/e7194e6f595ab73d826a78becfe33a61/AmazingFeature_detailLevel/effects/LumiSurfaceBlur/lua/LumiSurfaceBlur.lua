local isEditor = (Amaz.Macros and Amaz.Macros.EditorSDK) and true or false
local exports = exports or {}
local LumiSurfaceBlur = LumiSurfaceBlur or {}
LumiSurfaceBlur.__index = LumiSurfaceBlur
---@class LumiSurfaceBlur : ScriptComponent
---@field blurIntensity double [UI(Range={0.0, 1}, Slider)]
---@field InputTex Texture
---@field OutputTex Texture

local AE_EFFECT_TAG = 'AE_EFFECT_TAG LumiTag'

function LumiSurfaceBlur.new(construct, ...)
    local self = setmetatable({}, LumiSurfaceBlur)

    self.__lumi_type = "lumi_obj"
    self.__lumi_rt_pingpong_type = "custom"

    self.radius = 10
    self.blurIntensity = 1.0
    self.InputTex = nil
    self.OutputTex = nil

    self.NormalizationSize = 1000.
    self.SurfaceBlurThreshold = 0.05
    return self
end

function LumiSurfaceBlur:setEffectAttr(key, value, comp)
    local function _setEffectAttr(_key, _value, _force)
        if _force or self[_key] ~= nil then
            self[_key] = _value
            if comp and comp.properties ~= nil then
                comp.properties:set(_key, _value)
            end
        end
    end

    _setEffectAttr(key, value)
end

local function getXYScale(width, height)
    --[[
        Description: get XY scale for adjusting width/height ratio.
    ]]
    -- the following computation for baseSize is to avoid too small or too large width/height ratio.
    local size1 = math.min(width, height)
    local size2 = math.max(width, height) / 2.
    local baseSize = math.max(size1, size2)

    local xScale = baseSize / width
    local yScale = baseSize / height
    return xScale, yScale
end

function LumiSurfaceBlur:onStart(comp)
    self.entity = comp.entity
    self.TAG = AE_EFFECT_TAG .. ' ' .. self.entity.name
    Amaz.LOGI(self.TAG, 'onStart')

    -- Use entity instead of scene 
    self.camera = self.entity:searchEntity("CameraSurfaceBlur"):getComponent("Camera")
    self.material = self.entity:searchEntity("EntitySurfaceBlur"):getComponent("MeshRenderer").material
end

function LumiSurfaceBlur:onUpdate(comp, deltaTime)
    local textureWidth = self.OutputTex.width
    local textureHeight = self.OutputTex.height

    -- set the input and output textures to be displayed
    if self.OutputTex then
        self.camera.renderTexture = self.OutputTex
    end
    self.material:setTex("u_inputTexture", self.InputTex)

    -- surface blur parameters
    local xScale, yScale = getXYScale(textureWidth, textureHeight)
    local steps = self.blurIntensity * 5.0
    local dx = steps * xScale / self.NormalizationSize
    local dy = steps * yScale / self.NormalizationSize

    self.material:setFloat("u_threshold", self.SurfaceBlurThreshold)
    self.material:setFloat("u_stepX", dx)
    self.material:setFloat("u_stepY", dy)
end

exports.LumiSurfaceBlur = LumiSurfaceBlur
return exports
