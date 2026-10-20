
local exports = exports or {}
local SeekModeScript = SeekModeScript or {}
SeekModeScript.__index = SeekModeScript
function SeekModeScript.new(construct, ...)
    local self = setmetatable({}, SeekModeScript)
    if construct and SeekModeScript.constructor then
        SeekModeScript.constructor(self, ...)
    end

    self.intensity = 0.0

    self.ratio = 0.25
    self.seed = math.random()
    self.lastTime = 0.0
    self.curTime = 0.0
    return self
end


function SeekModeScript:constructor()
end


function SeekModeScript:onUpdate(comp, detalTime)
    if self.curTime ~= self.lastTime then
        self.seed = math.random()
        self.lastTime = self.curTime
    end
    self.noiseMaterial:setFloat("Intensity", self.intensity * self.ratio)
    self.noiseMaterial:setFloat("Seed", self.seed)
end


function SeekModeScript:onStart(comp)
    self.noiseMaterial = comp.entity.scene:findEntityBy("noise"):getComponent("MeshRenderer").material
end


function SeekModeScript:seekToTime(comp, time)
end


function SeekModeScript:onEvent(sys, event)
    if event.type == Amaz.AppEventType.SetEffectIntensity then
        if event.args:get(0) == "particle_intensity" then
            self.intensity = tonumber(event.args:get(1))
        end
        if event.args:size() == 2 and event.args:get(0) == "reset_params" and event.args:get(1) == 1 then
            self.intensity = 0.
        end
    end
end

exports.SeekModeScript = SeekModeScript
return exports
