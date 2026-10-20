---@class SeekModeScript: ScriptComponent


local exports = exports or {}
local SeekModeScript = SeekModeScript or {}
SeekModeScript.__index = SeekModeScript


------------ util functions ------------
local function clamp(value, min, max)
    --[[
        Description: clamp value between [min, max]
    ]]
    return math.min(math.max(value, min), max)
end


local function mix(x, y, a)
    return x * (1. - a) + y * a
end


local function boolToInt(flag)
    --[[
        Description: convert flag from bool to int
    ]]
    if flag then return 1 end
    return 0
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


function SeekModeScript.new(construct, ...)
    local self = setmetatable({}, SeekModeScript)

    if construct and SeekModeScript.constructor then SeekModeScript.constructor(self, ...) end
    -- intensity parameters
    self.temperatureIntensity = 0.
    self.tintIntensity = 0.
    self.saturationIntensity = 0.
    self.brightnessIntensity = 0.
    self.contrastIntensity = 0.
    self.highlightIntensity = 0.
    self.shadowIntensity = 0.
    self.whiteIntensity = 0.
    self.blackIntensity = 0.
    self.lightSensationIntensity = 0.
    self.fadeIntensity = 0.

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

    -- other parameters
    self.MinIntensity = -1.
    self.MaxIntensity = 1.
    self.ContrastPivot = 0.435
    self.Eps = 1e-5

    return self
end

function SeekModeScript:constructor()
end


------------ parameter acquisition functions ------------
local function getTemperatureTintParam(temperatureIntensity, tintIntensity)
    local t = temperatureIntensity
    local t2 = t * t
    local t3 = t2 * t
    local temperatureParam = 6500. - 1970. * t + 876. * t2 - 2630. * t3
    local tintParam = tintIntensity * 100.
    return temperatureParam, tintParam
end


function SeekModeScript:computeTemperatureTintVec3(temperature, tint)
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


function SeekModeScript:computeTemperatureTintMat3(vec3, vec3Base)
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
    return intensity + 1.0
end


local function getBrightnessParam(intensity)
    local param = intensity
    if intensity > 0 and intensity <= 0.7 then
        param = 0.3 * intensity
    elseif intensity > 0.7 and intensity <= 1.0 then
        param = 0.6333 * intensity - 0.2333
    end
    return param
end


local function getContrastSigmoidXFactor(intensity)
    --[[
        intensity is between [0, 2]
    ]]
    return math.exp(8.33 * intensity - 12.16) + 5.82 * intensity - 1.72
end


local function getContrastSigmoidParam(x, intensity, pivot)
    --[[
        intensity is between [0, 2]
    ]]
    local a = getContrastSigmoidXFactor(intensity)
    local s = 1.0 / (1.0 + math.exp(-a * (x - pivot)))
    local y = s + pivot - 0.5       -- sigmoid value
    local k = a * s * (1.0 - s)     -- sigmoid derivative
    return y, k
end


local function getContrastParam(intensity, pivot)
    local param = 1.
    if intensity <= 0.0 then
        param = intensity * 0.4 + 1.0
    else
        param = intensity * 0.6 + 1.0
    end

    local xFactor = getContrastSigmoidXFactor(param)
    local leftValue, leftSlope = getContrastSigmoidParam(0.0, param, pivot)
    local rightValue, rightSlope = getContrastSigmoidParam(1.0, param, pivot)
    local pivotValue, pivotSlope = getContrastSigmoidParam(pivot, param, pivot)
    local leftDiff = 0.0 - leftValue
    local rightDiff = 1.0 - rightValue
    local leftSlopeDiff = pivotSlope - leftSlope
    local rightSlopeDiff = pivotSlope - rightSlope

    return param, xFactor, leftDiff, rightDiff, leftSlopeDiff, rightSlopeDiff, pivotSlope
end


-- Formula of shadow is y = x^a + (a-1)(x^2 - x^3). 
-- Formula of highlight is y = 1 - (1-x)^a - (a-1)((1-x)^2 - (1-x)^3)
-- The following two functions is to obtain parameter `a` in shadow and highlight formulae
local function getHighlightParam(intensity)
    local p = intensity
    local p2 = p * p
    local p3 = p2 * p
    local p4 = p3 * p
    local param = 1.0 + 0.503 * p + 0.183 * p2 + 0.147 * p3 + 0.067 * p4
    return param
end


local function getShadowParam(intensity)
    local p = intensity
    local p2 = p * p
    local p3 = p2 * p
    local p4 = p3 * p
    local param = 1.0 - 0.503 * p + 0.183 * p2 - 0.147 * p3 + 0.067 * p4
    return param
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


function SeekModeScript:onStart(comp)
    self.material = comp.entity:searchEntity("EntityColorAdjust"):getComponent("MeshRenderer").material
end


function SeekModeScript:onEvent(sys, event)
    if event.type == Amaz.AppEventType.SetEffectIntensity then
        -- tint
        if event.args:get(0) == "tint_intensity" then
            local intensity = event.args:get(1)
            self.tintIntensity = intensity
        end

        -- temperature
        if event.args:get(0) == "temperature_intensity" then
            local intensity = event.args:get(1)
            self.temperatureIntensity = intensity
        end

        -- saturation
        if event.args:get(0) == "saturation_intensity" then
            local intensity = event.args:get(1)
            self.saturationIntensity = intensity
        end

        -- brightness
        if event.args:get(0) == "brightness_intensity" then
            local intensity = event.args:get(1)
            self.brightnessIntensity = intensity
        end
        
        -- contrast
        if event.args:get(0) == "contrast_intensity" then
            local intensity = event.args:get(1)
            self.contrastIntensity = intensity
        end

        -- highlight
        if event.args:get(0) == "highlight_intensity" then
            local intensity = event.args:get(1)
            self.highlightIntensity = intensity
        end

        -- shadow
        if event.args:get(0) == "shadow_intensity" then
            local intensity = event.args:get(1)
            self.shadowIntensity = intensity
        end

        -- white
        if event.args:get(0) == "white_intensity" then
            local intensity = event.args:get(1)
            self.whiteIntensity = intensity
        end

        -- black
        if event.args:get(0) == "black_intensity" then
            local intensity = event.args:get(1)
            self.blackIntensity = intensity
        end

        -- light_sensation
        if event.args:get(0) == "light_sensation_intensity" then
            local intensity = event.args:get(1)
            self.lightSensationIntensity = intensity
        end

        -- fade
        if event.args:get(0) == "fade_intensity" then
            local intensity = event.args:get(1)
            self.fadeIntensity = intensity
        end
        if event.args:size() == 2 and event.args:get(0) == "reset_params" and event.args:get(1) == 1 then
            self.temperatureIntensity = 0.
            self.tintIntensity = 0.
            self.saturationIntensity = 0.
            self.brightnessIntensity = 0.
            self.contrastIntensity = 0.
            self.highlightIntensity = 0.
            self.shadowIntensity = 0.
            self.whiteIntensity = 0.
            self.blackIntensity = 0.
            self.lightSensationIntensity = 0.
            self.fadeIntensity = 0.
        end
    end
end


function SeekModeScript:onUpdate(comp, deltaTime)
end

function SeekModeScript:onUpdate(sys, deltaTime)
    local enableTemperatureTint = boolToInt(math.abs(self.temperatureIntensity) > self.Eps or math.abs(self.tintIntensity) > self.Eps)
    local enableSaturation = boolToInt(math.abs(self.saturationIntensity) > self.Eps)
    local enableBrightness = boolToInt(math.abs(self.brightnessIntensity) > self.Eps)
    local enableContrast = boolToInt(math.abs(self.contrastIntensity) > self.Eps)
    local enableHighlight = boolToInt(math.abs(self.highlightIntensity) > self.Eps)
    local enableShadow = boolToInt(math.abs(self.shadowIntensity) > self.Eps)
    local enableBlackWhite = boolToInt(math.abs(self.blackIntensity) > self.Eps or math.abs(self.whiteIntensity) > self.Eps)
    local enableLightSensation = boolToInt(math.abs(self.lightSensationIntensity) > self.Eps)
    local enableFade = boolToInt(math.abs(self.fadeIntensity) > self.Eps)

    -- initialize parameters
    local temperatureParam = 6500.
    local tintParam = 0.
    local ttParam = {{1., 0., 0.}, {0., 1., 0.}, {0., 0., 1.}}  -- tt = temperature-tint
    local saturationParam = 1.
    local brightnessParam = 0.
    local contrastParam = 1.
    local contrastXFactor = 1.
    local contrastLeftDiff = 0.
    local contrastRightDiff = 0.
    local contrastLeftSlopeDiff = 0.
    local contrastRightSlopeDiff = 0.
    local contrastPivotSlope = 1.
    local highlightParam = 0.
    local shadowParam = 0.
    local blackWhiteSlope = 1.
    local blackWhiteBias = 0.


    -- compute parameters by intensities and enable state
    if enableTemperatureTint == 1 then
        self.temperatureIntensity = clamp(self.temperatureIntensity, self.MinIntensity, self.MaxIntensity)
        self.tintIntensity = clamp(self.tintIntensity, self.MinIntensity, self.MaxIntensity)
        temperatureParam, tintParam = getTemperatureTintParam(self.temperatureIntensity, self.tintIntensity)
        local vec3 = self:computeTemperatureTintVec3(temperatureParam, tintParam)
        local vec3Base = self:computeTemperatureTintVec3(self.baseTemperature, self.baseTint)
        ttParam = self:computeTemperatureTintMat3(vec3, vec3Base)
    end

    if enableSaturation == 1 then
        self.saturationIntensity = clamp(self.saturationIntensity, self.MinIntensity, self.MaxIntensity)
        saturationParam = getSaturationParam(self.saturationIntensity)
    end

    if enableBrightness == 1 then
        self.brightnessIntensity = clamp(self.brightnessIntensity, self.MinIntensity, self.MaxIntensity)
        brightnessParam = getBrightnessParam(self.brightnessIntensity)
    end

    if enableContrast == 1 then
        contrastParam, contrastXFactor, contrastLeftDiff, contrastRightDiff, contrastLeftSlopeDiff, contrastRightSlopeDiff, contrastPivotSlope =
            getContrastParam(self.contrastIntensity, self.ContrastPivot)
    end

    if enableHighlight == 1 then
        self.highlightIntensity = clamp(self.highlightIntensity, self.MinIntensity, self.MaxIntensity)
        highlightParam = getHighlightParam(self.highlightIntensity)
    end

    if enableShadow == 1 then
        self.shadowIntensity = clamp(self.shadowIntensity, self.MinIntensity, self.MaxIntensity)
        shadowParam = getShadowParam(self.shadowIntensity)
    end

    if enableBlackWhite == 1 then
        self.blackIntensity = clamp(self.blackIntensity, self.MinIntensity, self.MaxIntensity)
        self.whiteIntensity = clamp(self.whiteIntensity, self.MinIntensity, self.MaxIntensity)
        blackWhiteSlope, blackWhiteBias = getBlackWhiteParam(self.blackIntensity, self.whiteIntensity)
    end

    if enableLightSensation == 1 then
        self.lightSensationIntensity = clamp(self.lightSensationIntensity, self.MinIntensity, self.MaxIntensity)
    end

    if enableFade == 1 then
        self.fadeIntensity = clamp(self.fadeIntensity, 0., self.MaxIntensity)
    end

    -- set material parameters for enable state
    self.material:setInt("u_enableTemperatureTint", enableTemperatureTint)
    self.material:setInt("u_enableSaturation", enableSaturation)
    self.material:setInt("u_enableBrightness", enableBrightness)
    self.material:setInt("u_enableContrast", enableContrast)
    self.material:setInt("u_enableHighlight", enableHighlight)
    self.material:setInt("u_enableShadow", enableShadow)
    self.material:setInt("u_enableBlackWhite", enableBlackWhite)
    self.material:setInt("u_enableLightSensation", enableLightSensation)
    self.material:setInt("u_enableFade", enableFade)

    -- set material parameters for algorithm
    self.material:setVec3("u_temperatureTintRedVec3", Amaz.Vector3f(ttParam[1][1], ttParam[1][2], ttParam[1][3]))
    self.material:setVec3("u_temperatureTintGreenVec3", Amaz.Vector3f(ttParam[2][1], ttParam[2][2], ttParam[2][3]))
    self.material:setVec3("u_temperatureTintBlueVec3", Amaz.Vector3f(ttParam[3][1], ttParam[3][2], ttParam[3][3]))

    self.material:setFloat("u_saturationParam", saturationParam)
    self.material:setFloat("u_brightnessParam", brightnessParam)

    self.material:setFloat("u_contrastParam", contrastParam)
    self.material:setFloat("u_contrastPivot", self.ContrastPivot)
    self.material:setFloat("u_contrastXFactor", contrastXFactor)
    self.material:setFloat("u_contrastLeftDiff", contrastLeftDiff)
    self.material:setFloat("u_contrastRightDiff", contrastRightDiff)
    self.material:setFloat("u_contrastLeftSlopeDiff", contrastLeftSlopeDiff)
    self.material:setFloat("u_contrastRightSlopeDiff", contrastRightSlopeDiff)
    self.material:setFloat("u_contrastPivotSlope", contrastPivotSlope)

    self.material:setFloat("u_highlightParam", highlightParam)
    self.material:setFloat("u_shadowParam", shadowParam)

    self.material:setFloat("u_blackWhiteSlope", blackWhiteSlope)
    self.material:setFloat("u_blackWhiteBias", blackWhiteBias)

    self.material:setFloat("u_lightSensationParam", self.lightSensationIntensity)
    self.material:setFloat("u_fadeParam", self.fadeIntensity)
end


exports.SeekModeScript = SeekModeScript
return exports
