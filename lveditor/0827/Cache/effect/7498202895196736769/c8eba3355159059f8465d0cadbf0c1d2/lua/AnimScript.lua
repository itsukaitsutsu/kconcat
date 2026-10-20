local isEditor = (Amaz.Macros and Amaz.Macros.EditorSDK) and true or false
local exports = exports or {}
local AnimScript = AnimScript or {}

---@class AnimScript : ScriptComponent
---@field duration number
---@field progress number [UI(Range={0.0, 1.0}, Slider)]
---@field auto_play boolean
---@field start_p number [UI(Range={0.0, 1.0}, Slider)]
---@field end_p number [UI(Range={0.0, 1.0}, Slider)]
---@field offset number [UI(Range={-1.0, 1.0}, Slider)]
---@field shapeType number [UI(Range={0.0, 5.0}, Slider)]
---@field animtype number [UI(Range={0.0, 4.0}, Slider)]
---
local ANIMATIONTYPE = {
    IN = 1, --入场动画
    OUT = 2, --出场动画
    LOOP = 3, --循环动画
    SUBTITLE = 4 --字幕动画
}

local ae_attribute = {
	["ADBE_Position_0_0"]={
		{{0.166667, 0.166667, 0.14, 1, }, {1, 21, }, {{263.090942383, 371.899291992, 0, }, {263.090942383, 251.899291992, 0, }, {263.090942383, 351.899291992, 0, }, {263.090942383, 271.899291992, 0, }, }, {6413, }, {0, }, }, 
	}, 
	["ADBE_Opacity_0_1"]={
		{{0.166666667, 0.166666667, 0.833333333, 0.833333333, }, {0, 1, }, {{0, }, {100, }, }, {6417, }, {1, }, }, 
	}, 
	["ADBE_Position_1_2"]={
		{{0.166667, 0.166667, 0.14, 1, }, {6, 26, }, {{338.969848633, 372.485229492, 0, }, {338.969848633, 252.485229492, 0, }, {338.969848633, 352.485229492, 0, }, {338.969848633, 272.485229492, 0, }, }, {6413, }, {0, }, }, 
	}, 
	["ADBE_Opacity_1_3"]={
		{{0.166666667, 0.166666667, 0.833333333, 0.833333333, }, {5, 6, }, {{0, }, {100, }, }, {6417, }, {1, }, }, 
	}, 
	["ADBE_Position_2_4"]={
		{{0.166667, 0.166667, 0.14, 1, }, {11, 31, }, {{413.676879883, 372.485229492, 0, }, {413.676879883, 252.485229492, 0, }, {413.676879883, 352.485229492, 0, }, {413.676879883, 272.485229492, 0, }, }, {6413, }, {0, }, }, 
	}, 
	["ADBE_Opacity_2_5"]={
		{{0.166666667, 0.166666667, 0.833333333, 0.833333333, }, {10, 11, }, {{0, }, {100, }, }, {6417, }, {1, }, }, 
	}, 
	["ADBE_Position_3_6"]={
		{{0.166667, 0.166667, 0, 1, }, {16, 36, }, {{480.180786133, 371.606323242, 0, }, {480.180786133, 181.606323242, 0, }, {480.180786133, 339.939657211116, 0, }, {480.180786133, 181.606323242, 0, }, }, {6413, }, {0, }, }, 
		{{1, 0, 0.833333, 0.833333, }, {36, 56, }, {{480.180786133, 181.606323242, 0, }, {480.180786133, 251.606323242, 0, }, {480.180786133, 181.606323242, 0, }, {480.180786133, 251.606323242, 0, }, }, {6413, }, {0, }, }, 
	}, 
	["ADBE_Rotate_Z_3_7"]={
		{{0.015, 0.213203433, 0.09, 1, }, {16, 36, }, {{180, }, {360, }, }, {6417, }, {0, }, }, 
	}, 
	["ADBE_Opacity_3_8"]={
		{{0.166666667, 0.166666667, 0.833333333, 0.833333333, }, {15, 16, }, {{0, }, {100, }, }, {6417, }, {1, }, }, 
	}, 
	["ADBE_Position_4_9"]={
		{{0.166667, 0.166667, 0, 1, }, {21, 41, }, {{561.626098633, 371.12109375, 0, }, {561.626098633, 181.12109375, 0, }, {561.626098633, 339.454427719116, 0, }, {561.626098633, 181.12109375, 0, }, }, {6413, }, {0, }, }, 
		{{1, 0, 0.833333, 0.833333, }, {41, 61, }, {{561.626098633, 181.12109375, 0, }, {561.626098633, 251.12109375, 0, }, {561.626098633, 181.12109375, 0, }, {561.626098633, 251.12109375, 0, }, }, {6413, }, {0, }, }, 
	}, 
	["ADBE_Rotate_Z_4_10"]={
		{{0.015, 0.213203433, 0.09, 1, }, {21, 41, }, {{180, }, {360, }, }, {6417, }, {0, }, }, 
	}, 
	["ADBE_Opacity_4_11"]={
		{{0.166666667, 0.166666667, 0.833333333, 0.833333333, }, {20, 21, }, {{0, }, {100, }, }, {6417, }, {1, }, }, 
	}, 
	["ADBE_Position_5_12"]={
		{{0.166667, 0.166667, 0, 1, }, {26, 46, }, {{637.55078125, 372.050354004, 0, }, {637.55078125, 182.050354004, 0, }, {637.55078125, 372.050354004, 0, }, {637.55078125, 182.050354004, 0, }, }, {6413, }, {0, }, }, 
		{{1, 0, 0.833333, 0.833333, }, {46, 66, }, {{637.55078125, 182.050354004, 0, }, {637.55078125, 252.050354004, 0, }, {637.55078125, 182.050354004, 0, }, {637.55078125, 252.050354004, 0, }, }, {6413, }, {0, }, }, 
	}, 
	["ADBE_Rotate_Z_5_13"]={
		{{0.015, 0.213203433, 0.09, 1, }, {26, 46, }, {{180, }, {360, }, }, {6417, }, {0, }, }, 
	}, 
	["ADBE_Opacity_5_14"]={
		{{0.166666667, 0.166666667, 0.833333333, 0.833333333, }, {25, 26, }, {{0, }, {100, }, }, {6417, }, {1, }, }, 
	}, 
}


AnimScript.__index = AnimScript
function AnimScript.new()
    local self = {}
    self = setmetatable({}, AnimScript)

    self.transform = {}
    self.first = true
    self.duration = 1.0
    self.cur_time = 0.0
    self.progress = 0
    self.auto_play = false
    self.renderer = nil
    self.material = nil
    self.start_p = 0
    self.end_p = 1
    self.offset = 0
    self.shapeType = 0
    self.animtype  = 1

    self.util = includeRelativePath("Util")

    return self
end

---@param comp Component
function AnimScript:onStart(comp)
    self.textSelector = includeRelativePath("TextSelector"):new(self.util)
    self.comp = comp
    self.text = comp.entity:getComponent('SDFText')
    self.rich_text = comp.entity:getComponent('Text')
    self.trans = comp.entity:getComponent('Transform')
    self.parentTrans = self.trans.parent

    self.textEntity=comp.entity
    if self.text == nil then
        local text = comp.entity:getComponent('Text')
        if text ~= nil then
            self.text = comp.entity:addComponent('SDFText')
            self.text:setTextWrapper(text)
        end
    end
    --确保首帧拿到正确的chars:chout,rowth信息
    self.text:forceTypeSetting() 
    self.first = true
end


---@param comp Component
---@param deltaTime number
function AnimScript:onUpdate(comp, deltaTime)
    if isEditor then
        if self.auto_play then
            self.cur_time = self.cur_time + deltaTime
        end
        self:seek(self.cur_time)
    else
        self.cur_time = 0
    end
end


function AnimScript:getProgress(animtype,time)
    local progress = 0
    local duration = self.duration
    duration = duration <= 0 and 0.0001 or duration
    if animtype == ANIMATIONTYPE.IN then
        progress = self.util.clamp(0,1,time/duration)
    elseif animtype == ANIMATIONTYPE.OUT then
        --一般入场出场时间反向
        progress = self.util.clamp(0,1,1.0 - time/duration)
    elseif animtype == ANIMATIONTYPE.LOOP then
        progress = (time % duration) / duration
    elseif animtype == ANIMATIONTYPE.SUBTITLE then
        --字幕动画进度
        progress = self:getSubTitleProgress(time)
    end
    return progress
end


function AnimScript:isvalidletter(letter)
    local code = letter.utf8code or letter.utf8
    if code == ' ' or code == '\r' or code == '\n' then
        return false
    end
    return true
end

--获得字号
function AnimScript:getFontSize()
    local letters = self.rich_text.letters
    local fontsize = 8
    if letters:size() > 0 then
        local letter0 = letters:get(0)
        fontsize = letter0.letterStyle.fontSize
    end
    return fontsize
end

function AnimScript:getLineCount()
    local line_row = 0
	local len = self.count
	for i = 1, len do
		local letter = self.rich_text.letters:get(i - 1)
        if letter.rowth > line_row then
            line_row = letter.rowth
        end
	end
    return line_row + 1
end


function AnimScript:getTabLettersSplitByLine()
    local lineLetterTab = {}
    for i = 1, self.count do
        local letter = self.rich_text.letters:get(i - 1)
        local rowth = letter.rowth + 1
        if not lineLetterTab[rowth] then
            lineLetterTab[rowth] =  {}
        end
        if letter.utf8code ~= " " then
            table.insert(lineLetterTab[rowth],{letter})
        end
    end

end

function AnimScript:getTabSplitByWord()
    local word = {}
    local wordTab = {}
    local isValidLetter = function(letter)
        if letter.utf8code ~= " " and letter.utf8code ~= "\n" then
            return true
        else
            return false
        end
    end
    for i = 1, self.count do
        local letter = self.rich_text.letters:get(i - 1)
        if isValidLetter(letter) then
            table.insert(word, letter)
        elseif #letter.utf8code > 0 then
            table.insert(wordTab, letter)
            word = {}
        end
    end
    if #word > 0 then
        table.insert(wordTab, word)
    end
    return wordTab
end


function AnimScript:seek(time)
    if isEditor then
        if self.auto_play then
            self.progress = (time %self.duration) / self.duration
        end
    else
        self.progress = self:getProgress(ANIMATIONTYPE.IN,time)
    end
    if self.first then
        self.count =self.rich_text.letters:size()
        local getAvgSize = function()
            local totalsize = 0
            for i = 1, self.count do
                local char = self.text.chars:get(i-1)
                totalsize = char.width + totalsize
            end
            return totalsize/self.count
        end
        self.avgSize = getAvgSize()

        local getValidLetterCount = function()
            local count = 0
            for i = 1, self.count do
                local letter = self.text.chars:get(i-1)
                if self:isvalidletter(letter) then
                    count = count + 1
                end
            end
            return count
        end
        self.validCount = getValidLetterCount()
        self.first = false
    end
--------------------------动画逻辑代码区----------------------
    local getMaxLineCount = function(lineCharTab)
        local maxCount = 0
        for index, lineInfo in ipairs(lineCharTab) do
            local num  = #lineInfo
            maxCount = maxCount < num and num or maxCount
        end
        return maxCount
    end

    local typeSettingKind = self.rich_text.typeSettingParam.typeSettingKind
    self.count =self.rich_text.letters:size()

    local step_off1 = -self.avgSize
    local step_off2 = self.avgSize

    if self.count > 0 then
        if self.validCount <= 1 then 
            local totalFrame = 41
            local step1_frame = 21
            local offset = 0
            local curframe = math.floor(totalFrame * self.progress)
            if curframe <= step1_frame then
                local pt = curframe/step1_frame
                local cur_pt = self.util.bezier({0.166667, 0.166667, 0, 1, })(pt,0,1,1)
                offset = self.util.mix(step_off1,step_off2,cur_pt)
            else
                local pt = (curframe - step1_frame)/(totalFrame - step1_frame)
                local cur_pt = self.util.bezier({1, 0, 0.833333, 0.833333, })(pt,0,1,1)
                offset = self.util.mix(step_off2,0,cur_pt)
            end
            local pt = curframe/step1_frame
            local rot_pt = self.util.bezier({0.015, 0.213203433, 0.09, 1, })(pt,0,1,1)
            local rot = self.util.mix(-180,-360,rot_pt)
            local opt_pt = self.util.clamp(0,1,curframe)
            local opt_pt = self.util.bezier({0.166666667, 0.166666667, 0.833333333, 0.833333333,})(opt_pt,0,1,1)
            local opt = self.util.mix(0,1,opt_pt)
            for i = 1, self.count do
                local letter = self.rich_text.letters:get(i-1)
                if self:isvalidletter(letter) then
                    if typeSettingKind == 1 then
                        letter.position = Amaz.Vector2f(letter.initialPosition.x + offset,letter.initialPosition.y)
                    else
                        letter.position = Amaz.Vector2f(letter.initialPosition.x ,letter.initialPosition.y+offset)
                    end
                    letter.rotate = Amaz.Vector3f(0,0,rot)
                    letter.instanceColor = Amaz.Color(letter.instanceColor.r, letter.instanceColor.g, letter.instanceColor.b, opt)
                end
            end
        else
            --  only the last letter spring
            local validCount = 0
            local totalFrame = (self.validCount -1)*5 + 41
            local curframe = math.ceil(totalFrame * self.progress)
            local spring_num = self.validCount <=3 and 1 or 3
            for i = 1, self.count do
                local letter = self.rich_text.letters:get(i-1)
                local offset = 0
                local rot = 0
                local opt = 0
                if self:isvalidletter(letter) then
                    validCount = validCount + 1
                    if validCount <= self.validCount - spring_num then
                        local step_off1 = -self.avgSize
                        local start_frame1 = (validCount - 1)*5 +1
                        local end_frame1 = (validCount - 1)*5 + 20
                        rot = 0
                        local pt = self.util.clamp(0,1,(curframe - start_frame1)/(end_frame1 - start_frame1))
                        local cur_pt = self.util.bezier({0.166667, 0.166667, 0.14, 1,})(pt,0,1,1)
                        offset = self.util.mix(step_off1,0,cur_pt)
                        local opt_pt = self.util.clamp(0,1,(curframe - start_frame1+1))
                        opt_pt = self.util.bezier({0.166666667, 0.166666667, 0.833333333, 0.833333333,})(opt_pt,0,1,1)
                        opt = self.util.mix(0,1,opt_pt)

                    else
                        local start_frame1 = (validCount - 1)*5 +1
                        local end_frame1 = (validCount - 1)*5 + 21
                        local end_frame2 = (validCount - 1)*5 + 41
                        if curframe <= end_frame1 then
                            local pt = (curframe - start_frame1)/(end_frame1 - start_frame1)
                            pt = self.util.clamp(0,1,pt)
                            local cur_pt = self.util.bezier({0.166667, 0.166667, 0, 1, })(pt,0,1,1)
                            cur_pt = self.util.clamp(0,1,cur_pt)
                            offset = self.util.mix(step_off1,step_off2,cur_pt)

                        else
                            local pt = (curframe - end_frame1)/(end_frame2 - end_frame1)
                            pt = self.util.clamp(0,1,pt)
                            local cur_pt = self.util.bezier({1, 0, 0.833333, 0.833333, })(pt,0,1,1)
                            offset = self.util.mix(step_off2,0,cur_pt)

                        end

                        local pt = self.util.clamp(0,1,(curframe - start_frame1+1)/(end_frame1-start_frame1))
                        local rot_pt = self.util.bezier({0.015, 0.213203433, 0.09, 1, })(pt,0,1,1)
                        rot = self.util.mix(-180,-360,rot_pt)
                        local opt_pt = self.util.clamp(0,1,(curframe - start_frame1+1))
                        local opt_pt = self.util.bezier({0.166666667, 0.166666667, 0.833333333, 0.833333333,})(opt_pt,0,1,1)
                        opt = self.util.mix(0,1,opt_pt)
                    end

                    if typeSettingKind == 1 then
                        letter.position = Amaz.Vector2f(letter.initialPosition.x + offset,letter.initialPosition.y)
                    else
                        letter.position = Amaz.Vector2f(letter.initialPosition.x ,letter.initialPosition.y+offset)
                    end
                    letter.rotate = Amaz.Vector3f(0,0,rot)
                    letter.instanceColor = Amaz.Color(letter.instanceColor.r, letter.instanceColor.g, letter.instanceColor.b, opt)
                end
            end
        end
    end  

end


------------------------------------------------------------


function AnimScript:setDuration(duration)
   self.duration = duration
end


function AnimScript:resetData( ... )
	if not self.text then
        return
    end
    self.initstate  = false
    for i = 1, self.rich_text.letters:size() do
        local letter = self.rich_text.letters:get(i - 1)
        letter.position = letter.initialPosition

        letter.rotate = Amaz.Vector3f(0,0,0)
        letter.instanceColor = Amaz.Color(letter.instanceColor.r, letter.instanceColor.g, letter.instanceColor.b, 1)
    end

end

function AnimScript:onLeave()
    self:resetData()
end

function AnimScript:clear()
    self:resetData()
end

function AnimScript:onEnter()
    self.first = true
end


exports.AnimScript = AnimScript
return exports
