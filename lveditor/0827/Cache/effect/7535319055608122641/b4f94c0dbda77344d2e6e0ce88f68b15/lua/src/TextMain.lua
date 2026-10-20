local Utils = require("common/Utils")
local Helper = require("info_sticker/Helper")
local AEAdapter = require("common/AEAdapter")
local AE = require("AE")


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
    for i = 0, 71 do
        table.insert(self.MASK.images, string.format("image/mask/mask_%02d.png", i))
        table.insert(self.ANIM.images, string.format("image/anim/anim_%02d.png", i))
    end
    self.OFFSET = {
        {s = 28/72, e = 31/72},
    }
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

    local progress = 1 - elapsed / env.duration
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
    local maskIdx = Utils.clamp(math.floor(progress * #self.MASK.images + 0.5), 0, 71) + 1
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

return TextMain