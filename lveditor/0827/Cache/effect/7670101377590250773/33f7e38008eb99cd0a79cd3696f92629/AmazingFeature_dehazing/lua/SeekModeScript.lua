local exports = exports or {}
local SeekModeScript = SeekModeScript or {}
SeekModeScript.__index = SeekModeScript

---@class SeekModeScript : ScriptComponent
---@field debug boolean
---@field dehazeStrength number [UI(Range={0, 1}, Slider)]
---@field inputTexture Texture
---@field compareImage Texture

function SeekModeScript.new(construct, ...)
    local self = setmetatable({}, SeekModeScript)

    self.curTime = 0
    self.startTime = 0
    self.endTime = 3
    self.width = 0
    self.height = 0
    self.inputTexture = nil
    if construct and SeekModeScript.constructor then SeekModeScript.constructor(self, ...) end
    return self
end

function SeekModeScript:constructor()
    self.name = "scriptComp"
end

function SeekModeScript:onStart(comp)
    local scene = comp.entity.scene
    scene:setInputTexture(nil)
    self.entity = comp.entity
    local w = Amaz.BuiltinObject:getInputTextureWidth()
    local h = Amaz.BuiltinObject:getInputTextureHeight()
    self.width = w
    self.height = h
    self.renderer = self.entity.scene:findEntityBy("Entity"):getComponent("MeshRenderer")
    self.skinTexture = self.renderer.material:getTex("texture_skin")
    self.transmissionTexture = self.renderer.material:getTex("texture_trans")
    self.curveTexture = self.renderer.material:getTex("texture_curves")
    local input = self.renderer.material:getTex("texture_source")
    self.curveTexture.width = 256;
    self.curveTexture.height = 1;
    self:updateTextureWH(self.skinTexture)
    self:updateTextureWH(self.transmissionTexture)
    self.comp = comp
    self.comp.properties:set("debug", false)
    self.comp.properties:set("dehazeStrength", 1)
    self.comp.properties:set("inputTexture", input)

end

function SeekModeScript:updateTextureWH(tex)
    tex.width = self.width
    tex.height = self.height
end

function SeekModeScript:setAlgoTexture(src, dst)
    if self.quhuiInfo then
        if self.quhuiInfo:has(src) then
            local srcTex = self.quhuiInfo:get(src)
            local dstTex = self.renderer.material:getTex(dst)
            if srcTex and dstTex then
                -- Amaz.LOGE("ywt", "storage texture: "..src)
                dstTex:storage(srcTex)
            end
        end
    end
end

function SeekModeScript:getAlgoParams(name)
    if self.quhuiInfo then
        if self.quhuiInfo:has(name) then
            local param = self.quhuiInfo:get(name)
            return param
        end
    end
end

function SeekModeScript:seekToTime(comp, time)
    local graphName = self.entity.scene:getEffectName()
    local resNodeName = "nh_script_0"
    -- Amaz.Algorithm.setAlgorithmParamInt(graphName, resNodeName, "disIns", self.disIns) 

    local algoResult = Amaz.Algorithm.getAEAlgorithmResult()
    if algoResult ~= nil then
        local quhuiInfo = algoResult:getScriptInfo(graphName, resNodeName).outputMap
        if quhuiInfo ~= nil then
            self.quhuiInfo = quhuiInfo
            self:setAlgoTexture("skin_mask", "texture_skin")
            self:setAlgoTexture("transmission", "texture_trans")
            self:setAlgoTexture("curves", "texture_curves")
            local deHazeStrength = self:getAlgoParams("strength")
            -- Amaz.LOGE("ywt", "deHazeStrength: "..tostring(deHazeStrength))
            local human = self:getAlgoParams("human")
            -- Amaz.LOGE("ywt", "human: "..human)
            local tv = self:getAlgoParams("tv")
            local skinAlpha = self:getAlgoParams("alpha_skin")
            self.renderer.material:setFloat("strength", deHazeStrength*3.0)
            self.renderer.material:setFloat("human", human)
            self.renderer.material:setFloat("tv", tv)
            self.renderer.material:setFloat("alpha_s_full", skinAlpha)
            -- Amaz.LOGE("ywt", "alpha_s_full: "..skinAlpha)
            -- Amaz.LOGE("ywt", "deHazeStrength: "..tostring(deHazeStrength))
            -- Amaz.LOGE("ywt", "human: "..human)
            -- Amaz.LOGE("ywt", "tv: "..tv)
            -- Amaz.LOGE("ywt", "skinAlpha: "..skinAlpha)
        end
    end

    -- calculate some value for 
    self.renderer.material:setVec4("u_offsets", Amaz.Vector4f(0.008333,0.014434,0.005893,0.016667))
    self.renderer.material:setTex("texture_debug", comp.properties:get("compareImage"))
    self.renderer.material:setTex("texture_source", self.inputTexture)
    self.renderer.material:setVec4("u_sliderInfos", Amaz.Vector4f(self.comp.properties:get("dehazeStrength"), 0, 0, 0))
    self.renderer.material:setFloat("debug", self.comp.properties:get("debug") and 1 or 0)
end

function SeekModeScript:onUpdate(comp, deltaTime)
    -- Amaz.LOGE("ywt", comp.entity.scene.assetMgr.rootDir)
    -- Amaz.LOGE("ywt", comp.entity.scene:getInputTexture(Amaz.BuiltInTextureType.INPUT0).width)

    if Editor then
        self.curTime = self.curTime + deltaTime
    end
    local w = Amaz.BuiltinObject:getInputTextureWidth()
    local h = Amaz.BuiltinObject:getInputTextureHeight()
    if w ~= self.width or h ~= self.height then
        self.width = w
        self.height = h
        self:updateTextureWH(self.skinTexture)
        self:updateTextureWH(self.transmissionTexture)
    end
    self.inputTexture = comp.properties:get("inputTexture")
    self:seekToTime(comp, self.curTime - self.startTime)
end

function SeekModeScript:onEvent(comp, event)
    if event.args:get(0) == "dehazing_intensity" then
        local intensity = event.args:get(1)
        self.comp.properties:set("dehazeStrength", intensity)
    end
end

exports.SeekModeScript = SeekModeScript
return exports
