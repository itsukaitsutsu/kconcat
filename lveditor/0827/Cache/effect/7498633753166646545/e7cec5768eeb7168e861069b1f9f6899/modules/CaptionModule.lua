
--[[
    Script Name: CaptionModule.lua
    Version: 1.7
    Update Data: 2024.07.27
--]]
--[[  ****************************************
    Common Func
--    **************************************** ]]

local Common = {
    isEditor = (Amaz.Macros and Amaz.Macros.EditorSDK) and true or false,
    STICKER = nil,
    WORD_STATE = {
        READING = 0,
        BEFORE_READ = 1,
        AFTER_READ = 2
    },
    LINE_STATE = {
        READING = 0,
        BEFORE_READ = 1,
        AFTER_READ = 2
    },
    captionKeywordStyle = nil,
    getTightRect = function(letter, outlineMaxWidth, expandAffector)
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
    end,

    readStrFromFile = function(file_path)
        local file = io.input(file_path)
        local text = io.read("*a")
        return text
    end,

    remap01 = function(a, b, x)
        if x < a then return 0 end
        if x > b then return 1 end
        return (x - a) / (b - a)
    end,

    mix = function(a, b, x)
        return a * (1 - x) + b * x
    end,

    clamp = function(x, a, b)
        return math.max(math.min(x, b), a)
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

    isLegalStr = function(_str)
        local ilegal = _str == " " or _str == "\n"
        return not ilegal
    end,

    getTimeRange = function(_words)
        local startTime = 999999
        local endTime = 0
        for i = 1, #_words do
            local w = _words[i]
            if w.text ~= " " and w.text ~= "\n" then
                local st = w.pack_start_time == nil and w.start_time or w.pack_start_time
                local et = w.pack_end_time == nil and w.end_time or w.pack_end_time
                startTime = startTime == nil and st or math.min(startTime, st)
                endTime = endTime == nil and et or math.max(endTime, et)
            end
        end
        return startTime * 0.001, endTime * 0.001
    end,

    register = function(_obj, _name, _default, _getter, _setter)
        _obj[_name] = _default

        local funcName = _name:gsub("^%l", string.upper)
        local getFuncName = "get" .. funcName
        local setFuncName = "set" .. funcName

        if _getter then
            _obj[getFuncName] = _getter
        else
            if _obj[getFuncName] == nil then
                _obj[getFuncName] = function(_o)
                    return _o[_name]
                end
            end
        end

        if _setter and _obj[setFuncName] == nil then
            _obj[setFuncName] = _setter
        else
            if _obj[setFuncName] == nil then
                _obj[setFuncName] = function(_o, _val)
                    _o[_name] = _val
                end
            end
        end
    end,

    flushCmd = function(sticker)
        -- flush cmd based on oriStr
        if sticker then
            sticker.richText.str = sticker.oriStr
            sticker.richText:forceTypeSetting()
            if sticker.richText.forceFlushCommandQueue then
                sticker.richText:forceFlushCommandQueue()
            end
        end
    end,

    fakeDataForKeyWordMerge = {
        -- letterColorRGBA = Amaz.Color(0.5, 1, 0.5, 1),
        fontSize = 30
    },

    fakeMergeFunc = function(letter, fakeData)
        if fakeData.letterColorRGBA then
            letter.letterStyle.letterColorRGBA = fakeData.letterColorRGBA
        end
        if fakeData.fontSize then
            letter.letterStyle.fontSize = fakeData.fontSize
        end
    end,

}

local mergeModule = nil

--[[  ****************************************
    CWord
--    **************************************** ]]
local CWord = {}
CWord.__index = CWord

function CWord.new(_wordInfo, _page, _line)
    local self = setmetatable({}, CWord)


    self.progress = 0
    self.oriLetters = {}
    Common.register(self, "page", _page)
    Common.register(self, "line", _line)

    Common.register(self, "startTime", _wordInfo.pack_start_time * 0.001)
    Common.register(self, "endTime", _wordInfo.pack_end_time * 0.001)

    Common.register(self, "packStartTime", _wordInfo.pack_start_time * 0.001)
    Common.register(self, "packEndTime", _wordInfo.pack_end_time * 0.001)


    Common.register(self, "str", _wordInfo.text)
    Common.register(self, "idx", _wordInfo.idx)
    Common.register(self, "legalIdx", _wordInfo.legal_idx)

    Common.register(self, "idxInPage", -1)
    Common.register(self, "legalIdxInPage", -1)

    Common.register(self, "idxInLine", -1)
    Common.register(self, "legalIdxInLine", -1)

    Common.register(self, "isKey", _wordInfo.is_key)
    Common.register(self, "startLetterIdx", _wordInfo.ori_start_letter_idx)
    -- Common.register(self, "letterIdxOffset", 0)

    Common.register(self, "letters", {})
    Common.register(self, "oriLetters", {})
    Common.register(self, "letterCount", 0,
        function()
            return #self.letters
        end,
        nil
    )

    Common.register(self, "progress", 0)
    Common.register(self, "duration", 0)

    -- animations
    self.animingFuncs = {
        [Common.WORD_STATE.READING] = nil,
        [Common.WORD_STATE.BEFORE_READ] = nil,
        [Common.WORD_STATE.AFTER_READ] = nil,
    }

    self.animingEarlierFuncs = {
        [Common.WORD_STATE.READING] = nil,
        [Common.WORD_STATE.BEFORE_READ] = nil,
        [Common.WORD_STATE.AFTER_READ] = nil,
    }
    return self
end

function CWord:setNextWord(_other)
    self.nextWord = _other
end

function CWord:getNextWord()
    local closeTo = false
    if self.nextWord and (math.abs(self.nextWord:getStartTime() - self:getEndTime()) < 0.001) then
        closeTo = true
    end
    return self.nextWord, closeTo
end

function CWord:setLastWord(_other)
    self.lastWord = _other
end

function CWord:getLastWord()
    local closeTo = false
    if self.lastWord and (math.abs(self.lastWord:getStartTime() - self:getEndTime()) < 0.001) then
        closeTo = true
    end
    return self.lastWord, closeTo
end

function CWord:getProgress(_time)
    return Common.clamp((_time - self.startTime) / (self.endTime - self.startTime), 0, 1)
end

function CWord:getDuration()
    return self.endTime - self.startTime
end

function CWord:isLegal()
    return self.legalIdx > 0
end

function CWord:getInitalRect()
    return self:getRect(function(_l)
        return _l.initialPosition
    end)
end

function CWord:getRect(_posDelegate)
    if self.letters == nil then
        return Amaz.Rect(0, 0, 0, 0)
    end
    -- local mgdLetters = self.oriLetters
    local baseLetters = self.letters

    local function getWordRect(_letters)
        local ldp = nil
        local urp = nil
        for i = 1, #_letters do
            local let = _letters[i]
            if Common.isLegalStr(let.utf8) == true then
                local rect = Common.getTightRect(let, Common.STICKER.richText.outlineMaxWidth)
                local pos = _posDelegate == nil and let.position or _posDelegate(let)
                if ldp == nil then
                    ldp = {
                        pos.x - rect.width * 0.5,
                        pos.y - rect.height * 0.5,
                    }
                else
                    ldp = {
                        math.min(pos.x - rect.width * 0.5, ldp[1]),
                        math.min(pos.y - rect.height * 0.5, ldp[2]),
                    }
                end

                if urp == nil then
                    urp = {
                        pos.x + rect.width * 0.5,
                        pos.y + rect.height * 0.5,
                    }
                else
                    urp = {
                        math.max(pos.x + rect.width * 0.5, urp[1]),
                        math.max(pos.y + rect.height * 0.5, urp[2]),
                    }
                end
            end
        end
        if ldp == nil then
            return Amaz.Rect(0, 0, 0, 0)
        end
        local rect = Amaz.Rect(ldp[1], ldp[2], urp[1] - ldp[1], urp[2] - ldp[2])
        return rect
    end

    -- local mgdRect = getWordRect(mgdLetters)
    local baseRect = getWordRect(baseLetters)
    return baseRect
end

function CWord:getInitialCenter()
    return self:getCenter(function(_l)
        return _l.initialPosition
    end)
end

function CWord:getCenter(_centerDelegate)
    if self.letters == nil then
        return Amaz.Vector2f(-1, -1)
    end

    local rect = self:getRect(_centerDelegate)
    local center = Amaz.Vector2f(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
    return center
end

function CWord:setReadingAnim(_func, _isEarlier)
    if _isEarlier == true then
        self.animingEarlierFuncs[Common.WORD_STATE.READING] = _func
    else
        self.animingFuncs[Common.WORD_STATE.READING] = _func
    end
end

function CWord:setAfterReadAnim(_func, _isEarlier)
    if _isEarlier == true then
        self.animingEarlierFuncs[Common.WORD_STATE.AFTER_READ] = _func
    else
        self.animingFuncs[Common.WORD_STATE.AFTER_READ] = _func
    end
end

function CWord:setBeforeReadAnim(_func, _isEarlier)
    if _isEarlier == true then
        self.animingEarlierFuncs[Common.WORD_STATE.BEFORE_READ] = _func
    else
        self.animingFuncs[Common.WORD_STATE.BEFORE_READ] = _func
    end
end

function CWord:getState(_time)
    local state = nil
    if _time < self.startTime then
        state = Common.WORD_STATE.BEFORE_READ
    elseif _time >= self.startTime and _time < self.endTime then
        state = Common.WORD_STATE.READING
    elseif _time >= self.endTime then
        state = Common.WORD_STATE.AFTER_READ
    end
    return state
end

function CWord:setState(_val)
    self.state = _val
end

function CWord:hasWholeAnimDelegate(_isEarlier)
    if _isEarlier == true then
        return self.animingEarlierFuncs[Common.WORD_STATE.BEFORE_READ] and
            self.animingEarlierFuncs[Common.WORD_STATE.READING] and
            self.animingEarlierFuncs[Common.WORD_STATE.AFTER_READ]
    end
    return self.animingFuncs[Common.WORD_STATE.BEFORE_READ] and
        self.animingFuncs[Common.WORD_STATE.READING] and
        self.animingFuncs[Common.WORD_STATE.AFTER_READ]
end

function CWord:animing(_time)
    if self:isLegal() == false then
        return
    end

    if self:hasWholeAnimDelegate() then
        local p = self:getProgress(_time)
        self.state = self:getState(_time)

        if self.animingFuncs[self.state] then
            self.animingFuncs[self.state](self, p, _time)
        end
    end
end

function CWord:animingEarlier(_time)
    if self:isLegal() == false then
        return
    end

    if self:hasWholeAnimDelegate(true) then
        local p = self:getProgress(_time)
        self.state = self:getState(_time)

        if self.animingEarlierFuncs[self.state] then
            self.animingEarlierFuncs[self.state](self, p, _time)
        end
    end
end

function CWord:brushKeyWord()
    if self:getIsKey() then
        if mergeModule then
            for i = 1, #self.letters do
                mergeModule:merge(self.letters[i], self.oriLetters[i])
            end
        end
    end
end

function CWord:setAbsAnchor(_anchor)
    if self:isLegal() then
        for i = 1, #self.letters do
            local l = self.letters[i]
            l.anchor = Amaz.Vector2f(
                _anchor.x - l.initialPosition.x,
                _anchor.y - l.initialPosition.y
            )
        end
    end
end

function CWord:setAnchor(_anchor)
    local rect = self:getRect()
    local center = self:getCenter()
    for i = 1, #self.letters do
        local letter = self.letters[i]

        local lx = center.x - letter.position.x
        local ly = center.y - letter.position.y
        lx = lx + _anchor.x * rect.width * 0.5
        ly = ly + _anchor.y * rect.height * 0.5
        letter.anchor = Amaz.Vector2f(lx, ly)
    end
end

function CWord:destroy()
    self.letters = {}
    self.page = nil
    self.line = nil
end

--[[  ****************************************
    CLine: manager all the CWord
--    **************************************** ]]
local CLine = {}
CLine.__index = CLine

function CLine.new(_idx, _page)
    local self = setmetatable({}, CLine)

    self.progress = 0
    self.duration = 0
    Common.register(self, "page", _page)
    Common.register(self, "words", {})
    Common.register(self, "idx", _idx)
    Common.register(self, "idxInPage", -1)
    Common.register(self, "letters", {})
    Common.register(self, "str", "")

    Common.register(self, "startTime", 0)
    Common.register(self, "endTime", 0)

    -- animations
    self.animingFuncs = {
        [Common.LINE_STATE.READING] = nil,
        [Common.LINE_STATE.BEFORE_READ] = nil,
        [Common.LINE_STATE.AFTER_READ] = nil,
    }

    self.animingEarlierFuncs = {
        [Common.LINE_STATE.READING] = nil,
        [Common.LINE_STATE.BEFORE_READ] = nil,
        [Common.LINE_STATE.AFTER_READ] = nil,
    }

    return self
end

function CLine:setExtraFontSizeScale(_scale)
    local fsScale = _scale
    Amaz.LOGI("lrc cline /" .. tostring(self:getStr()), _scale)

    for i = 1, #self.words do
        local word = self.words[i]
        local letters = word:getLetters()
        local oriLetters = word:getOriLetters()
        -- Amaz.LOGE("lrc letters", #letters)
        for j = 1, #letters do
            local letter = letters[j]
            -- Amaz.LOGW("lrc letter "..tostring(letter.utf8), tostring(letter.letterStyle.fontSize) )
            letter.letterStyle.fontSize = oriLetters[j].letterStyle.fontSize * fsScale
        end
    end
end

function CLine:isLegal()
    return #self.words > 0
end

function CLine:getProgress(_time)
    return Common.clamp((_time - self.startTime) / (self.endTime - self.startTime), 0, 1)
end

function CLine:getDuration()
    return self.endTime - self.startTime
end

function CLine:isReading(_time)
    if self:getIdx() == 1 then
        return _time >= self.startTime and _time <= self.endTime
    end
    return _time > self.startTime and _time <= self.endTime
end

function CLine:addWord(_word)
    table.insert(self.words, _word)
end

function CLine:setPage(page)
    self.page = page
    for i = 1, #self.words do
        local word = self.words[i]
        word:setPage(page)
    end
end

function CLine:getLegalWordCount()
    if not self:isLegal() then
        return 0
    end
    local count = 0
    for i = 1, #self.words do
        local w = self.words[i]
        if w:isLegal() then
            count = count + 1
        end
    end
    return count
end

function CLine:getWordCount()
    if not self:isLegal() then
        return 0
    end
    return #self.words
end

function CLine:getLetterCount()
    if not self:isLegal() then
        return 0
    end
    local count = 0
    for i = 1, #self.words do
        local w = self.words[i]
        count = count + w:getLetterCount()
    end
    return count
end

function CLine:getLegalLetterCount()
    if not self:isLegal() then
        return 0
    end
    local count = 0
    for i = 1, #self.words do
        local w = self.words[i]
        if w:isLegal() then
            count = count + w:getLetterCount()
        end
    end
    return count
end

function CLine:getOriStartLetterIdx()
    if self:isLegal() then
        return self.words[1]:getStartLetterIdx()
    else
        return -1
    end
end

function CLine:getState(_time)
    local state = nil
    if _time < self.startTime then
        state = Common.LINE_STATE.BEFORE_READ
    elseif _time >= self.startTime and _time < self.endTime then
        state = Common.LINE_STATE.READING
    elseif _time >= self.endTime then
        state = Common.LINE_STATE.AFTER_READ
    end
    return state
end

function CLine:hasWholeAnimDelegate(_isEarlier)
    if _isEarlier == true then
        return self.animingEarlierFuncs[Common.LINE_STATE.BEFORE_READ] and
            self.animingEarlierFuncs[Common.LINE_STATE.READING] and
            self.animingEarlierFuncs[Common.LINE_STATE.AFTER_READ]
    end
    return self.animingFuncs[Common.LINE_STATE.BEFORE_READ] and
        self.animingFuncs[Common.LINE_STATE.READING] and
        self.animingFuncs[Common.LINE_STATE.AFTER_READ]
end

function CLine:animing(_time)
    if self:isLegal() == false then
        return
    end

    if self:hasWholeAnimDelegate() then
        local p = self:getProgress(_time)
        local state = self:getState(_time)

        if self.animingFuncs[state] then
            self.animingFuncs[state](self, p, _time)
        end
    end
end

function CLine:animingEarlier(_time)
    if self:isLegal() == false then
        return
    end

    if self:hasWholeAnimDelegate(true) then
        local p = self:getProgress(_time)
        local state = self:getState(_time)

        if self.animingEarlierFuncs[state] then
            self.animingEarlierFuncs[state](self, p, _time)
        end
    end
end

function CLine:setReadingAnim(_func, _isEarlier)
    if _isEarlier == true then
        self.animingEarlierFuncs[Common.LINE_STATE.READING] = _func
    else
        self.animingFuncs[Common.LINE_STATE.READING] = _func
    end
end

function CLine:setAfterReadAnim(_func, _isEarlier)
    if _isEarlier == true then
        self.animingEarlierFuncs[Common.LINE_STATE.AFTER_READ] = _func
    else
        self.animingFuncs[Common.LINE_STATE.AFTER_READ] = _func
    end
end

function CLine:setBeforeReadAnim(_func, _isEarlier)
    if _isEarlier == true then
        self.animingEarlierFuncs[Common.LINE_STATE.BEFORE_READ] = _func
    else
        self.animingFuncs[Common.LINE_STATE.BEFORE_READ] = _func
    end
end

function CLine:refresh()
    self.str = ""
    local curLegalWordIdx = 0
    for i = 1, #self.words do
        local word = self.words[i]

        word:setIdxInLine(i)
        word:setPage(self:getPage())
        if word:isLegal() then
            curLegalWordIdx = curLegalWordIdx + 1
            word:setLegalIdxInLine(curLegalWordIdx)
        end

        local letters = word:getLetters()
        for j = 1, #letters do
            table.insert(self.letters, letters[j])
        end

        self.str = self.str .. word:getStr()
    end

    self.startTime = self.words[1]:getStartTime()
    self.endTime = self.words[#self.words]:getEndTime()
end

function CLine:getInitalRect()
    return self:getRect(function(_l)
        return _l.initialPosition
    end)
end

function CLine:getRect(_posDelegate)
    if self:isLegal() == false then
        return Amaz.Rect(0, 0, 0, 0)
    end

    local leftDownPoint = nil
    local rightUpPoint = nil
    for i = 1, #self.words do
        local word = self.words[i]
        if word:isLegal() then
            local rect = word:getRect(_posDelegate)
            local curLeftDownPoint = { rect.x, rect.y }
            local curRightUpPoint = { rect.x + rect.width, rect.y + rect.height }
            if leftDownPoint == nil then
                leftDownPoint = curLeftDownPoint
            else
                leftDownPoint = {
                    math.min(leftDownPoint[1], curLeftDownPoint[1]),
                    math.min(leftDownPoint[2], curLeftDownPoint[2]),
                }
            end

            if rightUpPoint == nil then
                rightUpPoint = curRightUpPoint
            else
                rightUpPoint = {
                    math.max(rightUpPoint[1], curRightUpPoint[1]),
                    math.max(rightUpPoint[2], curRightUpPoint[2]),
                }
            end
        end
    end

    if leftDownPoint and rightUpPoint then
        local width = rightUpPoint[1] - leftDownPoint[1]
        local height = rightUpPoint[2] - leftDownPoint[2]
        return Amaz.Rect(leftDownPoint[1], leftDownPoint[2],
            width, height)
    end
    return Amaz.Rect(0, 0, 0, 0)
end

function CLine:getInitialCenter()
    return self:getCenter(function(_l)
        return _l.initialPosition
    end)
end

function CLine:getCenter(_centerDelegate)
    if self:isLegal() == false then
        return Amaz.Vector2f(-1, -1)
    end

    local rect = self:getRect(_centerDelegate)
    local center = Amaz.Vector2f(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
    return center
end

--[[  ****************************************
    CPage: manager all the CWord/CLine
--    **************************************** ]]
local CPage = {}
CPage.__index = CPage

function CPage.new(_idx)
    local self = setmetatable({}, CPage)

    self.oriStartLetterIdx = -1
    self.letterCount = -1
    Common.register(self, "words", {})
    Common.register(self, "lines", {})
    Common.register(self, "str", "")
    Common.register(self, "idx", _idx)
    Common.register(self, "letters", {})
    Common.register(self, "startTime", 0)
    Common.register(self, "endTime", 0)

    Common.register(self, "progress", 0)
    Common.register(self, "duration", 0)
    Common.register(self, "startLetterIdx", -1)

    Common.register(self, "enable", true)

    -- for fit with max line
    Common.register(self, "fitMaxLine", false)
    Common.register(self, "extraLineSpacing", 0)
    Common.register(self, "oriLineSpacing", 0)
    Common.register(self, "lastLineMaxScale", 1.3)
    Common.register(self, "lastLineWidthLimitFactor", 0.4)

    return self
end

function CPage:_setMaxLineFitable()
    -- Amaz.LOGI("lrc page ", tostring(self.fitMaxLine))
    if self.fitMaxLine == true and #self.lines > 0 then
        local maxLineScale = 1
        local maxWidth = -1
        local rectWidths = {}

        local maxWordCount = -1
        for i = 1, #self.lines do
            local line = self.lines[i]
            local curRect = line:getRect()
            maxWidth = math.max(curRect.width, maxWidth)
            table.insert(rectWidths, curRect.width)
            maxWordCount = math.max(maxWordCount, line:getWordCount())
        end

        for i = 1, #self.lines do
            local line = self.lines[i]
            local scale = maxWidth / rectWidths[i]

            -- if i == #self.lines and (maxWordCount * 0.55 > line:getWordCount()) then
            --     scale = math.min(scale, 1.2)
            -- end

            if i == #self.lines and (rectWidths[i] < maxWidth * self.lastLineWidthLimitFactor) then
                scale = math.min(scale, self.lastLineMaxScale)
            end

            maxLineScale = math.max(maxLineScale, scale)
            line:setExtraFontSizeScale(scale)
        end

        Common.STICKER.richText.typeSettingParam.lineSpacing = self.oriLineSpacing +
        self.extraLineSpacing * (1. / maxLineScale)
    end
end

function CPage:setAnchor(_anchor)
    if self:isLegal() then
        local rect = self:getRect()
        local anchor = Amaz.Vector2f(_anchor.x * 0.5 + 0.5, _anchor.y * 0.5 + 0.5)
        anchor = Amaz.Vector2f(
            rect.x + anchor.x * rect.width,
            rect.y + anchor.y * rect.height
        )
        for i = 1, #self.words do
            local word = self.words[i]
            word:setAbsAnchor(anchor)
        end
    end
end

function CPage:getLegalWordCount()
    if not self:isLegal() then
        return 0
    end
    local count = 0
    for i = 1, #self.words do
        local w = self.words[i]
        if w:isLegal() then
            count = count + 1
        end
    end
    return count
end

function CPage:getWordCount()
    if not self:isLegal() then
        return 0
    end
    return #self.words
end

function CPage:getLetterCount()
    if not self:isLegal() then
        return 0
    end
    local count = 0
    for i = 1, #self.words do
        local w = self.words[i]
        count = count + w:getLetterCount()
    end
    return count
end

function CPage:getLegalLetterCount()
    if not self:isLegal() then
        return 0
    end
    local count = 0
    for i = 1, #self.words do
        local w = self.words[i]
        if w:isLegal() then
            count = count + w:getLetterCount()
        end
    end
    return count
end

function CPage:getOriStartLetterIdx()
    if self:isLegal() then
        return self.words[1]:getStartLetterIdx()
    else
        return -1
    end
end

function CPage:getProgress(_time)
    return Common.clamp((_time - self.startTime) / (self.endTime - self.startTime), 0, 1)
end

function CPage:isReading(_time)
    if self:getIdx() == 1 then
        return _time >= self.startTime and _time <= self.endTime
    end
    return _time > self.startTime and _time <= self.endTime
end

function CPage:addWord(_word)
    table.insert(self.words, _word)
end

function CPage:addLine(_line)
    table.insert(self.lines, _line)
end

function CPage:getInitalRect()
    return self:getRect(function(_l)
        return _l.initialPosition
    end)
end

function CPage:getRect(_posDelegate)
    if self:isLegal() == false then
        return nil
    end

    local leftDownPoint = nil
    local rightUpPoint = nil
    for i = 1, #self.words do
        local word = self.words[i]
        if word:isLegal() then
            local rect = word:getRect(_posDelegate)
            local curLeftDownPoint = { rect.x, rect.y }
            local curRightUpPoint = { rect.x + rect.width, rect.y + rect.height }
            if leftDownPoint == nil then
                leftDownPoint = curLeftDownPoint
            else
                leftDownPoint = {
                    math.min(leftDownPoint[1], curLeftDownPoint[1]),
                    math.min(leftDownPoint[2], curLeftDownPoint[2]),
                }
            end

            if rightUpPoint == nil then
                rightUpPoint = curRightUpPoint
            else
                rightUpPoint = {
                    math.max(rightUpPoint[1], curRightUpPoint[1]),
                    math.max(rightUpPoint[2], curRightUpPoint[2]),
                }
            end
        end
    end

    if leftDownPoint and rightUpPoint then
        return Amaz.Rect(leftDownPoint[1], leftDownPoint[2],
            rightUpPoint[1] - leftDownPoint[1], rightUpPoint[2] - leftDownPoint[2])
    end
    return nil
end


function CPage:getRectPerLine(_posDelegate)

    if self:isLegal() == false then
        return nil
    end

    local lineSplit = {}
    local letters = self:getLetters()
    local lastRowth = -1
    local line = {}
    for i = 1, #letters do
        local let = letters[i]
        if lastRowth == -1 or lastRowth ~= let.rowth then
            if #line > 0 then
                table.insert(lineSplit, line)
            end
            line = {}
            lastRowth = let.rowth
        end
        table.insert(line, let)
    end

    if #line > 0 then
        table.insert(lineSplit, line)
    end

    local function getLettersRect(_letters)
        local ldp = nil
        local urp = nil
        for i = 1, #_letters do
            local let = _letters[i]
            if Common.isLegalStr(let.utf8) == true then
                local rect = Common.getTightRect(let, Common.STICKER.richText.outlineMaxWidth)
                local pos = _posDelegate == nil and let.position or _posDelegate(let)
                if ldp == nil then
                    ldp = {
                        pos.x - rect.width * 0.5,
                        pos.y - rect.height * 0.5,
                    }
                else
                    ldp = {
                        math.min(pos.x - rect.width * 0.5, ldp[1]),
                        math.min(pos.y - rect.height * 0.5, ldp[2]),
                    }
                end

                if urp == nil then
                    urp = {
                        pos.x + rect.width * 0.5,
                        pos.y + rect.height * 0.5,
                    }
                else
                    urp = {
                        math.max(pos.x + rect.width * 0.5, urp[1]),
                        math.max(pos.y + rect.height * 0.5, urp[2]),
                    }
                end
            end
        end
        if ldp == nil then
            return Amaz.Rect(0, 0, 0, 0)
        end
        local rect = Amaz.Rect(ldp[1], ldp[2], urp[1] - ldp[1], urp[2] - ldp[2])
        return rect
    end

    local lineRect = {}
    for i = 1, #lineSplit do
        local line = lineSplit[i]
        local rect = getLettersRect(line)
        table.insert(lineRect, rect)
    end
    return lineRect
end

function CPage:getOriRect(_posDelegate)
    if self.oriRect == nil then
        self.oriRect = self:getRect(_posDelegate)
    end
    return self.oriRect
end

function CPage:refresh()
    if not self:isLegal() then
        return
    end

    local function _syncWords()
        local curLegalWordCount = 0
        for i = 1, #self.words do
            local word = self.words[i]
            word:setIdxInPage(i)

            if word:isLegal() then
                curLegalWordCount = curLegalWordCount + 1
                word:setLegalIdxInPage(i)
            end

            local letters = word:getLetters()
            for j = 1, #letters do
                table.insert(self.letters, letters[j])
            end

            if i == #self.words then
                word:setNextWord(nil)
            end

            if i == #self.words then
                word:setLastWord(nil)
            end
        end
    end

    self.str = ""
    -- Amaz.LOGE("lrc line count 123", tostring(#self.lines))
    if #self.lines > 0 then
        self.words = {}

        for i = 1, #self.lines do
            local line = self.lines[i]
            line:setIdxInPage(i)
            line:refresh()
            self.str = self.str .. line:getStr()

            if i ~= #self.lines then
                self.str = self.str .. "\n"
            end

            local words = line:getWords()
            for j = 1, #words do
                local word = words[j]
                table.insert(self.words, word)
            end
        end

        _syncWords()
    else
        for i = 1, #self.words do
            local word = self.words[i]
            self.str = self.str .. word:getStr()
        end

        _syncWords()
    end
    -- Amaz.LOGI("lrc test " .. tostring(self:getIdx()), tostring(self.str))
    self:_setMaxLineFitable()

    self.startTime = self.words[1]:getStartTime()
    self.endTime = self.words[#self.words]:getEndTime()
end

function CPage:getDuration()
    if self:isLegal() then
        return self.endTime - self.startTime
    end
    return -1
end

function CPage:_syncStr()
    Common.flushCmd(Common.STICKER)
    Common.STICKER.richText.str = self:getStr()
    Common.STICKER.richText:forceTypeSetting()
end

function CPage:_syncLetters()
    local curLetters = Amaz.Vector()
    local curIdx = 0
    if #self.lines > 0 then
        for i = 1, #self.lines do
            local line = self.lines[i]
            local words = line:getWords()
            for j = 1, #words do
                local word = words[j]
                local letters = word:getLetters()
                -- word:setLetterIdxOffset(i - 1)
                for k = 1, #letters do
                    local letter = letters[k]
                    curLetters:pushBack(letter)
                    curIdx = curIdx + 1
                end
            end

            if i ~= #self.lines then
                local letter = Amaz.Letter()
                letter.utf8 = "\n"
                letter.letterStyle.fontSize = 0
                curLetters:pushBack(letter)
            end
        end
    else
        for i = 1, #self.words do
            local word = self.words[i]
            local letters = word:getLetters()
            for j = 1, #letters do
                curLetters:pushBack(letters[j])
            end
        end
    end

    Common.STICKER.richText.letters = curLetters
    Common.STICKER.richText:forceTypeSetting()
end

function CPage:setLineBeforeReadAnim(_func, _isEarlier)
    if #self.lines > 0 then
        for i = 1, #self.lines do
            self.lines[i]:setBeforeReadAnim(_func, _isEarlier)
        end
    end
end

function CPage:setLineReadingAnim(_func, _isEarlier)
    if #self.lines > 0 then
        for i = 1, #self.lines do
            self.lines[i]:setReadingAnim(_func, _isEarlier)
        end
    end
end

function CPage:setLineAfterReadAnim(_func, _isEarlier)
    if #self.lines > 0 then
        for i = 1, #self.lines do
            self.lines[i]:setAfterReadAnim(_func, _isEarlier)
        end
    end
end

function CPage:setWordBeforeReadAnim(_func, _isEarlier)
    if #self.words > 0 then
        for i = 1, #self.words do
            self.words[i]:setBeforeReadAnim(_func, _isEarlier)
        end
    end
end

function CPage:setWordReadingAnim(_func, _isEarlier)
    if #self.words > 0 then
        for i = 1, #self.words do
            self.words[i]:setReadingAnim(_func, _isEarlier)
        end
    end
end

function CPage:setWordAfterReadAnim(_func, _isEarlier)
    if #self.words > 0 then
        for i = 1, #self.words do
            self.words[i]:setAfterReadAnim(_func, _isEarlier)
        end
    end
end

function CPage:setReadingAnim(_func)
    self.readingAnim = _func
end

function CPage:setAfterReadAnim(_func)
    self.afterReadAnim = _func
end

function CPage:animing(_time)
    if self.enable == false then
        Common.flushCmd(Common.STICKER)
        Common.STICKER.richText.str = ""
        Common.STICKER.richText:forceTypeSetting()
        return
    end
    local t = _time

    if Common.STICKER.richText.str ~= self:getStr() then

    end

    self:_syncStr()
    self:_syncLetters()

    local p = (t - self.startTime) / (self.endTime - self.startTime)
    if self.readingAnim then
        self:readingAnim(p, self:getLetters(), _time)
    end

    -- line animations
    if #self.lines > 0 then
        for i = 1, #self.lines do
            self.lines[i]:animingEarlier(t)
        end

        for i = 1, #self.lines do
            local words = self.lines[i]:getWords()
            for j = 1, #words do
                words[j]:brushKeyWord()
            end
        end

        Common.STICKER.richText:forceTypeSetting()

        for i = 1, #self.lines do
            self.lines[i]:animing(t)
        end
    end

    -- word animations
    if self.words and #self.words > 0 then
        local readingWord = nil
        for i = 1, #self.words do
            local state = self.words[i]:getState(t)
            if state == Common.WORD_STATE.BEFORE_READ or state == Common.WORD_STATE.AFTER_READ then
                if self.words[i]:hasWholeAnimDelegate() then
                    self.words[i]:animing(t)
                end
            elseif state == Common.WORD_STATE.READING then
                if self.words[i]:hasWholeAnimDelegate() then
                    readingWord = self.words[i]
                end
            end
        end

        for i = 1, #self.words do
            if self.words[i]:hasWholeAnimDelegate() then
                self.words[i]:brushKeyWord()
            end
        end

        if readingWord then
            if readingWord:hasWholeAnimDelegate() then
                readingWord:animingEarlier(t)
                Common.STICKER.richText:forceTypeSetting()
                readingWord:animing(t)
            end
        end

    end

    if self.afterReadAnim then
        Common.STICKER.richText:forceTypeSetting()
        self:afterReadAnim(p, self:getLetters(), _time)
    end
end

function CPage:getReadingWord(_time)
    local t = _time
    local word = nil
    if self.words and #self.words > 0 then
        for i = 1, #self.words do
            local w = self.words[i]
            if w:getState(t) == Common.WORD_STATE.READING then
                word = w
                break
            end
        end
    end
    return word
end

function CPage:getReadingLine(_time)
    local t = _time
    local line = nil
    if self.lines and #self.lines > 0 then
        for i = 1, #self.lines do
            local l = self.lines[i]
            if l:getState(t) == Common.LINE_STATE.READING then
                line = l
                break
            end
        end
    end
    return line
end

function CPage:getInitialCenter()
    return self:getCenter(function(_l)
        return _l.initialPosition
    end)
end

function CPage:getCenter(_centerDelegate)
    if self:isLegal() == false then
        return Amaz.Vector2f(-1, -1)
    end

    local rect = self:getRect(_centerDelegate)
    local center = Amaz.Vector2f(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
    return center
end

function CPage:isLegal()
    return #self.words > 0 or #self.lines > 0
end

function CPage:destroy()
    self.words = {}
    self.str = ""
    self.letters = {}
    self.idx = -1
end

--[[  ****************************************
    CaptionModule: manager all the CPage
--    **************************************** ]]
local CaptionModule = {}
CaptionModule.__index = CaptionModule
CaptionModule.TEXT_STICKER = "TEXT_STICKER"

local json = cjson.new()

function CaptionModule.new(_sticker)
    local self = setmetatable({}, CaptionModule)

    self.sticker = _sticker
    Common.STICKER = self.sticker
    self.pages = {}
    self.words = {}
    return self
end

function CaptionModule:getLegalLetterCount()
    local count = 0
    for i = 1, self.captionInfo do
        local sw = self.captionInfo[i]
        if sw:Legal() then
            count = count + sw:getLetterCount()
        end
    end
    return count
end

function CaptionModule:getLetterCount()
    return Common.getLetterCount(Common.STICKER.richText.str)
end

-- custom change captionInfo
function CaptionModule:setCaptionInfoCustom(_customDelegate)
    if self.captionInfo == nil then
        self:splitSimpleCaption()
    end

    self.captionInfo = _customDelegate(self.captionInfo)
end

function CaptionModule:getDuration()
    local st, et = Common.getTimeRange(self.captionRawInfo.words)
    return et - st
end

function CaptionModule:isLegal()
    return not (self.captionRawInfo == nil)
end

function CaptionModule:registerMergeModule(_module)
    mergeModule = _module
end

function CaptionModule:init(captionInfo)
    if Common.isEditor then
        local rootDir = self.sticker.entity.scene.assetMgr.rootDir
        self.rawCaptionStr = Common.readStrFromFile(rootDir .. "test/caption_fake_data2.json")
    end

    if self.rawCaptionStr then
        self.captionRawInfo = json.decode(self.rawCaptionStr)
    end

    if captionInfo ~= nil then
        self.captionRawInfo = captionInfo
    end

    if mergeModule == nil and self.sticker.getModule then
        mergeModule = self.sticker:getModule("LetterStyleMergeModule")
    end
    if Common.isEditor then
        -- change text in editor
        Common.STICKER.richText.str = self.captionRawInfo.text
        Common.STICKER.richText:forceTypeSetting()
    end

    self.enable = true
    self.fitPageMaxLine = false
    self.extraLineSpacing = 1

    self.wordOverlapLeft = 0
    self.wordOverlapRight = 0
end

function CaptionModule:setRichText(_richText)
    if _richText == nil then
        return
    end
    -- self.sticker = _richText
    Common.STICKER = _richText
    -- self:init()
end

function CaptionModule:_getMergedLetters(_baseLetters)
    if mergeModule then
        return mergeModule:getMergedLetters(Common.STICKER)
    end
    return _baseLetters:clone()
end

function CaptionModule:initCaptionInfo(_extendLeft, _extendRight, _timeSpace)
    --[[    captionInfo include
        [
            {
                -- normal
                idx = 1,
                legal_idx = 1,
                text = "new",
                start_time = 0, ms
                end_time = 10,  ms
                ori_start_letter_idx = 1
                pack_start_time = 0,
                pack_end_time = 1,

                -- page about
                page_idx = 1

                -- line about
                line_idx = 1 or nil
            }
            ...
        ]
    ]]
    local extendRight = _extendRight == nil and 0.5 or Common.clamp(_extendRight, 0, 1)
    local extendLeft = _extendLeft == nil and 0.5 or Common.clamp(_extendLeft, 0, 1 - extendRight)
    local timeSpace = _timeSpace == nil and 1000 or _timeSpace

    local curLegalWordIdx = 1
    local curOriStartLetterIdx = 0

    local wordInfos = {}
    for i = 1, #self.captionRawInfo.words do
        local info = self.captionRawInfo.words[i]

        -- remove the illegal info if [first or last]
        local continue = false
        continue = (i == #self.captionRawInfo.words) and (Common.isLegalStr(info.text) == false)

        -- if first info.text == " " (stupid input)
        if (i == 1) and (Common.isLegalStr(info.text) == false) then
            continue = true
            curOriStartLetterIdx = curOriStartLetterIdx + Common.getLetterCount(info.text)
        end

        if not continue then
            info.idx = i
            info.pack_start_time = info.start_time
            info.pack_end_time = info.end_time

            if not Common.isLegalStr(info.text) then
                info.legal_idx = -1
            else
                info.legal_idx = curLegalWordIdx
                curLegalWordIdx = curLegalWordIdx + 1
            end
            info.ori_start_letter_idx = curOriStartLetterIdx
            table.insert(wordInfos, info)

            curOriStartLetterIdx = curOriStartLetterIdx + Common.getLetterCount(info.text)
        end
    end


    self.captionInfo = { words = {} }
    self.words = {}

    self.oriWordWrapWidth = Common.STICKER.richText.typeSettingParam.wordWrapWidth
    if Common.STICKER.richText.typeSettingParam.closestLineSpacing ~= nil then
        self.oriClosestLineSpacing = Common.STICKER.richText.typeSettingParam.closestLineSpacing
    end

    if self.fitPageMaxLine then
        -- self.oriWordWrapWidth = self.sticker.richText.typeSettingParam.wordWrapWidth
        Common.STICKER.richText.typeSettingParam.wordWrapWidth = 999999
        Common.STICKER.richText:forceTypeSetting()
    end

    local baseLetters = Common.STICKER.richText.letters:clone()
    local mergedLetters = self:_getMergedLetters(baseLetters)

    for i = 1, #wordInfos do
        local info = wordInfos[i]

        if info.legal_idx > 0 then
            local nextLegalFlag = true

            local idx = i + 1
            while nextLegalFlag and idx <= #wordInfos do
                local nextInfo = wordInfos[idx]
                nextLegalFlag = nextInfo.legal_idx < 0

                idx = idx + 1

                if nextLegalFlag then
                    nextInfo.pack_start_time = info.pack_end_time
                    nextInfo.pack_end_time = info.pack_end_time
                else
                    if nextInfo.start_time < info.end_time then
                        nextInfo.start_time = info.end_time
                    else
                        local duration = math.max(nextInfo.start_time - info.end_time, 0)
                        if duration > timeSpace then
                            local centerTime = (nextInfo.start_time + info.end_time) * 0.5
                            info.pack_end_time = info.end_time + (centerTime - info.end_time) * extendRight
                            nextInfo.pack_start_time = nextInfo.start_time - (centerTime - info.end_time) * extendLeft
                        else
                            info.pack_end_time = info.end_time + duration * extendRight
                            nextInfo.pack_start_time = nextInfo.start_time - duration * extendLeft
                        end
                    end
                end
            end
        end

        -- Amaz.LOGE("lrc caption init info ["..i, tostring(cjson.encode(info)))
        table.insert(self.captionInfo.words, info)

        local word = CWord.new(info)
        local letters = {}
        local mgdLetters = {}
        local oriStartLetterIdx = info.ori_start_letter_idx
        local lCount = Common.getLetterCount(word:getStr())
        for j = oriStartLetterIdx, oriStartLetterIdx + lCount - 1 do
            if j >= baseLetters:size() or j >= mergedLetters:size() then
                break
            end
            local letter = baseLetters:get(j)
            table.insert(letters, letter)

            local mgdLetter = mergedLetters:get(j)
            table.insert(mgdLetters, mgdLetter)
        end

        word:setLetters(letters)
        word:setOriLetters(mgdLetters)
        table.insert(self.words, word)
    end

    -- set the next word
    for i = 1, #self.words do
        local curWord = self.words[i]
        if curWord:isLegal() then
            local nextIdx = i
            local nextWord = nil
            while nextIdx <= #self.words and nextWord == nil do
                nextIdx = nextIdx + 1
                local next = self.words[nextIdx]
                if next and next:isLegal() then
                    nextWord = self.words[nextIdx]
                end
            end
            curWord:setNextWord(nextWord)


            local lastIdx = i
            local lastWord = nil
            while lastIdx > 1 and lastWord == nil do
                lastIdx = lastIdx - 1
                local last = self.words[lastIdx]
                if last and last:isLegal() then
                    lastWord = self.words[lastIdx]
                end
            end
            curWord:setLastWord(lastWord)
        end
    end
end

function CaptionModule:setWordOverlap(_left, _right)
    if _left < 0 then
        self.wordOverlapLeft = 0
    else
        self.wordOverlapLeft = math.min(_left, 1)
    end

    if _right < 0 then
        self.wordOverlapRight = 0
    else
        self.wordOverlapRight = math.min(_right, 1)
    end
end

function CaptionModule:_updateWordsOverlap(_overlapLeft, _overlapRight)
    for i = 1, #self.words do
        local word = self.words[i]

        local lastWord = word:getLastWord()
        local nextWord = word:getNextWord()

        if lastWord then
            word.startTime = word.packStartTime - (word.packStartTime - lastWord.packStartTime) * _overlapLeft
            word.startTime = math.max(word.startTime, lastWord.packStartTime)
        end

        if nextWord then
            word.endTime = word.packEndTime + (nextWord.packEndTime - word.packEndTime) * _overlapRight
            word.endTime = math.min(word.endTime, nextWord.endTime)
        end
    end
end

function CaptionModule:splitPage(_left, _right, _timeSpace)
    self:initCaptionInfo(_left, _right, _timeSpace)

    self:_updateWordsOverlap(self.wordOverlapLeft, self.wordOverlapRight)

    self.pages = {}
    local pageIdx = 1
    local page = CPage.new(pageIdx)

    local function getLinesLimitByWords(_wordCountList)
        local wordCountListIdx = 1
        local wordCountPerLineLimit = _wordCountList[wordCountListIdx]

        local lineList = {}
        local lineIdx = 1
        local lineObj = CLine.new(lineIdx, page)

        for i = 1, #self.words do
            local word = self.words[i]

            local curLineLegalWordCount = lineObj:getLegalWordCount()

            local curLineWidth = lineObj:getRect().width
            local fitWrapWidthFlag = curLineWidth + word:getRect().width + 50 > self.oriWordWrapWidth
            if curLineLegalWordCount < 1 then
                fitWrapWidthFlag = false
            end

            if curLineLegalWordCount >= wordCountPerLineLimit or fitWrapWidthFlag then
                lineObj:refresh()
                table.insert(lineList, lineObj)
                lineIdx = lineIdx + 1
                lineObj = CLine.new(lineIdx)

                wordCountListIdx = wordCountListIdx + 1
                wordCountPerLineLimit = _wordCountList[(wordCountListIdx - 1) % #_wordCountList + 1]
            end

            if lineObj:getWordCount() > 0 or word:isLegal() then
                lineObj:addWord(word)
                word:setLine(lineObj)
            end
        end

        if lineObj:isLegal() then
            lineObj:refresh()
            table.insert(lineList, lineObj)
        end

        return lineList
    end

    local function getLinesLimitByLetters(_letterCountList)
        local letterCountListIdx = 1
        local letterCountPerLineLimit = _letterCountList[letterCountListIdx]

        local lineList = {}
        local lineIdx = 1
        local lineObj = CLine.new(lineIdx, page)

        local curLetterCount = 0

        for i = 1, #self.words do
            local word = self.words[i]

            local wordLetterCount = word:getLetterCount()
            if word:isLegal() == false then
                wordLetterCount = 0
            end

            if lineObj:getWordCount() < 1 and word:isLegal() then
                lineObj:addWord(word)
                word:setLine(lineObj)
                curLetterCount = curLetterCount + wordLetterCount
            else
                if curLetterCount + wordLetterCount >= letterCountPerLineLimit then
                    lineObj:refresh()
                    table.insert(lineList, lineObj)
                    lineIdx = lineIdx + 1
                    lineObj = CLine.new(lineIdx)

                    curLetterCount = 0

                    letterCountListIdx = letterCountListIdx + 1
                    letterCountPerLineLimit = _letterCountList[(letterCountListIdx - 1) % #_letterCountList + 1]
                end

                if lineObj:getWordCount() > 0 or word:isLegal() then
                    lineObj:addWord(word)
                    word:setLine(lineObj)
                    curLetterCount = curLetterCount + wordLetterCount
                end
            end
        end

        if lineObj:isLegal() then
            lineObj:refresh()
            table.insert(lineList, lineObj)
        end

        return lineList
    end

    local function getPagesLimitByLines(_lines, _lineCountList)
        local lineCountIdx = 1
        local lineCountPerPageLimit = _lineCountList[lineCountIdx]
        local pageList = {}
        local curLineCount = 0
        for i = 1, #_lines do
            local line = _lines[i]

            if curLineCount >= lineCountPerPageLimit then
                -- page:refresh()
                table.insert(pageList, page)
                pageIdx = pageIdx + 1
                page = CPage.new(pageIdx)
                curLineCount = 0

                lineCountIdx = lineCountIdx + 1
                lineCountPerPageLimit = _lineCountList[(lineCountIdx - 1) % #_lineCountList + 1]
            end

            page:addLine(line)
            line:setPage(page)
            curLineCount = curLineCount + 1
        end

        if page:isLegal() then
            -- page:refresh()
            table.insert(pageList, page)
        end
        return pageList
    end

    return {
        Default = function()
            for i = 1, #self.words do
                local w = self.words[i]
                w:setPage(page)
                page:addWord(w)
            end
            -- page:refresh()
            table.insert(self.pages, page)
        end,

        wordCountPerPageLimit = function(_wordCountPerPage)
            local wordCount = _wordCountPerPage == nil and 1 or _wordCountPerPage

            local curLegalWordCount = 0

            for i = 1, #self.words do
                local word = self.words[i]

                if curLegalWordCount >= wordCount then
                    -- page:refresh()
                    table.insert(self.pages, page)
                    pageIdx = pageIdx + 1
                    page = CPage.new(pageIdx)
                    curLegalWordCount = 0
                end

                if page:getWordCount() > 0 or word:isLegal() then
                    if word:isLegal() then
                        curLegalWordCount = curLegalWordCount + 1
                    end
                    page:addWord(word)
                    word:setPage(page)
                end
            end

            if page:isLegal() then
                -- page:refresh()
                table.insert(self.pages, page)
            end
        end,

        LetterCountPerPageLimit = function(_letterCountPerPage, _percent)
            -- include illegal letter
            local letterCountLimit = _letterCountPerPage == nil and 1 or _letterCountPerPage
            local percent = _percent == nil and 0 or _percent
            percent = Common.clamp(percent, 0, 1)

            local curLetterCount = 0

            local newPageFlag = false
            for i = 1, #self.words do
                local word = self.words[i]

                if page:getLegalLetterCount() > 0 then
                    if word:isLegal() == false then
                        local nextIdx = i + 1
                        local nextLegalWord = nil
                        while nextIdx <= #self.words and nextLegalWord == nil do
                            if self.words[nextIdx]:isLegal() then
                                nextLegalWord = self.words[nextIdx]
                            end
                            nextIdx = nextIdx + 1
                        end

                        local curPageLetterCount = page:getLetterCount()

                        if nextLegalWord then
                            if (letterCountLimit - curPageLetterCount) < nextLegalWord:getLetterCount() * percent then
                                newPageFlag = true
                            end
                        end
                    else
                        local curPageLetterCount = page:getLetterCount()
                        if (letterCountLimit - curPageLetterCount) < word:getLetterCount() * percent then
                            newPageFlag = true
                        end
                    end
                end

                if newPageFlag then
                    if page:isLegal() then
                        -- page:refresh()
                        table.insert(self.pages, page)
                        pageIdx = pageIdx + 1
                        page = CPage.new(pageIdx)
                        newPageFlag = false
                    end
                    curLetterCount = 0
                else
                    curLetterCount = curLetterCount + word:getLetterCount()
                end

                if curLetterCount > 0 or word:isLegal() then
                    page:addWord(word)
                end
            end

            if page:isLegal() then
                -- page:refresh()
                table.insert(self.pages, page)
            end
        end,

        LineCountLimit = function(_wordCountPerLine, _lineCountPerPage)
            -- handle all the line
            local wordCountPerLineLimit = _wordCountPerLine == nil and 1 or _wordCountPerLine

            local lines = getLinesLimitByWords({ wordCountPerLineLimit })
            -- handle all the page
            local lineCountPerPageLimit = _lineCountPerPage == nil and 2 or _lineCountPerPage

            local pages = getPagesLimitByLines(lines, { lineCountPerPageLimit })
            for i = 1, #pages do
                table.insert(self.pages, pages[i])
            end
        end,

        singleLineInPageByRowth = function()
            local wordCountPerLine = {}
            local curRowth = -1
            for i = 1, #self.words do
                local word = self.words[i]
                if word:isLegal() then
                    local rowth = word:getOriLetters()[1].rowth + 1
                    if curRowth < 0 then
                        curRowth = rowth
                        wordCountPerLine[#wordCountPerLine + 1] = 0
                    elseif curRowth ~= rowth then
                        wordCountPerLine[#wordCountPerLine + 1] = 0
                    end
                    wordCountPerLine[#wordCountPerLine] = wordCountPerLine[#wordCountPerLine] + 1
                end
            end
            local lines = getLinesLimitByWords(wordCountPerLine)
            local pages = getPagesLimitByLines(lines, { 1 })
            for i = 1, #pages do
                table.insert(self.pages, pages[i])
            end
        end,

        letterCountPerLineLimitInPage = function(_letterCountPerLine, _lineCountPerPage)
            -- handle all the line
            local letterCountPerLineLimit = _letterCountPerLine

            local lines = getLinesLimitByLetters({ letterCountPerLineLimit })

            -- handle all the page
            local lineCountPerPageLimit = _lineCountPerPage == nil and 2 or _lineCountPerPage

            local pages = getPagesLimitByLines(lines, { lineCountPerPageLimit })
            for i = 1, #pages do
                table.insert(self.pages, pages[i])
            end
        end,

        customWordLineCountLimit = function(_wordLineList2D)
            local wordCountList = {}
            local lineCountList = {}
            for i = 1, #_wordLineList2D do
                for j = 1, #_wordLineList2D[i] do
                    table.insert(wordCountList, _wordLineList2D[i][j])
                end
                table.insert(lineCountList, #_wordLineList2D[i])
            end

            local lines = getLinesLimitByWords(wordCountList)

            local pages = getPagesLimitByLines(lines, lineCountList)
            for i = 1, #pages do
                table.insert(self.pages, pages[i])
            end
        end

    }
end

function CaptionModule:getReadingPage(_time)
    local t = _time

    local page = nil
    if self.pages and #self.pages > 0 then
        for i = 1, #self.pages do
            local p = self.pages[i]
            if p:isReading(t) and p:getEnable() then
                page = p
                break
            end
        end
    end

    return page
end

function CaptionModule:setFitMaxLine(_fitFlag)
    self.fitPageMaxLine = _fitFlag
end

function CaptionModule:setExtraLineSpacing(_lineSpace)
    self.oriLineSpacing = Common.STICKER.richText.typeSettingParam.lineSpacing
    self.extraLineSpacing = _lineSpace
end

function CaptionModule:animing(_time)
    if self.enable == false then
        return
    end

    local t = _time

    local page = self:getReadingPage(t)
    if page then
        page:animing(t)
        return page, page:getReadingWord(t), page:getReadingLine(t)
    end

    Common.flushCmd(Common.STICKER)
    Common.STICKER.richText.str = ""
end

function CaptionModule:onSetProperty(key, value)
    if key == "caption_duration_info" and value ~= "" then
        Amaz.LOGE("lrc onSetProperty", value)
        self.rawCaptionStr = value
    end
end

function CaptionModule:setEnable(_val)
    self.enable = _val
end

function CaptionModule:refresh()
    if self.fitPageMaxLine == true then
        if Common.STICKER.richText.typeSettingParam.closestLineSpacing ~= nil then
            Common.STICKER.richText.typeSettingParam.closestLineSpacing = true
        end
        for i = 1, #self.pages do
            self.pages[i]:setFitMaxLine(self.fitPageMaxLine)
            self.pages[i]:setExtraLineSpacing(self.extraLineSpacing)
            self.pages[i]:setOriLineSpacing(self.oriLineSpacing)
        end
    end

    for i = 1, #self.pages do
        self.pages[i]:refresh()
    end
end

function CaptionModule:getPages()
    self:refresh()
    return self.pages
end

function CaptionModule:getPagesWithoutRefresh()
    return self.pages
end

function CaptionModule:getWords()
    return self.words
end

function CaptionModule:getWordCount()
    return #self.words
end

function CaptionModule:getLegalWordCount()
    local count = 0
    for i = 1, #self.words do
        local w = self.words[i]
        if w:isLegal() then
            count = count + 1
        end
    end
    return count
end

function CaptionModule:getCaptionRawInfo()
    return self.captionRawInfo
end

function CaptionModule:seek(_time)
end

function CaptionModule:reset()
    self.captionRawInfo = {}
    self.captionInfo = { words = {} }
    self.pages = {}
    self.words = {}
    mergeModule = nil
    -- self.sticker = nil
    -- Common.STICKER = nil

    if self.oriWordWrapWidth then
        Common.STICKER.richText.typeSettingParam.wordWrapWidth = self.oriWordWrapWidth
    end

    if self.oriClosestLineSpacing ~= nil then
        Common.STICKER.richText.typeSettingParam.closestLineSpacing = self.oriClosestLineSpacing
    end

    if self.oriLineSpacing then
        Common.STICKER.richText.typeSettingParam.lineSpacing = self.oriLineSpacing
    end
end

return CaptionModule
