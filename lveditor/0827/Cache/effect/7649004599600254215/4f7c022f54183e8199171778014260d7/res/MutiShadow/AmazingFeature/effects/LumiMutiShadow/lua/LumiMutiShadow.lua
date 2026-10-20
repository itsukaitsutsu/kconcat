local isEditor = (Amaz.Macros and Amaz.Macros.EditorSDK) and true or false
local exports = exports or {}
local LumiMutiShadow = LumiMutiShadow or {}
LumiMutiShadow.__index = LumiMutiShadow
---@class LumiMutiShadow : ScriptComponent
---@field oriRotation number
---@field oriPositionX number
---@field oriPositionY number
---@field oriPivotX number
---@field oriPivotY number
---@field oriScaleX number
---@field oriScaleY number
---@field unifiedScale boolean
---@field oriColor Color [UI(NoAlpha)]
---@field oriAlpha number [UI(Range={0, 1}, Slider)]
---@field isAssociation boolean
---@field layerNum number [UI(Range={0, 8}, Slider)]
---@field rotation_1 number
---@field rotation_2 number
---@field rotation_3 number
---@field rotation_4 number
---@field rotation_5 number
---@field rotation_6 number
---@field rotation_7 number
---@field rotation_8 number
---@field positionX_1 number
---@field positionY_1 number
---@field positionX_2 number
---@field positionY_2 number
---@field positionX_3 number
---@field positionY_3 number
---@field positionX_4 number
---@field positionY_4 number
---@field positionX_5 number
---@field positionY_5 number
---@field positionX_6 number
---@field positionY_6 number
---@field positionX_7 number
---@field positionY_7 number
---@field positionX_8 number
---@field positionY_8 number
---@field scale_1 number
---@field scale_2 number
---@field scale_3 number
---@field scale_4 number
---@field scale_5 number
---@field scale_6 number
---@field scale_7 number
---@field scale_8 number
---@field color_1 Color [UI(NoAlpha)]
---@field color_2 Color [UI(NoAlpha)]
---@field color_3 Color [UI(NoAlpha)]
---@field color_4 Color [UI(NoAlpha)]
---@field color_5 Color [UI(NoAlpha)]
---@field color_6 Color [UI(NoAlpha)]
---@field color_7 Color [UI(NoAlpha)]
---@field color_8 Color [UI(NoAlpha)]
---@field alpha_1 number [UI(Range={0, 1}, Slider)]
---@field alpha_2 number [UI(Range={0, 1}, Slider)]
---@field alpha_3 number [UI(Range={0, 1}, Slider)]
---@field alpha_4 number [UI(Range={0, 1}, Slider)]
---@field alpha_5 number [UI(Range={0, 1}, Slider)]
---@field alpha_6 number [UI(Range={0, 1}, Slider)]
---@field alpha_7 number [UI(Range={0, 1}, Slider)]
---@field alpha_8 number [UI(Range={0, 1}, Slider)]
---@field InputTex Texture
---@field OutputTex Texture
---@field lumiSharedRt Vector [UI(Type="Texture")]

local AE_EFFECT_TAG = 'AE_EFFECT_TAG LumiTag'

local function createRenderTexture(width, height, filterMag, filterMin)
    local rt = Amaz.RenderTexture()
    rt.width = width
    rt.height = height
    rt.depth = 1
    rt.filterMag = filterMag or Amaz.FilterMode.LINEAR
    rt.filterMin = filterMin or Amaz.FilterMode.LINEAR
    rt.filterMipmap = Amaz.FilterMipmapMode.NONE
    rt.attachment = Amaz.RenderTextureAttachment.NONE
    return rt
end

local function updateTexSize(rt, width, height)
    if rt == nil or width <= 0 or height <= 0 then
        return
    end
    if rt.width ~= width or rt.height ~= height then
        rt.width = width
        rt.height = height
    end
end

function LumiMutiShadow.new(construct, ...)
    local self = setmetatable({}, LumiMutiShadow)

    self.__lumi_type = "lumi_obj"
    self.__lumi_rt_pingpong_type = "custom"

    self.oriRotation = 0.0
    self.oriPositionX = 0.0
    self.oriPositionY = 0.0
    self.oriPivotX = 0.0
    self.oriPivotY = 0.0
    self.oriScaleX = 1.0
    self.oriScaleY = 1.0
    self.oriColor = Amaz.Color(1., 1., 1.)
    self.oriAlpha = 1.0
    self.oriBgColor = Amaz.Color(0., 0., 20.)
    self.oriBgAlpha = 0.0
    self.oriBgPad = 0.05
    self.oriBgRadius = 0.0
    self.oriBgFeather = 0.0

    self.unifiedScale = true
    self.isAssociation = false

    self.layerNum = 1

    self.rotation_1 = 0.0
    self.rotation_2 = 0.0
    self.rotation_3 = 0.0
    self.rotation_4 = 0.0
    self.rotation_5 = 0.0
    self.rotation_6 = 0.0
    self.rotation_7 = 0.0
    self.rotation_8 = 0.0

    self.positionX_1 = 0.0
    self.positionY_1 = 0.0
    self.positionX_2 = 0.0
    self.positionY_2 = 0.0
    self.positionX_3 = 0.0
    self.positionY_3 = 0.0
    self.positionX_4 = 0.0
    self.positionY_4 = 0.0
    self.positionX_5 = 0.0
    self.positionY_5 = 0.0
    self.positionX_6 = 0.0
    self.positionY_6 = 0.0
    self.positionX_7 = 0.0
    self.positionY_7 = 0.0
    self.positionX_8 = 0.0
    self.positionY_8 = 0.0

    self.scale_1 = 1.0
    self.scale_2 = 1.0
    self.scale_3 = 1.0
    self.scale_4 = 1.0
    self.scale_5 = 1.0
    self.scale_6 = 1.0
    self.scale_7 = 1.0
    self.scale_8 = 1.0


    self.color_1 = Amaz.Color(1., 0., 0.)
    self.color_2 = Amaz.Color(1., 0., 0.)
    self.color_3 = Amaz.Color(1., 0., 0.)
    self.color_4 = Amaz.Color(1., 0., 0.)
    self.color_5 = Amaz.Color(1., 0., 0.)
    self.color_6 = Amaz.Color(1., 0., 0.)
    self.color_7 = Amaz.Color(1., 0., 0.)
    self.color_8 = Amaz.Color(1., 0., 0.)

    self.bgColor_1 = Amaz.Color(0., 0., 0.1)
    self.bgColor_2 = Amaz.Color(0., 0., 0.2)
    self.bgColor_3 = Amaz.Color(0., 0., 0.3)
    self.bgColor_4 = Amaz.Color(0., 0., 0.4)
    self.bgColor_5 = Amaz.Color(0., 0., 0.5)
    self.bgColor_6 = Amaz.Color(0., 0., 0.6)
    self.bgColor_7 = Amaz.Color(0., 0., 0.7)
    self.bgColor_8 = Amaz.Color(0., 0., 0.8)

    self.bgPad_1 = 0.05
    self.bgPad_2 = 0.05
    self.bgPad_3 = 0.05
    self.bgPad_4 = 0.05
    self.bgPad_5 = 0.05
    self.bgPad_6 = 0.05
    self.bgPad_7 = 0.05
    self.bgPad_8 = 0.05

    self.bgRadius_1 = 0.0
    self.bgRadius_2 = 0.0
    self.bgRadius_3 = 0.0
    self.bgRadius_4 = 0.0
    self.bgRadius_5 = 0.0
    self.bgRadius_6 = 0.0
    self.bgRadius_7 = 0.0
    self.bgRadius_8 = 0.0

    self.bgFeather_1 = 0.0
    self.bgFeather_2 = 0.0
    self.bgFeather_3 = 0.0
    self.bgFeather_4 = 0.0
    self.bgFeather_5 = 0.0
    self.bgFeather_6 = 0.0
    self.bgFeather_7 = 0.0
    self.bgFeather_8 = 0.0

    self.alpha_1 = 1.0
    self.alpha_2 = 1.0
    self.alpha_3 = 1.0
    self.alpha_4 = 1.0
    self.alpha_5 = 1.0
    self.alpha_6 = 1.0
    self.alpha_7 = 1.0
    self.alpha_8 = 1.0
    self.bgAlpha_1 = 1.0
    self.bgAlpha_2 = 1.0
    self.bgAlpha_3 = 1.0
    self.bgAlpha_4 = 1.0
    self.bgAlpha_5 = 1.0
    self.bgAlpha_6 = 1.0
    self.bgAlpha_7 = 1.0
    self.bgAlpha_8 = 1.0

    self.textExpandRatio = Amaz.Vector2f(1, 1)
    self.InputTex = nil
    self.OutputTex = nil

    return self
end

function LumiMutiShadow:setEffectAttr(key, value, comp)
    local function _setEffectAttr(_key, _value)
        self[_key] = _value
        if comp and comp.properties ~= nil then
            comp.properties:set(_key, _value)
        end
    end

    if key == "oriColor" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "oriBgColor" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "color_1" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "color_2" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "color_3" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "color_4" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "color_5" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "color_6" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "color_7" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "color_8" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "bgColor_1" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "bgColor_2" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "bgColor_3" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "bgColor_4" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "bgColor_5" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "bgColor_6" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "bgColor_7" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    elseif key == "bgColor_8" then
        local color = Amaz.Color(value:get(0), value:get(1), value:get(2), 1)
        _setEffectAttr(key, color)
    else
        _setEffectAttr(key, value)
    end
end

function LumiMutiShadow:onStart(comp)
    self.entity = comp.entity
    self.TAG = AE_EFFECT_TAG .. ' ' .. self.entity.name
    -- Amaz.LOGI(self.TAG, 'onStart')

    -- Use entity instead of scene
    self.camera = self.entity:searchEntity("BlitCamera"):getComponent("Camera")
    self.material = self.entity:searchEntity("BlitPass"):getComponent("MeshRenderer").material

    -- if temporary texture has the same dimensions as self.OutputTex, and need not persist, advised to used sharedRT
    if self.lumiSharedRt and self.lumiSharedRt:size() > 0 then
        self.midTex = self.lumiSharedRt:get(0)
    end

    -- if render texture needs to persist, or dimensions are exposed to be modified, then .rt files cannot be used, create the texture using the code below
    -- self.myTex = createRenderTexture(w, h)
end

function LumiMutiShadow:onUpdate(comp, deltaTime)
    -- set the input and output textures to be displayed
    if self.OutputTex then
        self.camera.renderTexture = self.OutputTex
    end
    self.material:setTex("u_inputTexture", self.InputTex)

    -- if using sharedRT, dimensions need to be consistent with self.InputTex or self.OutputTex
    if self.midTex then
        if self.midTex.width ~= self.OutputTex.width or self.midTex.height ~= self.OutputTex.height then
            self.midTex.width = self.OutputTex.width
            self.midTex.height = self.OutputTex.height
        end
    end

    -- temporary texture's dimensions need to be based on self.InputTex or self.OutputTex's dimensions
    if self.myTex then
        local scale = 1.0 -- the necessary scale factor
        local realW = self.OutputTex.width * scale
        local realH = self.OutputTex.height * scale

        -- when temporary texture type is RenderTexture
        self.myTex.width = realW
        self.myTex.height = realH

        -- when temporary texture type is ScreenRenderTexture
        self.myTex.pecentX = realW / Amaz.BuiltinObject.getOutputTextureWidth()
        self.myTex.pecentY = realH / Amaz.BuiltinObject.getOutputTextureHeight()
    end

    local oriPosition = Amaz.Vector2f(self.oriPositionX, self.oriPositionY)
    local oriPivot = Amaz.Vector2f(self.oriPivotX, self.oriPivotY)
    local oriScale = Amaz.Vector2f(self.oriScaleX, self.oriScaleY)

    if self.unifiedScale then
        oriScale.y = oriScale.x
    end
    self.material:setFloat("u_oriRotation", self.oriRotation)
    self.material:setVec2("u_oriPosition", oriPosition)
    self.material:setVec2("u_oriPivot", oriPivot)
    self.material:setVec2("u_oriScale", oriScale)

    local layerNum = math.floor(self.layerNum)

    local scaleList = {}
    local positionList = {}
    local rotationList = {}
    local layerColor = {}
    local bgLayerColor = {}
    local bgPadList = {}
    local bgRadiusList = {}
    local bgFeatherList = {}


    local colorProps = { "color_1", "color_2", "color_3", "color_4", "color_5", "color_6", "color_7", "color_8" }
    local alphaProps = { "alpha_1", "alpha_2", "alpha_3", "alpha_4", "alpha_5", "alpha_6", "alpha_7", "alpha_8" }
    local bgColorProps = { "bgColor_1", "bgColor_2", "bgColor_3", "bgColor_4", "bgColor_5", "bgColor_6", "bgColor_7",
        "bgColor_8" }
    local bgAlphaProps = { "bgAlpha_1", "bgAlpha_2", "bgAlpha_3", "bgAlpha_4", "bgAlpha_5", "bgAlpha_6", "bgAlpha_7",
        "bgAlpha_8" }
    local scaleProps = { "scale_1", "scale_2", "scale_3", "scale_4", "scale_5", "scale_6", "scale_7", "scale_8" }
    local bgPadProps = { "bgPad_1", "bgPad_2", "bgPad_3", "bgPad_4", "bgPad_5", "bgPad_6", "bgPad_7", "bgPad_8" }
    local bgRadiusProps = { "bgRadius_1", "bgRadius_2", "bgRadius_3", "bgRadius_4", "bgRadius_5", "bgRadius_6",
        "bgRadius_7", "bgRadius_8" }
    local bgFeatherProps = { "bgFeather_1", "bgFeather_2", "bgFeather_3", "bgFeather_4", "bgFeather_5", "bgFeather_6",
        "bgFeather_7", "bgFeather_8" }

    local positionXProps = { "positionX_1", "positionX_2", "positionX_3", "positionX_4", "positionX_5", "positionX_6",
        "positionX_7", "positionX_8" }
    local positionYProps = { "positionY_1", "positionY_2", "positionY_3", "positionY_4", "positionY_5", "positionY_6",
        "positionY_7", "positionY_8" }
    local rotationProps = { "rotation_1", "rotation_2", "rotation_3", "rotation_4", "rotation_5", "rotation_6",
        "rotation_7", "rotation_8" }

    for i = 1, layerNum do
        local colorProp = colorProps[i] or "color_1"
        local alphaProp = alphaProps[i] or "alpha_1"
        local bgColorProp = bgColorProps[i] or "bgColor_1"
        local bgAlphaProp = bgAlphaProps[i] or "bgAlpha_1"
        local scaleProp = scaleProps[i] or "scale_1"
        local posXProp = positionXProps[i] or "positionX_1"
        local posYProp = positionYProps[i] or "positionY_1"
        local rotationProp = rotationProps[i] or "rotation_1"
        local bgPadProp = bgPadProps[i] or "bgPad_1"
        local bgRadiusProp = bgRadiusProps[i] or "bgRadius_1"
        local bgFeatherProp = bgFeatherProps[i] or "bgFeather_1"



        local colorVec = self[colorProp] or Amaz.Color(1.0, 1.0, 1.0)
        local alpha = self[alphaProp] or 1.0
        local bgColorVec = self[bgColorProp] or Amaz.Color(0.0, 0.0, 0.0)
        local bgAlpha = self[bgAlphaProp] or 1.0
        local scale = self[scaleProp] or 1.0
        local positionX = self[posXProp] or 0.0
        local positionY = self[posYProp] or 0.0
        local rotation = self[rotationProp] or 0.0
        local bgPad = self[bgPadProp] or 0.05
        local bgRadius =  0.0
        local bgFeather = self[bgFeatherProp] or 0.0


        table.insert(layerColor, Amaz.Vector4f(colorVec.r, colorVec.g, colorVec.b, alpha))
        table.insert(bgLayerColor, Amaz.Vector4f(bgColorVec.r, bgColorVec.g, bgColorVec.b, bgAlpha))
        table.insert(scaleList, Amaz.Vector2f(scale, scale))
        table.insert(positionList, Amaz.Vector2f(positionX, positionY))
        table.insert(rotationList, Amaz.Vector2f(rotation, rotation))
        table.insert(bgPadList, bgPad)
        table.insert(bgRadiusList, bgRadius)
        table.insert(bgFeatherList, bgFeather)
    end

    self.material:setInt("u_layerNum", layerNum)
    self.material:setVec4Vector("u_layerColor", layerColor)
    self.material:setVec4Vector("u_bgColor", bgLayerColor)
    self.material:setVec2Vector("u_scale", scaleList)
    self.material:setVec2Vector("u_position", positionList)
    self.material:setVec2Vector("u_rotation", rotationList)
    self.material:setFloatVector("u_bgPad", bgPadList)
    self.material:setFloatVector("u_bgRadius", bgRadiusList)
    self.material:setFloatVector("u_bgFeather", bgFeatherList)

    if self.isAssociation then
        self.material:setInt("u_isAssociation", 1)
    else
        self.material:setInt("u_isAssociation", 0)
    end

    self.material:setFloat("u_oriAlpha", self.oriAlpha)
    self.material:setFloat("u_oriBgPad", self.oriBgPad)
    self.material:setFloat("u_oriBgRadius", 0.0)
    self.material:setFloat("u_oriBgFeather", self.oriBgFeather)

    -- Amaz.LOGE("oriColor", self.oriColor.r)
    self.material:setVec3("u_oriColor", Amaz.Vector3f(self.oriColor.r, self.oriColor.g, self.oriColor.b))
    self.material:setVec4("u_oriBgColor",
        Amaz.Vector4f(self.oriBgColor.r, self.oriBgColor.g, self.oriBgColor.b, self.oriBgAlpha))
   
    self.material:setVec2("textExpandRatio",
        Amaz.Vector2f(self.textExpandRatio.x, self.textExpandRatio.y))
end

exports.LumiMutiShadow = LumiMutiShadow
return exports
