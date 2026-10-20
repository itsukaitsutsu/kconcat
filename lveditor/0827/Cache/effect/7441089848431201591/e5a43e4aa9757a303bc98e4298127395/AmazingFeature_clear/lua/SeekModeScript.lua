local exports = exports or {}
local SeekModeScript = SeekModeScript or {}
SeekModeScript.__index = SeekModeScript

---@class SeekModeScript : ScriptComponent
---@field fact number
---@field sat_ratio number
---@field intensity number
---@field resOffset number

function SeekModeScript.new(construct, ...)
    local self = setmetatable({}, SeekModeScript)

    if construct and SeekModeScript.constructor then SeekModeScript.constructor(self, ...) end
    self.matPyr = {}
    self.camPyr = {}
    self.camPyrMid = {}
    self.camPyrMidUp = {}
    self.matPyrAdd = {}
    self.nlevel = 8
    self.fact = .55
    self.sat_ratio = 1.12
    self.sharpIntensity = 0.
    self.intensity = 1.
    self.resOffset = 0.
    return self
end

function SeekModeScript:constructor()
end

function SeekModeScript:onStart(comp)
    for i = 0, 7 do
        self.matPyr[i] = comp.entity.scene:findEntityBy("PassPyr"..i):getComponent("MeshRenderer").material
    end
    self.matFinal = comp.entity.scene:findEntityBy("PassFinal"):getComponent("MeshRenderer").material
    self.matSaturation = comp.entity.scene:findEntityBy("PassSaturation"):getComponent("MeshRenderer").material
    for i = 1, 7 do
        self.camPyr[i] = comp.entity.scene:findEntityBy("CameraPyr"..i):getComponent("Camera")
        self.camPyrMid[i] = comp.entity.scene:findEntityBy("CameraPyr"..i.."Mid"):getComponent("Camera")
        self.camPyrMidUp[i] = comp.entity.scene:findEntityBy("CameraPyr"..i.."MidUp"):getComponent("Camera")
        self.matPyrAdd[i] = comp.entity.scene:findEntityBy("PassPyr"..i.."Add"):getComponent("MeshRenderer").material
    end
    self.camPyr[0] = comp.entity.scene:findEntityBy("CameraPyr"..'0'):getComponent("Camera")
    self.camPyrMid[0] = comp.entity.scene:findEntityBy("CameraValue"):getComponent("Camera")
end

function SeekModeScript:onUpdate(comp, deltaTime)
    local w = Amaz.BuiltinObject:getInputTextureWidth()
    local h = Amaz.BuiltinObject:getInputTextureHeight()
    local whMinLog = math.ceil(math.log(math.min(w, h)) / math.log(2))
    self.nlevel = math.min(8, whMinLog + 1)
    -- Amaz.LOGI("ldrldr", self.nlevel)
    for i = 0, self.nlevel - 1 - 1 do
        self.matPyr[i]:setFloat("subFlag", 1.)
    end
    self.matPyr[self.nlevel - 1]:setFloat("subFlag", 0.)
    for i = 0, 6 do
        -- Amaz.LOGI("ldrldr", i)
        self.matPyr[i]:setFloat("fact", self.fact)
        self.matPyr[i]:setFloat("resOffset", self.resOffset)
    end
    for i = 1, 7 do
        if i <= self.nlevel then
            self.matPyrAdd[i]:setFloat("addFlag", 1.)
        else
            self.matPyrAdd[i]:setFloat("addFlag", 0.)
        end
    end
    self.matFinal:setFloat("nlevel", self.nlevel)
    self.matFinal:setFloat("intensity", self.intensity * self.sharpIntensity)
    self.matSaturation:setFloat("sat_ratio", self.sat_ratio)

    
    local standard = math.min(w, h)
    local standardMin = 10

    for i = self.nlevel, 7 do
        self.camPyr[i].renderTexture.pecentX = standardMin / w
        self.camPyr[i].renderTexture.pecentY = standardMin / h
        -- Amaz.LOGI("ldrldr-level camPyr", i..' '..self.camPyr[i].renderTexture.width)
        self.camPyrMid[i].renderTexture.pecentX = standardMin / w
        self.camPyrMid[i].renderTexture.pecentY = standardMin / h
        self.camPyrMidUp[i].renderTexture.pecentX = standardMin * 2. / w
        self.camPyrMidUp[i].renderTexture.pecentY = standardMin * 2. / h
    end
end

function SeekModeScript:onEvent(comp, event)
    if event.type == Amaz.AppEventType.SetEffectIntensity then
        if event.args:get(0) == "clear_intensity" then
            local intensity = event.args:get(1)
            self.sharpIntensity = intensity
        end
    end
    if event.args:size() == 2 and event.args:get(0) == "reset_params" and event.args:get(1) == 1 then
        self.sharpIntensity = 0.
    end
end

exports.SeekModeScript = SeekModeScript
return exports
