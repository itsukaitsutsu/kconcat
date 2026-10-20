local exports = exports or {}
local SeekModeScript = SeekModeScript or {}
SeekModeScript.__index = SeekModeScript
function SeekModeScript.new(construct, ...)
    local self = setmetatable({}, SeekModeScript)
    if construct and SeekModeScript.constructor then
        SeekModeScript.constructor(self, ...)
    end

    local vec2 = Amaz.Vec2Vector()
    vec2:pushBack(Amaz.Vector2f(0, 0))
    vec2:pushBack(Amaz.Vector2f(0.3, 0.3))
    vec2:pushBack(Amaz.Vector2f(0.6, 0.6))
    vec2:pushBack(Amaz.Vector2f(1, 1))

    self.control_points_count_y = 2
    self.control_points_y = vec2:copy()
    self.control_points_count_r = 2
    self.control_points_r = vec2:copy()
    self.control_points_count_g = 2
    self.control_points_g = vec2:copy()
    self.control_points_count_b = 2
    self.control_points_b = vec2:copy()
    return self
end

function SeekModeScript:constructor()
end

function SeekModeScript:onUpdate(comp, detalTime)
    self:seekToTime(comp, detalTime)
end

function SeekModeScript:start(comp)
    -- Amaz.LOGE('zglog: SeekModeScript:start'," ")
    self.material = comp.entity:getComponent("MeshRenderer").sharedMaterials:get(0)

    self.curveLUTY = comp.entity.scene:findEntityBy("curveLUTY"):getComponent("MeshRenderer").material
    self.curveLUTY:setVec2Vector("controlPoints", self.control_points_y)
    self.curveLUTY:setInt("pointsNums", self.control_points_count_y)

    self.curveLUTR = comp.entity.scene:findEntityBy("curveLUTR"):getComponent("MeshRenderer").material
    self.curveLUTR:setVec2Vector("controlPoints", self.control_points_r)
    self.curveLUTR:setInt("pointsNums", self.control_points_count_r)

    self.curveLUTG = comp.entity.scene:findEntityBy("curveLUTG"):getComponent("MeshRenderer").material
    self.curveLUTG:setVec2Vector("controlPoints", self.control_points_g)
    self.curveLUTG:setInt("pointsNums", self.control_points_count_g)

    self.curveLUTB = comp.entity.scene:findEntityBy("curveLUTB"):getComponent("MeshRenderer").material
    self.curveLUTB:setVec2Vector("controlPoints", self.control_points_b)
    self.curveLUTB:setInt("pointsNums", self.control_points_count_b)

end

function SeekModeScript:seekToTime(comp, time)
    -- Amaz.LOGE('zglog: seekToTime'," ")
    if self.first == nil then
        self.first = true
        self:start(comp)
    end
end

function SeekModeScript:onEvent(sys, event)
    if self.first == nil then
        self.first = true
        self:start(sys)
    end
    if self.material == nil or self.curveLUTY == nil or self.curveLUTR == nil or self.curveLUTG == nil or self.curveLUTB == nil then 
        return
    end
    if event.type == Amaz.AppEventType.SetEffectIntensity then
        if ("intensityY" == event.args:get(0)) then
            local intensity = event.args:get(1)
            self.material:setFloat("intensityY", intensity)
        elseif ("intensityR" == event.args:get(0)) then
            local intensity = event.args:get(1)
            self.material:setFloat("intensityR", intensity)
        elseif ("intensityG" == event.args:get(0)) then
            local intensity = event.args:get(1)
            self.material:setFloat("intensityG", intensity)
        elseif ("intensityB" == event.args:get(0)) then
            local intensity = event.args:get(1)
            self.material:setFloat("intensityB", intensity)
        elseif ("yPointsCount" == event.args:get(0)) then
            self.control_points_count_y = (event.args:get(1)+2)/3
            self.curveLUTY:setInt("pointsNums", self.control_points_count_y)
        elseif ("rPointsCount" == event.args:get(0)) then
            self.control_points_count_r = (event.args:get(1)+2)/3
            self.curveLUTR:setInt("pointsNums", self.control_points_count_r)
        elseif ("gPointsCount" == event.args:get(0)) then
            self.control_points_count_g = (event.args:get(1)+2)/3
            self.curveLUTG:setInt("pointsNums", self.control_points_count_g)
        elseif ("bPointsCount" == event.args:get(0)) then
            self.control_points_count_b = (event.args:get(1)+2)/3
            self.curveLUTB:setInt("pointsNums", self.control_points_count_b)
        elseif ("yPoints" == event.args:get(0)) then
            self.control_points_y = Amaz.Vec2Vector()
            for i = 0, event.args:get(1):size()/2 - 1 do
                self.control_points_y:pushBack(Amaz.Vector2f(event.args:get(1):get(i*2), event.args:get(1):get(i*2+1)))
            end
            -- Amaz.LOGE('zglog: yPoints size ', self.control_points_y:size())
            self.curveLUTY:setVec2Vector("controlPoints", self.control_points_y)
        elseif ("rPoints" == event.args:get(0)) then
            self.control_points_r = Amaz.Vec2Vector()
            for i = 0, event.args:get(1):size()/2 - 1 do
                self.control_points_r:pushBack(Amaz.Vector2f(event.args:get(1):get(i*2), event.args:get(1):get(i*2+1)))
            end
            self.curveLUTR:setVec2Vector("controlPoints", self.control_points_r)
        elseif ("gPoints" == event.args:get(0)) then
            self.control_points_g = Amaz.Vec2Vector()
            for i = 0, event.args:get(1):size()/2 - 1 do
                self.control_points_g:pushBack(Amaz.Vector2f(event.args:get(1):get(i*2), event.args:get(1):get(i*2+1)))
            end
            self.curveLUTG:setVec2Vector("controlPoints", self.control_points_g)
        elseif ("bPoints" == event.args:get(0)) then
            self.control_points_b = Amaz.Vec2Vector()
            for i = 0, event.args:get(1):size()/2 - 1 do
                self.control_points_b:pushBack(Amaz.Vector2f(event.args:get(1):get(i*2), event.args:get(1):get(i*2+1)))
            end
            self.curveLUTB:setVec2Vector("controlPoints", self.control_points_b)
        end
        if event.args:size() == 2 and event.args:get(0) == "reset_params" and event.args:get(1) == 1 then
            local intensity = 0.
            self.material:setFloat("intensityY", intensity)
            self.material:setFloat("intensityR", intensity)
            self.material:setFloat("intensityG", intensity)
            self.material:setFloat("intensityB", intensity)
        end
    end
end

exports.SeekModeScript = SeekModeScript
return exports
