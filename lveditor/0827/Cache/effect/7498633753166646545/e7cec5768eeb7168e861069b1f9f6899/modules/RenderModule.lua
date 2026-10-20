--[[
    Script Name: RenderModule.lua
    Version: 1.01
    Update Data: 2024.04.16
--]]

local CommonFunc = {
    ----- BlendFactor
    ----- ZERO	                0
    ----- ONE	                1
    ----- SRC_COLOR	            2
    ----- ONE_MINUS_SRC_COLOR	3
    ----- DST_COLOR	            4
    ----- ONE_MINUS_DST_COLOR	5
    ----- SRC_ALPHA	            6
    ----- ONE_MINUS_SRC_ALPHA	7
    ----- DST_ALPHA	            8
    ----- ONE_MINUS_DST_ALPHA	9
    blendMode = {
        ["Normal"] = {["src"] = 1, ["dst"] = 7, ["srcA"] = 1, ["dstA"] = 7},
        ["NormalWithAlpha"] = {["src"] = 6, ["dst"] = 7, ["srcA"] = 6, ["dstA"] = 7},
        ["Add"] = {["src"] = 1, ["dst"] = 1, ["srcA"] = 1, ["dstA"] = 1},
        ["Screen"] = {["src"] = 5, ["dst"] = 1, ["srcA"] = 9, ["dstA"] = 1},
        ["Mutiply"] = {["src"] = 5, ["dst"] = 0, ["srcA"] = 9, ["dstA"] = 0},
    }
}
function CommonFunc.materialClone(_mat)
    local oriPassSize = _mat.xshader.passes:size()
    local retMat = _mat:instantiate()
    if oriPassSize < 1 then return retMat end
    
    retMat.xshader =_mat.xshader:clone()
    retMat.xshader.passes = Amaz.Vector()
    for i = 0, oriPassSize - 2 do
        retMat.xshader:addPass(_mat.xshader.passes:get(i))
    end
    retMat.xshader:addPass(_mat.xshader.passes:get(oriPassSize - 1):clone())

    return retMat
end

function CommonFunc.setMatLastPassOutputRT(_mat, rt)
    local xshader = _mat.xshader
    local passesNum = xshader.passes:size()
    if passesNum < 1 then return -1 end
    local lastPass = xshader.passes:get(passesNum - 1)
    lastPass.renderTexture = rt
end

function CommonFunc._setColorBlendState(pass, mode)
    if mode == nil then
        return 
    end
    local colorBlend = pass.renderState.colorBlend
    if mode ~= "" then
        if colorBlend then
            local attachments = colorBlend.attachments
            if attachments:size() > 0 then
                local attach = attachments:get(0)
                attach.blendEnable = true
                attach.srcColorBlendFactor = CommonFunc.blendMode[mode].src
                attach.dstColorBlendFactor = CommonFunc.blendMode[mode].dst
                attach.srcAlphaBlendFactor = CommonFunc.blendMode[mode].srcA
                attach.dstAlphaBlendFactor = CommonFunc.blendMode[mode].dstA
                attach.colorWriteMask = 15
                attach.ColorBlendOp = 0
                attach.AlphaBlendOp = 0
            else
                local attach = Amaz.ColorBlendAttachmentState()
                attach.blendEnable = true
                attach.srcColorBlendFactor = CommonFunc.blendMode[mode].src
                attach.dstColorBlendFactor = CommonFunc.blendMode[mode].dst
                attach.srcAlphaBlendFactor = CommonFunc.blendMode[mode].srcA
                attach.dstAlphaBlendFactor = CommonFunc.blendMode[mode].dstA
                attach.colorWriteMask = 15
                attach.ColorBlendOp = 0
                attach.AlphaBlendOp = 0
                colorBlend:pushAttachment(attach)
            end
        else
            colorBlend = Amaz.ColorBlendState()
            local attach = Amaz.ColorBlendAttachmentState()
            attach.blendEnable = true
            attach.srcColorBlendFactor = CommonFunc.blendMode[mode].src
            attach.dstColorBlendFactor = CommonFunc.blendMode[mode].dst
            attach.srcAlphaBlendFactor = CommonFunc.blendMode[mode].srcA
            attach.dstAlphaBlendFactor = CommonFunc.blendMode[mode].dstA
            attach.colorWriteMask = 15
            attach.ColorBlendOp = 0
            attach.AlphaBlendOp = 0
            colorBlend:pushAttachment(attach)
            pass.renderState.colorBlend = colorBlend
        end
    else
        if colorBlend then
            local attachments = colorBlend.attachments
            if attachments:size() > 0 then
                local attach = attachments:get(0)
                attach.blendEnable = false
            end
        end
    end
end

function CommonFunc.setLastPassColorBlendMode(_mat, mode)
    local xshader = _mat.xshader
    local passesNum = xshader.passes:size()
    if passesNum < 1 then return -1 end
    local lastPass = xshader.passes:get(passesNum - 1)
    CommonFunc._setColorBlendState(lastPass, mode)
end


--[[  ****************************************
    storge the RenderNode info
--    **************************************** ]]  
local RenderNode = {}
RenderNode.__index = RenderNode

function RenderNode.new(_sticker, _idx, _mesh, _mat, _parent, _layer)
    local self = setmetatable({}, RenderNode)
    self.entity = _sticker.entity.scene:createEntity("RenderNode_".._idx)
    self.trans = self.entity:addComponent("Transform")
    self.renderer = self.entity:addComponent("MeshRenderer")
    if _sticker.trans then
        if _parent then
            self.trans.parent = _parent
            self.parent = _parent
            _parent.children:pushBack(self.trans)
        else
            self.trans.parent = _sticker.parent
            self.parent = _sticker.parent
            _sticker.parent.children:pushBack(self.trans)
        end
    end
    self:setMesh(_mesh)
    self.materials = self.renderer.materials
    self.materials:clear()
    self.material = self:addMaterial(_mat)
    self.renderer.sortingOrder = _sticker.renderer.sortingOrder


    if _layer ~= nil then
        self.entity.layer = _layer
    else
        self.entity.layer = _sticker.entity.layer
    end
    return self
end

function RenderNode:setMesh(_mesh)
    if _mesh == nil then
        return 
    end
    self.renderer.mesh = _mesh
    self.mesh = _mesh
end

function RenderNode:addMaterial(_mat, _rt, _mode, _order)
    if _mat == nil then
        return nil
    end
    local mat = CommonFunc.materialClone(_mat)
    CommonFunc.setMatLastPassOutputRT(mat, _rt)
    CommonFunc.setLastPassColorBlendMode(mat, _mode)
    if _order then
        mat.renderQueue = _order
    end
    self.materials:pushBack(mat)
    return mat
end

function RenderNode:getMaterial(idx)
    if idx == nil then
        idx = 0
    end
    if self.materials:size() <= 0 then
        return nil
    end
    return self.materials:get(idx)
end

function RenderNode:destroy()
    self.parent.children:erase(self.trans)
    self.entity.scene:removeEntity(self.entity)
    self.entity = nil
    self = {}
end

--[[  ****************************************
    RenderModule: manager the render
--    **************************************** ]]  
local RenderModule = {}
RenderModule.__index = RenderModule
RenderModule.TEXT_STICKER = "TEXT_STICKER"

function RenderModule.new(_sticker)
    local self = setmetatable({}, RenderModule)
    self.sticker = _sticker
    self.renderNodeList = {}
    self.quadInfo = {}
    return self
end

function RenderModule:createRenderTexture(width, height, shared, name, colorFormat)
    if name == nil then
        name = ""
    end
    local rt = Amaz.RenderTexture()
    rt.name = name
    rt.builtinType = Amaz.BuiltInTextureType.NORMAL
    rt.internalFormat = Amaz.InternalFormat.RGBA8
    rt.dataType = Amaz.DataType.U8norm
    rt.depth = 1
    rt.attachment = Amaz.RenderTextureAttachment.NONE
    rt.filterMag = Amaz.FilterMode.LINEAR
    rt.filterMin = Amaz.FilterMode.LINEAR
    rt.filterMipmap = Amaz.FilterMipmapMode.FilterMode_NONE
    rt.width = width
    rt.height = height
    rt.colorFormat = colorFormat or Amaz.PixelFormat.RGBA8Unorm
    if shared == nil then
        shared = false
    end
    rt.shared = shared
    return rt
end

function RenderModule:createScreenRenderTexture(pecentX, pecentY, shared, name, colorFormat)
    if name == nil then
        name = ""
    end
    if pecentX == nil then
        pecentX = 1
    end
    if pecentY == nil then
        pecentY = 1
    end
    local rt = Amaz.ScreenRenderTexture()
    rt.name = name
    rt.builtinType = Amaz.BuiltInTextureType.NORMAL
    rt.internalFormat = Amaz.InternalFormat.RGBA8
    rt.dataType = Amaz.DataType.U8norm
    rt.depth = 1
    rt.attachment = Amaz.RenderTextureAttachment.NONE
    rt.filterMag = Amaz.FilterMode.LINEAR
    rt.filterMin = Amaz.FilterMode.LINEAR
    rt.filterMipmap = Amaz.FilterMipmapMode.FilterMode_NONE
    rt.pecentX = pecentX
    rt.pecentY = pecentY
    rt.colorFormat = colorFormat or Amaz.PixelFormat.RGBA8Unorm
    if shared == nil then
        shared = false
    end
    rt.shared = shared
    return rt
end

function RenderModule:_createPass(backend, vertCode, fragCode, stencilMasked)
    local newPass = Amaz.Pass()

    local vs = Amaz.Shader()
    vs.type = Amaz.ShaderType.VERTEX
    vs.source = vertCode

    local fs = Amaz.Shader()
    fs.type = Amaz.ShaderType.FRAGMENT
    fs.source = fragCode

    local shaderVec = Amaz.Vector()
    shaderVec:pushBack(vs)
    shaderVec:pushBack(fs)

    local shaderMap = Amaz.Map()
    shaderMap:insert(backend, shaderVec)
    
    newPass.shaders = shaderMap

    local semantics = Amaz.Map()
    semantics:insert("position", Amaz.VertexAttribType.POSITION)   -- all post effect shaders must follow this tradition
    semantics:insert("texcoord0", Amaz.VertexAttribType.TEXCOORD0)  -- all post effect shaders must follow this tradition
    newPass.semantics = semantics
    
    local depthStencilState = Amaz.DepthStencilState()
    depthStencilState.depthTestEnable = false
    
    local renderState = Amaz.RenderState()
    renderState.depthstencil = depthStencilState

    if stencilMasked == true then
        depthStencilState.stencilTestEnable = true;

        local stencilOp = Amaz.StencilOpState();
        stencilOp.compareOp = Amaz.CompareOp.EQUAL;
        stencilOp.reference = 42;
        stencilOp.writeMask = 0;

        depthStencilState.stencilFront = stencilOp;
        depthStencilState.stencilBack = stencilOp;
    end

    newPass.renderState = renderState

    return newPass
end

function RenderModule:createClearScreenPass(_rt)
    local vShaderCode = [[
        precision highp float;
        attribute vec2 position;
        attribute vec2 texcoord0;
        varying vec2 uv0;
        void main() 
        { 
            gl_Position = (vec4(texcoord0.xy * 2.0 - 1.0, 0.0, 1.0));
            uv0 = texcoord0;
        }    
    ]]
    local fShaderCode = [[
        precision highp float;
        varying highp vec2 uv0;
        void main()
        {
            gl_FragColor = vec4(0.0);
        }
    ]]
    local newPass = RenderModule:_createPass("gles2", vShaderCode, fShaderCode, false)
    newPass.renderTexture = _rt
    return newPass
end

function RenderModule:createDefaultTextRenderPass(_rt, _mode)
    local vShaderCode = [[
        precision highp float;
        attribute vec2 position;
        attribute vec2 texcoord0;
        varying vec2 uv0;
        uniform mat4 u_MVP;
        void main() 
        { 
            gl_Position = u_MVP * vec4(position.xy, 0.0, 1.0);
            uv0 = texcoord0;
            uv0.y = 1.0 - uv0.y;
        }    
    ]]
    local fShaderCode = [[
        precision highp float;
        varying highp vec2 uv0;
        uniform sampler2D _MainTex;
        void main()
        {
            gl_FragColor = texture2D(_MainTex, uv0);
        }
    ]]
    local newPass = RenderModule:_createPass("gles2", vShaderCode, fShaderCode, false)
    newPass.renderTexture = _rt
    CommonFunc._setColorBlendState(newPass, _mode)
    return newPass
end

---- turn on the text post processing
---- param1 -> _rt: output rt of last pass, when _rt == nil, the output rt is the renderTexture of camera
---- param2 -> _mat: material for text render, when _mat == nil, use default text render
---- param3 -> _mode: blend mode of last pass, when _mode == nil or "", blend is disabled (support mode: Normal, NormalWithAlpha, Screen, Add, Mutiply)
---- param4 -> _order: render order of material
---- param5 -> _textInfo: need turn on the post processing of text entity, when _textInfo == nil, turn on the main tex
---- param6 -> _needClear: is need add clear pass at the front of material, default true
---- when _rt and _mode both are nil, _mode will be set as Normal
function RenderModule:textPostProcessing(_rt, _mat, _mode, _order, _textInfo, _needClear)
    if _needClear == nil then
        _needClear = true
    end
    local material = nil
    local clearPass = nil
    if _rt ~= nil and _needClear == true then
        clearPass = self:createClearScreenPass(_rt)
    end
    if _rt == nil and _mode == nil then
        _mode = "Normal"
    end
    if _mat == nil then
        material = Amaz.Material()
        material.renderQueue = 0
        material.xshader = Amaz.XShader()
        if clearPass then
            material.xshader.passes:pushBack(clearPass)
        end
        local defaultPass = self:createDefaultTextRenderPass(_rt, _mode)
        material.xshader.passes:pushBack(defaultPass)
    else
        material = CommonFunc.materialClone(_mat)
        if clearPass then
            material.xshader.passes:pushFront(clearPass)
        end
        CommonFunc.setMatLastPassOutputRT(material, _rt)
        CommonFunc.setLastPassColorBlendMode(material, _mode)
    end
    if _textInfo == nil then
        self.sticker.richText.canvas.renderToRT = true
        self.sticker.renderer.materials:clear()
        self.sticker.renderer.material = material
    else
        _textInfo.richText.canvas.renderToRT = true
        _textInfo.renderer.materials:clear()
        _textInfo.renderer.material = material
    end
    if _order ~= nil then
        material.renderQueue = _order
    end
    return material

end

function RenderModule:addQuadInfo(_rect, _modelInfo)
    local _model = nil
    if _modelInfo == nil then
        _model = Amaz.Matrix3x3f():setIdentity()
    else
        _model = self:calcModel(_modelInfo.s, _modelInfo.r, _modelInfo.t, _modelInfo.a)
    end
    if type(_rect) == "table" and _rect.x == nil then
        for i = 1, #_rect do
            local rectCenter = Amaz.Vector3f(_rect[i].x, _rect[i].y, 0.0)
            local info = {}
            info.pos = {
                _model:multiplyVector3(Amaz.Vector3f(-_rect[i].width, -_rect[i].height, 1)) + rectCenter,
                _model:multiplyVector3(Amaz.Vector3f( _rect[i].width, -_rect[i].height, 1)) + rectCenter,
                _model:multiplyVector3(Amaz.Vector3f( _rect[i].width,  _rect[i].height, 1)) + rectCenter,
                _model:multiplyVector3(Amaz.Vector3f(-_rect[i].width,  _rect[i].height, 1)) + rectCenter
            } 
            info.uv = {
                Amaz.Vector2f(0, 0),
                Amaz.Vector2f(1, 0),
                Amaz.Vector2f(1, 1),
                Amaz.Vector2f(0, 1),
            }
            info.indices = {0,1,3, 1,2,3}
            table.insert(self.quadInfo, info)
        end
    else
        local rectCenter = Amaz.Vector3f(_rect.x, _rect.y, 0.0)
        local info = {}
        info.pos = {
            _model:multiplyVector3(Amaz.Vector3f(-_rect.width, -_rect.height, 1)) + rectCenter,
            _model:multiplyVector3(Amaz.Vector3f( _rect.width, -_rect.height, 1)) + rectCenter,
            _model:multiplyVector3(Amaz.Vector3f( _rect.width,  _rect.height, 1)) + rectCenter,
            _model:multiplyVector3(Amaz.Vector3f(-_rect.width,  _rect.height, 1)) + rectCenter
        } 
        info.uv = {
            Amaz.Vector2f(0, 0),
            Amaz.Vector2f(1, 0),
            Amaz.Vector2f(1, 1),
            Amaz.Vector2f(0, 1),
        }
        info.indices = {0,1,3, 1,2,3}
        table.insert(self.quadInfo, info)
    end
end

function RenderModule:_combineQuad()
    local curQuad = {}
    curQuad.pos = {}
    curQuad.uv = {}
    curQuad.indices = {}
    for i = 1, #self.quadInfo do
        local info = self.quadInfo[i]

        for j = 1, #info.pos do
            local pos = info.pos[j]
            pos = Amaz.Vector3f(pos.x, pos.y, i)
            table.insert(curQuad.pos, pos)
        end

        for j = 1, #info.uv do
            table.insert(curQuad.uv, info.uv[j])
        end

        for j = 1, #info.indices do
            local indices = info.indices[j]
            indices = indices + (i-1) * 4
            table.insert(curQuad.indices, indices)
        end
    end
    self.quadInfo = {}
    return curQuad
end

function RenderModule:generateMesh(extendAttrs)
    local meshInfo = self:_combineQuad()
    if extendAttrs == nil then
        extendAttrs = 0
    end

    local mesh = Amaz.Mesh()

    local posVAD = Amaz.VertexAttribDesc()
    posVAD.semantic = Amaz.VertexAttribType.POSITION
    local uvVAD = Amaz.VertexAttribDesc()
    uvVAD.semantic = Amaz.VertexAttribType.TEXCOORD0
    local colorVAD = Amaz.VertexAttribDesc()
    colorVAD.semantic = Amaz.VertexAttribType.COLOR
    local vads = Amaz.Vector()
    vads:pushBack(posVAD)
    vads:pushBack(uvVAD)
    vads:pushBack(colorVAD)
    local uviArray = {}
    for i = 1, math.max(extendAttrs, 7) do
        local uvi = Amaz.VertexAttribDesc()
        uvi.semantic = Amaz.VertexAttribType.TEXCOORD0 + i
        vads:pushBack(uvi)
        uviArray[i] = Amaz.Vec2Vector()
    end
    
    mesh.vertexAttribs = vads

    local posArray = Amaz.Vec3Vector()
    local colorArray = Amaz.Vec4Vector()
    for i = 1, #meshInfo.pos do
        posArray:pushBack(meshInfo.pos[i])
        colorArray:pushBack(Amaz.Vector4f(0,0,0,0))
    end
    local indices = Amaz.UInt16Vector()
    for i = 1, #meshInfo.indices do
        indices:pushBack(meshInfo.indices[i])
    end
    
    local uvArray = Amaz.Vec2Vector()
    for i = 1, #meshInfo.uv do
        uvArray:pushBack(meshInfo.uv[i])

        for j = 1, math.max(extendAttrs, 7) do
            local uvi = Amaz.Vector2f(0, 0)
            uviArray[j]:pushBack(uvi)
        end
    end


    mesh:setVertexArray(posArray)
    mesh:setUvArray(0, uvArray)
    mesh:setColorArray(colorArray)
    for i = 1, math.max(extendAttrs, 7) do
        mesh:setUvArray(i, uviArray[i])
    end
    mesh.clearAfterUpload = true

    local subMesh = Amaz.SubMesh()
    subMesh.primitive = Amaz.Primitive.TRIANGLES
    subMesh.indices16 = indices
    subMesh.mesh = mesh
    mesh:addSubMesh(subMesh)

    return mesh
end

function RenderModule:addRenderNode(_mesh, _mat, _parent, _layer)
    if _mesh == nil then
        if self.defaultQuadMesh == nil then
            local backUpQuadInfo = self.quadInfo
            self.quadInfo = {}
            self:addQuadInfo({x = 0, y = 0, width = 1, height = 1})
            self.defaultQuadMesh = self:generateMesh()
            self.quadInfo = backUpQuadInfo
        end 
        _mesh = self.defaultQuadMesh
    end

    local renderNode = RenderNode.new(self.sticker, #self.renderNodeList, _mesh, _mat, _parent, _layer)
    table.insert(self.renderNodeList, renderNode)
    return renderNode

end

function RenderModule:getMatLastPassOutputRT(_mat)
    local xshader = _mat.xshader
    local passesNum = xshader.passes:size()
    if passesNum < 1 then return -1 end
    local lastPass = xshader.passes:get(passesNum - 1)
    if lastPass.renderTexture then
        return lastPass.renderTexture
    else
        return self.sticker.entity.scene:getOutputRenderTexture()
    end
end

function RenderModule:getCameraOutputRT()
    return self.sticker.entity.scene:getOutputRenderTexture()
end

function RenderModule:setMatLastPassOutputRT(_mat, rt)
    CommonFunc.setMatLastPassOutputRT(_mat, rt)
end

function RenderModule:setMatPassOutputRT(_mat, rt, passIdx)
    if passIdx == nil then
        passIdx = 0
    end
    local xshader = _mat.xshader
    local passesNum = xshader.passes:size()
    if passesNum < 1 then return -1 end
    local pass = xshader.passes:get(passIdx)
    pass.renderTexture = rt
end

function RenderModule:setMatInputRT(_mat, rt)
    _mat:setTex("u_inputTexture", rt)
end

function RenderModule:setPassColorBlendMode(_mat, mode, passIdx, passName)
    local xshader = _mat.xshader
    if passName ~= "" and passName ~= nil then
        if xshader.passes:size() > 0 then
            local pass = nil
            for i = 0, xshader.passes:size() - 1 do
                local pass_t = xshader.passes:get(i)
                if passName == pass_t.name then
                    pass = pass_t
                    break
                end
            end
            if pass then
                CommonFunc._setColorBlendState(pass, mode)
            else
                if xshader.passes:size() > passIdx and passIdx > 0 then
                    pass = xshader.passes:get(passIdx)
                    CommonFunc._setColorBlendState(pass, mode)
                end
            end
        end
    else
    end
end

function RenderModule:setLastPassColorBlendMode(_mat, mode)
    CommonFunc.setLastPassColorBlendMode(_mat, mode)
end

function RenderModule:materialClone(_mat)
    return CommonFunc.materialClone(_mat)
end

function RenderModule:addMaterial(_renderer, _mat, _order)
    local retMat = CommonFunc.materialClone(_mat)
    if _order then
        retMat.renderQueue = _order
    end
    _renderer.materials:pushBack(retMat)
    return retMat
end

function RenderModule:setMaterial(_renderer, _mat, _order)
    self.sticker.richText.canvas.renderToRT = true
    if _mat == nil then 
        return -1
    end
    local insMaterial = nil
    insMaterial = _mat:instantiate()
    if _order then
        insMaterial.renderQueue = _order
    end
    _renderer.materials:clear()
    _renderer.material = insMaterial
    local ret_material = _renderer.material
    return ret_material
end

--[[ ***************************************
    _s: scale,  ex. _s = {x = 1, y = 1}
    _r: angle,  ex. _r = 90
    _t: offset, ex. _t = {x = 0, y = 0}
    _a: anchor, ex. _a = {x = 0, y = 0}
     *************************************** ]]
function RenderModule:calcModel(_s, _r, _t, _a)
    local scale = _s
    if _s == nil then
        scale = {x = 1, y = 1}
    end

    local angle = _r
    if _r == nil then
        angle = 0
    end

    local offset = _t
    if _t == nil then
        offset = {x = 0, y = 0}
    end

    local anchor = _a
    if _a == nil then
        anchor = {x = 0, y = 0}
    end

    local theta = math.rad(angle)
    local cost = math.cos(theta)
    local sint = math.sin(theta)
    ---- combine all info of trans
    anchor = {x = anchor.x * (1 - scale.x * cost) + anchor.y * scale.y * sint, 
                    y = anchor.y * (1 - scale.y * cost) + anchor.x * scale.x * sint}
    local retMat3x3 = Amaz.Matrix3x3f(scale.x * cost, - scale.y * sint, anchor.x + offset.x,
                                        scale.x * sint, scale.y * cost, anchor.y + offset.y,
                                        0, 0, 1)
    return retMat3x3
end
    
function RenderModule:init()
    self.renderNodeList = {}
    self.quadInfo = {}
end

function RenderModule:reset()
    if self.sticker.richText then
        self.sticker.richText.canvas.renderToRT = false
    end
    if self.sticker.renderer then
        self.sticker.renderer.materials:clear()
    end

    for i = 1, #self.renderNodeList do
        local rn = self.renderNodeList[i]
        rn:destroy()
    end
    self.renderNodeList = {}
	collectgarbage("collect")
end

return RenderModule