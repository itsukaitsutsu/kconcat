local exports = exports or {}
local SeekModeScript = SeekModeScript or {}
SeekModeScript.__index = SeekModeScript
function SeekModeScript.new(construct, ...)
    local self = setmetatable({}, SeekModeScript)
    if construct and SeekModeScript.constructor then
        SeekModeScript.constructor(self, ...)
    end
    self.preview_effect_id = nil
    self.u_position = Amaz.Vector2f(0.0, 0.0)
    self.u_aspect = Amaz.BuiltinObject.getOutputTextureWidth() / Amaz.BuiltinObject.getOutputTextureHeight()
    self.u_rotation = 0.0
    self.u_scale = Amaz.Vector2f(1.0, 1.0)
    return self
end

function SeekModeScript:constructor()
end

function SeekModeScript:onStart(comp)
    self.material = comp.entity:searchEntity("blend"):getComponent("MeshRenderer").material

    if self.preview_effect_id ~= nil then
        self.previewMask = Amaz.BuiltinObject.getUserTexture(self.preview_effect_id)
        self.material:setTex("u_previewMask", self.previewMask)
    end

    self.material:setVec2("u_position", self.u_position)
    self.material:setFloat("u_rotation", self.u_rotation)
    self.material:setVec2("u_scale", self.u_scale)
    self.u_aspect = Amaz.BuiltinObject.getOutputTextureWidth() / Amaz.BuiltinObject.getOutputTextureHeight()
    self.material:setFloat("u_aspect", self.u_aspect)
end

function SeekModeScript:onUpdate(comp, detalTime)
    if self.preview_effect_id ~= nil then
        self.previewMask = Amaz.BuiltinObject.getUserTexture(self.preview_effect_id)
        self.material:setTex("u_previewMask", self.previewMask)
    end

    self.material:setVec2("u_position", self.u_position)
    self.material:setFloat("u_rotation", self.u_rotation)
    self.material:setVec2("u_scale", self.u_scale)
    self.u_aspect = Amaz.BuiltinObject.getOutputTextureWidth() / Amaz.BuiltinObject.getOutputTextureHeight()
    self.material:setFloat("u_aspect", self.u_aspect)
end

function SeekModeScript:seekToTime(comp, time)
end

function SeekModeScript:onEvent(sys, event)
    if event.type == Amaz.AppEventType.SetEffectIntensity then
        if event.args:get(0) == "preview_effect_id" then
            self.preview_effect_id = event.args:get(1)
        elseif event.args:get(0) == "px" then
            self.u_position.x = event.args:get(1)
        elseif event.args:get(0) == "py" then
            self.u_position.y = -1.0 * event.args:get(1)
        elseif event.args:get(0) == "rz" then
            local r = -1.0 * event.args:get(1)
            local rotation = (360.0 - r) * 2.0 * math.pi / 360.0
            self.u_rotation = rotation
        elseif event.args:get(0) == "sx" then
            self.u_scale.x = event.args:get(1)
        elseif event.args:get(0) == "sy" then
            self.u_scale.y = event.args:get(1)
        end
    end
end

exports.SeekModeScript = SeekModeScript
return exports