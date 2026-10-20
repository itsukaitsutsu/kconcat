local exports = exports or {}
local SeekModeScript = SeekModeScript or {}
SeekModeScript.__index = SeekModeScript
function SeekModeScript.new(construct, ...)
    local self = setmetatable({}, SeekModeScript)
    if construct and SeekModeScript.constructor then
        SeekModeScript.constructor(self, ...)
    end

    return self
end

function SeekModeScript:constructor()
end

function SeekModeScript:onUpdate(comp, detalTime)
    self:seekToTime(comp, detalTime)
end

function SeekModeScript:start(comp)
    self.material = comp.entity:getComponent("MeshRenderer").sharedMaterials:get(0)

    self.material:setFloat("Intensity", 1.);
    self.material:setFloat("LiftY", 0.);
    self.material:setFloat("LiftR", 0.);
    self.material:setFloat("LiftG", 0.);
    self.material:setFloat("LiftB", 0.);
    self.material:setFloat("LiftS", 0.);
    self.material:setFloat("GammaY", 1.);
    self.material:setFloat("GammaR", 1.);
    self.material:setFloat("GammaG", 1.);
    self.material:setFloat("GammaB", 1.);
    self.material:setFloat("GammaS", 1.);
    self.material:setFloat("GainY", 1.);
    self.material:setFloat("GainR", 1.);
    self.material:setFloat("GainG", 1.);
    self.material:setFloat("GainB", 1.);
    self.material:setFloat("GainS", 1.);
    self.material:setFloat("OffsetR", 0.);
    self.material:setFloat("OffsetG", 0.);
    self.material:setFloat("OffsetB", 0.);
    self.material:setFloat("OffsetS", 0.);
    self.material:setFloat("LumaMix", 1.);
end

function SeekModeScript:seekToTime(comp, time)
    if self.first == nil then
        self.first = true
        self:start(comp)
    end
end


local function clamp(val, min, max)
    return math.max(math.min(val, max), min)
end


local function adjustLift(lift)
    local res = lift
    if lift > 0.0 then
        res = lift * 0.5
    end
    return res
end


local function adjustLiftS(liftS)
    return liftS * 0.5
end


local function adjustGamma(gamma)
    local res = gamma
    if gamma < 0.0 then
        res = 1.0 - gamma * 4.0
    else
        res = 1.0 / (1.0 + gamma * 4.0)
    end
    return res
end


local function adjustGammaS(gammaS)
    local res = gammaS
    if gammaS < 0. then
        res = 1.0 - gammaS * 4.0
    else
        res = 1.0 / (1.0 + gammaS * gammaS * 4.0)
    end
    return res
end


local function adjustGain(Gain)
    local res = Gain
    if Gain < 0. then
        res = 1.0 + Gain * 0.999
    else
        res = 1.0 + (Gain * Gain) * 4.0
    end
    return res
end

local function adjustGainS(Gain)
    local res = Gain
    if Gain < 0. then
        res = 1.0 + Gain * 0.999
    else
        res = 1.0 + (Gain * Gain) * 2.0
    end
    return res
end

function SeekModeScript:onEvent(sys, event)
    if self.first == nil then
        self.first = true
        self:start(sys)
    end
    if self.material == nil then 
        return
    end
    if event.type == Amaz.AppEventType.SetEffectIntensity then
		if (type(event.args:get(1)) == 'number') then
			local intensity = clamp(event.args:get(1), -1, 1)
			if ("Intensity" == event.args:get(0)) then
				self.material:setFloat("Intensity", intensity);
			elseif ("LiftY" == event.args:get(0)) then
				self.material:setFloat("LiftY", adjustLift(intensity));
			elseif ("LiftR" == event.args:get(0)) then
				self.material:setFloat("LiftR", adjustLift(intensity));
			elseif ("LiftG" == event.args:get(0)) then
				self.material:setFloat("LiftG", adjustLift(intensity));
			elseif ("LiftB" == event.args:get(0)) then
				self.material:setFloat("LiftB", adjustLift(intensity));
			elseif ("LiftS" == event.args:get(0)) then
				self.material:setFloat("LiftS", adjustLiftS(intensity));
			elseif ("GammaY" == event.args:get(0)) then
				self.material:setFloat("GammaY", adjustGamma(intensity));
			elseif ("GammaR" == event.args:get(0)) then
				self.material:setFloat("GammaR", adjustGamma(intensity));
			elseif ("GammaG" == event.args:get(0)) then
				self.material:setFloat("GammaG", adjustGamma(intensity));
			elseif ("GammaB" == event.args:get(0)) then
				self.material:setFloat("GammaB", adjustGamma(intensity));
			elseif ("GammaS" == event.args:get(0)) then
				self.material:setFloat("GammaS", adjustGammaS(intensity));
			elseif ("GainY" == event.args:get(0)) then
				self.material:setFloat("GainY", adjustGain(intensity));
			elseif ("GainR" == event.args:get(0)) then
				self.material:setFloat("GainR", adjustGain(intensity));
			elseif ("GainG" == event.args:get(0)) then
				self.material:setFloat("GainG", adjustGain(intensity));
			elseif ("GainB" == event.args:get(0)) then
				self.material:setFloat("GainB", adjustGain(intensity));
			elseif ("GainS" == event.args:get(0)) then
				self.material:setFloat("GainS", adjustGainS(intensity));
			elseif ("OffsetR" == event.args:get(0)) then
				self.material:setFloat("OffsetR", intensity);
			elseif ("OffsetG" == event.args:get(0)) then
				self.material:setFloat("OffsetG", intensity);
			elseif ("OffsetB" == event.args:get(0)) then
				self.material:setFloat("OffsetB", intensity);
			elseif ("OffsetS" == event.args:get(0)) then
				self.material:setFloat("OffsetS", intensity);
			elseif ("LumaMix" == event.args:get(0)) then
				self.material:setFloat("LumaMix", intensity);
			end
		end
    end
end

exports.SeekModeScript = SeekModeScript
return exports
