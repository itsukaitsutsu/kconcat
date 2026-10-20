---@class SeekModeScript: ScriptComponent
---@field vignettingIntensity number [UI(Range={-1, 1}, Drag)]


local exports = exports or {}
local SeekModeScript = SeekModeScript or {}
SeekModeScript.__index = SeekModeScript

function SeekModeScript.new(construct, ...)
    local self = setmetatable({}, SeekModeScript)

    if construct and SeekModeScript.constructor then SeekModeScript.constructor(self, ...) end
    self.vignettingIntensity = 0.
    self.vignettingParam = 0.
    return self
end

local function getBezierValue(controls, t)
    local ret = {}
    local xc1 = controls[1]
    local yc1 = controls[2]
    local xc2 = controls[3]
    local yc2 = controls[4]
    ret[1] = 3 * xc1 * (1 - t) * (1 - t) * t + 3 * xc2 * (1 - t) * t * t + t * t * t
    ret[2] = 3 * yc1 * (1 - t) * (1 - t) * t + 3 * yc2 * (1 - t) * t * t + t * t * t
    return ret
end

local function getBezierTfromX(controls, x)
    local ts = 0
    local te = 1
    -- divide and conque
    repeat
        local tm = (ts + te) / 2
        local value = getBezierValue(controls, tm)
        if (value[1] > x) then
            te = tm
        else
            ts = tm
        end
    until (te - ts < 0.0001)

    return (te + ts) / 2
end

local function bezier(controls)
    return function(t, b, c, d)
        t = t / d
        local tvalue = getBezierTfromX(controls, t)
        local value = getBezierValue(controls, tvalue)
        return b + c * value[2]
    end
end

function SeekModeScript:constructor()
end

function SeekModeScript:onStart(comp)
    self.material = comp.entity:searchEntity("EntityVignetting"):getComponent("MeshRenderer").material
end

function SeekModeScript:onUpdate(comp, deltaTime)

    -- local vignettingParam = 0.
    -- if self.vignettingIntensity > 0 then
    --     vignettingParam = bezier({0.2, 0.45, 0.63, 0.81})(self.vignettingIntensity * 0.5 + 0.5, 0, 1, 1) * 2 - 1
    -- else
    --     vignettingParam = bezier({.65, .2, .81, .63})(self.vignettingIntensity * 0.5 + 0.5, 0, 1, 1) * 2 - 1
    -- end

    self.material:setFloat("u_vignettingParam", self.vignettingParam)
    self.material:setFloat("u_vignettingIntensity", self.vignettingIntensity)
end


function SeekModeScript:onEvent(sys, event)
    if event.type == Amaz.AppEventType.SetEffectIntensity then
        if event.args:get(0) == "vignetting_intensity" then
            local intensity = event.args:get(1)
            self.vignettingIntensity = intensity
        end
        if event.args:size() == 2 and event.args:get(0) == "reset_params" and event.args:get(1) == 1 then
            self.vignettingIntensity = 0.
        end
        if self.vignettingIntensity > 0 then
            self.vignettingParam = bezier({0.2, 0.45, 0.63, 0.81})(self.vignettingIntensity * 0.5 + 0.5, 0, 1, 1) * 2 - 1
        else
            self.vignettingParam = bezier({.65, .2, .81, .63})(self.vignettingIntensity * 0.5 + 0.5, 0, 1, 1) * 2 - 1
        end
    end
end


exports.SeekModeScript = SeekModeScript
return exports
