--[[
    Script Name: ComToolsModule.lua
    Version: 1.3
    Update Data: 2024.07.26
--]]

--[[  ****************************************
    Common Func
--    **************************************** ]]

local Common = {
    json = cjson.new(),
    isEditor = (Amaz.Macros and Amaz.Macros.EditorSDK) and true or false,
    STICKER = nil,
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
}

local LetterGroup = {}
LetterGroup.__index = LetterGroup
function LetterGroup.new()
    local self = setmetatable({}, LetterGroup)
    self.letters = {}
    self.oriLetters = {}
    return self
end

function LetterGroup:getLetters()
    return self.letters
end

function LetterGroup:getOriLetters()
    return self.oriLetters
end

function LetterGroup:addLetter(letter, oriLetter)
    table.insert(self.letters, letter)
    table.insert(self.oriLetters, oriLetter)
end

function LetterGroup:getLegalLetterCount()
    self.legalCount = 0
    for i = 1, #self.letters do
        local letter = self.letters[i]
        if Common.isLegalStr(letter.utf8) then
            self.legalCount = self.legalCount + 1
        end
    end
    return self.legalCount
end

function LetterGroup:getCount()
    return #self.letters
end

function LetterGroup:isEmpty()
    return not (self:getLegalLetterCount() > 0)
end

function LetterGroup:getRect(_posDelegate)
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

function LetterGroup:getInitialCenter()
    return self:getCenter(function(_l)
        return _l.initialPosition
    end)
end

function LetterGroup:getCenter(_centerDelegate)
    if self.letters == nil then
        return Amaz.Vector2f(-1, -1)
    end

    local rect = self:getRect(_centerDelegate)
    local center = Amaz.Vector2f(rect.x + rect.width * 0.5, rect.y + rect.height * 0.5)
    return center
end

--[[  ****************************************
    ComToolsModule: common tools func
--    **************************************** ]]  
local ComToolsModule = {}
ComToolsModule.__index = ComToolsModule
ComToolsModule.TEXT_STICKER = "TEXT_STICKER"

--- new func of module
--- receives the 'self' variable of the original script as an argument
function ComToolsModule.new(_sticker)
    local self = setmetatable({}, ComToolsModule)
    self.sticker = _sticker
    Common.STICKER = _sticker
    self.letterGroupList = {}
    return self
end

function ComToolsModule:remap01(a, b, x)
    if x < a then return 0 end
    if x > b then return 1 end
    return self:clamp((x - a) / (b - a), 0, 1)
end

function ComToolsModule:mix(a, b, x)
    return a * (1 - x) + b * x
end

function ComToolsModule:clamp(x, a, b)
    return math.max(math.min(x, b), a)
end

function ComToolsModule:soomthstep(a, b, x)
    x = self:remap01(a, b, x)
    return x * x * (3.0 - 2.0 * x)
end

function ComToolsModule:invSoomthstep(a, b, x)
    x = self:remap01(a, b, x)
    return 0.5 - math.sin(math.asin(1.0 - 2.0 * x) / 3.0)
end

function ComToolsModule:soomthstepQuartic(a, b, x)
    x = self:remap01(a, b, x)
    return x * x * (2.0 - x * x)
end

function ComToolsModule:invSoomthstepQuartic(a, b, x)
    x = self:remap01(a, b, x)
    return math.sqrt(1.0 - math.sqrt(1.0 - x))
end

function ComToolsModule:invSoomthstepQuintic(a, b, x)
    x = self:remap01(a, b, x)
    return x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
end

function ComToolsModule:smax(a, b, k)
    local h = self:clamp((b - a) / k + 0.5,0.,1.)
    return self:mix(a,b,h) + h * (1. - h) * k * .5
end

function ComToolsModule:smin(a, b, k)
    local h = self:clamp((b - a) / k + 0.5,0.,1.)
	return self:mix(b,a,h) - h * (1. - h) * k * .5
end

function ComToolsModule:readStrFromFile(file_path)
    local file = io.input(file_path)
    local text = io.read("*a")
    return text
end

function ComToolsModule:readJson(file_path)
    local jsonData = self:readStrFromFile(file_path)
    if jsonData then
        jsonData = Common.json.decode(jsonData)
    end
    return jsonData
end

function ComToolsModule:GetStringWordNum(str)
    return Common.getLetterCount(str)
end

---- return 2 when the char is fullwidth, else return 1
function ComToolsModule:charIsFullWidth(char)
    local charCode = string.byte(char)
    if charCode ~= nil then
        if (charCode >= 0x0020 and charCode <= 0x007E) or
        (charCode >= 0xFF61 and charCode <= 0xFF9F) then
            return 1
        end
    end
    return 2
end

---- return true when the char is chinese, else return false
function ComToolsModule:charIsChinese(char)
    local byte = string.byte(char, 1)
    if byte ~= nil then
        if byte >= 228 and byte <= 233 then
            local byte2 = string.byte(char, 2)
            if byte2 ~= nil and byte2 >= 128 and byte2 <= 191 then
                return true -- 
            end
        end
    end
    return false -- 
end

function ComToolsModule:charIncludedInList(char, list)
    for k, v in pairs(list) do
        if v == char then
            return true
        elseif self:GetStringWordNum(tostring(v)) > 1 and string.match(char, tostring(v)) then
            return true
        end
    end
    return false
end

function ComToolsModule:letterGroupBy(sticker)

    local mergeGroup = function(srcGroup, disGroup)
        for i = 1, srcGroup:getCount() do
            local letter = srcGroup.letters[i]
            local oriLetter = srcGroup.oriLetters[i]
            disGroup:addLetter(letter, oriLetter)
        end
        return disGroup
    end

    if sticker == nil then
        sticker = Common.STICKER
    end
    local letters = sticker.richText.letters
    local oriLetters = nil
    if sticker.oriLetters then
        oriLetters = sticker.oriLetters
    else
        oriLetters = sticker.richText.letters:clone()
        sticker.oriLetters = oriLetters
    end
    return {
        letterCountLimit = function(countPerGroup, exclusionCharsList, minLength)
            if letters:size() <= 0 then
                return
            end
            if exclusionCharsList == nil then
                exclusionCharsList = {}
            elseif type(exclusionCharsList) == "string" then
                exclusionCharsList = {exclusionCharsList}
            end
            if minLength == nil then
                minLength = 1
            end

            local retGroup = {}
            local lastRowth = letters:get(0).rowth
            local letterGroup = LetterGroup.new()
            local lastGroup = nil
            for i = 0, letters:size() - 1 do
                local letter = letters:get(i)
                local oriLetter = oriLetters:get(i)
                if lastRowth ~= letter.rowth then
                    Amaz.LOGI("qdy", 1)
                    if letterGroup:getLegalLetterCount() >= math.min(countPerGroup, minLength) then
                        table.insert(retGroup, letterGroup)
                        lastGroup = nil
                        letterGroup = LetterGroup:new()
                    elseif letterGroup:isEmpty() == false and lastGroup ~= nil then
                        mergeGroup(letterGroup, lastGroup)
                        lastGroup = nil
                        letterGroup = LetterGroup:new()
                    end
                    lastRowth = letter.rowth
                end

                if #exclusionCharsList >= 0 and self:charIncludedInList(letter.utf8, exclusionCharsList) == false then
                    letterGroup:addLetter(letter, oriLetter)
                    if letterGroup:getLegalLetterCount() >= countPerGroup then
                        table.insert(retGroup, letterGroup)
                        lastGroup = letterGroup
                        letterGroup = LetterGroup:new()
                    end
                end
            end

            if letterGroup:getLegalLetterCount() >= math.min(countPerGroup, minLength) then
                table.insert(retGroup, letterGroup)
            elseif letterGroup:isEmpty() == false and lastGroup ~= nil then
                mergeGroup(letterGroup, lastGroup)
            end
            
            retGroup.sticker = sticker
            table.insert(self.letterGroupList, retGroup)
            return retGroup
        end,
        splitBySymbol = function(countPerGroup, symbol, exclusionCharsList, minLength)
            if letters:size() <= 0 then
                return
            end
            if exclusionCharsList == nil then
                exclusionCharsList = {}
            elseif type(exclusionCharsList) == "string" then
                exclusionCharsList = {exclusionCharsList}
            end
            if symbol == nil then
                symbol= {" "}
            elseif type(symbol) == "string" then
                symbol = {symbol}
            end

            if minLength == nil then
                minLength = 1
            end

            local counter = 0
            local retGroup = {}
            local lastRowth = letters:get(0).rowth
            local letterGroup = LetterGroup.new()
            local lastGroup = nil
            for i = 0, letters:size() - 1 do
                local letter = letters:get(i)
                local oriLetter = oriLetters:get(i)
                if lastRowth ~= letter.rowth then
                    if counter >= countPerGroup then
                        table.insert(retGroup, letterGroup)
                        lastGroup = letterGroup
                        letterGroup = LetterGroup:new()
                    elseif letterGroup:getLegalLetterCount() > minLength and letterGroup:isEmpty() == false and lastGroup ~= nil then
                        mergeGroup(letterGroup, lastGroup)
                        lastGroup = nil
                        letterGroup = LetterGroup:new()
                    end
                    counter = 0
                    lastRowth = letter.rowth
                end

                if #exclusionCharsList <= 0 or (#exclusionCharsList > 0 and self:charIncludedInList(letter.utf8, exclusionCharsList) == false) then
                    letterGroup:addLetter(letter, oriLetter)
                end
                if self:charIncludedInList(letter.utf8, symbol) then
                    counter = counter + 1
                    if counter >= countPerGroup then
                        table.insert(retGroup, letterGroup)
                        lastGroup = letterGroup
                        letterGroup = LetterGroup:new()
                        counter = 0
                    end
                end
            end

            if counter >= (countPerGroup - 1) then
                table.insert(retGroup, letterGroup)
            elseif letterGroup:getLegalLetterCount() > minLength and letterGroup:isEmpty() == false and lastGroup ~= nil then
                mergeGroup(letterGroup, lastGroup)
            end
            retGroup.sticker = sticker
            table.insert(self.letterGroupList, retGroup)
            return retGroup
        end,
        lineCountLimit = function(countPerGroup, exclusionCharsList, minLength)
            if letters:size() <= 0 then
                return
            end
            if exclusionCharsList == nil then
                exclusionCharsList = {}
            elseif type(exclusionCharsList) == "string" then
                exclusionCharsList = {exclusionCharsList}
            end

            if minLength == nil then
                minLength = 1
            end

            local counter = 0
            local retGroup = {}
            local lastRowth = letters:get(0).rowth
            local letterGroup = LetterGroup.new()
            local lastGroup = nil
            for i = 0, letters:size() - 1 do
                local letter = letters:get(i)
                local oriLetter = oriLetters:get(i)
                if lastRowth ~= letter.rowth then
                    counter = counter + 1
                    if counter >= countPerGroup then
                        table.insert(retGroup, letterGroup)
                        lastGroup = letterGroup
                        letterGroup = LetterGroup:new()
                        counter = 0
                    elseif letterGroup:getLegalLetterCount() > minLength and letterGroup:isEmpty() == false and lastGroup ~= nil then
                        mergeGroup(letterGroup, lastGroup)
                        letterGroup = LetterGroup:new()
                    end
                    lastRowth = letter.rowth
                end

                if #exclusionCharsList >= 0 and self:charIncludedInList(letter.utf8, exclusionCharsList) == false then
                    letterGroup:addLetter(letter, oriLetter)
                end
            end


            if counter >= countPerGroup then
                table.insert(retGroup, letterGroup)
            elseif letterGroup:getLegalLetterCount() > minLength and letterGroup:isEmpty() == false and lastGroup ~= nil then
                mergeGroup(letterGroup, lastGroup)
            end
            retGroup.sticker = sticker
            table.insert(self.letterGroupList, retGroup)
            return retGroup
        end
    }
end

function ComToolsModule:group2CaptionInfo(group, duration, curveFunc)
    local sticker = group.sticker
    if curveFunc == nil then
        curveFunc = function(x)
            return x
        end
    end
    local captionInfo = {}
    captionInfo.text = sticker.richText.str
    local words = {}
    local dt = duration * 1000
    for i = 1, #group do
        local startTime = dt * curveFunc((i - 1) / #group)
        local endTime = dt * curveFunc(i / #group)
        local letters = group[i]:getLetters()
        local word = {}
        word.end_time = endTime
        word.start_time = startTime
        local subText = ""
        local count = #letters - 1
        if i == #group then
            count = #letters
        end
        for j = 1, count do
            local letter = letters[j]
            subText = subText .. letter.utf8
        end
        word.text = subText
        table.insert(words, word)
        if i < #group then
            local word = {}
            word.end_time = endTime
            word.start_time = endTime
            word.text = letters[#letters].utf8
            table.insert(words, word)
        end
    end
    captionInfo.words = words
    return captionInfo

end
--- init the script data (optional)
function ComToolsModule:init()
end

--- seek per frame (optional)
function ComToolsModule:seek()
end

--- reset the script data (optional)
function ComToolsModule:reset()
end

return ComToolsModule