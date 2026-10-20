--@input float curTime = 0.0{"widget":"slider","min":0,"max":10}

local exports = exports or {}
local SeekModeScript = SeekModeScript or {}
SeekModeScript.__index = SeekModeScript

function SeekModeScript.new(construct, ...)
    local self = setmetatable({}, SeekModeScript)
    if construct and SeekModeScript.constructor then SeekModeScript.constructor(self, ...) end
    self.startTime = 0.0
    self.endTime = 10.0  
    self.curTime = 0.0
    self.width = 0
    self.height = 0
    return self
end

function SeekModeScript:onStart(comp)

    self.tableCom = comp.entity.scene:findEntityBy("Table"):getComponent("TableComponent").table

    self.props    = comp.entity:getComponent("ScriptComponent").properties
    
    self.pass0mat = comp.entity.scene:findEntityBy("Pass0"):getComponent("MeshRenderer").material
    
    
    --"Transform"、"AnimSeqComponent"
end

function SeekModeScript:onUpdate(comp, detalTime)
    self.curTime=self.curTime+detalTime
    self:seekToTime(comp, self.curTime - self.startTime)
end

function SeekModeScript:seekToTime(comp, time)
    self.pass0mat:setFloat("iTime", time)
end


exports.SeekModeScript = SeekModeScript
return exports






