--write by editor  EffectSDK:10.5.0 EngineVersion:10.66.0 EditorBuildTime:Dec__7_2021_12_06_29
--sliderVersion: 20210901  Lua generation date: Thu Sep  8 23:23:40 2022


local exports = exports or {}
local ImageBusinessSlider = ImageBusinessSlider or {}
ImageBusinessSlider.__index = ImageBusinessSlider


function ImageBusinessSlider.new(construct, ...)
    local self = setmetatable({}, ImageBusinessSlider)
    if construct and ImageBusinessSlider.constructor then
        ImageBusinessSlider.constructor(self, ...)
    end
    return self
end


local function remap(x, a, b)
    return x * (b - a) + a
end


function ImageBusinessSlider:onStart(sys)
    self.Pass0Material0 = sys.scene:findEntityBy("Pass0"):getComponent("Renderer").material
end


function ImageBusinessSlider:onEvent(sys,event)
    if event.args:get(0) == "splendor_intensity" then
        local intensity = event.args:get(1)
        intensity = intensity * 0.5 + 0.5
        self.Pass0Material0["ins"] = remap(intensity,0,1)
    end
end


exports.ImageBusinessSlider = ImageBusinessSlider
return exports