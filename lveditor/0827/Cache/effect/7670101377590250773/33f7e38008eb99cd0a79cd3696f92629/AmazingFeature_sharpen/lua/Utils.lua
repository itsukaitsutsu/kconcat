local Utils = Utils or {}
function Utils.CreateRenderTexture(name, width, height, colorFormat, renderTextureType)
    local rt = nil
    if renderTextureType == "RenderTexture" then
        rt = Amaz.RenderTexture()
    elseif renderTextureType == "ScreenRenderTexture" then
        rt = Amaz.ScreenRenderTexture()
    end
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
    rt.shared = true
    return rt
end

function Utils.setMaterialProp(self, comp)
    local link = self.renderChain.link
    for i = 1, #link do
        local materialProp = link[i].materialProp
        if materialProp then
            for type, props in pairs(materialProp) do
                if type == "Float" then
                    for key, value in pairs(props) do
                        link[i].material:setFloat(key, value)
                    end
                elseif type == "Int" then
                    for key, value in pairs(props) do
                        link[i].material:setInt(key, value)
                    end
                elseif type == "Tex" then
                    for key, value in pairs(props) do
                        local png = comp.entity.scene.assetMgr:SyncLoad(value)
                        link[i].material:setTex(key, png)
                    end
                elseif type == "Vec2" then
                    for key, value in pairs(props) do
                        link[i].material:setVec2(key, Amaz.Vector3f(value.x, value.y))
                    end
                elseif type == "Vec3" then
                    for key, value in pairs(props) do
                        link[i].material:setVec3(key, Amaz.Vector3f(value.x, value.y, value.z))
                    end
                elseif type == "Vec4" then
                    for key, value in pairs(props) do
                        link[i].material:setVec4(key, Amaz.Vector3f(value.x, value.y, value.z, value.w))
                    end
                end
            end
        end
    end
end

function Utils.split(s, p)
    local rt = {}
    string.gsub(
        s,
        "[^" .. p .. "]+",
        function(w)
            table.insert(rt, w)
        end
    )
    return rt
end

function Utils.buildRenderChain(self, comp)
    local colorFormat = Amaz.PixelFormat.RGBA8Unorm
    local RTCounter = self.RTCounter
    local link = self.renderChain.link
    local w = Amaz.BuiltinObject:getInputTextureWidth()
    local h = Amaz.BuiltinObject:getInputTextureHeight()
    -- init RTCounter
    RTCounter[1] = {}
    -- RTCounter[1].rt = comp.entity.scene:getOutputRenderTexture()
    RTCounter[1].rt = comp.entity.scene.assetMgr:SyncLoad("rt/outputTex.rt")
    RTCounter[1].width = w
    RTCounter[1].height = h
    for i = 1, #RTCounter do
        RTCounter[i].count = -1
    end

    -- set final node's renderTexture as output
    for i = #link, 1, -1 do
        if link[i].visible then
            link[i].renderTextureType = "ScreenRenderTexture"
            break
        end
    end

    -- set outputSize of ScreenRenderTexture
    for i = 1, #link do
        if link[i].visible and link[i].renderTextureType == "ScreenRenderTexture" then
            link[i].outputSize.width = w
            link[i].outputSize.height = h
        end
    end

    -- set simple input
    for i = 1, #link do
        if link[i].visible then
            for k, v in pairs(link[i].input) do
                if tostring(v) ~= "userdata: 0x0" then
                    local spiltRet = Utils.split(v, ".")
                    if #spiltRet == 3 and spiltRet[2] == "input" then
                        for j = 1, #link do
                            if link[j].visible then
                                if spiltRet[1] == link[j].entityName then
                                    link[i].input[k] = link[j].input[spiltRet[3]]
                                end
                            end
                        end
                    end
                end
            end
        end
    end

    Amaz.LOGI("RenderTex", "----------------------------------------------------------------------------")
    -- create renderTexture and link the node
    for i = 1, #link do
        if link[i].visible then
            -- search availableRT
            local availableRT = 0
            if not link[i].output then
                local countFlag = 0
                if not self.renderChain.isUseRTShare then
                    countFlag = -1
                end
                for j = 1, #RTCounter do
                    if
                        RTCounter[j].count <= countFlag and RTCounter[j].width == link[i].outputSize.width and
                            RTCounter[j].height == link[i].outputSize.height
                     then
                        if link[i].RTShare then
                            availableRT = j
                            break
                        elseif RTCounter[j].count < 0 then
                            availableRT = j
                            break
                        end
                    end
                end
            else
                for j = i - 1, 1, -1 do
                    if link[j].visible and link[j].entityName == link[i].output then
                        availableRT = link[j].renderTextureIndex
                        break
                    end
                end
            end

            -- create renderTexture
            if availableRT == 0 then
                availableRT = #RTCounter + 1
                RTCounter[availableRT] = {}
                RTCounter[availableRT].rt =
                    Utils.CreateRenderTexture(
                    "__tmp_rt_" .. (availableRT - 1),
                    link[i].outputSize.width,
                    link[i].outputSize.height,
                    colorFormat,
                    link[i].renderTextureType
                )
                local rtStr = "create __tmp_rt_ " .. (availableRT - 1)
                rtStr = rtStr .. " " .. link[i].outputSize.width .. " * " .. link[i].outputSize.height
                Amaz.LOGI("RenderTex", rtStr)
                RTCounter[availableRT].width = link[i].outputSize.width
                RTCounter[availableRT].height = link[i].outputSize.height
            end

            -- use the availableRT
            if link[i].RTShare then
                if not link[i].output then
                    RTCounter[availableRT].count = 0
                end
            else
                RTCounter[availableRT].count = 999
            end
            link[i].renderTextureIndex = availableRT

            -- update the use information of rt
            for j = i + 1, #link do
                if link[j].visible and link[j].input then
                    for k, v in pairs(link[j].input) do
                        if v == link[i].entityName then
                            RTCounter[availableRT].count = RTCounter[availableRT].count + 1
                        end
                    end
                end
            end
            if link[i].input then
                for k, v in pairs(link[i].input) do
                    for j = i - 1, 1, -1 do
                        if link[j].visible and link[j].entityName == v then
                            local m = link[j].renderTextureIndex
                            RTCounter[m].count = RTCounter[m].count - 1
                        end
                    end
                end
            end
        end
    end

    -- reorder the output rendertexture and tmp rendertexture
    local lastIndex = 1
    for i = #link, 1, -1 do
        if link[i].visible then
            lastIndex = link[i].renderTextureIndex
            break
        end
    end
    for i = 1, #link do
        if link[i].visible then
            if link[i].renderTextureIndex == lastIndex then
                link[i].renderTextureIndex = 1
            elseif link[i].renderTextureIndex == 1 then
                link[i].renderTextureIndex = lastIndex
            end
        end
    end
    Amaz.LOGI("RenderTex", "----------------------------------------------------------------------------")
    -- set the output rt of xshader
    for i = 1, #link do
        if link[i].visible then
            local availableRT = link[i].renderTextureIndex
            if availableRT == 1 then
                link[i].material.xshader.passes:get(0).renderTexture = nil
            else
                link[i].material.xshader.passes:get(0).renderTexture = RTCounter[availableRT].rt
            end
            if link[i].renderTextureIndex == 1 then
                Amaz.LOGI("RenderTex", link[i].entityName .. ": output_rt " .. w .. " * " .. h)
            else
                local rtStr = link[i].entityName .. ": __tmp_rt_ " .. (link[i].renderTextureIndex - 1)
                rtStr = rtStr .. " " .. link[i].outputSize.width .. " * " .. link[i].outputSize.height
                Amaz.LOGI("RenderTex", rtStr)
            end
        end
    end

    -- set the input rt of material
    for i = 1, #link do
        if link[i].visible then
            local material = link[i].material
            if link[i].input then
                for k, v in pairs(link[i].input) do
                    if v == "INPUT0" then
                        material:setTex(k, self.inputTexture)
                    else
                        for j = i - 1, 1, -1 do
                            if link[j].visible and link[j].entityName == v then
                                material:setTex(k, RTCounter[link[j].renderTextureIndex].rt)
                            end
                        end
                    end
                end
            end
        end
    end
end

return Utils
