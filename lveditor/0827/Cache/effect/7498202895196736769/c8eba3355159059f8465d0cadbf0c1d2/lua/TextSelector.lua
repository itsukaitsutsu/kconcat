local TextSelector = TextSelector or {}
TextSelector.__index = TextSelector

local SHAPETYPE = {
    CIRCLE = 0,
    SMOOTH = 1,
    SQUARE = 2,
    UNPILL = 3,
    DOWNPILL = 4,
    TRIANGLE = 5
}

local ANIMATIONTYPE= {
    CHAR = 0,
    WORD = 1,
    LINE = 2
}


function TextSelector:new(_util)
    local self = setmetatable({}, TextSelector)
    self.begin_pos = 0 --Standardize to 0-1
    self.end_pos = 0 --Standardize to 0-1
    self.curType = 0
    self.offset = 0 --Standardize to 0-1
    self.util = _util
    return self
end


function TextSelector:getCirclePercent(pt,start_p,end_p)
    local R = end_p - start_p
    local mid = (start_p + end_p)/2
    local s_r = 0.25*R*R

    local s_d = (pt - mid)*(pt - mid)
    if s_d < s_r then
        return 1.0-s_d/s_r
    else
        return 0
    end

    -- if typeSettingKind == 0 then
    --     -- local x = 2.0*math.abs(linePos.x - pos.x)/lineWidth
    --     -- local p = math.exp(-2.5*x*x)
    --     -- p = self.util.remap(0.25,1.0,p)
    --     local s_d = (pos.x - c_pos.x)*(pos.x - c_pos.x)

    --     if s_d < s_r then
    --         local d = 2*math.sqrt(s_r-s_d)/r
    --         return self.util.remap(0.6,1.0,d)
    --     else
    --         return 0
    --     end
    -- elseif typeSettingKind == 1 then
    --     local s_d = (pos.y - c_pos.y)*(pos.y - c_pos.y)

    --     if s_d < s_r then
    --         return 2*math.sqrt(s_r-s_d)/r
    --     else
    --         return 0
    --     end
    -- end
end


function TextSelector:getSmoothPercent(pt,start_p,end_p)
    local mid = (start_p+end_p)*0.5
    local x = 2.0*(pt-mid)/(end_p-start_p)
    local p = math.exp(-2.5*x*x)
    p = self.util.remap(0.05,1.0,p)
    return p
end

function TextSelector:getSquarePercent(pt,start_p,end_p)
    if pt < start_p or pt > end_p then
        return 0
    else
        return 1
    end
end

function TextSelector:getTrianglePercent(pt,start_p,end_p)
    local mid = (start_p+end_p)*0.5
    if pt < start_p or pt > end_p then
        return 0
    else
        if pt <=  mid then
            return 1.0-(mid - pt)/(mid-start_p)
        else
            return 1.0-(pt-mid)/(end_p-mid)
        end
    end
end



function TextSelector:getUpLinePercent(pt,start_p,end_p)
    if pt < start_p then
        return 0
    elseif pt > end_p then
        return 1
    else
        return (pt-start_p)/(end_p-start_p)
    end
end


function TextSelector:getDownLinePercent(pt, start_p, end_p)
    return 1.0 - self:getUpLinePercent(pt, start_p, end_p)
end


function TextSelector:getShapPercent(aeInfo,textInfo)
    local start_pos = aeInfo.start_p
    local end_pos = aeInfo.end_p
    local offset = aeInfo.offset
    local shapeType = aeInfo.shapeType

    local index = textInfo.index
    local totalnum = textInfo.totalnum
    local pt = 0.5

    if totalnum > 1 then
        pt = self.util.remap(0,1,(index-1)/(totalnum-1))
    end

    if start_pos > end_pos then
        local tmp = start_pos
        start_pos = end_pos
        end_pos = tmp
    end

    start_pos = start_pos + offset
    end_pos = end_pos + offset

    if shapeType == SHAPETYPE.UNPILL then
       return  self:getUpLinePercent(pt,start_pos,end_pos)
    elseif shapeType == SHAPETYPE.DOWNPILL then
        return self:getDownLinePercent(pt,start_pos,end_pos)
    elseif shapeType == SHAPETYPE.CIRCLE then
        return self:getCirclePercent(pt,start_pos,end_pos)
    elseif shapeType == SHAPETYPE.SMOOTH then
        return self:getSmoothPercent(pt,start_pos,end_pos)
    elseif shapeType == SHAPETYPE.SQUARE then
        return self:getSquarePercent(pt,start_pos,end_pos)
    elseif shapeType == SHAPETYPE.TRIANGLE then
        return self:getTrianglePercent(pt,start_pos,end_pos)
    end
end

return TextSelector
