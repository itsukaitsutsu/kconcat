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


function SeekModeScript:start(comp)
    self.material = comp.entity:getComponent("MeshRenderer").sharedMaterials:get(0)

     self.Intensity = 1.0
     self.RngDown = 0.333
     self.RngUp = 0.550
     self.ShadowR = 0.0
     self.ShadowG = 0.0
     self.ShadowB = 0.0
     self.ShadowS = 0.0
     self.MidtoneR = 0.0
     self.MidtoneG = 0.0
     self.MidtoneB = 0.0
     self.MidtoneS = 0.0
     self.HightlightsR = 0.0
     self.HightlightsG = 0.0
     self.HightlightsB = 0.0
     self.HightlightsS = 0.0
     self.OffsetR = 0.0
     self.OffsetG = 0.0
     self.OffsetB = 0.0
     self.OffsetS = 0.0

    self.material:setFloat("Intensity", self.Intensity);
    self.material:setFloat("RngDown", self.RngDown);
    self.material:setFloat("RngUp", self.RngUp);
    self.material:setFloat("ShadowR", self.ShadowR);
    self.material:setFloat("ShadowG", self.ShadowG);
    self.material:setFloat("ShadowB", self.ShadowB);
    self.material:setFloat("ShadowS", self.ShadowS);
    self.material:setFloat("MidtoneR", self.MidtoneR);
    self.material:setFloat("MidtoneG", self.MidtoneG);
    self.material:setFloat("MidtoneB", self.MidtoneB);
    self.material:setFloat("MidtoneS", self.MidtoneS);
    self.material:setFloat("HightlightsR", self.HightlightsR);
    self.material:setFloat("HightlightsG", self.HightlightsG);
    self.material:setFloat("HightlightsB", self.HightlightsB);
    self.material:setFloat("HightlightsS", self.HightlightsS);
    self.material:setFloat("OffsetR", self.OffsetR);
    self.material:setFloat("OffsetG", self.OffsetG);
    self.material:setFloat("OffsetB", self.OffsetB);
    self.material:setFloat("OffsetS", self.OffsetS);

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


-- Interpretations for some common parameters in the following functions
-- NOTE: to avoid global parameters pollution, the following common parameters 
-- are defined in each function by `local` type

-- EPS = 1e-6           small positive number to avoid zero division
-- MAX_SLPOE = 9.0      maximum slope for shadow or highlights range
-- RNG_T = 0.05         half length for transition range


-- Functions for computing cubic polynomial coefficients
local function getShadow(x, shadow, rngDown)
    local EPS = 1e-6
    local MAX_SLOPE = 9.0
    local res = 0.0
    if shadow > 0.0 then
        res = x * (1.0 - shadow) + rngDown * shadow
    else
        local slope = (rngDown - shadow) / math.max(rngDown, EPS)
        if slope <= MAX_SLOPE then
            res = x * slope + shadow
        else
            res = x * MAX_SLOPE + (1 - MAX_SLOPE) * rngDown
        end
    end
    return res
end


local function getShadowDerivative(shadow, rngDown)
    local EPS = 1e-6
    local MAX_SLOPE = 9.0
    local res = 0.0
    if shadow > 0.0 then
        res = 1.0 - shadow
    else
        local slope = (rngDown - shadow) / math.max(rngDown, EPS)
        res = math.min(slope, MAX_SLOPE)
        -- extreme case
        if rngDown < 0.01 then
            res = 1.0 + 100.0 * (MAX_SLOPE - 1) * rngDown
        end
    end
    return res
end


local function getHighlights(x, hightlights, rngUp)
    local EPS = 1e-6
    local MAX_SLOPE = 9.0
    local res = 0.0
    if hightlights > 0.0 then
        local slope = (1.0 - rngUp + hightlights) / math.max(1.0 - rngUp, EPS)
        if slope <= MAX_SLOPE then
            res = x * slope - rngUp * hightlights / math.max(1.0 - rngUp, EPS)
        else
            res = x * MAX_SLOPE + (1.0 - MAX_SLOPE) * rngUp
        end
    else
        res = x * (1.0 + hightlights) - rngUp * hightlights
    end
    return res
end


local function getHighlightsDerivative(hightlights, rngUp)
    local EPS = 1e-6
    local MAX_SLOPE = 9.0
    local res = 0.0
    if hightlights > 0.0 then
        local slope = (1.0 - rngUp + hightlights) / math.max(1.0 - rngUp, EPS)
        res = math.min(slope, MAX_SLOPE)
        -- extreme case
        if rngUp > 0.99 then
            res = 1 + 100.0 * (MAX_SLOPE - 1) * (1.0 - rngUp)
        end
    else
        res = 1.0 + hightlights
    end
    return res
end


local function mappingMidtoneParam(midtone)
    local m1 = midtone
    local m2 = m1 * m1
    local m3 = m2 * m1
    local m4 = m3 * m1
    local m5 = m4 * m1
    local m6 = m5 * m1
    local m7 = m6 * m1
    local res = 1.0 - 1.10145 * m1 + 0.906076 * m2 - 0.681492 * m3 - 
                0.253415 * m4 + 0.272653 * m5 + 1.09324 * m6 - 0.909166 * m7
    return res
end


local function getMidtone(x, midtone, rngDown, rngUp)
    local EPS = 1e-6
    local res = 0.0
    local rngDiff = rngUp - rngDown
    local m = mappingMidtoneParam(midtone)

    -- adjust rng for horizontal direction
    x = (x - rngDown) / math.max(rngDiff, EPS)
    res = 1.0 - x^m
    res = 1.0 - res^(1.0 / m)
    -- adjust rng for vertical direction
    res = res * rngDiff + rngDown
    return res
end


local function getMidtoneDerivative(x, midtone, rngDown, rngUp)
    local EPS = 1e-6
    local res = 0.0
    local rngDiff = rngUp - rngDown
    local m = mappingMidtoneParam(midtone)

    x = (x - rngDown) / math.max(rngDiff, EPS)
    res = 1.0 - x^m
    x = math.max(x, EPS)
    res = res^(1.0 / m - 1.0) * x^(m - 1.0)
    return res
end


local function getCubicCoefficients(x1, y1, k1, x2, y2, k2)
    -- solve cubic coefficients by the continuous conditions of 
    -- curve values (y) and first derivatives (k)
    local EPS = 1e-6
    local f = x1 - x2
    if x1 == x2 then
        f = -EPS
    end

    x1Square = x1 * x1
    x1Cubic = x1Square * x1

    local a3 = -(2.0 * (y1 - y2) - (k1 + k2) * f) / (f^3)
    local a2 = ((k1 - k2) / f - 3.0 * a3 * (x1 + x2)) * 0.5
    local a1 = k1 - 2.0 * a2 * x1 - 3.0 * a3 * x1Square
    local a0 = y1 - a1 * x1 - a2 * x1Square - a3 * x1Cubic
    return a0, a1, a2, a3
end


local function getShadow2MidtoneCoef(shadow, midtone, rngDown, rngUp)
    local RNG_T = 0.05
    local rngMid = (rngDown + rngUp) / 2.0
    local x1 = math.max(rngDown - RNG_T, -rngDown)
    local y1 = getShadow(x1, shadow, rngDown)
    local k1 = getShadowDerivative(shadow, rngDown)
    local x2 = math.min(rngDown + RNG_T, rngMid)
    local y2 = getMidtone(x2, midtone, rngDown, rngUp)
    local k2 = getMidtoneDerivative(x2, midtone, rngDown, rngUp)
    local a0, a1, a2, a3 = getCubicCoefficients(x1, y1, k1, x2, y2, k2)
    return a0, a1, a2, a3
end


local function getHighlights2MidtoneCoef(highlights, midtone, rngDown, rngUp)
    local RNG_T = 0.05
    local rngMid = (rngDown + rngUp) / 2.0
    local x1 = math.max(rngUp-RNG_T, rngMid)
    local y1 = getMidtone(x1, midtone, rngDown, rngUp)
    local k1 = getMidtoneDerivative(x1, midtone, rngDown, rngUp)
    local x2 = math.min(rngUp + RNG_T, 2.0 - rngUp)
    local y2 = getHighlights(x2, highlights, rngUp)
    local k2 = getHighlightsDerivative(highlights, rngUp)
    local a0, a1, a2, a3 = getCubicCoefficients(x1, y1, k1, x2, y2, k2)
    return a0, a1, a2, a3
end


local function getShadow2HighlightsCoef(shadow, highlights, rngDown, rngUp)
    local RNG_T = 0.05
    local x1 = math.max(rngDown - RNG_T, -rngDown)
    local y1 = getShadow(x1, shadow, rngDown)
    local k1 = getShadowDerivative(shadow, rngDown)
    local x2 = math.min(rngUp + RNG_T, 2.0 - rngUp)
    local y2 = getHighlights(x2, highlights, rngUp)
    local k2 = getHighlightsDerivative(highlights, rngUp)
    local a0, a1, a2, a3 = getCubicCoefficients(x1, y1, k1, x2, y2, k2)
    return a0, a1, a2, a3
end


-- Functions for parameters adjustment
local function adjustShadow(shadow)
    local res = shadow
    if shadow < 0. then
        res = -2.7 * shadow * shadow
    end
    return res
end


local function adjustHightlights(hightlights)
    local res = hightlights
    if hightlights > 0. then
        res = 3.6 * hightlights * hightlights
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
			-- Interface parameters
			local intensity = clamp(event.args:get(1), -1, 1)
			if ("Intensity" == event.args:get(0)) then
				self.material:setFloat("Intensity", intensity);
			elseif ("RngDown" == event.args:get(0)) then
				self.RngDown = intensity
				self.material:setFloat("RngDown", self.RngDown);
			elseif ("RngUp" == event.args:get(0)) then
				self.RngUp = intensity
				self.material:setFloat("RngUp", self.RngUp);
			elseif ("ShadowR" == event.args:get(0)) then
				self.ShadowR = adjustShadow(intensity)
				self.material:setFloat("ShadowR", self.ShadowR);
			elseif ("ShadowG" == event.args:get(0)) then
				self.ShadowG = adjustShadow(intensity)
				self.material:setFloat("ShadowG", self.ShadowG);
			elseif ("ShadowB" == event.args:get(0)) then
				self.ShadowB = adjustShadow(intensity)
				self.material:setFloat("ShadowB", self.ShadowB);
			elseif ("ShadowS" == event.args:get(0)) then
				self.ShadowS = adjustShadow(intensity * 0.5)
				self.material:setFloat("ShadowS", self.ShadowS);
			elseif ("MidtoneR" == event.args:get(0)) then
				self.MidtoneR = intensity
				self.material:setFloat("MidtoneR", self.MidtoneR);
			elseif ("MidtoneG" == event.args:get(0)) then
				self.MidtoneG = intensity
				self.material:setFloat("MidtoneG", self.MidtoneG);
			elseif ("MidtoneB" == event.args:get(0)) then
				self.MidtoneB = intensity
				self.material:setFloat("MidtoneB", self.MidtoneB);
			elseif ("MidtoneS" == event.args:get(0)) then
				self.MidtoneS = intensity
				self.material:setFloat("MidtoneS", self.MidtoneS);
			elseif ("HightlightsR" == event.args:get(0)) then
				self.HightlightsR = adjustHightlights(intensity)
				self.material:setFloat("HightlightsR", self.HightlightsR);
			elseif ("HightlightsG" == event.args:get(0)) then
				self.HightlightsG = adjustHightlights(intensity)
				self.material:setFloat("HightlightsG", self.HightlightsG);
			elseif ("HightlightsB" == event.args:get(0)) then
				self.HightlightsB = adjustHightlights(intensity)
				self.material:setFloat("HightlightsB", self.HightlightsB);
			elseif ("HightlightsS" == event.args:get(0)) then
				self.HightlightsS = adjustHightlights(intensity * 0.5)
				self.material:setFloat("HightlightsS", self.HightlightsS);
			elseif ("OffsetR" == event.args:get(0)) then
				self.OffsetR = intensity
				self.material:setFloat("OffsetR", self.OffsetR);
			elseif ("OffsetG" == event.args:get(0)) then
				self.OffsetG = intensity
				self.material:setFloat("OffsetG", self.OffsetG);
			elseif ("OffsetB" == event.args:get(0)) then
				self.OffsetB = intensity
				self.material:setFloat("OffsetB", self.OffsetB);
			elseif ("OffsetS" == event.args:get(0)) then
				self.OffsetS = intensity
				self.material:setFloat("OffsetS", self.OffsetS);
			end
		end
    end
end


function SeekModeScript:onUpdate(comp, detalTime)
    self:seekToTime(comp, detalTime)

    -- NONE-interface parameters
    local rngDownMod = math.min(self.RngDown, self.RngUp)
    local a0, a1, a2, a3 = getShadow2MidtoneCoef(self.ShadowR, self.MidtoneR, rngDownMod, self.RngUp)
    self.material:setVec4("s2mR", Amaz.Vector4f(a0, a1, a2, a3))
    a0, a1, a2, a3 = getShadow2MidtoneCoef(self.ShadowG, self.MidtoneG, rngDownMod, self.RngUp)
    self.material:setVec4("s2mG", Amaz.Vector4f(a0, a1, a2, a3))
    a0, a1, a2, a3 = getShadow2MidtoneCoef(self.ShadowB, self.MidtoneB, rngDownMod, self.RngUp)
    self.material:setVec4("s2mB", Amaz.Vector4f(a0, a1, a2, a3))
    a0, a1, a2, a3 = getShadow2MidtoneCoef(self.ShadowS, self.MidtoneS, rngDownMod, self.RngUp)
    self.material:setVec4("s2mS", Amaz.Vector4f(a0, a1, a2, a3))

    a0, a1, a2, a3 = getHighlights2MidtoneCoef(self.HightlightsR, self.MidtoneR, rngDownMod, self.RngUp)
    self.material:setVec4("h2mR", Amaz.Vector4f(a0, a1, a2, a3))
    a0, a1, a2, a3 = getHighlights2MidtoneCoef(self.HightlightsG, self.MidtoneG, rngDownMod, self.RngUp)
    self.material:setVec4("h2mG", Amaz.Vector4f(a0, a1, a2, a3))
    a0, a1, a2, a3 = getHighlights2MidtoneCoef(self.HightlightsB, self.MidtoneB, rngDownMod, self.RngUp)
    self.material:setVec4("h2mB", Amaz.Vector4f(a0, a1, a2, a3))
    a0, a1, a2, a3 = getHighlights2MidtoneCoef(self.HightlightsS, self.MidtoneS, rngDownMod, self.RngUp)
    self.material:setVec4("h2mS", Amaz.Vector4f(a0, a1, a2, a3))

    a0, a1, a2, a3 = getShadow2HighlightsCoef(self.ShadowR, self.HightlightsR, rngDownMod, self.RngUp)
    self.material:setVec4("s2hR", Amaz.Vector4f(a0, a1, a2, a3))
    a0, a1, a2, a3 = getShadow2HighlightsCoef(self.ShadowG, self.HightlightsG, rngDownMod, self.RngUp)
    self.material:setVec4("s2hG", Amaz.Vector4f(a0, a1, a2, a3))
    a0, a1, a2, a3 = getShadow2HighlightsCoef(self.ShadowB, self.HightlightsB, rngDownMod, self.RngUp)
    self.material:setVec4("s2hB", Amaz.Vector4f(a0, a1, a2, a3))
    a0, a1, a2, a3 = getShadow2HighlightsCoef(self.ShadowS, self.HightlightsS, rngDownMod, self.RngUp)
    self.material:setVec4("s2hS", Amaz.Vector4f(a0, a1, a2, a3))
end

exports.SeekModeScript = SeekModeScript
return exports
