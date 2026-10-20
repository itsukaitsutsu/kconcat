      
local isEditor = (Amaz.Macros and Amaz.Macros.EditorSDK) and true or false
local exports = exports or {}
local LumiHub = LumiHub or {}
LumiHub.__index = LumiHub

includeRelativePath("LumiUtils")
-- includeRelativePath("LumiObjectExtension")
-- Limitations:
-- 1, if we add a LumiEffect on a renderer, the renderer should be visible to only one camera;
--    depth map is not handled, so 3D rendering may be incorrect
-- 2, These entities should be listed after all original entities in the entity tree: LumiRoot and LumiExtraNodeRoot, 
--    so that entity sortingOrders are not affected by entities created by LumiHub
-- 3, possible class name conflucts in prefab luas are not addressed by the framework.

-- Builtin prefab properties: startTime/endTime/curTime
-- Builtin prefab functions:
--   RECOMMENDED: updateMaterial
--   OPTIONAL: updateRenderTexture, setInputTexSize, setInputOriginalTexSize, getOutputTexSize, getOutputOriginalTexSize

function LumiHub.new(construct, ...)
    local self = setmetatable({}, LumiHub)
    if construct and LumiHub.constructor then LumiHub.constructor(self, ...) end
    self.inited = false
    self.lazyInited = false
    return self
end

function LumiHub:_getUnsedEffectID(_prefabPath, _effectID)
    for _, effectID in pairs(self.lumiTable.unusedLumiEffects) do
        if _effectID == nil or _effectID == effectID then
            local prefabPath = self.effectIDToPrefabPath[effectID]
            if prefabPath == _prefabPath then
                return effectID
            end
        end
    end
    return nil
end

function LumiHub:_getLumiCamera(target)
    for _, v in pairs(self.lumiCameras) do
        if v.entity.name == target then
            return v
        end
    end
    return nil
end

function LumiHub:_getLumiRenderer(target)
    for _, v in pairs(self.lumiRenderers) do
        if v.entity.name == target then
            return v
        end
    end
    return nil
end

function LumiHub:_getLumiTarget(targetType, target)
    local lumiTarget = nil
    if targetType == LUMI_TARGET_TYPE_CAMERA_RT then
        lumiTarget = self:_getLumiCamera(target)
    elseif targetType == LUMI_TARGET_TYPE_RENDERER then
        lumiTarget = self:_getLumiRenderer(target)
    elseif targetType == LUMI_TARGET_TYPE_INPUT_RT then
        lumiTarget = LUMI_TARGET_INPUT_RT_IDENTIFIER
    elseif targetType == LUMI_TARGET_TYPE_OUTPUT_RT then
        lumiTarget = LUMI_TARGET_OUTPUT_RT_IDENTIFIER
    else
        printe("_getLumiTarget: Invalid targetType")
    end
    return lumiTarget
end

-- return errorCode, effectID
function LumiHub:_addToScene(stickerPath, prefabPath, effectID)
    local absolutePath = stickerPath .. "/" .. prefabPath

    if not fileExists(absolutePath) then
        printe("_addToScene: prefab not found:", absolutePath)
        return LUMI_FILE_NOT_FOUND, nil
    end
    local prefab = Amaz.PrefabManager.loadPrefab(stickerPath, prefabPath)
    local prefabRoot = prefab:instantiateToEntity(self.scene, self.lumiRoot, true)

    local scriptComp = prefabRoot:getComponent("ScriptComponent")
    if scriptComp == nil then
        printe("_addToScene: script component not found on the instantiated prefab")
        return LUMI_INVALID_PREFAB, nil
    end
    if effectID == nil then
        effectID = self:_getLumiName("LE_")
    end
    prefabRoot.name = effectID

    scriptComp:reloadScript()
    local lumiObj = self.lumiObjExtension.register(prefabRoot)
    if lumiObj == nil then
        printe("_addToScene: lumiObj is nil")
    else
        self.effectIDToLumiObject[effectID] = lumiObj
        -- also set effectID for lumiObj (to get custom texture by effectID at effect_lua)
        lumiObj:setEffectAttr("effectID", effectID)
    end
    
    return LUMI_SUCCESS, effectID
end

-- return effectID if success else tostring(errorCode)
-- example 1: stickerPath="/Users/bytedance/stickerA/", prefabPath="prefabs/a.prefab"
-- example 2: stickerPath="/Users/bytedance/stickerA/prefabs/", prefabPath="a.prefab"
-- all resources refered by the prefab will be searched with stickerPath as the current directory
function LumiHub:addLumiEffect(stickerPath, prefabPath, targetType, target, _effectID)
    if targetType ~= LUMI_TARGET_TYPE_INPUT_RT and targetType ~= LUMI_TARGET_TYPE_OUTPUT_RT and targetType ~= LUMI_TARGET_TYPE_CAMERA_RT and targetType ~= LUMI_TARGET_TYPE_RENDERER then return LUMI_INVALID_TARGET_TYPE end
    local lumiTarget = self:_getLumiTarget(targetType, target)
    if lumiTarget == nil then
        printe("addLumiEffect: target not found: ", target)
        return tostring(LUMI_TARGET_NOT_FOUND)
    end
    local effectID = nil
    if self.enableLumiEffectCache then
        effectID = self:_getUnsedEffectID(prefabPath, _effectID)
    end

    if effectID == nil then
        local errorCode = nil
        errorCode, effectID = self:_addToScene(stickerPath, prefabPath, _effectID)
        if errorCode ~= LUMI_SUCCESS then return tostring(errorCode) end
        self.lumiTable:insert(lumiTarget, effectID)
        self.effectIDToPrefabPath[effectID] = prefabPath
    else
        -- reused prefab should maintain visible, (order, layer, RT)--automatic
        self:_reuseLumiEffect(lumiTarget, effectID)
        print("addLumiEffect: reused lumi:", effectID)
    end
        
    if self:_needExtraNode(lumiTarget) then
        local lumiCamera = nil
        for _, v in pairs(self.lumiCameras) do
            if v.layerVisibleMask:test(lumiTarget.layer) then  -- lumiCamera can see renderer
                lumiCamera = v
                break  -- assume only one camera can see the renderer, more cameras are ignored
            end
        end
        local extraNode = self:_reuseExtraNode(effectID, lumiCamera.camera, lumiTarget)
        if extraNode == nil then
            extraNode = self:_createExtraNode(effectID, lumiCamera.camera, lumiTarget)
        end
    end

    self.lumiDirty = true
    print("addLumiEffect: added lumi:", effectID)
    return effectID
end

function LumiHub:updateLumiPara(effectID, key, value)
    -- print("updateLumiPara start")
    local lumiObj = self.effectIDToLumiObject[effectID]
    if lumiObj == nil then
        printe("updateLumiPara: effectID not found")
        return LUMI_EFFECT_ID_NOT_FOUND
    end
    lumiObj:setEffectAttr(key, value)
    -- print("updateLumiPara succeeded")
    return LUMI_SUCCESS
end

function LumiHub:enableLumiEffect(effectID, enable)
    if effectID == nil then 
        return LUMI_EFFECT_ID_NOT_FOUND 
    end
    if enable == false then
        self:_disableLumiEffect(effectID)
    else
        if self.lumiTable ~= nil and self.lumiTable.effectIDTolumiTarget[effectID] ~= nil then
            for _, lumiTarget in pairs(self.lumiTable.effectIDTolumiTarget[effectID]) do
                self:_reuseLumiEffect(lumiTarget, effectID)
            end
            -- test
            -- local lumiTarget = self:_getLumiTarget(LUMI_TARGET_TYPE_CAMERA_RT, "Camera_Mask")
            -- self:_reuseLumiEffect(lumiTarget, effectID)
        end
    end
    return LUMI_SUCCESS
end

function LumiHub:updateAllLumiPara(key, value)
    local ret = LUMI_SUCCESS
    for _, v1 in pairs(self.lumiTable.lumiTable) do
        for _, effectID in pairs(v1) do
            ret = ret + self:updateLumiPara(effectID, key, value)
        end
    end
    if ret == LUMI_SUCCESS then
        return LUMI_SUCCESS
    else
        return LUMI_ERROR
    end
end

function LumiHub:changeOrder(effectID, index)
    self.lumiDirty = true
    print("changeOrder succeeded")
    return self.lumiTable:changeOrder(effectID, index)
end

function LumiHub:getLumiPara(effectID, key)
    local lumiObj = self.effectIDToLumiObject[effectID]
    if lumiObj == nil then
        printe("getLumiPara: effectID not found in getLumiPara")
        return LUMI_EFFECT_ID_NOT_FOUND
    end
    return lumiObj:getEffectAttr(key)
end

-- setInputTexSize and getOutputTexSize are used for calculating RT size, which will be used outside LumiHub,
-- i.e., LumiHub will not change internal RT size according to inputTexSize or outputTexSize
-- In fact, LumiHub holds the same RT size in the whole pipeline as the outputTexture's real size
function LumiHub:setInputTexSize(w, h)
    self.inputTexSize = {w, h}
end

function LumiHub:_getOutputTexSize()
    local texToSizes = {}
    local maxSize = self.inputTexSize
    for i=1, #self.nodes do
        local node = self.nodes[i]
        if node.nodeType == LUMI_NODE_EFFECT then  -- effectID
            local effectID = node.node
            local lumiObj = self.effectIDToLumiObject[effectID]
            local input = lumiObj:getInputTex()
            local inputSize, origSize = nil, nil
            if texToSizes[input] ~= nil then
                local sizes = texToSizes[input]
                inputSize, origSize = sizes[1], sizes[2]
            else
                inputSize, origSize = self.inputTexSize, self.inputTexSize
            end
            lumiObj:setInputTexSizes(inputSize, origSize)
            local outputSize, outputOrigSize = lumiObj:getOutputTexSizes()
            if outputSize[1] ~= inputSize[1] or outputSize[2] ~= inputSize[2] then
                local output = lumiObj:getOutputTex()
                texToSizes[output] = {outputSize, outputOrigSize}
                if outputSize[1] > maxSize[1] or outputSize[2] > maxSize[2] then
                    maxSize = outputSize
                end
            end
        end
    end
    return maxSize
end

function LumiHub:getOutputTexSize()
    self:_update()
    local size = self:_getOutputTexSize()
    return Amaz.Vector2f(size[1], size[2])
end

function LumiHub:_removeFromScene(effectID, enableLumiEffectCache)
    local prefabRoot = self.lumiRoot:searchEntity(effectID)
    if prefabRoot == nil then
        printe("_removeFromScene: prefab not found")
        return LUMI_EFFECT_ID_NOT_FOUND
    end
    if enableLumiEffectCache then
        prefabRoot.visible = false
    else
        self.scene:removeEntity(prefabRoot)
    end
    return LUMI_SUCCESS
end
 
function LumiHub:_reuseLumiEffect(lumiTarget, effectID)
    local prefabRoot = self.lumiRoot:searchEntity(effectID)
    if prefabRoot ~= nil then
        prefabRoot.visible = true
        self.lumiTable:reuse(lumiTarget, effectID)
        return LUMI_SUCCESS
    end
    return LUMI_EFFECT_ID_NOT_FOUND
end

function LumiHub:removeLumiEffect(effectID)
    -- zhelicunzaiyouhuaxiang，self.lumiTable.unusedLumiEffectshuiyuelaiyueda, yinweimeicideeffectIDdoubutong
    -- print("test self.lumiTable.unusedLumiEffects: ", self.lumiTable.unusedLumiEffects)
    if self.enableLumiEffectCache then
        if table.getIndex(self.lumiTable.unusedLumiEffects, effectID) > 0 then
            return LUMI_SUCCESS
        end
    end
    local ret = self.lumiTable:remove(effectID, self.enableLumiEffectCache)
    if ret ~= LUMI_SUCCESS then
        printe("removeLumiEffect: remove from lumiTable failed with code ", ret)
        return ret
    end
    ret = self:_removeFromScene(effectID, self.enableLumiEffectCache)
    if ret ~= LUMI_SUCCESS then
        printe("removeLumiEffect: remove from scene failed with code ", ret)
        return ret
    end
    self:_tryRemoveExtraNode(effectID)
    self.lumiDirty = true
    print("removed", effectID)
    return LUMI_SUCCESS
end

function LumiHub:_disableLumiEffect(effectID)
    if table.getIndex(self.lumiTable.unusedLumiEffects, effectID) > 0 then
        return LUMI_SUCCESS
    end
    local enableLumiEffectCache = true;
    local ret = self.lumiTable:remove(effectID, enableLumiEffectCache)
    if ret ~= LUMI_SUCCESS then
        printe("removeLumiEffect: remove from lumiTable failed with code ", ret)
        return ret
    end
    ret = self:_removeFromScene(effectID, enableLumiEffectCache)
    if ret ~= LUMI_SUCCESS then
        printe("removeLumiEffect: remove from scene failed with code ", ret)
        return ret
    end
    self:_tryRemoveExtraNode(effectID)
    self.lumiDirty = true
    print("disable", effectID)
    return LUMI_SUCCESS
end

function LumiHub:clearLumiEffectForTarget(targetType, target)
    local lumiTarget = self:_getLumiTarget(targetType, target)
    if lumiTarget == nil then return LUMI_ERROR end
    if self.lumiTable.lumiTable[lumiTarget] == nil then return LUMI_ERROR end
    for _, effectID in pairs(self.lumiTable.lumiTable[lumiTarget]) do
        self:_removeFromScene(effectID, self.enableLumiEffectCache)
        self:_tryRemoveExtraNode(effectID)
    end
    self.lumiTable:clearForTarget(lumiTarget, self.enableLumiEffectCache)
    self.lumiDirty = true
    return LUMI_SUCCESS
end

function LumiHub:clearLumiEffect()
    for _, v1 in pairs(self.lumiTable.lumiTable) do
        for _, effectID in pairs(v1) do
            self:_removeFromScene(effectID, self.enableLumiEffectCache)
            self:_tryRemoveExtraNode(effectID)
        end
    end
    self.lumiTable:clear(self.enableLumiEffectCache)
    self.lumiDirty = true
    return LUMI_SUCCESS
end

function LumiHub:_scanMainScene()
    local entities = self.scene.entities
    self.lumiRenderers = {}
    self.maxLayer = 0
    self.lumiCameras = {}
    for i = 1, entities:size() do
        local ent = entities:get(i-1)
        if not table.exists(LUMI_EXCLUDE_ENTITY_NAMES, ent.name) then
            local renderer = ent:getComponent("Renderer")
            if renderer ~= nil then
                local lumiRenderer = createLumiRenderer(renderer)
                table.insert(self.lumiRenderers, lumiRenderer)
            end
            
            local cam = ent:getComponent("Camera")
            if cam ~= nil then
                local lumiCamera = createLumiCamera(cam)
                local index = -1
                for j=1, #self.lumiCameras do
                    if self.lumiCameras[j].order > lumiCamera.order then
                        index = j
                        break
                    end
                end
                if index < 0 then
                    index = #self.lumiCameras + 1
                end
                table.insert(self.lumiCameras, index, lumiCamera)
            end
        end
    end
    for _, lr in pairs(self.lumiRenderers) do
        if lr.entity.layer > self.maxLayer then
            self.maxLayer = lr.entity.layer
        end
    end
end

function LumiHub:_getPingPongTexture(index)
    if index == 1 then
        if self.pingPongTexture1 == nil then
            self.pingPongTexture1 = createRenderTexture("lumi_1", self.width, self.height)
        end
        return self.pingPongTexture1
    elseif index == 2 then
        if self.pingPongTexture2 == nil then
            self.pingPongTexture2 = createRenderTexture("lumi_2", self.width, self.height)
        end
        return self.pingPongTexture2
    else
        return nil
    end
end

function LumiHub:_getLumiObjPingPongTexture()
    if self.lumiObjPingPongTexture == nil then
        self.lumiObjPingPongTexture = createRenderTexture("pingPong", self.width, self.height)
    end
    return self.lumiObjPingPongTexture
end

function LumiHub:_getLumiName(prefix)
    assert(self.lumiNameIndex < 10000, "lumiNameIndex is too large")
    local name = prefix .. "0000"
    local indexStr = tostring(self.lumiNameIndex)
    name = string.sub(name, 1, #name - #indexStr) .. indexStr
    self.lumiNameIndex = self.lumiNameIndex + 1
    return name
end

function LumiHub:_initLumi(comp)
    if self.inited then return end
    print("lumi init start with version: ", LUMI_VERSION)
    self.scene = comp.entity.scene
    self.lumiRoot = self.scene:findEntityBy("LumiRoot")
    if self.lumiRoot == nil then
        self.lumiRoot = self.scene:createEntity("LumiRoot")
        self.lumiRoot:addComponent(Amaz.Transform())
    end
    self.LumiExtraNodeRoot = self.scene:findEntityBy("LumiExtraNodeRoot")
    if self.LumiExtraNodeRoot == nil then
        self.LumiExtraNodeRoot = self.scene:createEntity("LumiExtraNodeRoot")
        self.LumiExtraNodeRoot:addComponent(Amaz.Transform())
    end
    self.lumiNameIndex = 1
    self.effectIDToExtraNode = {}
    self.unusedExtraNodes = {}
    self.nodes = nil

    self.lumiObjExtension = includeRelativePath("LumiObjectExtension")

    self.lumiTable = LumiTable.new()
    self.effectIDToPrefabPath = {}
    self.effectIDToLumiObject = {}
    self:_scanMainScene()
    self.quadMeshPath = "effects/LumiHub/LumiExtra.mesh"
    self.quadMesh = nil
    self.lumiImageMaterialPath = "effects/LumiHub/LumiExtra.material"
    self.lumiImageMaterial = nil
    -- used only by setInputTexSize and getOutputTexSize for computing RT size used outside LumiHub
    self.inputTexSize = nil
    -- real RT size
    self.texResolutionDirty = true

    self.pingPongTexture1 = nil
    self.pingPongTexture2 = nil
    self.newOutputTexture = nil
    self.extraTexture = nil
    self.lumiObjPingPongTexture = nil
    self.width, self.height = 720, 1080
    
    self.lumiDirty = true
    self.inited = true

    self.enableLumiEffectCache = false
    print("lumi enableLumiEffectCache: ", self.enableLumiEffectCache)
    print("lumi inited")
end

-- business may require to createLumiHub and addLumiEffect without waiting onStart to execute, and positively call onStart earlier
function LumiHub:_lasyInitLumi()
    if self.lazyInited then return end
    self.inputTexture = self.scene:getInputTexture(Amaz.BuiltInTextureType.INPUT0)
    self.outputTexture = self.scene:getOutputRenderTexture()
    if self.inputTexture ~= nil and self.outputTexture ~= nil then
        self.inputTexture.name = "input"
        self.outputTexture.name = "output"
        self.width, self.height = Amaz.BuiltinObject:getOutputTextureWidth(), Amaz.BuiltinObject:getOutputTextureHeight()
        self.lazyInited = true
    end
end

function LumiHub:_needExtraNode(lumiTarget)
    -- already created extraNode
    for k, v in pairs(self.effectIDToExtraNode) do
        if v.lumiTarget == lumiTarget then
            return false
        end
    end

    local renderer = lumiTarget.renderer
    if renderer == nil then return false end

    local lumiCamera = nil
    for _, v in pairs(self.lumiCameras) do
        if v.layerVisibleMask:test(lumiTarget.entity.layer) then
            lumiCamera = v
        end
    end
    if lumiCamera == nil then return false end

    for _, v in pairs(self.lumiRenderers) do
        if v ~= renderer and lumiCamera.layerVisibleMask:test(v.layer) then
            return true
        end
    end
    return false
end

function LumiHub:_createExtraNode(effectID, _camera, lumiTarget)
    if self.extraTexture == nil then
        self.extraTexture = createRenderTexture("extra", self.width, self.height)
    end

    local camEntityName = self:_getLumiName("CAM_")
    local camEntity = self.scene:createEntity(camEntityName)
    local camTrans = camEntity:addComponent("Transform")
    self.LumiExtraNodeRoot:getComponent("Transform"):addTransform(camTrans)
    camTrans:setWorldMatrix(_camera.entity:getComponent("Transform"):getWorldMatrix())
    local camera = camEntity:cloneComponentOf(_camera)
    camera.clearColor = Amaz.Color(0, 0, 0, 0)
    camera.clearType = Amaz.CameraClearType.COLOR_DEPTH

    local rendererEntityName = self:_getLumiName("REND_")
    local rendererEntity = self.scene:createEntity(rendererEntityName)
    local rendTrans = rendererEntity:addComponent("Transform")
    camTrans:addTransform(rendTrans)
    rendTrans:setWorldMatrix(lumiTarget.entity:getComponent("Transform"):getWorldMatrix())
    local renderer = rendererEntity:addComponent("MeshRenderer")
    renderer.sortingOrder = lumiTarget.order

    if self.quadMesh == nil then
        self.quadMesh = self.scene.assetMgr:SyncLoad(self.quadMeshPath)
        if self.quadMesh == nil then
            printe("_createExtraNode: load quadMesh failed")
        end
    end
    renderer.mesh = self.quadMesh
    if self.lumiImageMaterial == nil then
        self.lumiImageMaterial = self.scene.assetMgr:SyncLoad(self.lumiImageMaterialPath)
        if self.lumiImageMaterial == nil then
            printe("_createExtraNode: load lumiImageMaterial failed")
        end
        self.lumiImageMaterial:setTex("_MainTex", self.extraTexture)
    end
    renderer.material = self.lumiImageMaterial
    local lumiExtraNode = createLumiExtraNode(camEntity, camera, renderer, lumiTarget)
    self.effectIDToExtraNode[effectID] = lumiExtraNode
    return lumiExtraNode
end

function LumiHub:_tryRemoveExtraNode(effectID)
    local lumiExtraNode = self.effectIDToExtraNode[effectID]
    if lumiExtraNode == nil then
        return LUMI_EFFECT_ID_NOT_FOUND
    end
    self.effectIDToExtraNode[effectID] = nil
    local needRemove = true
    for k, v in pairs(self.effectIDToExtraNode) do
        if v == lumiExtraNode then
            needRemove = false
        end
    end
    if needRemove then
        lumiExtraNode.entity.visible = false
        table.insert(self.unusedExtraNodes, lumiExtraNode)
    end
    return LUMI_SUCCESS
end

function LumiHub:_reuseExtraNode(effectID, _camera, lumiTarget)
    local lumiExtraNode = nil
    for k, v in pairs(self.unusedExtraNodes) do
        if v.lumiTarget == lumiTarget then
            lumiExtraNode = v
            self.unusedExtraNodes[k] = nil
        end
    end
    if lumiExtraNode == nil then return nil end
    lumiExtraNode.entity.visible = true

    local camEntity = lumiExtraNode.entity
    camEntity:removeComponent("Camera")
    local camera = camEntity:cloneComponentOf(_camera)
    camera.clearColor = Amaz.Color(0, 0, 0, 0)
    camera.clearType = Amaz.CameraClearType.COLOR_DEPTH

    lumiExtraNode.entity:getComponent("Transform"):setWorldMatrix(_camera.entity:getComponent("Transform"):getWorldMatrix())
    lumiExtraNode.renderer.entity:getComponent("Transform"):setWorldMatrix(lumiTarget.entity:getComponent("Transform"):getWorldMatrix())
    lumiExtraNode.renderer.sortingOrder = lumiTarget.order
    self.effectIDToExtraNode[effectID] = lumiExtraNode
    camera.renderOrder = lumiExtraNode.lumiTarget.order
    lumiExtraNode.camera = camera  -- camera is rebuilt, need to update
    print("reused extra node")
    return lumiExtraNode
end

function LumiHub:_recoverStates()
    for _, lc in pairs(self.lumiCameras) do
        lc.camera.inputTexture = lc.inputTexture
        lc.camera.renderTexture = lc.renderTexture
    end
    for _, lr in pairs(self.lumiRenderers) do
        lr.entity.layer = lr.layer
    end
    for _, v in pairs(self.effectIDToExtraNode) do
        v.camera.inputTexture = nil
        v.camera.renderTexture = self.extraTexture
    end
end

function LumiHub:_buildCamNodes()
    self.nodes = {}
    for _, v in pairs(self.lumiCameras) do
        table.insert(self.nodes, {nodeType=LUMI_NODE_CAMERA, node=v})
    end
    for lumiTarget, effects in pairs(self.lumiTable.lumiTable) do
        if lumiTarget == LUMI_TARGET_INPUT_RT_IDENTIFIER then
            for i=#effects, 1, -1 do
                table.insert(self.nodes, 1, {nodeType=LUMI_NODE_EFFECT, node=effects[i]})
            end
        elseif lumiTarget == LUMI_TARGET_OUTPUT_RT_IDENTIFIER then
            for i=1, #effects do
                table.insert(self.nodes, {nodeType=LUMI_NODE_EFFECT, node=effects[i]})
            end
        elseif lumiTarget.camera ~= nil then  -- LumiTarget is LumiCamera
            local nodeIndex = -1
            for i = 1, #self.nodes do
                local node = self.nodes[i].node
                if lumiTarget == node then
                    nodeIndex = i
                    break
                end
            end
            if nodeIndex > 0 then
                for i=1, #effects do
                    table.insert(self.nodes, nodeIndex+1, {nodeType=LUMI_NODE_EFFECT, node=effects[i]})
                end
            end
        else  -- LumiTarget is LumiRenderer
            local nodeIndex = -1
            for i = 1, #self.nodes do
                local node = self.nodes[i].node
                if node.camera ~= nil and node.layerVisibleMask:test(lumiTarget.layer) then  -- node is LumiCamera and can see renderer
                    nodeIndex = i
                    break  -- assume only one camera can see the renderer, more cameras are ignored
                end
            end
            if nodeIndex > 0 then
                local lumiCamera = self.nodes[nodeIndex].node
                for i=1, #effects do
                    local extraNode = self.effectIDToExtraNode[effects[i]]
                    table.insert(self.nodes, nodeIndex, {nodeType=LUMI_NODE_EXTRA, node=extraNode})
                    table.insert(self.nodes, nodeIndex+1, {nodeType=LUMI_NODE_EFFECT, node=effects[i]})
                end
            end
        end
    end
end

function LumiHub:_setExtraNodeLayer(extraNode, currLayer)
    setLayer(extraNode.camera, currLayer)
    local lumiRenderer = extraNode.lumiTarget
    lumiRenderer.entity.layer = currLayer
    extraNode.entity.layer = lumiRenderer.layer
end

function LumiHub:_updateCameraOrderAndLayer()
    local currCamOrder = 1
    local currLayer = self.maxLayer + 1
    for i=1, #self.nodes do
        local node = self.nodes[i]
        if node.nodeType == LUMI_NODE_EFFECT then  -- effectID
            local effectID = node.node
            local lumiObj = self.effectIDToLumiObject[effectID]
            lumiObj:updateCameraLayerAndOrder(currLayer, currCamOrder)
            local count = lumiObj:getCameraCount()
            currCamOrder, currLayer = currCamOrder + count, currLayer + count
        elseif node.nodeType == LUMI_NODE_CAMERA then  -- LumiCamera
            local oldOrder = node.node.order
            if oldOrder < currCamOrder then
                node.node.camera.renderOrder = currCamOrder
                currCamOrder = currCamOrder + 1
            else
                currCamOrder = oldOrder + 1
            end
        elseif node.nodeType == LUMI_NODE_EXTRA then  -- extraNode
            local oldOrder = node.node.lumiTarget.order
            if oldOrder < currCamOrder then
                node.node.camera.renderOrder = currCamOrder
                currCamOrder = currCamOrder + 1
            else
                currCamOrder = oldOrder + 1
            end
            self:_setExtraNodeLayer(node.node, currLayer)
            currLayer = currLayer + 1
        end
    end
end

function LumiHub:_updateRTs()
    local debug = false
    local rtList = {}
    local inputPiece, outputPiece = {}, {}
    local nodePieces = {}  -- create input nodes and output nodes; insert camera nodes in between
    local currLumiCamera = nil
    for i=1, #self.nodes do
        local node = self.nodes[i]
        if node.nodeType == LUMI_NODE_EFFECT then  -- effectID
            local effectID = node.node
            local lumiTarget, index = self.lumiTable:getIndex(effectID)
            if lumiTarget == LUMI_TARGET_INPUT_RT_IDENTIFIER then
                table.insert(inputPiece, node)
            elseif lumiTarget == LUMI_TARGET_OUTPUT_RT_IDENTIFIER then
                table.insert(outputPiece, node)
            elseif lumiTarget.camera ~= nil then
                table.insert(nodePieces[#nodePieces], node)
            else
                table.insert(nodePieces[#nodePieces], node)
            end
        elseif node.nodeType == LUMI_NODE_CAMERA then  -- LumiCamera
            table.insert(nodePieces, {node})
        elseif node.nodeType == LUMI_NODE_EXTRA then  -- extraNode
            table.insert(nodePieces, {node})
        end
    end
    
    local pingPongIndex = 1
    for i=1, #inputPiece do
        local effectID = inputPiece[i].node
        local lumiObj = self.effectIDToLumiObject[effectID]
        local input = nil
        if i > 1 then
            input = self:_getPingPongTexture(3-pingPongIndex)
        else
            input = self.inputTexture
        end
        lumiObj:updateRt(input, self:_getPingPongTexture(pingPongIndex), self:_getLumiObjPingPongTexture())
        pingPongIndex = 3 - pingPongIndex
        if debug then
            table.insert(rtList, {input=lumiObj:getInputTex(), output=lumiObj:getOutputTex()})
        end
    end

    -- change inputTexture to newInputTexture with effects, for cameras and renderers
    if #inputPiece > 0 then
        local newInputTex = self:_getPingPongTexture(3-pingPongIndex)
        for _, lc in pairs(self.lumiCameras) do
            if lc.camera.inputTexture ~= nil and lc.camera.inputTexture.builtinType == Amaz.BuiltInTextureType.INPUT0 then
                lc.camera.inputTexture = newInputTex
            end
        end

        for _, lr in pairs(self.lumiRenderers) do
            local mat = lr.renderer.material
            local texmap = mat.properties.texmap
            local keys = texmap:getVectorKeys()
            for i=1, keys:size() do
                local key = keys:get(i-1)
                local value = texmap:get(key)
                if value ~= nil and value.builtinType == Amaz.BuiltInTextureType.INPUT0 then
                    texmap:set(key, newInputTex)
                end
            end
        end
    end
    
    local outTex = nil  -- the tex which outputRT effects work on
    if #outputPiece % 2 == 1 then
        if self.newOutputTexture == nil then
            self.newOutputTexture = createRenderTexture("lumi_3", self.width, self.height)
        end
        outTex = self.newOutputTexture
        for _, lc in pairs(self.lumiCameras) do
            if lc.camera.renderTexture ~= nil and lc.camera.renderTexture.builtinType == Amaz.BuiltInTextureType.OUTPUT then
                lc.camera.renderTexture = outTex
            end
        end
    else
        outTex = self.outputTexture
    end

    for _, piece in pairs(nodePieces) do
        local node1 = piece[1].node
        if #piece > 1 then
            -- name all textures as: i-self.inputTexture, o-outputTexture, a-pingPongTextures[3-pingPongIndex], b-pingPongTextures[pingPongIndex], c-newOutputTexture;
            -- I-input is {i, a}, O-output is {o, c}, P-pingPong is b;
            -- textures are [I, P, O] or [I, O, P, O], so no neighbors are the same
            local textures = getPingPongTextures(node1.camera.inputTexture, node1.camera.renderTexture, self:_getPingPongTexture(pingPongIndex), #piece)

            if debug then
                for _, v in pairs(textures) do
                    table.insert(rtList, {input=v.input, output=v.output})
                end
            end

            node1.camera.renderTexture = textures[1].output
            for i=2, #piece do
                local effectID = piece[i].node
                local lumiObj = self.effectIDToLumiObject[effectID]
                lumiObj:updateRt(textures[i].input, textures[i].output, self:_getLumiObjPingPongTexture())
            end
        else
            table.insert(rtList, {input=node1.camera.inputTexture, output=node1.camera.renderTexture})
        end
    end
    
    if #outputPiece > 0 then
        local textures = getPingPongTextures(outTex, self.outputTexture, self:_getPingPongTexture(pingPongIndex), #outputPiece)
        if debug then
            for _, v in pairs(textures) do
                table.insert(rtList, {input=v.input, output=v.output})
            end
        end
        for i=1, #outputPiece do
            local effectID = outputPiece[i].node
            local lumiObj = self.effectIDToLumiObject[effectID]
            lumiObj:updateRt(textures[i].input, textures[i].output, self:_getLumiObjPingPongTexture())
            pingPongIndex = 3 - pingPongIndex
        end
    end
    if debug then
        local str = ""
        for _, v in pairs(rtList) do
            local item = "("
            if v.input ~= nil then
                item = item .. v.input.name
            end
            item = item .. ", "
            if v.output ~= nil then
                item = item .. v.output.name
            end
            item = item .. ") "
            str = str .. item
        end
        print("rtlist:", str)
    end
end

function LumiHub:_update()
    if self.lumiDirty then
        self.lumiDirty = false
        self:_recoverStates()
        local nodes = self:_buildCamNodes()
        self:_updateCameraOrderAndLayer(nodes)
        self:_updateRTs(nodes)
        self.scene:getSystem("ScriptSystem"):sortScriptCompsByDepth()
    end
end
-- make sure LumiHub update before other onUpdate
function LumiHub:beforeScriptSystemUpdate(comp, deltaTime)
    self:_lasyInitLumi()
    self:_updateTexResolution()
    self:_updateLumiObjMaterials(deltaTime)
    self:_update()

    -- if self.tick == nil then
    --     self.tick = 0
    -- else
    --     self.tick = self.tick + 1
    -- end
    -- -- print(self.tick)
    -- if self.tick == 300 then
    --     -- print(100)
    --     table.insert(self.effects, self:addLumiEffect("/Users/bytedance/work/LumiHub/", "Filter_Effect.prefab", LUMI_TARGET_TYPE_OUTPUT_RT, ""))
    -- end
end

function LumiHub:_updateLumiObjMaterials(time)
    for _, v1 in pairs(self.lumiTable.lumiTable) do
        for _, effectID in pairs(v1) do
            local lumiObj = self.effectIDToLumiObject[effectID]
            lumiObj:updateMaterials(time)
        end
    end
end

function LumiHub:_updateTexResolution()
    local width = Amaz.BuiltinObject:getOutputTextureWidth()
    local height = Amaz.BuiltinObject:getOutputTextureHeight()
    if self.width ~= width or self.height ~= height then
        self.width = width
        self.height = height
        self.texResolutionDirty= true
    end
    if self.texResolutionDirty then
        self.inputTexture.width, self.inputTexture.height = self.width, self.height
        self.outputTexture.width, self.outputTexture.height = self.width, self.height
        if self.pingPongTexture1 then
            self.pingPongTexture1.width, self.pingPongTexture1.height = self.width, self.height
        end
        if self.pingPongTexture2 then
            self.pingPongTexture2.width, self.pingPongTexture2.height = self.width, self.height
        end
        if self.newOutputTexture then
            self.newOutputTexture.width, self.newOutputTexture.height = self.width, self.height
        end
        if self.extraTexture then
            self.extraTexture.width, self.extraTexture.height = self.width, self.height
        end
        if self.lumiObjPingPongTexture then
            self.lumiObjPingPongTexture.width, self.lumiObjPingPongTexture.height = self.width, self.height
        end
    end
end

function LumiHub:_testCreateEffects()
    self.effects = {}
    -- local rootDir = "/Users/bytedance/work/tmp/"
    -- local rootDir = "/Users/bytedance/work/LumiHub/"
    -- table.insert(self.effects, self:addLumiEffect(rootDir, "prefabs/Filter_Effect.prefab", LUMI_TARGET_TYPE_OUTPUT_RT, ""))
    -- table.insert(self.effects, self:addLumiEffect(rootDir, "prefabs/Filter_Effect.prefab", LUMI_TARGET_TYPE_INPUT_RT, ""))
    -- table.insert(self.effects, self:addLumiEffect(rootDir, "prefabs/Filter_Effect.prefab", LUMI_TARGET_TYPE_OUTPUT_RT, ""))
    -- table.insert(self.effects, self:addLumiEffect(rootDir, "prefabs/Filter_Effect.prefab", LUMI_TARGET_TYPE_CAMERA_RT, "Camera"))
    -- table.insert(self.effects, self:addLumiEffect(rootDir, "prefabs/Filter_Effect.prefab", LUMI_TARGET_TYPE_RENDERER, "Cube1"))
    
    local rootDir = "/Users/bytedance/work/LumiHub/"
    table.insert(self.effects, self:addLumiEffect(rootDir, "prefabs/Shadow.prefab", LUMI_TARGET_TYPE_OUTPUT_RT, ""))
    table.insert(self.effects, self:addLumiEffect(rootDir, "prefabs/Outline.prefab", LUMI_TARGET_TYPE_OUTPUT_RT, "", "test_effectID"))
    table.insert(self.effects, self:addLumiEffect(rootDir, "prefabs/Glow.prefab", LUMI_TARGET_TYPE_OUTPUT_RT, ""))
    -- table.insert(self.effects, self:addLumiEffect(rootDir, "prefabs/Filter_Effect.prefab", LUMI_TARGET_TYPE_OUTPUT_RT, ""))
    local effectID = self.effects[1]

    -- local img = self.scene.assetMgr:SyncLoad("effects/6.png")
    -- self.inputTexture = img
    -- self:updateLumiPara(effectID, "intensity", "0.5")
end

function LumiHub:onDestroy()
    self:_recoverStates()
    self.lumiNameIndex = 1
    self.effectIDToExtraNode = {}
    self.unusedExtraNodes = {}
    self.nodes = nil
    self.effectIDToPrefabPath = {}
    self.effectIDToLumiObject = {}
    self.quadMesh = nil
    self.lumiImageMaterial = nil
    -- used only by setInputTexSize and getOutputTexSize for computing RT size used outside LumiHub
    self.inputTexSize = nil
    self.pingPongTexture1 = nil
    self.pingPongTexture2 = nil
    self.newOutputTexture = nil
    self.extraTexture = nil
    self.lumiObjPingPongTexture = nil
    self.enableLumiEffectCache = false
    print("LumiHub onDestroy end!")
end

function LumiHub:onStart(comp)
    print("onStart begin")

    self:_initLumi(comp)
    self:_lasyInitLumi()

    -- test code start
    if isEditor then
        self:_testCreateEffects()
        local value = Amaz.Vector()
        value:pushBack(0.0)
        value:pushBack(1.0)
        value:pushBack(0.0)
        self:updateLumiPara(self.effects[1], "color", value)
        self:updateLumiPara(self.effects[1], "blurIntensity", 0.5)
        self:updateLumiPara(self.effects[1], "intensity", 0.5)
        self:updateLumiPara(self.effects[1], "opacity", 0.5)
        self:updateLumiPara(self.effects[2], "opacity", 0.3)
        print('effects:', self.effects)
        local opacity = self:getLumiPara(self.effects[1], "opacity")
        print('opacity:', opacity)
        opacity = self:getLumiPara(self.effects[2], "opacity")
        print('opacity:', opacity)
        Amaz.LOGE("haha", tostring(self.effectIDToLumiObject[self.effects[1]].effect_lua["opacity"]))
        Amaz.LOGE("haha", tostring(self.effectIDToLumiObject[self.effects[1]].effect_lua["opacity"]))
        Amaz.LOGE("haha", tostring(self.effectIDToLumiObject[self.effects[1]].effect_lua["opacity"]))
        Amaz.LOGE("haha", tostring(self.effectIDToLumiObject[self.effects[2]].effect_lua["opacity"]))
        -- for _, v in pairs(self.effects) do
        --     self:removeLumiEffect(v)
        -- end
        print(self.lumiTable.lumiTable)
        self:clearLumiEffectForTarget(LUMI_TARGET_TYPE_CAMERA_RT, "Camera")
        print(self.lumiTable.lumiTable)
        self:clearLumiEffectForTarget(LUMI_TARGET_TYPE_RENDERER, "Cube1")
        print(self.lumiTable.lumiTable)
        self:clearLumiEffect()
        print(self.lumiTable.lumiTable)
        self.effects = {}
        self:_testCreateEffects()
        self:setInputTexSize(800, 600)
        local outputSize = self:getOutputTexSize()
        print("outputSize:", outputSize)
    end
    -- test code end
    print("onStart end")
end

function LumiHub:onUpdate(comp, deltaTime)
end

exports.LumiHub = LumiHub
return exports
