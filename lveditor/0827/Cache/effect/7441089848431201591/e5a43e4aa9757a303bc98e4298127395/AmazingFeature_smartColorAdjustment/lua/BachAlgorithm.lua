local graphName = 'smartColorAdjustment_VVdcRcCbhcmbVcfcCb06cIbccKb5c'
local exports = exports or {}
local BachAlgorithm = BachAlgorithm or {}
local Amaz = Amaz or {}
BachAlgorithm.__index = BachAlgorithm
local entityName = "BachAlgorithm"

function BachAlgorithm.new(construct, ...)
    local self = setmetatable({}, BachAlgorithm)
    self.comps = {}
    self.compsdirty = true
    self.intensity = 0.0
    return self
end

function BachAlgorithm:constructor()
    
end

function BachAlgorithm:onComponentAdded(sys, comp)
    if comp:isInstanceOf("MeshRenderer") and comp.entity.name == entityName then
        self.MeshRenderer = comp
    end
end

function BachAlgorithm:onComponentRemoved(sys, comp)
    if comp:isInstanceOf("MeshRenderer") and comp.entity.name == entityName then
        self.MeshRenderer = nil
    end
end

function BachAlgorithm:onStart(sys)
    if self.MeshRenderer then
        self.MeshRenderer.mesh.clearAfterUpload = false
    end
end

function BachAlgorithm:onUpdate(sys, deltaTime)
    local material = self.MeshRenderer.material
    local intensity = self.intensity

    -- srcTexture
    local srcTexture = sys.scene:getInputTexture(Amaz.BuiltInTextureType.INPUT0)
    material:setTex("srcTexture", srcTexture)

    -- resultTexture
    local resultTexture = srcTexture
    local result = Amaz.Algorithm.getAEAlgorithmResult()
    if result then
        local extraInfo = result:getLensGeneralInfo(graphName, 'vhdr_0', 0)
        if extraInfo then
            local errorCode = extraInfo:get("errorCode")
            if errorCode and errorCode == 1 then
                -- 1 means no error 
                local info = result:getAlgorithmInfo(graphName, 'vhdr_0', 'general_lens', 0)
                if info then
                    resultTexture = info.texture
                end
            else
                -- error handle
                Amaz.LOGE("VhdrLuaError", "errorCode:"..tostring(errorCode))
                intensity = 0
            end
        end
    end
    material:setTex("resultTexture", resultTexture)

    -- output texture
    material:setFloat("intensity", intensity)

    self.MeshRenderer.enabled = true
end

function BachAlgorithm:onEvent(comp, event)
    if "smartcoloradjustment_intensity" == event.args:get(0) then
        local intensity = event.args:get(1)
        self.intensity = intensity
        -- Amaz.LOGE("wenjie.123", "onEvent"..tostring(self.intensity))
    end
    if event.args:size() == 2 and event.args:get(0) == "reset_params" and event.args:get(1) == 1 then
        self.intensity = 0.
    end
end

exports.BachAlgorithm = BachAlgorithm
return exports
