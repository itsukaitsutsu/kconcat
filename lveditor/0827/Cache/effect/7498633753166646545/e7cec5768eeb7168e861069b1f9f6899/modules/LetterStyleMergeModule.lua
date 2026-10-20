--[[  ****************************************
    Common Func
--    **************************************** ]]
local Common = {
    isEditor = (Amaz.Macros and Amaz.Macros.EditorSDK) and true or false,

    readStrFromFile = function(file_path)
        local file = io.input(file_path)
        local text = io.read("*a")
        return text
    end,

    getLetterCount = function(_str)
        local str = _str
        local lenInByte = #str
        local count = 0
        local i = 1
        while true do
            local curByte = string.byte(str, i)
            if i > lenInByte then
                break
            end
            local byteCount = 1
            if curByte > 0 and curByte < 128 then
                byteCount = 1
            elseif curByte >= 128 and curByte < 224 then
                byteCount = 2
            elseif curByte >= 224 and curByte < 240 then
                byteCount = 3
            elseif curByte >= 240 and curByte <= 247 then
                byteCount = 4
            else
                break
            end
            -- local char = string.sub(str, i, i+byteCount-1)
            i = i + byteCount
            count = count + 1
        end
        return count
    end,

    fakeMergeData = {
        -- letterColorRGBA = Amaz.Color(0.5, 1, 0.5, 1),
        -- fontSize = 30
    },

}
--[[  ****************************************
    LetterStyleMergeModule: manager all the CloneText
--    **************************************** ]]
local LetterStyleMergeModule = {}
LetterStyleMergeModule.__index = LetterStyleMergeModule
LetterStyleMergeModule.TEXT_STICKER = "TEXT_STICKER"


function LetterStyleMergeModule.new(_sticker)
    local self = setmetatable({}, LetterStyleMergeModule)

    self.sticker = _sticker
    return self
end

function LetterStyleMergeModule:merge(_baseLetter, _mergeLetter, _blendMode)
    -- Amaz.LOGE("lrc merge base "..tostring(_baseLetter.utf8), tostring(_baseLetter.letterStyle.letterColorRGBA))
    -- Amaz.LOGE("lrc merge merged "..tostring(_mergeLetter.utf8), tostring(_mergeLetter.letterStyle.letterColorRGBA))
    if Common.isEditor then
        self:fakeMerge(_baseLetter, _mergeLetter)
    else
        self:realMerge(_baseLetter, _mergeLetter, _blendMode)
    end
end

function LetterStyleMergeModule:realMerge(_baseLetter, _mergeLetter, _blendMode)
    local keywordStyle = self:getKeywordStyle()
    if keywordStyle == nil then
        _baseLetter.letterStyle.letterColorRGBA = _mergeLetter.letterStyle.letterColorRGBA
        -- _baseLetter.letterStyle.fontSize = _mergeLetter.letterStyle.fontSize
        return
    end

    if keywordStyle.styles == nil or not (#keywordStyle.styles > 0) then
        return
    end

    local style = keywordStyle.styles[1]

    if style == nil then
        return
    end

    local function setNonEffectStyle()
        if style.size then
            _baseLetter.letterStyle.fontSize = _mergeLetter.letterStyle.fontSize
        end

        -- set font's about
        if style.font then
            _baseLetter.letterStyle.fontId = _mergeLetter.letterStyle.fontId
            _baseLetter.letterStyle.fontfamily = _mergeLetter.letterStyle.fontfamily
        end

        -- set fontStyle's about
        if style.bold == true or style.italic == true then
            _baseLetter.letterStyle.fontStyle = _mergeLetter.letterStyle.fontStyle
        end

        if style.underline then
            _baseLetter.letterStyle.fontDecoration = _mergeLetter.letterStyle.fontDecoration
        end

        -- fill's content
        if style.fill then
            _baseLetter.letterStyle.letterColorRGBA = _mergeLetter.letterStyle.letterColorRGBA
            _baseLetter.letterStyle.fill = _mergeLetter.letterStyle.fill:clone()
        end

        -- stroke's content
        if style.strokes and #style.strokes > 0 then
            _baseLetter.letterStyle.strokes = _mergeLetter.letterStyle.strokes:clone()
        end

        -- shadows's content
        if style.shadows and #style.shadows > 0 then
            _baseLetter.letterStyle.shadows = _mergeLetter.letterStyle.shadows:clone()
        end

        -- inner_shadows's content, ONLY flower word has
        if style.inner_shadows and #style.inner_shadows > 0 then
            _baseLetter.letterStyle.innerShadows = _mergeLetter.letterStyle.innerShadows:clone()
        end
    end

    if style.effectStyle then
        _baseLetter.letterStyle = _mergeLetter.letterStyle:clone()
        return
        -- _baseLetter.letterStyle.effectStyleId = _mergeLetter.letterStyle.effectStyleId
        -- _baseLetter.letterStyle.effectStylePath = _mergeLetter.letterStyle.effectStylePath
    else
        if _baseLetter.letterStyle.effectStylePath ~= _mergeLetter.letterStyle.effectStylePath then
            _baseLetter.letterStyle = _mergeLetter.letterStyle:clone()
            return
        end
    end

    setNonEffectStyle()
end

function LetterStyleMergeModule:init()
    if Common.isEditor then
        local rootDir = self.sticker.entity.scene.assetMgr.rootDir
        self.rawKeywordStyleStr = Common.readStrFromFile(rootDir .. "test/fake_kw_style1.json")
    end

    self.keywordStyle = nil
end

function LetterStyleMergeModule:seek()

end

function LetterStyleMergeModule:fakeMerge(baseLetter, mergedLetter)
    if Common.fakeMergeData.letterColorRGBA then
        baseLetter.letterStyle.letterColorRGBA = Common.fakeMergeData.letterColorRGBA
    end
    if Common.fakeMergeData.fontSize then
        baseLetter.letterStyle.fontSize = Common.fakeMergeData.fontSize
    end
end

function LetterStyleMergeModule:getKeywordStyle()
    if self.rawKeywordStyleStr and self.keywordStyle == nil then
        self.keywordStyle = cjson.decode(self.rawKeywordStyleStr)
    end
    return self.keywordStyle
end

function LetterStyleMergeModule:getMergedLetters()
    if Common.isEditor then
        return self.sticker.richText.letters:clone()
    end

    if self.rawKeywordStyleStr and self.rawCaptionStr then
        if self.sticker.richText.applyTextStyle then
            local range = Amaz.Vector()
            local curRange = 0
            local captionInfo = cjson.decode(self.rawCaptionStr)
            local words = captionInfo.words
            for i = 1, #words do
                local word = words[i]
                if word.is_key then
                    range:pushBack(curRange)
                    range:pushBack(curRange + Common.getLetterCount(word.text))
                end
                curRange = curRange + Common.getLetterCount(word.text)
            end
            self.sticker.richText:applyTextStyle(self.rawKeywordStyleStr, range)
            self.sticker.richText:forceTypeSetting()
            return self.sticker.richText.letters:clone()
        end
    end
    return self.sticker.richText.letters:clone()
end

function LetterStyleMergeModule:onSetProperty(key, value)
    if key == "keyword_rich_text" and value ~= "" then
        Amaz.LOGE("lrc keyword_rich_text", value)
        self.rawKeywordStyleStr = value
    end

    if key == "caption_duration_info" and value ~= "" then
        Amaz.LOGE("lrc keyword_rich_text", value)
        self.rawCaptionStr = value
    end
end

function LetterStyleMergeModule:reset()
end

return LetterStyleMergeModule
