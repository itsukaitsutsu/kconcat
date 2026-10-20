local isEditor = (Amaz.Macros and Amaz.Macros.EditorSDK) and true or false
local exports = exports or {}
local LumiColorAdjustBundle = LumiColorAdjustBundle or {}
LumiColorAdjustBundle.__index = LumiColorAdjustBundle
---@class LumiColorAdjustBundle : ScriptComponent
---@field colorRange string [UI(Option={"Shadow", "Highlight", "Skin"})]
---@field mapType string [UI(Option={"Hard", "Soft"})]
---@field shadowMapParam number [UI(Range={0., 2.}, Drag)]
---@field highlightMapParam number [UI(Range={0., 2.}, Drag)]
---@field displayOriginalColor boolean
---@field displayRangeMap boolean
---@field displaySaturationSuppressionFactor boolean
---@field useRangeMap boolean
---@field highlightIntensity number [UI(Range={-1., 1.}, Drag)]
---@field shadowIntensity number [UI(Range={-1., 1.}, Drag)]
---@field blackIntensity number [UI(Range={-1., 1.}, Drag)]
---@field whiteIntensity number [UI(Range={-1., 1.}, Drag)]
---@field exposureIntensity number [UI(Range={-1., 1.}, Drag)]
---@field offsetIntensity number [UI(Range={-1., 1.}, Drag)]
---@field contrastIntensity number [UI(Range={-1., 1.}, Drag)]
---@field contrastPivot number [UI(Range={0., 1.}, Drag)]
---@field saturationSuppressionIntensity number [UI(Range={0., 1.}, Drag)]
---@field saturationSuppressionThreshold number [UI(Range={0., 1.}, Drag)]
---@field saturationIntensity number [UI(Range={-1., 1.}, Drag)]
---@field temperatureIntensity number [UI(Range={-1., 1.}, Drag)]
---@field tintIntensity number [UI(Range={-1., 1.}, Drag)]
---@field dahazeIntensity number [UI(Range={0., 1.}, Drag)]
---@field surfaceBlurIntensity number [UI(Range={0., 1.}, Drag)]
---@field lutIntensity number [UI(Range={0., 1.}, Drag)]
---@field lutColorSample number [UI(Range={1., 257.}, Drag)]
---@field lutHorizontalGridNum number [UI(Range={1., 65.}, Drag)]
---@field lutVerticalGridNum number [UI(Range={1., 65.}, Drag)]
---@field lutTex Texture
---@field InputTex Texture
---@field OutputTex Texture
---@field lumiSharedRt Vector [UI(Type="Texture")]

local AE_EFFECT_TAG = 'AE_EFFECT_TAG LumiTag'


function LumiColorAdjustBundle.new(construct, ...)
    local self = setmetatable({}, LumiColorAdjustBundle)

    self.__lumi_type = "lumi_obj"
    self.__lumi_rt_pingpong_type = "custom"

    -- user parameters
    self.colorRange = "Shadow"
    self.mapType = "Hard"
    self.shadowMapParam = 0.9
    self.highlightMapParam = 1.8
    self.displayOriginalColor = false
    self.displayRangeMap = false
    self.displaySaturationSuppressionFactor = false
    self.useRangeMap = true

    self.highlightIntensity = 0.
    self.shadowIntensity = 0.
    self.blackIntensity = 0.
    self.whiteIntensity = 0.
    self.exposureIntensity = 0.
    self.offsetIntensity = 0.
    self.contrastIntensity = 0.
    self.contrastPivot = 0.435
    self.saturationSuppressionIntensity = 0.
    self.saturationSuppressionThreshold = 0.
    self.saturationIntensity = 0.
    self.temperatureIntensity = 0.
    self.tintIntensity = 0.
    self.surfaceBlurIntensity = 0.
    self.lutIntensity = 0.
    self.lutColorSample = 64.
    self.lutHorizontalGridNum = 8.
    self.lutVerticalGridNum = 8.
    self.dahazeIntensity = 0.5

    -- constant parameters
    self.Eps = 1e-5
    self.HighlightIntensityScale = 0.8
    self.ShadowIntensityScale = -0.8
    self.MaxLutColorSample = 1024.
    self.MaxLutGridNum = 100.
    self.NormalizationSize = 1000.
    self.SurfaceBlurThreshold = 0.05
    self.AirLightThreshold = 0.5

    self.lutTex = nil
    self.InputTex = nil
    self.OutputTex = nil

    -- algorithm parameters for temperature and tint 
    self.baseTemperature = 6500
    self.baseTint = 0
    self.extraData = {
        0, 0.18006, 0.26352, -0.24341,
        10, 0.18066, 0.26589, -0.25479,
        20, 0.18133, 0.26846, -0.26876,
        30, 0.18208, 0.27119, -0.28539,
        40, 0.18293, 0.27407, -0.30470,
        50, 0.18388, 0.27709, -0.32675,
        60, 0.18494, 0.28021, -0.35156,
        70, 0.18611, 0.28342, -0.37915,
        80, 0.18740, 0.28668, -0.40955,
        90, 0.18880, 0.28997, -0.44278,
        100, 0.19032, 0.29326, -0.47888,
        125, 0.19462, 0.30141, -0.58204,
        150, 0.19962, 0.30921, -0.70471,
        175, 0.20525, 0.31647, -0.84901,
        200, 0.21142, 0.32312, -1.0182,
        225, 0.21807, 0.32909, -1.2168,
        250, 0.22511, 0.33439, -1.4512,
        275, 0.23247, 0.33904, -1.7298,
        300, 0.24010, 0.34308, -2.0637,
        325, 0.24792, 0.34655, -2.4681,
        350, 0.25591, 0.34951, -2.9641,
        375, 0.26400, 0.35200, -3.5814,
        400, 0.27218, 0.35407, -4.3633,
        425, 0.28039, 0.35577, -5.3762,
        450, 0.28863, 0.35714, -6.7262,
        475, 0.29685, 0.35823, -8.5955,
        500, 0.30505, 0.35907, -11.324,
        525, 0.31320, 0.35968, -15.628,
        550, 0.32129, 0.36011, -23.325,
        575, 0.32931, 0.36038, -40.770,
        600, 0.33724, 0.36051, -116.45,
    }
    self.A = {{0.8951, 0.2664, -0.1614}, {-0.7502, 1.7135, 0.0367}, {0.0389, -0.0685, 1.0296}}
    self.B = {{0.987, -0.1471, 0.16}, {0.4323, 0.5184, 0.0493}, {-0.0085, 0.04, 0.9685}}
    self.P = {
        {0.4123907992659595, 0.357584339383878, 0.1804807884018343},
        {0.21263900587151036, 0.715168678767756, 0.07219231536073371},
        {0.019330818715591832, 0.11919477979462598, 0.9505321522496607}
    }
    self.Q = {
        {3.2409699419045213, -1.5373831775700935, -0.4986107602930033},
        {-0.9692436362808796, 1.8759675015077208, 0.04155505740717562},
        {0.05563007969699364, -0.20397695888897655, 1.0569715142428784}
    }
    return self
end


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

local function clamp(value, min, max)
    --[[
        Description: clamp value between [min, max]
    ]]
    return math.min(math.max(value, min), max)
end


local function remap(value, srcMin, srcMax, dstMin, dstMax)
    --[[
        Description: linearly remap value from [srcMin, srcMax] to [dstMin, dstMax]
    ]]
    return dstMin + (value - srcMin) * (dstMax - dstMin) / (srcMax - srcMin)
end


local function boolToInt(flag)
    --[[
        Description: convert flag from bool to int
    ]]
    if flag then return 1 end
    return 0
end


local function getMapType(mapType)
    --[[
        Description: convert mapType from string to int
    ]]
    local mapTypeTable = {
        ["Hard"] = 0, -- default
        ["Soft"] = 1
    }
    local flag = mapTypeTable[mapType]
    if flag == nil then flag = 0 end
    return flag
end


local function getColorRange(colorRange)
    --[[
        Description: convert colorRange from string to int
    ]]
    local colorRangeTable = {
        ["Shadow"] = 0, -- default
        ["Highlight"] = 1,
        ["Skin"] = 2
    }
    local flag = colorRangeTable[colorRange]
    if flag == nil then flag = 0 end
    return flag
end


local function mix(x, y, a)
    return x * (1. - a) + y * a
end


local function Mat3xVec3(mat3, vec3)
    --[[
        Description: matrix multiplication for `mat3x3 matmul vec3x1`
    ]]
    return {
        mat3[1][1] * vec3[1] + mat3[1][2] * vec3[2] + mat3[1][3] * vec3[3],
        mat3[2][1] * vec3[1] + mat3[2][2] * vec3[2] + mat3[2][3] * vec3[3],
        mat3[3][1] * vec3[1] + mat3[3][2] * vec3[2] + mat3[3][3] * vec3[3]
    }
end


local function Vec3xMat3(vec3, mat3)
    --[[
        Description: matrix multiplication for `vec1x3 matmul mat3x3`
    ]]
    return {
        mat3[1][1] * vec3[1] + mat3[2][1] * vec3[2] + mat3[3][1] * vec3[3],
        mat3[1][2] * vec3[1] + mat3[2][2] * vec3[2] + mat3[3][2] * vec3[3],
        mat3[1][3] * vec3[1] + mat3[2][3] * vec3[2] + mat3[3][3] * vec3[3]
    }
end


local function Mat3xMat3(mat3_1, mat3_2)
    --[[
        Description: matrix multiplication for `mat3x3 matmul mat3x3`
    ]]
    return {Vec3xMat3(mat3_1[1], mat3_2),
            Vec3xMat3(mat3_1[2], mat3_2),
            Vec3xMat3(mat3_1[3], mat3_2)}
end


local function diag(vec3)
    return {
        {vec3[1], 0.0, 0.0},
        {0.0, vec3[2], 0.0},
        {0.0, 0.0, vec3[3]}
    }
end


local function getTemperatureTintParam(temperatureIntensity, tintIntensity)
    local t = temperatureIntensity
    local t2 = t * t
    local t3 = t2 * t
    local temperatureParam = 6500. - 1970. * t + 876. * t2 - 2630. * t3
    local tintParam = tintIntensity * 100.
    return temperatureParam, tintParam
end


function LumiColorAdjustBundle:computeTemperatureTintVec3(temperature, tint)
    local x = 1000000.0 / temperature
    local y = tint * 0.0001
    local index = 5
    local data = self.extraData[index]
    index = index + 4
    while (data <= x and index < #self.extraData + 1) do
        data = self.extraData[index]
        index = index + 4
    end
    local factor = (data - x) / (data - self.extraData[index - 2 * 4])
    local temp_1 = {
        mix(self.extraData[index - 3], self.extraData[index - 7], factor),
        mix(self.extraData[index - 2], self.extraData[index - 6], factor),
    }
    local a = self.extraData[index - 5]
    local b = self.extraData[index - 1]
    local sqA = math.sqrt(a * a + 1.0)
    local sqB = math.sqrt(b * b + 1.0)
    local temp_2 = {
        mix(1. / sqB, 1. / sqA, factor),
        mix(b / sqB, a / sqA, factor)
    }
    factor = math.sqrt(temp_2[1] * temp_2[1] + temp_2[2] * temp_2[2])
    temp_1 = {
        y * temp_2[1] / factor + temp_1[1],
        y * temp_2[2] / factor + temp_1[2],
    }
    temp_2 = -4.0 * temp_1[2] + temp_1[1] + 2.0
    a = temp_1[1] * 1.5 / temp_2
    b = temp_1[2] / temp_2
    a = clamp(a, 0.000001, 0.999999)
    b = clamp(b, 0.000001, 0.999999)
    if (a + b > 0.999999) then
        local t = 0.999999 / (a + b)
        a = a / t
        b = b / t
    end
    return {a / b, 1.0, (1. - a - b) / b}
end


function LumiColorAdjustBundle:computeTemperatureTintMat3(vec3, vec3Base)
    vec3 = Mat3xVec3(self.A, vec3)
    vec3Base = Mat3xVec3(self.A, vec3Base)
    local D = diag({vec3[1] / vec3Base[1], vec3[2] / vec3Base[2], vec3[3] / vec3Base[3]})
    local param = Mat3xMat3(D, self.A)
    param = Mat3xMat3(self.B, param)
    param = Mat3xMat3(param, self.P)
    param = Mat3xMat3(self.Q, param)
    return param
end


local function getSaturationParam(intensity)
    if intensity <= 0. then
        return intensity + 1.0
    else
        return (intensity + 1.0) ^ 2.0
    end
end


local function getBlackWhiteParam(blackIntensity, whiteIntensity)
    -- initialization, (xw, yw) is white control point, (xb, yb) is black control point
    local xw = 1.0
    local yw = 1.0
    local xb = 0.0
    local yb = 0.0
    local MAX_RANGE = 0.5
    local EPS = 0.005

    -- white control points are located at top or right border
    -- black control points are located at bottom or left border
    if whiteIntensity >= 0.0 then
        xw = 1.0 - whiteIntensity * MAX_RANGE
        yw = 1.0
    else
        xw = 1.0
        yw = 1.0 + whiteIntensity * MAX_RANGE
    end

    if blackIntensity >= 0.0 then
        xb = 0.0
        yb = blackIntensity * MAX_RANGE
    else
        xb = -blackIntensity * MAX_RANGE
        yb = 0.0
    end

    -- rectify the points
    xw = math.max(xw, EPS)
    yw = math.max(yw, EPS)
    xb = math.min(xb, xw - EPS)
    yb = math.min(yb, yw - EPS)

    local slope = (yw - yb) / (xw - xb)
    local bias = yb - slope * xb
    return slope, bias
end


local function getDehazeParam(oriDehazeFactor)
     --[[
        Description: remap dehaze factor
    ]]
    local dehazeParam = oriDehazeFactor
    if dehazeParam > 0.5 then
        dehazeParam = remap(oriDehazeFactor, 0.5, 1., 0.5, 0.8)
    end
    return dehazeParam
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


function LumiColorAdjustBundle:setEffectAttr(key, value, comp)
    local function _setEffectAttr(_key, _value, _forceState)
        if self[_key] ~= nil or _forceState then
            self[_key] = _value
            if comp and comp.properties ~= nil then
                comp.properties:set(_key, _value)
            end
        end
    end

    if key == "colorRange" then
        local typeTable = {"Shadow", "Highlight", "Skin"}
        local rangeType = typeTable[value + 1]
        if rangeType == nil then rangeType = "Shadow" end
        _setEffectAttr(key, rangeType)
    elseif key == "mapType" then
        local typeTable = {"Hard", "Soft"}
        local mapType = typeTable[value + 1]
        if mapType == nil then mapType = "Hard" end
        _setEffectAttr(key, mapType)
    elseif key == "lutTex" then
        _setEffectAttr(key, value, true)
    else
        _setEffectAttr(key, value)
    end
end


function LumiColorAdjustBundle:onStart(comp)
    self.camColorAdjustBundle = comp.entity:searchEntity("CameraColorAdjustBundle"):getComponent("Camera")
    self.camSurfaceBlur = comp.entity:searchEntity("CameraSurfaceBlur"):getComponent("Camera")
    self.camDarkChannel = comp.entity:searchEntity("CameraDarkChannel"):getComponent("Camera")
    self.camDCDownSample = comp.entity:searchEntity("CameraDCDownSample"):getComponent("Camera")
    self.camAirLight = comp.entity:searchEntity("CameraAirLight"):getComponent("Camera")
    self.camDehaze = comp.entity:searchEntity("CameraDehaze"):getComponent("Camera")

    self.matColorAdjustBundle = comp.entity:searchEntity("EntityColorAdjustBundle"):getComponent("MeshRenderer").material
    self.matSurfaceBlur = comp.entity:searchEntity("EntitySurfaceBlur"):getComponent("MeshRenderer").material
    self.matDarkChannel = comp.entity:searchEntity("EntityDarkChannel"):getComponent("MeshRenderer").material
    self.matAirLight = comp.entity:searchEntity("EntityAirLight"):getComponent("MeshRenderer").material
    self.matDehaze = comp.entity:searchEntity("EntityDehaze"):getComponent("MeshRenderer").material

    self.colorAdjustTex = createRenderTexture(1, 1, Amaz.FilterMode.LINEAR, Amaz.FilterMode.LINEAR)
    self.darkChannelTex = createRenderTexture(1, 1, Amaz.FilterMode.LINEAR, Amaz.FilterMode.LINEAR)
    self.dcDownSampleTex = createRenderTexture(256, 256, Amaz.FilterMode.LINEAR, Amaz.FilterMode.LINEAR)
    self.airLightTex = createRenderTexture(1, 1, Amaz.FilterMode.LINEAR, Amaz.FilterMode.LINEAR)
    self.dehazeTex = createRenderTexture(1, 1, Amaz.FilterMode.LINEAR, Amaz.FilterMode.LINEAR)

    -- create skin segment texture, whose type is Texture2D
    self.skinSegTex = Amaz.Texture2D()
    self.skinSegTex.filterMin = Amaz.FilterMode.LINEAR
    self.skinSegTex.filterMag = Amaz.FilterMode.LINEAR
end


function LumiColorAdjustBundle:onUpdate(comp, deltaTime)
    local textureWidth = self.OutputTex.width
    local textureHeight = self.OutputTex.height

    -- preprocess parameters
    self.shadowMapParam = clamp(self.shadowMapParam, 0., 2.)
    self.highlightMapParam = clamp(self.highlightMapParam, 0., 2.)
    local displayOriginalColor = boolToInt(self.displayOriginalColor)
    local displayRangeMap = boolToInt(self.displayRangeMap)
    local colorRange = getColorRange(self.colorRange)
    local mapType = getMapType(self.mapType)
    local useRangeMap = boolToInt(self.useRangeMap)
    local displaySaturationSuppressionFactor = boolToInt(self.displaySaturationSuppressionFactor)

    self.highlightIntensity = clamp(self.highlightIntensity, -1., 1.)
    self.shadowIntensity = clamp(self.shadowIntensity, -1., 1.)
    self.blackIntensity = clamp(self.blackIntensity, -1., 1.)
    self.whiteIntensity = clamp(self.whiteIntensity, -1., 1.)
    self.exposureIntensity = clamp(self.exposureIntensity, -1., 1.)
    self.offsetIntensity = clamp(self.offsetIntensity, -1., 1.)
    self.contrastIntensity = clamp(self.contrastIntensity, -1., 1.)
    self.contrastPivot = clamp(self.contrastPivot, 0., 1.)
    self.saturationSuppressionIntensity = clamp(self.saturationSuppressionIntensity, 0., 1.)
    self.saturationSuppressionThreshold = clamp(self.saturationSuppressionThreshold, 0., 1.)
    self.saturationIntensity = clamp(self.saturationIntensity, -1., 1.)
    self.temperatureIntensity = clamp(self.temperatureIntensity, -1., 1.)
    self.tintIntensity = clamp(self.tintIntensity, -1., 1.)
    self.surfaceBlurIntensity = clamp(self.surfaceBlurIntensity, 0., 1.)
    self.lutIntensity = clamp(self.lutIntensity, 0., 1.)
    self.lutColorSample = clamp(self.lutColorSample, 1., self.MaxLutColorSample)
    self.lutHorizontalGridNum = clamp(self.lutHorizontalGridNum, 1., self.MaxLutGridNum)
    self.lutVerticalGridNum = clamp(self.lutVerticalGridNum, 1., self.MaxLutGridNum)
    self.dahazeIntensity = clamp(self.dahazeIntensity, 0., 1.)

    -- surface blur parameters
    local xScale, yScale = getXYScale(textureWidth, textureHeight)
    local steps = self.surfaceBlurIntensity * 5.0
    local dx = steps * xScale / self.NormalizationSize
    local dy = steps * yScale / self.NormalizationSize

    local enableHighlight = boolToInt(math.abs(self.highlightIntensity) > self.Eps)
    local enableShadow = boolToInt(math.abs(self.shadowIntensity) > self.Eps)
    local enableBlackWhite = boolToInt(math.abs(self.blackIntensity) > self.Eps or math.abs(self.whiteIntensity) > self.Eps)
    local enableExposure = boolToInt(math.abs(self.exposureIntensity) > self.Eps)
    local enableOffset = boolToInt(math.abs(self.offsetIntensity) > self.Eps)
    local enableContrast = boolToInt(math.abs(self.contrastIntensity) > self.Eps)
    local enableSaturation = boolToInt(math.abs(self.saturationIntensity) > self.Eps)
    local enableTemperatureTint = boolToInt(math.abs(self.temperatureIntensity) > self.Eps or math.abs(self.tintIntensity) > self.Eps)
    local enableLut = boolToInt(math.abs(self.lutIntensity) > self.Eps and self.lutTex ~= nil)
    local enableSurfaceBlur = boolToInt(math.abs(self.surfaceBlurIntensity) > self.Eps)
    local enableLinearOps = boolToInt(enableBlackWhite or enableExposure or enableOffset)
    local enableSaturationSuppresion = boolToInt(math.abs(self.saturationSuppressionIntensity) > self.Eps and self.colorRange == 'Shadow')
    local enableDehaze = boolToInt(math.abs(self.dahazeIntensity) > self.Eps)

    -- initialize parameters
    local highlightParam = 0.
    local shadowParam = 0.
    local blackWhiteSlope = 1.
    local blackWhiteBias = 0.
    local exposure = 1.
    local saturationParam = 1.
    local temperatureParam = 6500.
    local tintParam = 0.
    local ttParam = {{1., 0., 0.}, {0., 1., 0.}, {0., 0., 1.}}  -- tt = temperature-tint
    local dehazeParam = 0.5
    local lutColorSample = 64.
    local lutHorizontalGridNum = 8.
    local lutVerticalGridNum = 8.

    -- compute parameters by intensities (some complicate computations will rely to enable state)
    highlightParam = self.highlightIntensity * self.HighlightIntensityScale
    shadowParam = self.shadowIntensity * self.ShadowIntensityScale
    exposure = 10 ^ self.exposureIntensity
    dehazeParam = getDehazeParam(self.dahazeIntensity)
    lutColorSample = math.floor(self.lutColorSample)
    lutHorizontalGridNum = math.floor(self.lutHorizontalGridNum)
    lutVerticalGridNum = math.floor(self.lutVerticalGridNum)

    if enableBlackWhite == 1 then
        blackWhiteSlope, blackWhiteBias = getBlackWhiteParam(self.blackIntensity, self.whiteIntensity)
    end

    if enableSaturation == 1 then
        saturationParam = getSaturationParam(self.saturationIntensity)
    end

    if enableTemperatureTint == 1 then
        temperatureParam, tintParam = getTemperatureTintParam(self.temperatureIntensity, self.tintIntensity)
        local vec3 = self:computeTemperatureTintVec3(temperatureParam, tintParam)
        local vec3Base = self:computeTemperatureTintVec3(self.baseTemperature, self.baseTint)
        ttParam = self:computeTemperatureTintMat3(vec3, vec3Base)
    end

    -- call for skin segment algorithm (nn model)
    if self.colorRange == "Skin" then
        local skinSegInfo = Amaz.Algorithm.getAEAlgorithmResult():getSkinSegInfo()
        if skinSegInfo then
            local skinSegMask = skinSegInfo.data
            self.skinSegTex:storage(skinSegMask)
        end
    end

    -- set textures
    self.matColorAdjustBundle:setTex("u_inputTexture", self.InputTex)
    self.matColorAdjustBundle:setTex("u_lutTexture", self.lutTex)
    self.matColorAdjustBundle:setTex("u_skinSegTexture", self.skinSegTex)
    if enableDehaze == 1 then
        self.colorAdjustTex.width = textureWidth
        self.colorAdjustTex.height = textureHeight
        self.darkChannelTex.width = textureWidth
        self.darkChannelTex.height = textureHeight
        self.camDarkChannel.entity.visible = true
        self.camDCDownSample.entity.visible = true
        self.camAirLight.entity.visible = true
        self.camDehaze.entity.visible = true

        self.camColorAdjustBundle.renderTexture = self.colorAdjustTex
        self.matDarkChannel:setTex("u_inputTexture", self.colorAdjustTex)
        self.camDarkChannel.renderTexture = self.darkChannelTex
        self.camDCDownSample.inputTexture = self.darkChannelTex
        self.camDCDownSample.renderTexture = self.dcDownSampleTex
        self.matAirLight:setTex("u_inputTexture", self.dcDownSampleTex)
        self.camAirLight.renderTexture = self.airLightTex
        self.matDehaze:setTex("u_inputTexture", self.colorAdjustTex)
        self.matDehaze:setTex("u_darkChannelTex", self.darkChannelTex)
        self.matDehaze:setTex("u_airLightTex", self.airLightTex)

        if enableSurfaceBlur == 1 then
            self.dehazeTex.width = textureWidth
            self.dehazeTex.height = textureHeight
            self.camSurfaceBlur.entity.visible = true

            self.camDehaze.renderTexture = self.dehazeTex
            self.matSurfaceBlur:setTex("u_inputTexture", self.dehazeTex)
            self.camSurfaceBlur.renderTexture = self.OutputTex
        else
            self.dehazeTex.width = 1
            self.dehazeTex.height = 1
            self.camSurfaceBlur.entity.visible = false
            self.camDehaze.renderTexture = self.OutputTex
        end
    else
        self.colorAdjustTex.width = 1
        self.colorAdjustTex.height = 1
        self.darkChannelTex.width = 1
        self.darkChannelTex.height = 1
        self.dehazeTex.width = 1
        self.dehazeTex.height = 1
        self.camDarkChannel.entity.visible = false
        self.camDCDownSample.entity.visible = false
        self.camAirLight.entity.visible = false
        self.camDehaze.entity.visible = false

        if enableSurfaceBlur == 1 then
            self.colorAdjustTex.width = textureWidth
            self.colorAdjustTex.height = textureHeight
            self.camSurfaceBlur.entity.visible = true

            self.camColorAdjustBundle.renderTexture = self.colorAdjustTex
            self.matSurfaceBlur:setTex("u_inputTexture", self.colorAdjustTex)
            self.camSurfaceBlur.renderTexture = self.OutputTex
        else
            self.colorAdjustTex.width = 1
            self.colorAdjustTex.height = 1
            self.camSurfaceBlur.entity.visible = false
            self.camColorAdjustBundle.renderTexture = self.OutputTex
        end
    end

    -- set parameters
    self.matColorAdjustBundle:setFloat("u_shadowMapParam", self.shadowMapParam)
    self.matColorAdjustBundle:setFloat("u_highlightMapParam", self.highlightMapParam)
    self.matColorAdjustBundle:setInt("u_displayOriginalColor", displayOriginalColor)
    self.matColorAdjustBundle:setInt("u_displayRangeMap", displayRangeMap)
    self.matColorAdjustBundle:setInt("u_useRangeMap", useRangeMap)
    self.matColorAdjustBundle:setInt("u_displaySaturationSuppressionFactor", displaySaturationSuppressionFactor)
    self.matColorAdjustBundle:setInt("u_colorRange", colorRange)
    self.matColorAdjustBundle:setInt("u_mapType", mapType)

    self.matColorAdjustBundle:setInt("u_enableHighlight", enableHighlight)
    self.matColorAdjustBundle:setInt("u_enableShadow", enableShadow)
    self.matColorAdjustBundle:setInt("u_enableLinearOps", enableLinearOps)
    self.matColorAdjustBundle:setInt("u_enableSaturation", enableSaturation)
    self.matColorAdjustBundle:setInt("u_enableContrast", enableContrast)
    self.matColorAdjustBundle:setInt("u_enableTemperatureTint", enableTemperatureTint)
    self.matColorAdjustBundle:setInt("u_enableLut", enableLut)
    self.matColorAdjustBundle:setInt("u_enableSaturationSuppresion", enableSaturationSuppresion)

    self.matColorAdjustBundle:setFloat("u_highlightParam", highlightParam)
    self.matColorAdjustBundle:setFloat("u_shadowParam", shadowParam)
    self.matColorAdjustBundle:setFloat("u_blackWhiteSlope", blackWhiteSlope)
    self.matColorAdjustBundle:setFloat("u_blackWhiteBias", blackWhiteBias)
    self.matColorAdjustBundle:setFloat("u_exposure", exposure)
    self.matColorAdjustBundle:setFloat("u_offsetIntensity", self.offsetIntensity)
    self.matColorAdjustBundle:setFloat("u_contrastIntensity", self.contrastIntensity)
    self.matColorAdjustBundle:setFloat("u_contrastPivot", self.contrastPivot)
    self.matColorAdjustBundle:setFloat("u_saturationSuppressionIntensity", self.saturationSuppressionIntensity)
    self.matColorAdjustBundle:setFloat("u_saturationSuppressionThreshold", self.saturationSuppressionThreshold)
    self.matColorAdjustBundle:setFloat("u_saturationParam", saturationParam)
    self.matColorAdjustBundle:setVec3("u_temperatureTintRedVec3", Amaz.Vector3f(ttParam[1][1], ttParam[1][2], ttParam[1][3]))
    self.matColorAdjustBundle:setVec3("u_temperatureTintGreenVec3", Amaz.Vector3f(ttParam[2][1], ttParam[2][2], ttParam[2][3]))
    self.matColorAdjustBundle:setVec3("u_temperatureTintBlueVec3", Amaz.Vector3f(ttParam[3][1], ttParam[3][2], ttParam[3][3]))
    self.matColorAdjustBundle:setFloat("u_lutIntensity", self.lutIntensity)
    self.matColorAdjustBundle:setFloat("u_lutColorSample", lutColorSample)
    self.matColorAdjustBundle:setFloat("u_lutHorizontalGridNum", lutHorizontalGridNum)
    self.matColorAdjustBundle:setFloat("u_lutVerticalGridNum", lutVerticalGridNum)

    self.matAirLight:setFloat("u_airLightThreshold", self.AirLightThreshold)
    self.matDehaze:setFloat("u_dehazeParam", dehazeParam)

    self.matSurfaceBlur:setFloat("u_threshold", self.SurfaceBlurThreshold)
    self.matSurfaceBlur:setFloat("u_stepX", dx)
    self.matSurfaceBlur:setFloat("u_stepY", dy)
end

exports.LumiColorAdjustBundle = LumiColorAdjustBundle
return exports
