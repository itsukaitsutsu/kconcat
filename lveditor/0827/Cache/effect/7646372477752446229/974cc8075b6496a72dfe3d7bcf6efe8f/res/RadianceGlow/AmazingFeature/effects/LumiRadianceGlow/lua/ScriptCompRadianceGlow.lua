---@class ScriptCompRadianceGlow: ScriptComponent
---@field thresholdType string [UI(Option={"S", "D"})]
---@field thresholdLow number [UI(Range={0., 1.}, Drag)]
---@field thresholdHigh number [UI(Range={0., 1.}, Drag)]
---@field thresholdSmooth number [UI(Range={0., 1.}, Drag)]
---@field grayScale number [UI(Range={0., 1.}, Drag)]
---@field erodeIterations int [UI(Range={0, 5}, Slider)]
---@field erodeStepMulplier number [UI(Range={0.5, 5.}, Drag)]
---@field glowIntensity number [UI(Range={0., 1000.}, Drag)]
---@field angle number
---@field directionNum number [UI(Range={1., 4.}, Drag)]
---@field exposure number [UI(Range={0., 20.}, Drag)]
---@field quality number [UI(Range={0., 1.}, Drag)]
---@field spaceDither number [UI(Range={0, 1.}, Drag)]
---@field borderType string [UI(Option={"Black", "Mirror"})]
---@field glowColor Color [UI(NoAlpha)]
---@field displayGlow boolean
---@field useMask boolean
---@field InputTex Texture
---@field OutputTex Texture
---@field maskTex Texture
---@field lumiSharedRt Vector [UI(Type="Texture")]


local exports = exports or {}
local ScriptCompRadianceGlow = ScriptCompRadianceGlow or {}
ScriptCompRadianceGlow.__index = ScriptCompRadianceGlow


function ScriptCompRadianceGlow.new(construct, ...)
    local self = setmetatable({}, ScriptCompRadianceGlow)

    if construct and ScriptCompRadianceGlow.constructor then ScriptCompRadianceGlow.constructor(self, ...) end
    -- user parameters
    self.thresholdType = "S"
    self.thresholdLow = 0.5
    self.thresholdHigh = 1.
    self.thresholdSmooth = 0.
    self.grayScale = 0.
    self.erodeIterations = 0
    self.erodeStepMulplier = 1.
    self.glowIntensity = 0.
    self.angle = 0.
    self.directionNum = 1
    self.exposure = 1.
    self.glowColor = Amaz.Color(1., 1., 1.)
    self.quality = 0.5
    self.spaceDither = 0.
    self.borderType = "Black"
    self.displayGlow = false
    self.useMask = false

    -- other parameters
    self.NormalizationSize = 1000.
    self.MaxIntensity = 1000.
    self.MaxExposure = 20.
    self.RadiusOverSigma = 3.
    self.MaxDirectionNum = 4
    self.MaxErodeNum = 5
    self.Eps = 1e-5

    self.InputTex = nil
    self.OutputTex = nil
    self.downsampleTextures = nil
    self.textures = nil
    self.midTex = nil
    self.maskTex = nil

    return self
end


------------ util functions ------------
local function createRenderTexture(width, height)
    local rt = Amaz.RenderTexture()
    rt.width = width
    rt.height = height
    rt.depth = 1
    rt.filterMag = Amaz.FilterMode.LINEAR
    rt.filterMin = Amaz.FilterMode.LINEAR
    rt.attachment = Amaz.RenderTextureAttachment.NONE
    return rt
end


local function clamp(value, min, max)
    --[[
        Description: clamp value between [min, max]
    ]]
    return math.min(math.max(value, min), max)
end


local function boolToInt(flag)
    --[[
        Description: convert flag from bool to int
    ]]
    if flag then return 1 end
    return 0
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


local function getBorderType(borderType)
    --[[
        Description: convert borderType from string to int
    ]]
    local borderTypeTable = {
        ["Black"] = 0, -- default
        ["Mirror"] = 1
    }
    local flag = borderTypeTable[borderType]
    if flag == nil then flag = 0 end
    return flag
end


local function getThresholdType(thresholdType)
    --[[
        Description: convert thresholdType from string to int
    ]]
    local thresholdTypeTable = {
        ["S"] = 0, -- default
        ["D"] = 1
    }
    local flag = thresholdTypeTable[thresholdType]
    if flag == nil then flag = 0 end
    return flag
end


local function getSampleNum(intensity)
    --[[
        Description: get sample number and downsample scale
    ]]
    local scale = 1.
    local bias = 0.
    local s = scale
    if intensity <= 10 then
        scale = 1
        bias = 0.
        s = 1.0
    elseif intensity <= 50 then
        scale = 0.7
        bias = 2.
        s = 0.78
    elseif intensity <= 200 then
        scale = 0.5
        bias = 10
        s = 0.66
    else
        scale = 0.25
        bias = 60.
        s = 0.7
    end

    local sampleNum = scale * intensity + bias
    sampleNum = sampleNum * s
    if sampleNum < 2. then
        sampleNum = math.floor(sampleNum + 0.5)
    end
    return sampleNum, scale
end


local function getQualityParam(quality)
    local qualityParam = quality * 2. - 1.
    if qualityParam < 0. then
        qualityParam = 10 ^ qualityParam
    else
        qualityParam = qualityParam * 2. + 1.
    end
    return qualityParam
end


------------ class functions for ScriptComponent ------------
function ScriptCompRadianceGlow:setEffectAttr(key, value, comp)
    local function _setEffectAttr(_key, _value, _forceState)
        if self[_key] ~= nil or _forceState then
            self[_key] = _value
            if comp and comp.properties ~= nil then
                comp.properties:set(_key, _value)
            end
        end
    end
    if type(value) == 'userdata' and value.size then
        local size = value:size()
        if size == 2 then
            value = Amaz.Vector2f(value:get(0), value:get(1))
        elseif size == 3 then
            value = Amaz.Vector3f(value:get(0), value:get(1), value:get(2))
        elseif size == 4 then
            value = Amaz.Color(value:get(0), value:get(1), value:get(2), value:get(3))
        end
    end
    if key == "borderType" then
        local typeTable = {"Black", "Mirror"}
        local borderType = typeTable[value + 1]
        if borderType == nil then borderType = "Normal" end
        _setEffectAttr(key, borderType)
    elseif key == "thresholdType" then
        local typeTable = { "S", "D"}
        local thresholdType = typeTable[value + 1]
        if thresholdType == nil then thresholdType = "S" end
        _setEffectAttr(key, thresholdType)
    elseif key == "maskTex" then
        _setEffectAttr(key, value, true)
    else
        _setEffectAttr(key, value)
    end
end


function ScriptCompRadianceGlow:onStart(comp)
    self.camThreshold = comp.entity:searchEntity("CameraThreshold"):getComponent("Camera")
    self.camDownScale = comp.entity:searchEntity("CameraDownScale"):getComponent("Camera")
    self.camBlend = comp.entity:searchEntity("CameraBlend"):getComponent("Camera")
    self.matThreshold = comp.entity:searchEntity("EntityThreshold"):getComponent("MeshRenderer").material
    self.matBlend = comp.entity:searchEntity("EntityBlend"):getComponent("MeshRenderer").material

    self.camBlurs = {}
    for i = 1, self.MaxDirectionNum, 1 do
        self.camBlurs[i] = comp.entity:searchEntity("CameraDirectionalBlurs_" .. i):getComponent("Camera")
    end

    self.matBlurs = {}
    for i = 1, self.MaxDirectionNum, 1 do
        self.matBlurs[i] = comp.entity:searchEntity("EntityDirectionalBlurs_" .. i):getComponent("MeshRenderer").material
    end

    self.camErodes = {}
    for i = 1, self.MaxErodeNum, 1 do
        self.camErodes[i] = comp.entity:searchEntity("CameraErode_" .. i):getComponent("Camera")
    end

    self.matErodes = {}
    for i = 1, self.MaxErodeNum, 1 do
        self.matErodes[i] = comp.entity:searchEntity("EntityErode_" .. i):getComponent("MeshRenderer").material
    end

    self.downsampleTextures = {}
    self.downsampleTextures[1] = createRenderTexture(1, 1)
    self.downsampleTextures[2] = createRenderTexture(1, 1)

    self.textures = {}
    for i = 1, self.MaxDirectionNum, 1 do
        self.textures[i] = createRenderTexture(1, 1)
    end
    self.midTex = self.lumiSharedRt:get(0)
end


function ScriptCompRadianceGlow:onUpdate(comp, deltaTime)
    local textureWidth = self.OutputTex.width
    local textureHeight = self.OutputTex.height

    -- preprocess parameters
    local thresholdType = getThresholdType(self.thresholdType)
    self.thresholdLow = clamp(self.thresholdLow, 0., 1.)
    self.thresholdHigh = clamp(self.thresholdHigh, 0., 1.)
    local thresholdHigh = math.max(self.thresholdHigh, self.thresholdHigh)
    self.thresholdSmooth = clamp(self.thresholdSmooth, 0., 1.)
    self.grayScale = clamp(self.grayScale, 0., 1.)
    self.erodeIterations = clamp(self.erodeIterations, 0, self.MaxErodeNum)
    self.erodeStepMulplier = clamp(self.erodeStepMulplier, 0.5, 5.)
    local erodeIterations = math.floor(self.erodeIterations)
    local displayGlow = boolToInt(self.displayGlow)
    local useMask = boolToInt(self.useMask)
    self.glowIntensity = clamp(self.glowIntensity, 0., self.MaxIntensity)
    local startAngle = (-self.angle - 90.) * math.pi / 180.
    self.directionNum = clamp(self.directionNum, 1., self.MaxDirectionNum)
    local directionNum = math.floor(self.directionNum)
    local deltaAngle = math.pi / directionNum
    local exposure = clamp(self.exposure, 0., self.MaxExposure)
    local glowIntensity = self.glowIntensity
    if exposure < 1 then
        glowIntensity = glowIntensity * exposure
        exposure = 1
    end
    local sampleNum, downScale = getSampleNum(glowIntensity)
    self.quality = clamp(self.quality, 0., 1.)
    local qualityParam = getQualityParam(self.quality)
    local borderType = getBorderType(self.borderType)
    self.spaceDither = clamp(self.spaceDither, 0., 1.)

    -- set texture size
    if self.midTex.width ~= textureWidth or self.midTex.height ~= textureHeight then
        self.midTex.width = textureWidth
        self.midTex.height = textureHeight
    end
    local downScaleWidth = textureWidth * downScale
    local downScaleHeight = textureHeight * downScale

    self.downsampleTextures[1].width = downScaleWidth
    self.downsampleTextures[1].height = downScaleHeight
    if erodeIterations > self.Eps then
        self.downsampleTextures[2].width = downScaleWidth
        self.downsampleTextures[2].height = downScaleHeight
    else
        self.downsampleTextures[2].width = 1
        self.downsampleTextures[2].height = 1
    end

    for i = 1, self.MaxDirectionNum, 1 do
        if i <= directionNum then
            self.textures[i].width = downScaleWidth
            self.textures[i].height = downScaleHeight
        else
            self.textures[i].width = 1
            self.textures[i].height = 1
        end
    end

    -- compute sigma, sample, dx/dy for blur
    local xScale, yScale = getXYScale(textureWidth, textureHeight)
    local radius = glowIntensity / self.NormalizationSize
    local sigma = radius / self.RadiusOverSigma
    local sample = sampleNum * qualityParam
    local ds = radius / math.max(sample, self.Eps)

    -- compute dx/dy for erode
    local erodeDx = self.erodeStepMulplier / self.NormalizationSize * xScale
    local erodeDy = self.erodeStepMulplier / self.NormalizationSize * yScale

    -- set Textures
    if self.useMask then
        self.matThreshold:setTex("u_maskTex", self.maskTex)
    end
    self.matThreshold:setTex("u_inputTexture", self.InputTex)
    self.camThreshold.renderTexture = self.midTex
    self.camDownScale.inputTexture = self.midTex
    self.camDownScale.renderTexture = self.downsampleTextures[1]

    for i = 1, self.MaxErodeNum, 1 do
        if i % 2 == 1 then
            self.matErodes[i]:setTex("u_inputTexture", self.downsampleTextures[1])
            self.camErodes[i].renderTexture = self.downsampleTextures[2]
        else
            self.matErodes[i]:setTex("u_inputTexture", self.downsampleTextures[2])
            self.camErodes[i].renderTexture = self.downsampleTextures[1]
        end

        if i <= erodeIterations then
            self.camErodes[i].entity.visible = true
        else
            self.camErodes[i].entity.visible = false
        end
    end
    local index = 1
    if erodeIterations % 2 == 1 then
        index = 2
    end

    for i = 1, self.MaxDirectionNum, 1 do
        self.matBlurs[i]:setTex("u_inputTexture", self.downsampleTextures[index])
        self.camBlurs[i].renderTexture = self.textures[i]
        if i <= directionNum then
            self.camBlurs[i].entity.visible = true
        else
            self.camBlurs[i].entity.visible = false
        end
    end

    for i = 1, directionNum, 1 do
        self.matBlend:setTex("u_tex" .. i, self.textures[i])
    end
    self.matBlend:setTex("u_inputTexture" , self.InputTex)
    self.camBlend.renderTexture = self.OutputTex

    -- set parameters for materials
    self.matThreshold:setFloat("u_thresholdLow", self.thresholdLow)
    self.matThreshold:setFloat("u_thresholdHigh", thresholdHigh)
    self.matThreshold:setFloat("u_thresholdSmooth", self.thresholdSmooth)
    self.matThreshold:setFloat("u_grayScale", self.grayScale)
    self.matThreshold:setInt("u_thresholdType", thresholdType)
    self.matThreshold:setInt("u_useMask", useMask)

    for i = 1, self.MaxErodeNum, 1 do
        self.matErodes[i]:setFloat("u_stepX", erodeDx)
        self.matErodes[i]:setFloat("u_stepY", erodeDy)
    end

    for i = 1, self.MaxDirectionNum, 1 do
        local angle = startAngle + (i - 1) * deltaAngle
        local dx = ds * math.cos(angle) * xScale
        local dy = ds * math.sin(angle) * yScale

        self.matBlurs[i]:setFloat("u_sample", sample)
        self.matBlurs[i]:setFloat("u_sigma", sigma)
        self.matBlurs[i]:setFloat("u_stepX", dx)
        self.matBlurs[i]:setFloat("u_stepY", dy)
        self.matBlurs[1]:setFloat("u_spaceDither", self.spaceDither)
        self.matBlurs[1]:setInt("u_borderType", borderType)
    end
    self.matBlurs[1]:setFloat("u_exposure", exposure)
    self.matBlurs[1]:setInt("u_directionNum", directionNum)

    self.matBlend:setFloat("u_exposure", exposure)
    self.matBlend:setInt("u_directionNum", directionNum)
    self.matBlend:setInt("u_displayGlow", displayGlow)
    self.matBlend:setVec3("u_glowColor", Amaz.Vector3f(self.glowColor.r, self.glowColor.g, self.glowColor.b))
end


exports.ScriptCompRadianceGlow = ScriptCompRadianceGlow
return exports
