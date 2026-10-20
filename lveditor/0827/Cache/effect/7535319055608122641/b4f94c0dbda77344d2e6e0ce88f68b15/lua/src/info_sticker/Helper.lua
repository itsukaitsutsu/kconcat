local Utils = require("common/Utils")

local Helper = {}


function Helper.getRootDir ()
    return debug.getinfo(2, "S").source:match("@?(.*/)")
end
function Helper.getRootDirInLuaDir ()
    local path = debug.getinfo(2, "S").source
    local len = #path
    local counter = 0
    for i = len, 1, -1 do
        local leadingByte = string.byte(path, i)
        if leadingByte == 47 then
            counter = counter + 1
            if counter == 2 then
                return string.sub(path, 1, i)
            end
        end
    end
end
function Helper.getStyle0 (comp)
    if not comp then
        return
    end
    if comp.forceFlushCommandQueue then
        comp:forceFlushCommandQueue()
    end
    local letters = comp.letters
    if letters:size() > 0 then
        local letter0 = letters:get(0)
        return letter0.letterStyle
    end
end
function Helper.setFontSize (comp, size)
    local letters = comp.letters
    for i = 0, letters:size() - 1 do
        local letter = letters:get(i)
        letter.letterStyle.fontSize = size
    end
    if comp.forceFlushCommandQueue then
        comp:forceFlushCommandQueue()
    end
end
function Helper.flushStyle (comp)
    if comp.forceFlushCommandQueue then
        comp:forceFlushCommandQueue()
    end
end

function Helper.disableText (node, transparentMaterial)
    local materials = Amaz.Vector()
    materials:pushBack(transparentMaterial)
    local Text = node:getComponent("Text")
    Text.canvas.renderToRT = true
    local Renderer = node:getComponent("MeshRenderer")
    Renderer.materials = materials
end
function Helper.enableText (node)
    local Text = node:getComponent("Text")
    Text.canvas.renderToRT = false
end

function Helper.attachCopy (node, name)
    local copy = node:searchEntity(name)
    if copy then
        node.scene:removeEntity(copy)
    end
    copy = node.scene:createEntity(name)

    local Transform0 = node:getComponent("Transform")
    local Transform1 = copy:cloneComponentOf(Transform0)
    Transform1.parent = Transform0
    Transform0.children:pushBack(Transform1)

    local MeshRenderer0 = node:getComponent("MeshRenderer")
    local MeshRenderer1 = copy:cloneComponentOf(MeshRenderer0)

    local Text0 = node:getComponent("Text")
    local Text1 = copy:cloneComponentOf(Text0)
    Text1.bloomEnable = false

    return {
        Entity = copy,
        Transform = Transform1,
        Renderer = MeshRenderer1,
        Text = Text1,
    }
end
function Helper.detachCopy (copy)
    if copy then
        copy.Transform.parent.children:erase(copy.Transform)
        copy.Entity.scene:removeEntity(copy.Entity)
    end
end

function Helper.getTightRect (letter, outlineMaxWidth, expandAffector)
    local pixelHeight = letter.letterStyle.fontSize * 300 / 72
    local rectDistance = pixelHeight * outlineMaxWidth
    local expand = expandAffector == nil and 0.2 or expandAffector
    local tightRect = Amaz.Rect(
        letter.rect.x + rectDistance,
        letter.rect.y + rectDistance,
        letter.rect.width - rectDistance * 2 + expand * pixelHeight,
        letter.rect.height - rectDistance * 2 + expand * pixelHeight
    )
    return tightRect
end

function Helper.setMaterialToText (text, renderer, material)
    text.canvas.renderToRT = true
    material = material:instantiate()
    local materials = Amaz.Vector()
    materials:pushBack(material)
    renderer.materials = materials
    return material
end

function Helper.cloneText(env, name)
    local ctx = {}
    ctx.env = env
    ctx.scene = env.scene
    ctx.entity = env.scene:createEntity("clone_entity_"..name)
    ctx.transform = ctx.entity:cloneComponentOf(env.transform)
    ctx.text = ctx.entity:cloneComponentOf(env.text)
    ctx.renderer = ctx.entity:cloneComponentOf(env.renderer)
    ctx.text.richStr = env.text.richStr
    ctx.transform.parent = env.transform
    env.transform.children:pushBack(ctx.transform)
    return ctx
end
function Helper.clonePrefab(env, prefab, name)
    local ctx = {}
    ctx.scene = env.scene
    ctx.entity = prefab:instantiateToEntity(env.scene, env.entity)
    ctx.entity.name = "clone_entity_"..name
    ctx.transform = ctx.entity:getComponent("Transform")
    ctx.script = ctx.entity:getComponent("ScriptComponent")
    return ctx
end
function Helper.removeNode(ctx)
    ctx.transform.parent.children:erase(ctx.transform)
    ctx.scene:removeEntity(ctx.entity)
end


function Helper.isBreakLine (lastRowID, char)
    if char.rowth ~= lastRowID then
        return true
    end
    return char.utf8 == "\n"
end
function Helper.isVisibleChar (char)
    local lb = string.byte(char.utf8)
    return lb > 32 -- space
end
function Helper.splitByChar (chars)
    local frags = {}
    for i = 0, chars:size() - 1 do
        local char = chars:get(i)
        if Helper.isVisibleChar(char) then
            table.insert(frags, { line = char.rowth, char })
        end
    end
    return frags
end
function Helper.splitByWord (chars)
    local frag = { line = 0 }
    local frags = {}
    for i = 0, chars:size() - 1 do
        local char = chars:get(i)
        if Helper.isVisibleChar(char) then
            table.insert(frag, char)
            frag.line = char.rowth
        elseif #frag > 0 then
            table.insert(frags, frag)
            frag = {}
        end
    end
    if #frag > 0 then
        table.insert(frags, frag)
    end
    return frags
end
function Helper.splitByLine (chars)
    local frag = {}
    local frags = {}
    local rowID = 0
    for i = 0, chars:size() - 1 do
        local char = chars:get(i)
        if Helper.isVisibleChar(char) then
            table.insert(frag, char)
        elseif Helper.isBreakLine(rowID, char) then
            if #frag > 0 then
                table.insert(frags, frag)
                frag = {}
            end
            rowID = rowID + 1
        end
    end
    if #frag > 0 then
        table.insert(frags, frag)
    end
    return frags
end
function Helper.splitByNone (chars)
    local frag = {}
    local frags = {}
    for i = 0, chars:size() - 1 do
        local char = chars:get(i)
        if Helper.isVisibleChar(char) then
            table.insert(frag, char)
        end
    end
    if #frag > 0 then
        table.insert(frags, frag)
    end
    return frags
end


function Helper.getLuaObject (comp)
    local script = comp:getScript()
    if script then
        return Amaz.ScriptUtils.getLuaObj(script)
    end
end


function Helper.convertSubtitle0 (data)
    local src = data.words
    local dst = {}
    for _, word in ipairs(src) do
        local text = word.text
        local startFragIndex = #dst + 1
        local totalVisibleCharCount = 0
        local charCount = 0
        local charIndex = 1
        for i = 1, #text do
            local code = string.byte(text, i)
            if code <= 32 then
                if charCount > 0 then
                    table.insert(dst, {text = string.sub(text, charIndex, i - 1), visibleCharCount = charCount})
                    charCount = 0
                    charIndex = i
                end
            else
                charCount = charCount + 1
                totalVisibleCharCount = totalVisibleCharCount + 1
            end
        end
        table.insert(dst, {text = string.sub(text, charIndex, #text), visibleCharCount = charCount})

        local t0 = word.start_time
        local dt = word.end_time - t0
        for i = startFragIndex, #dst do
            local frag = dst[i]
            frag.start_time = t0
            frag.end_time = t0 + dt * (frag.visibleCharCount / totalVisibleCharCount)
            t0 = frag.end_time
        end
    end
    for i = #dst, 1, -1 do
        local frag = dst[i]
        local next = dst[i + 1]
        if next then
            if frag.visibleCharCount == 0 then
                frag.start_time = next.start_time
            end
            frag.end_time = next.start_time
        end
        if frag.visibleCharCount == 0 then
            frag.start_time = frag.end_time
        end
    end
    data.words = dst
    return data
end

function Helper.splitSubtitle0 (words, chars, letters)
    local frags = {}
    local frag = {}
    local wordI = 1
    local charN = 0
    for charI = 0, chars:size() - 1 do
        local char = chars:get(charI)
        local letter = letters:get(charI)
        local word = words[wordI]
        if not word then
            break
        end
        table.insert(frag, {char = char, letter = letter})
        charN = charN + #char.utf8code
        if charN >= #word.text then
            frag.word = word
            table.insert(frags, frag)
            frag = {}
            charN = 0
            wordI = wordI + 1
        end
    end
    return frags
end
function Helper.splitSubtitle1 (words, start, chars, letters)
    local frags = {}
    local frag = {}
    local wordI = 1
    local charN = start
    for charI = 0, chars:size() - 1 do
        local char = chars:get(charI)
        local letter = letters:get(charI)
        local word = words[wordI]
        if not word then
            break
        end
        table.insert(frag, {char = char, letter = letter})
        charN = charN + #char.utf8code
        if charN >= #word.text then
            frag.word = word
            table.insert(frags, frag)
            frag = {}
            start = charN
            charN = 0
            wordI = wordI + 1
        end
    end
    return frags, start
end

function Helper.createFramebuffer (w, h)
    local rb = Amaz.RenderTexture()
    rb.attachment = Amaz.RenderTextureAttachment.NONE
    rb.filterMag = Amaz.FilterMode.LINEAR
    rb.filterMin = Amaz.FilterMode.LINEAR
    rb.depth = 1
    rb.width = w or 720
    rb.height = h or 1280
    return rb
end
function Helper.createMesh (locations, primitive)
    if not locations then
        locations = {Amaz.VertexAttribType.POSITION, Amaz.VertexAttribType.TEXCOORD0}
    end
    local attribs = Amaz.Vector()
    for _, loc in ipairs(locations) do
        local descriptor = Amaz.VertexAttribDesc()
        descriptor.semantic = loc
        attribs:pushBack(descriptor)
    end

    local mesh = Amaz.Mesh()
    mesh.vertexAttribs = attribs

    local mesh0 = Amaz.SubMesh()
    mesh0.mesh = mesh
    mesh0.primitive = primitive or Amaz.Primitive.TRIANGLES
    mesh:addSubMesh(mesh0)

    return mesh
end
function Helper.createTexture (w, h)
    local tex = Amaz.Texture2D()
    tex.filterMin = Amaz.FilterMode.LINEAR
    tex.filterMag = Amaz.FilterMode.LINEAR
    if w and h then
        tex.width = w
        tex.height = h
    end
    return tex
end



return Helper