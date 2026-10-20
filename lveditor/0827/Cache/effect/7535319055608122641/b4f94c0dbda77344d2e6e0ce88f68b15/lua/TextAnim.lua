




---------- common/Utils.lua ----------
local Utils = {}



-- System
---@param fmt string
---@vararg any
function Utils.log (fmt, ...)
    ---#ifdef DEV
--//    local args = { ... }
--//    for i, v in ipairs(args) do
--//        local type = type(v)
--//        if type == "table" then
--//            args[i] = cjson.encode(v)
--//        elseif type == "number" then
--//            args[i] = v
--//        else
--//            args[i] = tostring(v)
--//        end
--//    end
--//    if Editor then
--//        Amaz.LOGW("MoreFive", string.format(fmt, unpack(args)))
--//    elseif EffectSdk then
--//        EffectSdk.LOG_LEVEL(8, string.format(fmt, unpack(args)))
--//    end
    ---#endif
end



-- Container
---@param src table
---@return table
function Utils.table_clone (src)
    if not src then
        return src
    end
    local dst = {}
    for k, v in pairs(src) do
        dst[k] = type(v) == "table" and Utils.table_clone(v) or v
    end
    return dst
end
---@param src any[]
---@param si number|nil
---@param ei number|nil
---@return any[]
function Utils.table_slice (src, si, ei)
    si = si or 1
    ei = ei or #src
    local dst = {}
    for i = si, ei do
        table.insert(dst, src[i])
    end
    return dst
end

---@vararg any[]
---@return any[]
function Utils.array_concat(...)
    local dst = {}
    for _, src in ipairs({...}) do
        for _, ele in ipairs(src) do
            table.insert(dst, ele)
        end
    end
    return dst
end
---@param src any[]
---@return any[]
function Utils.array_shuffle (src)
    local dst = {}
    if type(src) == "number" then
        for i = 1, src do
            dst[i] = i
        end
    else
        for i, v in ipairs(src) do
            dst[i] = v
        end
    end
    for n = #dst, 1, -1 do
        local i = math.floor(math.random(n))
        local v = dst[i]
        dst[i] = dst[n]
        dst[n] = v
    end
    return dst
end



-- Math
function Utils.round (value)
    return math.floor(value + 0.5)
end
function Utils.clamp (value, min, max)
    return math.min(math.max(min, value), max)
end
function Utils.mix (x, y, a)
    return x + (y - x) * a
end
function Utils.step (edge0, edge1, value)
    return math.min(math.max(0, (value - edge0) / (edge1 - edge0)), 1)
end
function Utils.fract (value)
    return value - math.floor(value)
end

function Utils.step0 (edge0, edge1, value)
    return (value - edge0) / (edge1 - edge0)
end
function Utils.linearstep (edge0, edge1, value)
    return math.min(math.max(0, (value - edge0) / (edge1 - edge0)), 1)
end
function Utils.smoothstep (edge0, edge1, value)
    local t = math.min(math.max(0, (value - edge0) / (edge1 - edge0)), 1)
    return t * t * (3 - t - t)
end
function Utils.mirror (range, value)
    local round = value / range
    local roundF = 1 - math.abs(round % 2 - 1)
    local roundI = math.floor(round)
    return roundF, roundI
end
function Utils.bezier4 (q, x1, x2, x3, x4, y1, y2, y3, y4, z1, z2, z3, z4)
    local p = 1 - q
    local p2 = p * p
    local p3 = p2 * p
    local q2 = q * q
    local q3 = q2 * q
    local x = x1*p3 + 3*x2*p2*q + 3*x3*p*q2 + x4*q3
    local y = y4 and y1*p3 + 3*y2*p2*q + 3*y3*p*q2 + y4*q3 or 0
    local z = z4 and z1*p3 + 3*z2*p2*q + 3*z3*p*q2 + z4*q3 or 0
    return x, y, z
end
function Utils.bezier4x2y (x1, x2, x3, x4, y1, y2, y3, y4, x)
    local t_ = 0
    local _t = 1
    local bezier4 = Utils.bezier4
    repeat
        local _t_ = (t_ + _t) * 0.5
        local _x_ = bezier4(_t_, x1, x2, x3, x4)
        if _x_ > x then
            _t = _t_
        else
            t_ = _t_
        end
    until _t - t_ < 0.0078125

    local t = (t_ + _t) * 0.5
    return bezier4(t, y1, y2, y3, y4)
end



-- Easing
function Utils.sineIn (t)
    return 1 - math.cos(math.pi * t * .5)
end
function Utils.sineOut (t)
    return math.sin(math.pi * t * .5)
end
function Utils.sineInOut (t)
    return -(math.cos(math.pi * t) - 1) * .5
end
function Utils.quadIn (t)
    return t * t
end
function Utils.quadOut (t)
    return (2 - t) * t
end
function Utils.quadInOut (t)
    return t < .5 and 2 * t * t or t * (4 - t - t) - 1
end
function Utils.cubicIn (t)
    return t * t * t
end
function Utils.cubicOut (t)
    t = 1 - t
    return 1 - t * t * t
end
function Utils.cubicInOut (t)
    if t < .5 then
        return 4 * t * t * t
    else
        t = 2 - t - t
        return 1 - t * t * t * .5
    end
end
function Utils.quartIn (t)
    t = t * t
    return t * t
end
function Utils.quartOut (t)
    t = 1 - t
    t = t * t
    return 1 - t * t
end
function Utils.quartInOut (t)
    if t < .5 then
        t = t * t
        return 8 * t * t
    else
        t = 2 - t - t
        t = t * t
        return 1 - t * t * .5
    end
end
function Utils.expoIn (t)
    return t ~= 0 and math.pow(2, 10 - t - 10) or 0
end
function Utils.expoOut (t)
    return t ~= 1 and 1 - math.pow(2, -10 * t) or 1
end
function Utils.expoInOut (t)
    if t == 0 then
        return 0
    elseif t == 1 then
        return 1
    elseif t < .5 then
        return math.pow(2, 20 * t - 10) * .5
    else
        return 1 - math.pow(2, -20 * t + 10) * .5
    end
end
function Utils.circIn (t) return 1 - math.sqrt(1 - t * t) end
function Utils.circOut (t)
    t = t - 1
    return math.sqrt(1 - t * t)
end
function Utils.circInOut (t)
    if t < .5 then
        return .5 - math.sqrt(1 - 4 * t * t) * .5
    else
        t = 2 - t - t
        return .5 + math.sqrt(1 - t * t) * 0.5
    end
end
function Utils.backIn (t)
    local tt = t * t
    return 2.70158 * tt * t - 1.70158 * tt
end
function Utils.backOut (t)
    t = t - 1
    local tt = t * t
    return 1 + 2.70158 * tt * t + 1.70158 * tt
end
function Utils.backInOut (t)
    if t < .5 then
        t = t + t
        return (t * t * (3.5949095 * t - 2.5949095)) * .5
    else
        t = t + t - 2
        return (t * t * (3.5949095 * t + 2.5949095) + 2) * .5
    end
end
function Utils.elasticIn (t)
    if t == 0 then
        return 0
    elseif t == 1 then
        return 1
    else
        return -math.pow(2, 10 * t - 10) * math.sin((t * 10 - 10.75) * math.pi * 2 / 3)
    end
end
function Utils.elasticOut (t)
    if t == 0 then
        return 0
    elseif t == 1 then
        return 1
    else
        return math.pow(2, -10 * t) * math.sin((t * 10 - .75) * math.pi * 2 / 3) + 1
    end
end
function Utils.elasticInOut (t)
    if t == 0 then
        return 0
    elseif t == 1 then
        return 1
    elseif t < 0.5 then
        return -(math.pow(2, 20 * t - 10) * math.sin((t * 20 - 11.125) * math.pi * 2 / 4.5)) * .5
    else
        return (math.pow(2, -20 * t + 10) * math.sin((t * 20 - 11.125) * math.pi * 2 / 4.5)) * .5 + 1
    end
end
function Utils.bounceIn (t)
    return 1 - Utils.bounceOut(1 - t)
end
function Utils.bounceOut (t)
    local n1 = 7.5625;
    local d1 = 2.75;
    if t < 1 / d1 then
        return n1 * t * t;
    elseif t < 2 / d1 then
        t = t - 1.5 / d1
        return n1 * t * t + .75;
    elseif t < 2.5 / d1 then
        t = t - 2.25 / d1
        return n1 * t * t + .9375;
    else
        t = t - 2.625 / d1
        return n1 * t * t + .984375;
    end
end
function Utils.bounceInOut (t)
    if t < .5 then
        return (1 - Utils.bounceOut(1 - t + t)) * .5
    else
        return (1 + Utils.bounceOut(t + t - 1)) * .5
    end
end



-- Vector Math
---@param screenSize Vector2f|nil
---@param layers table[]
---@return Matrix4x4f
function Utils.uvTransform (screenSize, layers)
    local matrix = Amaz.Matrix4x4f()
    if screenSize then
        local last = layers[#layers]
        matrix:setScale(Amaz.Vector3f(1/last.w, 1/last.h, 1))
    else
        matrix:setIdentity()
    end
    for i = #layers, 1, -1 do
        local layer = layers[i]
        if layer.ax or layer.ay then
            local dx = layer.ax and layer.ax * layer.w or 0
            local dy = layer.ay and layer.ay * layer.h or 0
            matrix:translate(Amaz.Vector3f(dx, dy, 0))
        end
        if layer.r then
            local rotate = Amaz.Matrix4x4f()
            rotate:setTR(Amaz.Vector3f(0, 0, 0), Amaz.Quaternionf.eulerToQuaternion(Amaz.Vector3f(0, 0, -math.rad(layer.r))))
            matrix = matrix * rotate
        end
        if layer.s then
            matrix:scale(Amaz.Vector3f(1/layer.s, 1/layer.s, 1))
        elseif layer.sx or layer.sy then
            matrix:scale(Amaz.Vector3f(1/layer.sx, 1/layer.sy, 1))
        end
        if layer.x or layer.y then
            local x = layer.x and -layer.x or 0
            local y = layer.y and -layer.y or 0
            matrix:translate(Amaz.Vector3f(x, y, 0))
        end
    end
    if screenSize then
        matrix:scale(Amaz.Vector3f(screenSize.x, screenSize.y, 1))
    end
    return matrix
end

function Utils.vtTransform (layers)
    local matrix = Amaz.Matrix4x4f()
    local rotate = Amaz.Matrix4x4f()
    matrix:setIdentity()
    for i = 1, #layers do
        local layer = layers[i]
        local dx = layer.x and layer.x or 0
        local dy = layer.y and layer.y or 0
        local dz = layer.z and layer.z or 0
        matrix:translate(Amaz.Vector3f(dx, dy, dz))
        if layer.rx then
            rotate:setTR(Amaz.Vector3f(0, 0, 0), Amaz.Quaternionf.eulerToQuaternion(Amaz.Vector3f(math.rad(layer.rx), 0, 0)))
            matrix = matrix * rotate
        end
        if layer.ry then
            rotate:setTR(Amaz.Vector3f(0, 0, 0), Amaz.Quaternionf.eulerToQuaternion(Amaz.Vector3f(0, -math.rad(layer.ry), 0)))
            matrix = matrix * rotate
        end
        if layer.rz then
            rotate:setTR(Amaz.Vector3f(0, 0, 0), Amaz.Quaternionf.eulerToQuaternion(Amaz.Vector3f(0, 0, -math.rad(layer.rz))))
            matrix = matrix * rotate
        end
        local sx = layer.sx or layer.s or 1
        local sy = layer.sy or layer.s or 1
        local sz = layer.sz or layer.s or 1
        matrix:scale(Amaz.Vector3f(sx, sy, sz))

        local wx = layer.wx or layer.w or 0
        local wy = layer.wy or layer.w or 0
        local wz = layer.wz or layer.w or 0
        local ax = layer.ax or layer.a or 0.5
        local ay = layer.ay or layer.a or 0.5
        local az = layer.az or layer.a or 0.5
        matrix:translate(Amaz.Vector3f(-wx*ax, -wy*ay, -wz*az))
    end
    return matrix
end


-- Convert
---@param arr number[]
---@param si number
---@param ei number
---@return number|Vector2f|Vector3f|Vector4f
function Utils.arr2vec (arr, si, ei)
    si = si or 1
    ei = ei or #arr
    if si == ei then
        return arr[si]
    end
    local n = ei - si + 1
    if n == 3 then
        return Amaz.Vector3f(arr[si], arr[si + 1], arr[si + 2])
    elseif n == 2 then
        return Amaz.Vector2f(arr[si], arr[si + 1])
    elseif n == 4 then
        return Amaz.Vector4f(arr[si], arr[si + 1], arr[si + 2], arr[si + 3])
    end
end
function Utils.rgb2hsl (R, G, B)
    B = B or R.z
    G = G or R.y
    R = B and R or R.x
    local H, S, L;
    local max = math.max(R, G, B);
    local min = math.min(R, G, B);
    local delta = max - min

    L = (max + min) * 0.5
    S = delta == 0 and 0 or 1 - math.abs(L + L - 1)

    if delta == 0 then
        H = 0
    elseif max == R then
        H = (G - B) / delta % 6
    elseif max == G then
        H = (B - R) / delta + 2
    else
        H = (R - G) / delta + 4
    end
    H = H / 6

    if B then
        return H, S, L
    else
        return Amaz.Vector3f(H, S, L)
    end
end
function Utils.hsl2rgb (H, S, L)
    L = L or H.z
    S = S or H.y
    H = L and H or H.x
    H = H * 360
    local R, G, B
    local C = (1 - math.abs(L + L - 1)) * S
    local X = C * (1 - math.abs((H / 60) % 2 - 1))
    local m = L - C * 0.5

    if H < 60 then
        R, G, B = C, X, 0
    elseif H < 120 then
        R, G, B = X, C, 0
    elseif H < 180 then
        R, G, B = 0, C, X
    elseif H < 240 then
        R, G, B = 0, X, C
    elseif H < 300 then
        R, G, B = X, 0, C
    else
        R, G, B = C, 0, X
    end

    R = R + m
    G = G + m
    B = B + m

    if L then
        return R, G, B
    else
        return Amaz.Vector3f(R, G, B)
    end
end
function Utils.hueShift (R, G, B, shift)
    local H, S, L = Utils.rgb2hsl(R, G, B)
    H = H + shift
    R, G, B = Utils.hsl2rgb(H, S, L)
    return Amaz.Vector3f(R, G, B)
end



---@param range number
---@param step number
---@return number, number
function Utils.solveSamples (range, step)
    local samples = math.ceil(range / step)
    step = range / (samples + 0.1)
    return samples, step
end



-- String
function Utils.startswith (str, prefix)
    local n0 = string.len(str)
    local n1 = string.len(prefix)
    return n0 >= n1 and string.sub(str, 1, n1) == prefix
end
function Utils.endswith (str, suffix)
    local n0 = string.len(str)
    local n1 = string.len(suffix)
    return n0 > n1 and string.sub(str, n0 - n1 + 1, n0) == suffix
end
function Utils.trim (str, mode)
    local si = 1
    local ei = #str
    mode = mode or 0
    if mode >= 0 then
        while si <= ei and string.byte(str, si) <= 32 do
            si = si + 1
        end
    end
    if mode <= 0 then
        while ei > si and string.byte(str, ei) <= 32 do
            ei = ei - 1
        end
    end
    if si ~= 1 or ei ~= #str then
        return string.sub(str, si, ei)
    else
        return str
    end
end
function Utils.rfind (str, pattern)
    local rStr = str:reverse()
    local rPattern = pattern:reverse()
    local rIdx = string.find(rStr, rPattern, 1, true)
    if not rIdx then return nil end
    local sIdx = #str - (rIdx + #pattern - 1) + 1
    local eIdx = #str - rIdx + 1
    return sIdx, eIdx
end



-- UTF-8
---@param lead number
---@return number
function Utils.ucs4_size (lead)
    if lead < 128 then
        return 1
    elseif lead < 192 then
        return 0
    elseif lead < 224 then
        return 2
    elseif lead < 240 then
        return 3
    elseif lead < 248 then
        return 4
    elseif lead < 252 then
        return 5
    else
        return 6
    end
end
---@param str string
---@return number
function Utils.utf8_len (str)
    local n = #str
    local i = 1
    local l = 0
    while i <= n do
        local bytes = Utils.ucs4_size(string.byte(str, i))
        if bytes > 0 then
            i = i + bytes
            l = l + 1
        else
            i = i + 1
        end
    end
    return l
end
---@param str string
---@param si number|nil
---@param ei number|nil
---@return string
function Utils.utf8_sub (str, si, ei)
    local n = #str
    si = si or 1
    ei = ei or n
    ei = ei - si
    local i = 1
    while i <= n and si > 1 do
        local bytes = Utils.ucs4_size(string.byte(str, i))
        i = i + (bytes > 0 and bytes or 1)
        si = si - 1
    end
    local j = i
    while j <= n and ei > 0 do
        local bytes = Utils.ucs4_size(string.byte(str, j))
        j = j + (bytes > 0 and bytes or 1)
        ei = ei - 1
    end
    return string.sub(str, i, j)
end
---@param str string
---@param cb fun(str: string, index: number, size: number): boolean
function Utils.utf8_for (str, cb)
    local n = #str
    local i = 1
    while i <= n do
        local code = string.byte(str, i)
        local bytes = Utils.ucs4_size(code)
        if bytes > 0 then
            if cb(i, bytes, code) then
                return
            end
            i = i + bytes
        else
            i = i + 1
        end
    end
end



-- Size Fit
---@param dstSize Vector2f
---@param srcSizeOrAspect Vector2f|number
---@return Vector2f
function Utils.sizeFill (dstSize, srcSizeOrAspect)
    return dstSize:copy()
end
---@param dstSize Vector2f
---@param srcSizeOrAspect Vector2f|number
---@return Vector2f
function Utils.sizeFitX (dstSize, srcSizeOrAspect)
    srcSizeOrAspect = type(srcSizeOrAspect) == "number" and srcSizeOrAspect or srcSizeOrAspect.x / srcSizeOrAspect.y
    return Amaz.Vector2f(dstSize.x, dstSize.x / srcSizeOrAspect)
end
---@param dstSize Vector2f
---@param srcSizeOrAspect Vector2f|number
---@return Vector2f
function Utils.sizeFitY (dstSize, srcSizeOrAspect)
    srcSizeOrAspect = type(srcSizeOrAspect) == "number" and srcSizeOrAspect or srcSizeOrAspect.x / srcSizeOrAspect.y
    return Amaz.Vector2f(dstSize.y * srcSizeOrAspect, dstSize.y)
end
---@param dstSize Vector2f
---@param srcSizeOrAspect Vector2f|number
---@return Vector2f
function Utils.sizeContains (dstSize, srcSizeOrAspect)
    srcSizeOrAspect = type(srcSizeOrAspect) == "number" and srcSizeOrAspect or srcSizeOrAspect.x / srcSizeOrAspect.y
    return Amaz.Vector2f(math.min(dstSize.x, dstSize.y * srcSizeOrAspect), math.min(dstSize.x / srcSizeOrAspect, dstSize.y))
end
---@param dstSize Vector2f
---@param srcSizeOrAspect Vector2f|number
---@return Vector2f
function Utils.sizeCover (dstSize, srcSizeOrAspect)
    srcSizeOrAspect = type(srcSizeOrAspect) == "number" and srcSizeOrAspect or srcSizeOrAspect.x / srcSizeOrAspect.y
    return Amaz.Vector2f(math.max(dstSize.x, dstSize.y * srcSizeOrAspect), math.max(dstSize.x / srcSizeOrAspect, dstSize.y))
end








---------- info_sticker/Helper.lua ----------

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








---------- common/AEAdapter.lua ----------


---@class AEAdapter
local AEAdapter = {}
AEAdapter.__index = AEAdapter


function AEAdapter:new ()
    local self = setmetatable({}, AEAdapter)
    self._tracks = {}
    return self
end


---@param layerName string
---@param layerData table<string, table>
function AEAdapter:addKeyframes (layerName, layerData)
    for trackName, trackData in pairs(layerData) do
        local trackPath = layerName.."/"..trackName
        trackData.from_ae = true
        self._tracks[trackPath] = trackData
        if trackData.v then
            AEAdapter._solveBezierLength(trackData)
        end
    end
end


---@param layerName string
---@param layerData table
function AEAdapter:addFrames (layerName, layerData)
    local fps = layerData.frameRate
    layerData = layerData.layer0
    for attrName, attrData in pairs(layerData) do
        if type(attrData) == "table" then
            local trackPath = layerName.."."..attrName
            attrData.fps = fps
            self._tracks[trackPath] = attrData
        end
    end
end


---@param layerName string
---@param layerData table
function AEAdapter:addAnimations (layerName, layerData)
    for attrName, attrData in pairs(layerData) do
        local trackPath = layerName..":"..attrName
        self._tracks[trackPath] = attrData
    end
end


---@param path string
---@param time number
---@return number|number[]
function AEAdapter:get (path, time)
    local track = self._tracks[path]
    if not track then
        return
    end

    if track.fps then
        return self:_solveFrame(track, time * track.fps)
    elseif track.from_ae then
        return self:_solveKeyframe(track, time)
    else
        return self:_solveAnimation(track, time)
    end
end
---@param path string
---@param time number
---@param delta number|nil
---@return number|number[]
function AEAdapter:getVelocity (path, time, delta)
    delta = delta or 0.001
    local v0 = self:get(path, time - delta)
    local v1 = self:get(path, time + delta)
    delta = delta + delta
    local v = {}
    for i = 1, #v0 do
        v[i] = (v1[i] - v0[i]) / delta
    end
    return v
end


function AEAdapter:_solveFrame (track, frame)
    if #track > 0 then
        local v = self._interpolateFrame(track, frame)
        return type(v) == "number" and {v} or v
    else
        local x = self._interpolateFrame(track.x, frame)
        local y = self._interpolateFrame(track.y, frame)
        return {x, y}
    end
end
function AEAdapter._interpolateFrame (array, frame)
    local n = #array
    local t, k0, k1

    if frame <= 0 then
        k0 = array[1]
        k1 = array[1]
        t = 0
    elseif frame >= n - 1 then
        k0 = array[n]
        k1 = array[n]
        t = 0
    else
        local i = math.floor(frame)
        k0 = array[i + 1]
        k1 = array[i + 2]
        t = frame - i
    end

    if type(k0) == "table" then
        local x = k0.x + (k1.x - k0.x) * t
        local y = k0.y + (k1.y - k0.y) * t
        if k0.z then
            local z = k0.z + (k1.z - k0.z) * t
            return {x, y, z}
        else
            return {x, y}
        end
    else
        return k0 + (k1 - k0) * t
    end
end


function AEAdapter:_solveKeyframe (track, time)
    local K = track.k
    local N = #K
    if time <= K[1][1] then
        return Utils.table_slice(K[1], 2)
    elseif time >= K[N][1] then
        return Utils.table_slice(K[N], 2)
    end

    local interpolator = track.v and self._interpolateVector or self._interpolateScalar
    for i = 2, N do
        if time < K[i][1] then
            return interpolator(K[i - 1], K[i], track.s[i - 1], time)
        end
    end
    return Utils.table_slice(K[N], 2)
end
function AEAdapter._interpolateScalar (K0, K1, S, T)
    if S.h then
        return Utils.table_slice(K0, 2)
    end
    local C = S.c
    local y = {}
    for i = 1, #K0 - 1 do
        local x1 = K0[1]
        local x2 = C[i][1]
        local x3 = C[i][3]
        local x4 = K1[1]
        local y1 = K0[i + 1]
        local y2 = C[i][2]
        local y3 = C[i][4]
        local y4 = K1[i + 1]
        y[i] = Utils.bezier4x2y(x1, x2, x3, x4, y1, y2, y3, y4, T)
    end
    return y
end
function AEAdapter._interpolateVector (K0, K1, S, T)
    if S.h then
        return Utils.table_slice(K0, 2)
    end

    local C = S.c
    local P1 = S.p1
    local P2 = S.p2
    local L = S.l

    local t1 = K0[1]
    local t2 = C[1]
    local t3 = C[3]
    local t4 = K1[1]
    local l1 = 0
    local l2 = C[2]
    local l3 = C[4]
    local l4 = L[#L]
    local l = Utils.bezier4x2y(t1, t2, t3, t4, l1, l2, l3, l4, T)
    l = math.min(l, L[#L])

    local si = 1
    local ei = #L
    while si < ei - 1 do
        local i = math.floor((si + ei) * 0.5)
        if l < L[i] then
            ei = i
        else
            si = i
        end
    end

    local t = Utils.step(L[si], L[ei], l)
    t = ((si-1) + t) / (#L-1)

    local x1 = K0[2]
    local x2 = P1[1]
    local x3 = P2[1]
    local x4 = K1[2]
    local y1 = K0[3]
    local y2 = P1[2]
    local y3 = P2[2]
    local y4 = K1[3]
    local z1, z2, z3, z4
    if K0[4] then
        z1 = K0[4]
        z2 = P1[3]
        z3 = P2[3]
        z4 = K1[4]
    end
    local x, y, z = Utils.bezier4(t, x1, x2, x3, x4, y1, y2, y3, y4, z1, z2, z3, z4)
    return {x, y, z}
end


function AEAdapter:_solveAnimation (track, time)
    local N = #track
    if time <= track[1][1] then
        return track[1][3]
    elseif time >= track[N][2] then
        return track[N][4]
    end

    for i = 1, N do
        local frag = track[i]
        if time <= frag[2] then
            local x = Utils.step(frag[1], frag[2], time)
            local m = frag[5]
            local y = type(m) == "function" and m(x) or Utils.bezier4x2y(0, m[1], m[3], 1, 0, m[2], m[4], 1, x)
            local v0 = frag[3]
            local v1 = frag[4]
            local v = {}
            for j = 1, #v0 do
                v[j] = Utils.mix(v0[j], v1[j], y)
            end
            return v
        end
    end
    return track[N][4]
end

function AEAdapter._splitBezierCurve (segments, K0, P1, P2, K3)
    local L = {0}
    local x0, y0, z0 = K0[2], K0[3], K0[4]
    for i = 1, segments do
        local t = i / segments
        local x1, y1, z1 = Utils.bezier4(t, K0[2], P1[1], P2[1], K3[2], K0[3], P1[2], P2[2], K3[3], K0[4], P1[3], P2[3], K3[4])
        local dx = x1 - x0
        local dy = y1 - y0
        local dz = z0 and z1 - z0 or 0
        local dL = math.sqrt(dx*dx + dy*dy + dz*dz)
        table.insert(L, L[#L] + dL)
        x0, y0, z0 = x1, y1, z1
    end
    return L
end
function AEAdapter._solveBezierLength (track)
    for i, s in ipairs(track.s) do
        if not s.h then
            local k0 = track.k[i]
            local k3 = track.k[i+1]
            s.l = AEAdapter._splitBezierCurve(128, k0, s.p1, s.p2, k3)
        end
    end
end







---------- AE.lua ----------
local AE = {
    ["Offset"] = {
        v = true,
        d = 3,
        k = {
            {0.76666666666667, 448, 252, 0},
            {0.93333333333333, 526, 273, 0},
            {1.03333333333333, 448, 252, 0},
            {1.23333333333333, 412.5, 242.75, 0},
            {1.36666666666667, 434.5, 248.25, 0},
            {1.46666666666667, 480.5, 261.75, 0},
            {1.56666666666667, 458.5, 255.75, 0},
            {1.63333333333333, 448, 252, 0}
        },
        s = {
            {h = true},
            {h = true},
            {h = true},
            {h = true},
            {h = true},
            {h = true},
            {h = true}
        }
    },
    ["DeepGlow_Exposure"] = {
        d = 1,
        k = {
            {1.03333333333333, 1},
            {2.4, 0}
        },
        s = {
            {c = {{1.26111111111567, 0.83333333333, 2.17222222221767, 0.16666666667}}}
        }
    }
}






---------- TextMain.lua ----------


local TextMain = {}
TextMain.__index = TextMain
function TextMain.new (env)
    local self = setmetatable({}, TextMain)
    self.MASK = {
        ax = 0.5,
        ay = 0.55,
        ws = 1.2,
        hs = 4.0,
        images = {},
    }
    self.ANIM = {
        ax = 0.5,
        ay = 0.445,
        hs = function (lineCount, textHeight) return Utils.mix(3.0, 1.5, Utils.step(1, 4, lineCount)) * textHeight end,
        asp = 960 / 300,
        images = {},
    }
    table.insert(self.MASK.images, "image/mask/mask_72.png")
    for i = 0, 71 do
        table.insert(self.MASK.images, string.format("image/mask/mask_%02d.png", i))
        table.insert(self.ANIM.images, string.format("image/anim/anim_%02d.png", i))
    end
    self.AE = AE
    return self
end


function TextMain:onCreate (env)
    ---#ifdef DEV
--//    env.text.activeTextStyle.fontSize = 30
--//    env.text.typeSettingParam.typeSettingKind = Amaz.TypeSettingKind.HORIZONTAL
--//    env.text.str = "first line"
--//    --env.text.str = "first line\nsecond line"
--//    --env.text.str = "first line\nsecond line\nthrid line"
    ---#endif
    self.ae = AEAdapter.new()
    self.ae:addKeyframes("", self.AE)
    self.offset0 = self.ae:get("/Offset", 0)
    self.offset0 = Amaz.Vector2f(self.offset0[1], self.offset0[2])
end

function TextMain:onDestroy (env)

end

function TextMain:onShow (env)
    local style0 = Helper.getStyle0(env.text)
    self.lineCount = self:getLineCount(env)
    self.vertical = env.text.typeSettingParam.typeSettingKind == Amaz.TypeSettingKind.VERTICAL
    self.fontSize = style0 and style0.fontSize or 15

    --- >>>
    local canvasRect0 = env.text:getRectExpanded()
    local canvasSize0 = Amaz.Vector2f(canvasRect0.width, canvasRect0.height)
    Amaz.LOGE("canvasSize0", string.format("[%.3f %.3f]", canvasSize0.x, canvasSize0.y))
    Amaz.LOGE("canvasBase0", string.format("[%.3f %.3f]", canvasRect0.x, canvasRect0.y))
    local expandSize = self.solveExpandSize(self.vertical, self.lineCount, self.fontSize, canvasSize0)--Amaz.Vector2f(canvasSize0.y, canvasSize0.y * 2)
    --- <<<

    local contourCtx = Helper.cloneText(env, "contour")
    local spritesCtx = Helper.cloneText(env, "sprites")
    self.contourCtx = contourCtx
    self.spritesCtx = spritesCtx

    --- >>>
    contourCtx.text.canvas.targetRTExtraSize = expandSize
    spritesCtx.text.canvas.targetRTExtraSize = expandSize
    --- <<<

    contourCtx.material = Helper.setMaterialToText(contourCtx.text, contourCtx.renderer, env.contour)
    spritesCtx.material = Helper.setMaterialToText(spritesCtx.text, spritesCtx.renderer, env.sprites)

    --- >>>
    self.canvasSize = Amaz.Vector2f(canvasSize0.x + expandSize.x, canvasSize0.y + expandSize.y)
    local extraSize = contourCtx.text:getRectExpanded()
    Amaz.LOGE("extraSize.x", extraSize.x)
    Amaz.LOGE("extraSize.y", extraSize.y)
    --self.canvasBase = Amaz.Vector2f(canvasRect0.x, canvasRect0.y)
    self.canvasBase = Amaz.Vector2f(canvasRect0.x - expandSize.x * 0.5, canvasRect0.y - expandSize.y * 0.5)
    contourCtx.material:setVec2("u_size", self.canvasSize)
    --- <<<

    local rtText0 = contourCtx.material.xshader.passes:get(0).renderTexture
    rtText0.width = self.canvasSize.x * 1.5
    rtText0.height = self.canvasSize.y * 1.5
    self.rtText0 = rtText0
    local rtText1 = contourCtx.material.xshader.passes:get(1).renderTexture
    rtText1.width = self.canvasSize.x * 1.5
    rtText1.height = self.canvasSize.y * 1.5
    self.rtText1 = rtText1

    Helper.flushStyle(contourCtx.text)
    Helper.flushStyle(spritesCtx.text)

    self.lines = self:splitLines(contourCtx, spritesCtx)
    self:initDeepGlow(spritesCtx.material, 2, self.canvasSize.x, self.canvasSize.y)

    if not self.vertical then
        self:solveWordMotion(self.lines)
        self:solveLayoutH(self.lines)
    else
        self:solveWordMotion(self.lines)
        self:solveLayoutV(self.lines)
    end
    env.text.canvas.renderToRT = true
end

function TextMain:onHide (env)
    Helper.removeNode(self.contourCtx)
    Helper.removeNode(self.spritesCtx)
    env.text.canvas.renderToRT = false
    collectgarbage("collect")
end

function TextMain:onUpdate (env, elapsed)
    if not self.lines then return end

    local progress = 1 - (elapsed / env.duration)*1.05
    progress = math.min(1, math.max(0, progress))
    self:updateSeqAnim(env, progress)
    self:updateWordMotion(progress)
    self:updateDeepGlow(progress)

    if not self.vertical then
        self:updateUVOffsetH(progress)
    else
        self:updateUVOffsetV(progress)
    end

    --local mainTex = self.contourCtx.material:getTex("_MainTex")
    --if mainTex then
    --    Amaz.LOGE("_MainTex.w", mainTex.width)
    --    Amaz.LOGE("_MainTex.h", mainTex.height)
    --end
    self.spritesCtx.material:setFloat("u_textVisible", progress > 0.8 and 1 or 0)
end

function TextMain:onDurationChanged (env, duration)
end

function TextMain:getLineCount (ctx)
    local letters = ctx.text.letters
    local N = letters:size()
    if N == 0 then return 0 end

    local id = letters:get(0).rowth
    local count = 1
    for i = 0, N - 1 do
        local letter = letters:get(i)
        if letter.rowth ~= id then
            count = count + 1
        end
    end
    return count
end

function TextMain:splitLines ()
    self.contourCtx.text:forceTypeSetting()
    self.spritesCtx.text:forceTypeSetting()

    local lettersContour = self.contourCtx.text.letters
    local lettersSprites = self.spritesCtx.text.letters
    local N = lettersContour:size()
    if N == 0 then return end

    self.rects = {}
    local lines = { x = 0, y = 0, X = 0, Y = 0 }
    local line = { id = lettersContour:get(0).rowth }
    local word = nil
    for i = 0, N - 1 do
        local letterContour = lettersContour:get(i)
        local letterSprites = lettersSprites:get(i)
        if letterContour.rowth ~= line.id and #line > 0 then
            table.insert(lines, line)
            line = { id = letterContour.rowth }
        end

        local letterRect = Helper.getTightRect(letterContour, 0.2)
        local leadingByte = string.byte(letterContour.utf8)
        if leadingByte > 32 then
            --- >>>
            local x0 = letterContour.initialPosition.x - 0.25 * (letterRect.width - letterContour.rect.width)
            local y0 = letterContour.initialPosition.y
            local hw = letterRect.width * 0.5
            local hh = letterRect.height * 0.5
            lines.x = math.min(x0 - hw, lines.x)
            lines.y = math.min(y0 - hh, lines.y)
            lines.X = math.max(x0 + hw, lines.X)
            lines.Y = math.max(y0 + hh, lines.Y)
            table.insert(self.rects, Amaz.Vector4f(
                    (x0 - hw - self.canvasBase.x) / self.canvasSize.x,
                    (y0 - hh - self.canvasBase.y) / self.canvasSize.y,
                    (x0 + hw - self.canvasBase.x) / self.canvasSize.x,
                    (y0 + hh - self.canvasBase.y) / self.canvasSize.y)
            )
            --- <<<

            if leadingByte < 128 then -- ascii char
                word = word or {
                    contour = {},
                    sprites = {},
                }
                table.insert(word.contour, letterContour)
                table.insert(word.sprites, letterSprites)
            else
                if word then -- last ascii word
                    table.insert(line, word)
                end
                -- one char one word
                table.insert(line, {
                    contour = {letterContour},
                    sprites = {letterSprites},
                })
                word = nil
            end

        elseif word then -- invisible char
            table.insert(line, word)
            word = nil
        end
    end
    self.spritesCtx.material:setVec4("u_rect", self.rects[3])
    if word then
        table.insert(line, word)
    end
    if #line > 0 then
        table.insert(lines, line)
    end
    return lines
end

function TextMain:solveWordMotion (lines)
    for _, line in ipairs(lines) do
        local wordCount = #line
        if wordCount >= 3 then
            local sideWords = math.floor(wordCount / 3)
            local middleWords = wordCount - sideWords * 2
            for i = sideWords + 1, sideWords + middleWords do
                line[i].delay = true
            end
        end
    end
end

function TextMain:solveLayoutH (lines)
    local textW = lines.X - lines.x
    local textH = lines.Y - lines.y
    local maskW = textW * self.MASK.ws
    local maskH = textH * self.MASK.hs
    local animH = self.ANIM.hs(#self.lines, textH)
    local animW = animH * self.ANIM.asp
    local textX = (lines.x + lines.X) * 0.5 - self.canvasBase.x
    local textY = (lines.y + lines.Y) * 0.5 - self.canvasBase.y
    self.spritesCtx.material:setMat4("u_matMask", Utils.uvTransform(self.canvasSize, {
        {
            x = textX,
            y = textY,
            w = maskW,
            h = maskH,
            ax = self.MASK.ax,
            ay = self.MASK.ay,
        }
    }))
    self.spritesCtx.material:setMat4("u_matAnim", Utils.uvTransform(self.canvasSize, {
        {
            x = textX,
            y = textY,
            w = animW,
            h = animH,
            ax = self.ANIM.ax,
            ay = self.ANIM.ay,
        }
    }))
end

function TextMain:solveLayoutV (lines)
    local textW = lines.X - lines.x
    local textH = lines.Y - lines.y
    local maskW = textH * self.MASK.ws
    local maskH = textW * self.MASK.hs
    local animH = self.ANIM.hs(self.lineCount, textW)
    local animW = animH * self.ANIM.asp
    local textX = (lines.x + lines.X) * 0.5 - self.canvasBase.x
    local textY = (lines.y + lines.Y) * 0.5 - self.canvasBase.y
    self.spritesCtx.material:setMat4("u_matMask", Utils.uvTransform(self.canvasSize, {
        {
            x = textX,
            y = textY,
            w = maskW,
            h = maskH,
            ax = self.MASK.ax,
            ay = self.MASK.ay,
            r = 90,
        }
    }))
    self.spritesCtx.material:setMat4("u_matAnim", Utils.uvTransform(self.canvasSize, {
        {
            x = textX,
            y = textY,
            w = animW,
            h = animH,
            ax = self.ANIM.ax,
            ay = self.ANIM.ay,
            r = 90,
        }
    }))
end

function TextMain:updateSeqAnim (env, progress)
    local maskIdx = Utils.clamp(math.floor(progress * #self.MASK.images + 0.5), 0, 72) + 1
    local animIdx = Utils.clamp(math.floor(progress * #self.ANIM.images + 0.5), 0, 71) + 1
    local maskTex = env.scene.assetMgr:SyncLoad(env.path..self.MASK.images[maskIdx])
    local animTex = env.scene.assetMgr:SyncLoad(env.path..self.ANIM.images[animIdx])
    self.spritesCtx.material:setTex("_MaskTex", maskTex)
    self.spritesCtx.material:setTex("_AnimTex", animTex)
end

function TextMain:updateWordMotion (progress)
    local generalRange = {s = 0.50, e = 0.72}
    local delayedRange = {s = 0.65, e = 0.87}
    local rdx, rdy = self.wiggle(progress)
    local intensity = Utils.step(1.0, 0.8, progress)
    rdx = rdx * self.fontSize * 0.2 * intensity
    rdy = rdy * self.fontSize * 0.2 * intensity
    self.contourCtx.material:setVec2("u_offset", Amaz.Vector2f(rdx * 0.35, rdy * 0.35))

    for _, line in ipairs(self.lines) do
        for _, word in ipairs(line) do
            local range = word.delay and delayedRange or generalRange
            local progress0 = Utils.step(range.s, range.e, progress)
            progress0 = Utils.quadOut(progress0)
            local dy = Utils.mix(self.fontSize * 3.5, 0, progress0)
            for _, char in ipairs(word.contour) do
                local p0 = char.initialPosition
                char.position = Amaz.Vector2f(p0.x, p0.y + dy)
            end
            for _, char in ipairs(word.sprites) do
                local p0 = char.initialPosition
                char.position = Amaz.Vector2f(p0.x + rdx, p0.y + dy + rdy)
            end
        end
    end
end

function TextMain:updateUVOffsetH (progress)
    local offset0 = self.offset0
    local offset = self.ae:get("/Offset", progress * 72/30)
    offset = Amaz.Vector2f(offset[1], offset[2])
    local cp = offset0.x * offset.y - offset0.y * offset.x
    if cp ~= 0 then
        cp = -cp / math.abs(cp)
    end
    local distance = Amaz.Vector2f.distance(offset, offset0)
    self.spritesCtx.material:setFloat("u_offsetLength", cp * distance / 1000)

    local x = (self.lines.x - self.canvasBase.x) / self.canvasSize.x
    local y = (self.lines.y - self.canvasBase.y) / self.canvasSize.y
    local X = (self.lines.X - self.canvasBase.x) / self.canvasSize.x
    local Y = (self.lines.Y - self.canvasBase.y) / self.canvasSize.y
    self.spritesCtx.material:setVec2("u_offsetPoint0", Amaz.Vector2f(Utils.mix(x, X, 0.30), Utils.mix(y, Y, 0.00)))
    self.spritesCtx.material:setVec2("u_offsetPoint1", Amaz.Vector2f(Utils.mix(x, X, 1.00), Utils.mix(y, Y, 0.85)))
    self.spritesCtx.material:setFloat("u_offsetRadius", self.fontSize / (self.canvasSize.x + self.canvasSize.y) * 2.2)
end

function TextMain:updateUVOffsetV (progress)
    local offset0 = self.offset0
    local offset = self.ae:get("/Offset", progress * 72/30)
    offset = Amaz.Vector2f(offset[1], offset[2])
    local cp = offset0.x * offset.y - offset0.y * offset.x
    if cp ~= 0 then
        cp = -cp / math.abs(cp)
    end
    local distance = Amaz.Vector2f.distance(offset, offset0)
    self.spritesCtx.material:setFloat("u_offsetLength", cp * distance / 1000)

    local x = (self.lines.x - self.canvasBase.x) / self.canvasSize.x
    local y = (self.lines.y - self.canvasBase.y) / self.canvasSize.y
    local X = (self.lines.X - self.canvasBase.x) / self.canvasSize.x
    local Y = (self.lines.Y - self.canvasBase.y) / self.canvasSize.y
    self.spritesCtx.material:setVec2("u_offsetPoint0", Amaz.Vector2f(Utils.mix(x, X, 1.00), Utils.mix(y, Y, 0.30)))
    self.spritesCtx.material:setVec2("u_offsetPoint1", Amaz.Vector2f(Utils.mix(x, X, 0.15), Utils.mix(y, Y, 1.00)))
    self.spritesCtx.material:setFloat("u_offsetRadius", self.fontSize / (self.canvasSize.x + self.canvasSize.y) * 2.2)
end

function TextMain:initDeepGlow (material, passStartIndex, w, h)
    local passes = material.xshader.passes
    local midRT = passes:get(passStartIndex).renderTexture
    midRT.width = w
    midRT.height = h
    local midRT_1 = passes:get(passStartIndex+1).renderTexture
    midRT_1.width = w
    midRT_1.height = h
    w = w * 0.4
    h = h * 0.4
    local midRT_2 = passes:get(passStartIndex+3).renderTexture
    midRT_2.width = w
    midRT_2.height = h
    local midRT_3 = passes:get(passStartIndex+5).renderTexture
    midRT_3.width = w
    midRT_3.height = h
    local midRT_4 = passes:get(passStartIndex+7).renderTexture
    midRT_4.width = w
    midRT_4.height = h
    local midRT_5 = passes:get(passStartIndex+9).renderTexture
    midRT_5.width = w
    midRT_5.height = h
    local midRT_6 = passes:get(passStartIndex+11).renderTexture
    midRT_6.width = w
    midRT_6.height = h
    local midRT_7 = passes:get(passStartIndex+13).renderTexture
    midRT_7.width = w
    midRT_7.height = h

end

function TextMain:stepDeepGlow (material, w, h, radius, intensity, alpha)
    local RadiusOverSigma = 2.5
    local downSample1 = 1
    local downSample2 = 1
    local downSample3 = math.max(math.min(radius / 128, 2.0), 1.0)
    local downSample4 = math.max(math.min(radius / 64, 4.0), 1.0)
    local downSample5 = math.max(radius / 32, 1.0)
    local downSample6 = math.max(radius / 16, 1.0)
    local downSample7 = math.max(radius / 8, 1.0)
    local sample1 = radius / 16 * 0.125
    local sample2 = radius / 16 * 0.25
    local sample3 = radius / 16 / downSample3 * 0.5
    local sample4 = radius / 16 / downSample4
    local sample5 = radius / downSample5 * 0.125
    local sample6 = radius / downSample6 * 0.25
    local sample7 = radius / downSample7 * 0.5

    local dx1 = downSample1 / w
    local dy1 = downSample1 / h
    local dx2 = downSample2 / w
    local dy2 = downSample2 / h
    local dx3 = downSample3 / w
    local dy3 = downSample3 / h
    local dx4 = downSample4 / w
    local dy4 = downSample4 / h
    local dx5 = downSample5 / w
    local dy5 = downSample5 / h
    local dx6 = downSample6 / w
    local dy6 = downSample6 / h
    local dx7 = downSample7 / w
    local dy7 = downSample7 / h

    material:setFloat("u_sampleX1", sample1)
    material:setFloat("u_sampleY1", sample1)
    material:setFloat("u_sampleX2", sample2)
    material:setFloat("u_sampleY2", sample2)
    material:setFloat("u_sampleX3", sample3)
    material:setFloat("u_sampleY3", sample3)
    material:setFloat("u_sampleX4", sample4)
    material:setFloat("u_sampleY4", sample4)
    material:setFloat("u_sampleX5", sample5)
    material:setFloat("u_sampleY5", sample5)
    material:setFloat("u_sampleX6", sample6)
    material:setFloat("u_sampleY6", sample6)
    material:setFloat("u_sampleX7", sample7)
    material:setFloat("u_sampleY7", sample7)

    material:setFloat("u_dx1", dx1)
    material:setFloat("u_dy1", dy1)
    material:setFloat("u_dx2", dx2)
    material:setFloat("u_dy2", dy2)
    material:setFloat("u_dx3", dx3)
    material:setFloat("u_dy3", dy3)
    material:setFloat("u_dx4", dx4)
    material:setFloat("u_dy4", dy4)
    material:setFloat("u_dx5", dx5)
    material:setFloat("u_dy5", dy5)
    material:setFloat("u_dx6", dx6)
    material:setFloat("u_dy6", dy6)
    material:setFloat("u_dx7", dx7)
    material:setFloat("u_dy7", dy7)

    local sigmaX1 = dx1 * sample1 / RadiusOverSigma
    local sigmaY1 = dy1 * sample1 / RadiusOverSigma
    local sigmaX2 = dx2 * sample2 / RadiusOverSigma
    local sigmaY2 = dy2 * sample2 / RadiusOverSigma
    local sigmaX3 = dx3 * sample3 / RadiusOverSigma
    local sigmaY3 = dy3 * sample3 / RadiusOverSigma
    local sigmaX4 = dx4 * sample4 / RadiusOverSigma
    local sigmaY4 = dy4 * sample4 / RadiusOverSigma
    local sigmaX5 = dx5 * sample5 / RadiusOverSigma
    local sigmaY5 = dy5 * sample5 / RadiusOverSigma
    local sigmaX6 = dx6 * sample6 / RadiusOverSigma
    local sigmaY6 = dy6 * sample6 / RadiusOverSigma
    local sigmaX7 = dx7 * sample7 / RadiusOverSigma
    local sigmaY7 = dy7 * sample7 / RadiusOverSigma

    material:setFloat("u_sigmaX1", sigmaX1)
    material:setFloat("u_sigmaY1", sigmaY1)
    material:setFloat("u_sigmaX2", sigmaX2)
    material:setFloat("u_sigmaY2", sigmaY2)
    material:setFloat("u_sigmaX3", sigmaX3)
    material:setFloat("u_sigmaY3", sigmaY3)
    material:setFloat("u_sigmaX4", sigmaX4)
    material:setFloat("u_sigmaY4", sigmaY4)
    material:setFloat("u_sigmaX5", sigmaX5)
    material:setFloat("u_sigmaY5", sigmaY5)
    material:setFloat("u_sigmaX6", sigmaX6)
    material:setFloat("u_sigmaY6", sigmaY6)
    material:setFloat("u_sigmaX7", sigmaX7)
    material:setFloat("u_sigmaY7", sigmaY7)

    material:setFloat("u_GlowIntensity", intensity)
    material:setFloat("u_Alpha", alpha)
end

function TextMain:updateDeepGlow (progress)
    local time = progress * 72 / 30
    local intensity = self.ae:get("/DeepGlow_Exposure", time)[1]
    intensity = Utils.cubicIn(intensity)
    self:stepDeepGlow(self.spritesCtx.material, self.canvasSize.x, self.canvasSize.y, self.fontSize * 5, intensity * 1, 1.0)
end

function TextMain.random (rand)
    local randL = math.floor(rand)
    local randC = rand - randL
    local rand0 = Utils.fract(math.sin(randL + 0) * 43758.5453123)
    local rand1 = Utils.fract(math.sin(randL + 1) * 43758.5453123)
    return Utils.mix(rand0, rand1, randC)
end

function TextMain.wiggle (progress)
    local scale = 37.3124365
    local x = TextMain.random((progress + 0.0000) * scale)
    local y = TextMain.random((progress + 1.7311) * scale)
    return x * 2 - 1, y * 2 - 1
end

function TextMain.solveExpandSize (vertical, lineCount, fontSize, rawTextSize)
    if not vertical then
        local x = fontSize * 4 * (lineCount + 2)
        local y = fontSize * 4 * (lineCount + 2)
        return Amaz.Vector2f(x, y)
    else
        local x = fontSize * 4 * (lineCount + 2)
        local y = fontSize * 4 * (lineCount + 2)
        return Amaz.Vector2f(x, y)
    end
end






---------- TextEntry.lua ----------


local TextAnim = {}
TextAnim.__index = TextAnim


function TextAnim.new ()
    local self = setmetatable({}, TextAnim)
    self._main = TextMain.new(self)
    return self
end

function TextAnim:onStart (comp)
    self.curTime = 0.0
    self.duration = 5.0
    self._visible = false

    self.script = comp
    self.entity = comp.entity
    self.scene = comp.entity.scene
    self.path = Helper.getRootDirInLuaDir()
    self.transform = comp.entity:getComponent("Transform")
    self.renderer = comp.entity:getComponent("MeshRenderer")
    self.text = comp.entity:getComponent("Text")
    self._main:onCreate(self)
end

function TextAnim:onEnter ()
    if not self.visible then
        self._main:onShow(self)
        self._visible = true
    end
end

function TextAnim:onLeave ()
    if self._visible then
        self._main:onHide(self)
        self._visible = false
    end
    collectgarbage("collect")
end

function TextAnim:clear ()
    self:onLeave()
    if self._main.onDestroy then
        self._main:onDestroy(self)
    end
end

function TextAnim:setDuration (duration)
    self.duration = duration
end

function TextAnim:seek (time)
    if self._visible then
        self._main:onUpdate(self, time)
    end
end


---#ifdef DEV
--//function TextAnim:onUpdate (comp, dt)
--//    self:seek(self.seekTime or self.curTime)
--//    self.curTime = self.curTime + dt
--//end
--//function TextAnim:onEvent (comp, event)
--//    if event.type ~= Amaz.EventType.TOUCH then
--//        return
--//    end
--//    local pointer = event.args:get(0)
--//    if pointer.type == Amaz.TouchType.TOUCH_BEGAN then
--//        self.curTime = 0
--//        self:onEnter()
--//    elseif pointer.type == Amaz.TouchType.TOUCH_MOVED then
--//        self.seekTime = pointer.x * self.duration
--//    elseif pointer.type == Amaz.TouchType.TOUCH_ENDED or pointer.type == Amaz.TouchType.TOUCH_CANCELLED then
--//        self.seekTime = nil
--//        self:onLeave()
--//    end
--//end
---#endif


local exports = exports or {}
exports.TextAnim = TextAnim
return exports