local isEditor = (Amaz.Macros and Amaz.Macros.EditorSDK) and true or false
local exports = exports or {}
local TextAnim = TextAnim or {}
TextAnim.__index = TextAnim
---@class TextAnim : ScriptComponent
---@field autoPlay boolean
---@field duration number
---@field curTime number
---@field progress number [UI(Range={0, 1}, Slider)]


function TextAnim.new()
    local self = {}
    setmetatable(self, TextAnim)

    -- deep glow
    self.NormalizationSize = 720.
    self.RadiusOverSigma = 2.5
    return self
end

function TextAnim:textNew()
    -- vat Attr ----
    self.registerModules = {
        "LetterStyleMergeModule",
        "ComToolsModule",
        "RenderModule",
        "CaptionModule",
        "SceneMgrModule"
    }


    return self
end

local function findMinValue(array)
    local minValue = array[1]

    for i = 2, #array do
        if array[i] < minValue then
            minValue = array[i]
        end
    end

    return minValue
end

local function countPianyiliang(index, letterNum, jiange)
    local pianyishu = 0
    local pianyiliang = 0

    local midNum = math.floor(letterNum * 0.5) + 1
    pianyishu = index - midNum
    pianyiliang = pianyishu * jiange
    return pianyiliang
end

local function isWhite(_letter)
    local col = _letter.letterStyle.letterColorRGBA
    return math.abs(col.r - 1) < 0.01 and math.abs(col.g - 1) < 0.01 and math.abs(col.b - 1) < 0.01
end

local function getXYScale(width, height)
    local size1 = math.min(width, height)
    local size2 = math.max(width, height) / 2.
    local baseSize = math.max(size1, size2)

    local xScale = baseSize / width
    local yScale = baseSize / height
    return xScale, yScale
end

function TextAnim:setDeepGlowParams(mat, radius, glow)
    self.width = Amaz.BuiltinObject:getOutputTextureWidth()
    self.height = Amaz.BuiltinObject:getOutputTextureHeight()
    local xScale, yScale = getXYScale(self.width, self.height)
    local downSample1 = 1
    local downSample2 = 1
    local downSample3 = math.max(math.min(radius / 128, 2.0), 1.0)
    local downSample4 = math.max(math.min(radius / 64, 4.0), 1.0)
    local downSample5 = math.max(radius / 32, 1.0)
    local downSample6 = math.max(radius / 16, 1.0)
    local downSample7 = math.max(radius / 8, 1.0)
    local downSample8 = math.max(radius / 4, 1.0)
    local sample1 = radius / 16 * 0.125
    local sample2 = radius / 16 * 0.25
    local sample3 = radius / 16 / downSample3 * 0.5
    local sample4 = radius / 16 / downSample4
    local sample5 = radius / downSample5 * 0.125
    local sample6 = radius / downSample6 * 0.25
    local sample7 = radius / downSample7 * 0.5
    local sample8 = radius / downSample8

    local dx1 = downSample1 * xScale / self.NormalizationSize
    local dy1 = downSample1 * yScale / self.NormalizationSize
    local dx2 = downSample2 * xScale / self.NormalizationSize
    local dy2 = downSample2 * yScale / self.NormalizationSize
    local dx3 = downSample3 * xScale / self.NormalizationSize
    local dy3 = downSample3 * yScale / self.NormalizationSize
    local dx4 = downSample4 * xScale / self.NormalizationSize
    local dy4 = downSample4 * yScale / self.NormalizationSize
    local dx5 = downSample5 * xScale / self.NormalizationSize
    local dy5 = downSample5 * yScale / self.NormalizationSize
    local dx6 = downSample6 * xScale / self.NormalizationSize
    local dy6 = downSample6 * yScale / self.NormalizationSize
    local dx7 = downSample7 * xScale / self.NormalizationSize
    local dy7 = downSample7 * yScale / self.NormalizationSize

    mat:setFloat("u_sampleX1", sample1)
    mat:setFloat("u_sampleY1", sample1)
    mat:setFloat("u_sampleX2", sample2)
    mat:setFloat("u_sampleY2", sample2)
    mat:setFloat("u_sampleX3", sample3)
    mat:setFloat("u_sampleY3", sample3)
    mat:setFloat("u_sampleX4", sample4)
    mat:setFloat("u_sampleY4", sample4)
    mat:setFloat("u_sampleX5", sample5)
    mat:setFloat("u_sampleY5", sample5)
    mat:setFloat("u_sampleX6", sample6)
    mat:setFloat("u_sampleY6", sample6)
    mat:setFloat("u_sampleX7", sample7)
    mat:setFloat("u_sampleY7", sample7)

    mat:setFloat("u_dx1", dx1)
    mat:setFloat("u_dy1", dy1)
    mat:setFloat("u_dx2", dx2)
    mat:setFloat("u_dy2", dy2)
    mat:setFloat("u_dx3", dx3)
    mat:setFloat("u_dy3", dy3)
    mat:setFloat("u_dx4", dx4)
    mat:setFloat("u_dy4", dy4)
    mat:setFloat("u_dx5", dx5)
    mat:setFloat("u_dy5", dy5)
    mat:setFloat("u_dx6", dx6)
    mat:setFloat("u_dy6", dy6)
    mat:setFloat("u_dx7", dx7)
    mat:setFloat("u_dy7", dy7)

    local sigmaX1 = dx1 * sample1 / self.RadiusOverSigma
    local sigmaY1 = dy1 * sample1 / self.RadiusOverSigma
    local sigmaX2 = dx2 * sample2 / self.RadiusOverSigma
    local sigmaY2 = dy2 * sample2 / self.RadiusOverSigma
    local sigmaX3 = dx3 * sample3 / self.RadiusOverSigma
    local sigmaY3 = dy3 * sample3 / self.RadiusOverSigma
    local sigmaX4 = dx4 * sample4 / self.RadiusOverSigma
    local sigmaY4 = dy4 * sample4 / self.RadiusOverSigma
    local sigmaX5 = dx5 * sample5 / self.RadiusOverSigma
    local sigmaY5 = dy5 * sample5 / self.RadiusOverSigma
    local sigmaX6 = dx6 * sample6 / self.RadiusOverSigma
    local sigmaY6 = dy6 * sample6 / self.RadiusOverSigma
    local sigmaX7 = dx7 * sample7 / self.RadiusOverSigma
    local sigmaY7 = dy7 * sample7 / self.RadiusOverSigma

    mat:setFloat("u_sigmaX1", sigmaX1)
    mat:setFloat("u_sigmaY1", sigmaY1)
    mat:setFloat("u_sigmaX2", sigmaX2)
    mat:setFloat("u_sigmaY2", sigmaY2)
    mat:setFloat("u_sigmaX3", sigmaX3)
    mat:setFloat("u_sigmaY3", sigmaY3)
    mat:setFloat("u_sigmaX4", sigmaX4)
    mat:setFloat("u_sigmaY4", sigmaY4)
    mat:setFloat("u_sigmaX5", sigmaX5)
    mat:setFloat("u_sigmaY5", sigmaY5)
    mat:setFloat("u_sigmaX6", sigmaX6)
    mat:setFloat("u_sigmaY6", sigmaY6)
    mat:setFloat("u_sigmaX7", sigmaX7)
    mat:setFloat("u_sigmaY7", sigmaY7)

    mat:setFloat("u_GlowIntensity", glow)
end

function TextAnim:textStart(comp)
    self.r_module = self:getModule("RenderModule")
    self.t_module = self:getModule("ComToolsModule")
    self.c_module = self:getModule("CaptionModule")
    self:registingModule("CM2", "CaptionModule")
    self.c_module2 = self:getModule("CM2")
    self.s_module = self:getModule("SceneMgrModule")

    self.richText:forceTypeSetting()
    self.group = self.t_module:letterGroupBy().splitBySymbol(1, { " ", "\n" }, {})
    self.captionInfo = self.t_module:group2CaptionInfo(self.group, self.duration)
    self.attrs = self.getAEData()
end

function TextAnim:textInit()
    self.richText.canvas.renderToRT = false
    self.oriLetterSpacing = self.richText.typeSettingParam.letterSpacing
    if isEditor then
    else
    end

    self:setTextBloomState(false)

    self.c_module:init(self.captionInfo)
    -- self.c_module:splitPage(0, 1).LineCountLimit(1, 1, 50)
    self.c_module:splitPage(0, 1).Default()
    self.capDuration = self.c_module:getDuration()

    self.c_module2:init(self.captionInfo)
    -- self.c_module2:splitPage(0, 1).LineCountLimit(1, 1, 50)
    self.c_module2:splitPage(0, 1).Default()
    self.cloneTextSticker2 = self.s_module:addCloneText()
    self.c_module2:setRichText(self.cloneTextSticker2)


    local rt_1 = self:loadAsset("rt/rt1.rt")
    local rt_2 = self:loadAsset("rt/rt2.rt")
    -- local seqMat = self:loadAsset("material/seqAnimation.material")
    -- self.r_module:textPostProcessing(rt_1)
    -- self.r_module:textPostProcessing()
    -- local node1 = self.r_module:addRenderNode()
    -- -- local nodeMat1 = node1:addMaterial(seqMat, rt_2, "Normal")
    -- local nodeMat1 = node1:addMaterial(seqMat, nil, "Normal")
    -- self.node = node1

    local picture_1 = self:loadAsset("image/lightLine.png")
    local quadMat = self:loadAsset("effects/wrap/wrap.material")
    local seqMat = self:loadAsset("effects/seq/seq.material")
    local glowMat = self:loadAsset("effects/deep_glow/MatAndShader/deepGlow.material")

    local quadMat_0 = self.r_module:textPostProcessing(rt_1, quadMat, "Normal", 0)
    local defaultMat2 = self.r_module:textPostProcessing(nil, seqMat, "Normal", 3, self.cloneTextSticker2, true)

    quadMat_0:setTex("u_picture1", picture_1)

    self.node = self.r_module:addRenderNode()
    self.NormalizationSize = 720.
    self.RadiusOverSigma = 2.5
    self.nodeMat_0 = self.node:addMaterial(glowMat, nil, "Normal", 2) 
    self.nodeMat_0:setTex("u_inputTexture", rt_1)
    self:setDeepGlowParams(self.nodeMat_0, 160.0, 0.1)
    
    local pages = self.c_module:getPages()
    local expandedSize = self.text:getRectExpanded()
    self.richText.canvas.targetRTExtraSize = Amaz.Vector2f(expandedSize.width, expandedSize.height)
    
    self.sequenceFrames = {}
    for i = 1, 90 do  -- Adjust the number based on your actual frame count
        local framePath = string.format("seq/Lz_%05d.png", i-1)
        table.insert(self.sequenceFrames, self:loadAsset(framePath))
    end

    local readingPageFunc = function(_obj, _p, letters)
        self.richText:forceTypeSetting()
        local expandedSize = self.text:getRectExpanded()
        self.richText.canvas.targetRTExtraSize = Amaz.Vector2f(expandedSize.width * 0.5, expandedSize.height * 0.5)

        local feather = self.attrs:GetVal("ADBE_IE_269a507356ee6026561592e_0003_0_1", _p)[1]/100.0
        local wrapProgress = self.attrs:GetVal("ADBE_IE_269a507356ee6026561592e_0001_0_0", _p)[1]/100.0
        quadMat_0:setFloat("u_feather", feather * 0.5)
        quadMat_0:setFloat("u_progress", wrapProgress)

        local frameIndex = math.floor(_p * #self.sequenceFrames) + 1
        frameIndex = math.min(frameIndex, #self.sequenceFrames)  -- Ensure we don't go out of bounds
        
        -- Set the current frame texture
        quadMat_0:setTex("u_sequenceTex", self.sequenceFrames[frameIndex])

        local glowIns = self.attrs:GetVal("ADBE_IE_8cb17a2f56325a949dc60f2_0002_0_2", _p)[1]/12.0
        self:setDeepGlowParams(self.nodeMat_0, 80.0, glowIns)
    end

    local function beforeReadLineFunc(_obj, _p)

    end

    local function readingLineFunc(_obj, _p)

    end

    local function afterReadLineFunc(_obj, _p)

    end

    local beforeReadWordFunc = function(_obj, _p)

    end

    local readingWordFunc = function(_obj, _p)

    end

    local afterReadWordFunc = function(_obj, _p)

    end

    ------------------------------------cloneTextSticker2--------------------------------
    local function readingPageFunc2(_obj, _p, letters)

    end
    local afterReadPageFunc2 = function(_obj, _p, letters)
        self.cloneTextSticker2.richText:forceTypeSetting()
        local expandedSize = self.text:getRectExpanded()
        self.cloneTextSticker2.richText.canvas.targetRTExtraSize = Amaz.Vector2f(expandedSize.width * 0.75, expandedSize.height * 0.75)
        local frameIndex = math.floor(_p * #self.sequenceFrames) + 1
        frameIndex = math.min(frameIndex, #self.sequenceFrames)  -- Ensure we don't go out of bounds
        
        -- Set the current frame texture
        defaultMat2:setTex("u_sequenceTex", self.sequenceFrames[frameIndex])
        
    end
    local function readingLineFunc2(_obj, _p)

    end

    local function afterLineFunc2(_obj, _p)
    end

    local function beforeLineFunc2(_obj, _p)

    end
    local function beforeReadWordFunc2(_obj, _p)

    end

    local function readingWordFunc2(_obj, _p)

    end

    local function afterReadWordFunc2(_obj, _p)

    end

    for i = 1, #pages do
        pages[i]:setReadingAnim(readingPageFunc)
        pages[i]:setWordBeforeReadAnim(beforeReadWordFunc)
        pages[i]:setWordReadingAnim(readingWordFunc)
        pages[i]:setWordAfterReadAnim(afterReadWordFunc)
        pages[i]:setLineReadingAnim(readingLineFunc)
        pages[i]:setLineBeforeReadAnim(beforeReadLineFunc)
        pages[i]:setLineAfterReadAnim(afterReadLineFunc)
    end

    local pages2 = self.c_module2:getPages()
    for i = 1, #pages2 do
        pages2[i]:setReadingAnim(readingPageFunc2)
        pages2[i]:setAfterReadAnim(afterReadPageFunc2)
        pages2[i]:setLineReadingAnim(readingLineFunc2)
        pages2[i]:setLineBeforeReadAnim(beforeLineFunc2)
        pages2[i]:setLineAfterReadAnim(afterLineFunc2)
        pages2[i]:setWordBeforeReadAnim(beforeReadWordFunc2)
        pages2[i]:setWordReadingAnim(readingWordFunc2)
        pages2[i]:setWordAfterReadAnim(afterReadWordFunc2)
    end
end

function TextAnim:textSeek(_time)
    -- local time = _time
    local time = self.progress

    if isEditor then
        time = self.capDuration * self.progress
    else
        -- self.progress = math.min(time, self.capDuration)
        -- self.progress = _time % self.duration / self.duration
        self.progress = _time / self.duration
    end
    -- local page, word = self.c_module:animing(math.min(time, self.capDuration))
    local page, word = self.c_module:animing(time)
    local page1, word1 = self.c_module2:animing(time)
    -- if page == nil then
    -- self.emojiSticker.entity.visible = false
    -- return
    -- end
    -- self.emojiSticker.entity.visible = true
end

function TextAnim:textReset()
    self.richText.typeSettingParam.letterSpacing = self.oriLetterSpacing
end

exports.TextAnim = TextAnim
return exports
