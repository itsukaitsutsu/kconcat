local exports = exports or {}
local SeekModeScript = SeekModeScript or {}
SeekModeScript.__index = SeekModeScript
function SeekModeScript.new(construct, ...)
    local self = setmetatable({}, SeekModeScript)
    if construct and SeekModeScript.constructor then
        SeekModeScript.constructor(self, ...)
    end
    -- This HSL effect has two color origins: 8 default colors and <=10 user picked colors
    self.defaultColorNum = 8
    self.userColorNum = 10
    self.maxSliderValue = 100

    self.defaultSliderParam = {}
    for i = 1, self.defaultColorNum do
        self.defaultSliderParam[i] = Amaz.Vector3f(0.0, 0.0, 0.0)
    end

    self.defaultColorName = {"HSLRed", "HSLOrange", "HSLYellow", "HSLGreen", "HSLCyan", "HSLBlue", "HSLPurple", "HSLMagenta"}
    self.defaultColorMap = {}
    for i = 1, self.defaultColorNum do
        local colorName = self.defaultColorName[i]
        self.defaultColorMap[colorName] = i
    end

    self.userColor = {}
    self.userSliderParam = {}
    for i = 1, self.userColorNum do
        self.userColor[i] = Amaz.Vector3f(0.0, 0.0, 0.0)
        self.userSliderParam[i] = Amaz.Vector3f(0.0, 0.0, 0.0)
    end

    -- use deltaE to find hue range for user colors, key infos are already pre-computed as follows
    -- the first number is base hue, the second number is left or right offset with respect to deltaE=20
    self.leftOffsetKeyPts = {
        {0, 24},    {18, 38},   {24, 38},   {34, 18},
        {42, 14},   {54, 14},   {92, 34},   {150, 80},
        {160, 80},  {170, 32},  {188, 14},  {222, 14},
        {284, 66},  {288, 36},  {298, 26},  {316, 38},
        {326, 38},  {336, 26},  {350, 20},  {360, 24}
    }

    self.rightOffsetKeyPts = {
        {0, 30},    {26, 14},   {42, 14},   {60, 36},
        {70, 80},   {82, 80},   {142, 30},  {174, 14},
        {208, 14},  {214, 20},  {218, 66},  {246, 42},
        {272, 26},  {280, 38},  {290, 38},  {320, 22},
        {334, 22},  {342, 38},  {348, 38},  {360, 30}
    }
    self.softEdgeRatio = 0.7

    return self
end


local function isArrayContain(array, value)
    for i = 1, #array do
        if array[i] == value then
            return true
        end
    end
    return false
end


local function rgb2hsl(rgb)
    --[[
        rgb: Amaz.Vector3f type
    ]]
    local eps = 1e-6
    local r = rgb.x
    local g = rgb.y
    local b = rgb.z
    local cmax = math.max(r, math.max(g, b))
    local cmin = math.min(r, math.min(g, b))
    local delta = cmax - cmin
    local l = (cmax + cmin) / 2.0
    local h = 0.0
    local s = 0.0

    if delta > eps then
        if l <= 0.5 then
            s = delta / (cmax + cmin)
        else
            s = delta / (2.0 - (cmax + cmin))
        end

        if cmax == r then
            if g >= b then
                h = 60.0 * (g - b) / delta
            else
                h = 60.0 * (g - b) / delta + 360.0
            end
        elseif cmax == g then
            h = 60.0 * (b - r) / delta + 120.0
        else
            h = 60.0 * (r - g) / delta + 240.0
        end
    end

    return h, s, l
end


local function parseColorName(colorName)
    --[[
        Convert string type color name to number type color, normalized by 255 so as to between [0, 1].

        colorName: string type color name, e.g. "01#ad8a86"， 01 before # means order of color, ad8a86 after # means color code
    ]]
    local hexOrder = string.sub(colorName, 1, 2)
    local hexR = string.sub(colorName, 4, 5)
    local hexG = string.sub(colorName, 6, 7)
    local hexB = string.sub(colorName, 8, 9)

    local order = tonumber(hexOrder, 16) + 1
    local r = tonumber(hexR, 16) / 255.
    local g = tonumber(hexG, 16) / 255.
    local b = tonumber(hexB, 16) / 255.
    return order, r, g, b
end


local function splitColorString(colorString)
    local idx = string.find(colorString, "_")
    if idx == nil then
        return nil, nil
    else
        local colorName = string.sub(colorString, 1, idx - 1)
        local sliderName = string.sub(colorString, idx + 1, string.len(colorString))
        return colorName, sliderName
    end
end


local function linearInterpolate(pts, x)
    local n = #pts
    x = x % 360
    local y = 20.
    for i = 1, n - 1 do
        if x >= pts[i][1] and x < pts[i + 1][1] then
            local x1 = pts[i][1]
            local y1 = pts[i][2]
            local x2 = pts[i + 1][1]
            local y2 = pts[i + 1][2]
            y = y1 + (x - x1) * (y2 - y1) / (x2 - x1)
            break
        end
    end
    return y
end


local function getHueRange(rgb, leftOffsetKeyPts, rightOffsetKeyPts, softEdgeRatio)
    --[[
        rgb: user picked color, Amaz.Vector3f type
        leftOffsetKeyPts, rightOffsetKeyPts, softEdgeRatio: defined in `SeekModeScript.new()`
    ]]
    local h, s, l = rgb2hsl(rgb)
    local flatRatio = 1. - softEdgeRatio
    local leftOffset = linearInterpolate(leftOffsetKeyPts, h)
    local rightOffset = linearInterpolate(rightOffsetKeyPts, h)
    local outerLeft = h - leftOffset
    local outerRight = h + rightOffset
    local innerLeft = h - leftOffset * flatRatio
    local innerRight = h + rightOffset * flatRatio
    outerLeft = outerLeft % 360
    outerRight = outerRight % 360
    innerLeft = innerLeft % 360
    innerRight = innerRight % 360
    if s < 1e-5 then
        return Amaz.Vector4f(0., 0., 0., 0.)
    else
        return Amaz.Vector4f(outerLeft, innerLeft, innerRight, outerRight)
    end
end


function SeekModeScript:reset_param()
    for i = 1, self.defaultColorNum do
        self.defaultSliderParam[i] = Amaz.Vector3f(0.0, 0.0, 0.0)
    end

    for i = 1, self.userColorNum do
        self.userSliderParam[i] = Amaz.Vector3f(0.0, 0.0, 0.0)
    end
end


function SeekModeScript:constructor()
end


function SeekModeScript:onStart(comp)
    self.material = comp.entity:getComponent("MeshRenderer").sharedMaterials:get(0)
end

function SeekModeScript:seekToTime(comp, time)
end

function SeekModeScript:onUpdate(comp, detalTime)
    for i = 1, self.defaultColorNum do
        self.material:setVec3("u_defaultParam_" .. i, self.defaultSliderParam[i])
    end

    -- use Vec3Vector to save user picked colors
    local userColorHueRange = Amaz.Vec4Vector()
    local userParam = Amaz.Vec3Vector()
    for i = 1, self.userColorNum do
        local hueRange = getHueRange(self.userColor[i], self.leftOffsetKeyPts, self.rightOffsetKeyPts, self.softEdgeRatio)
        userColorHueRange:pushBack(hueRange)
        userParam:pushBack(self.userSliderParam[i])
    end
    self.material:setVec4Vector("u_userColorHueRange", userColorHueRange)
    self.material:setVec3Vector("u_userParam", userParam)
end


function SeekModeScript:onEvent(sys, event)
    if event.args:size() == 2 and event.args:get(0) == "reset_params" and event.args:get(1) == 1 then
        self.reset_param(self)
    else
        local colorString = event.args:get(0)
        local sliderParam = event.args:get(1)
        local colorName, sliderName = splitColorString(colorString)

        if colorName ~= nil and isArrayContain({"H", "S", "L"}, sliderName) then
            if isArrayContain(self.defaultColorName, colorName) == true then
                local order = self.defaultColorMap[colorName]
                if sliderName == "H" then
                    self.defaultSliderParam[order].x = (sliderParam * 2. - 1.) * self.maxSliderValue
                elseif sliderName == "S" then
                    self.defaultSliderParam[order].y = (sliderParam * 2. - 1.) * self.maxSliderValue
                elseif sliderName == "L" then
                    self.defaultSliderParam[order].z = (sliderParam * 2. - 1.) * self.maxSliderValue
                end
            elseif string.sub(colorName, 3, 3) == "#" then
                local order, r, g, b = parseColorName(colorName)
                self.userColor[order] = Amaz.Vector3f(r, g, b)
                if sliderName == "H" then
                    self.userSliderParam[order].x = (sliderParam * 2. - 1.) * self.maxSliderValue
                    self.userSliderParam[order].x = self.userSliderParam[order].x * 0.3
                elseif sliderName == "S" then
                    self.userSliderParam[order].y = (sliderParam * 2. - 1.) * self.maxSliderValue
                elseif sliderName == "L" then
                    self.userSliderParam[order].z = (sliderParam * 2. - 1.) * self.maxSliderValue
                    self.userSliderParam[order].z = self.userSliderParam[order].z * 0.37
                end
            end
        end
    end
end


exports.SeekModeScript = SeekModeScript
return exports
