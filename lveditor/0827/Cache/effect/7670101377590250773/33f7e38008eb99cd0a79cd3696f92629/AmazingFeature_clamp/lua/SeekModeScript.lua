
--@input float curTime = 0.0{"widget":"slider","min":0,"max":1}
local isEditor = (Amaz.Macros and Amaz.Macros.EditorSDK) and true or false
local exports = exports or {}
local SeekModeScript = SeekModeScript or {}
SeekModeScript.__index = SeekModeScript

function SeekModeScript.new(construct, ...)
    local self = setmetatable({}, SeekModeScript)
    if construct and SeekModeScript.constructor then SeekModeScript.constructor(self, ...) end
    return self
end

function SeekModeScript:onStart(comp)
    
end

function SeekModeScript:onUpdate(comp, detalTime)
    
end

exports.SeekModeScript = SeekModeScript
return exports






