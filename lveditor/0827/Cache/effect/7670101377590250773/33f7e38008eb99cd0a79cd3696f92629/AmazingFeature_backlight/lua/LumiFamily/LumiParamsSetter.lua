local LumiParamsSetter = {}
LumiParamsSetter.__index = LumiParamsSetter

local AE_EFFECT_TAG = 'AE_EFFECT_TAG LumiTag'

function LumiParamsSetter.new(params, keyframes, effectType, sliderInfos, fadeoutInfos)
    local self = setmetatable({}, LumiParamsSetter)
    self.params = params
    self.keyframes = keyframes
    self.sliderInfos = sliderInfos
    self.sliderParams = {}
    self.XTEvent = (effectType == 'xtFilter' or  effectType == 'xtEffect')
    self.fadeoutInfos = fadeoutInfos
    self.afterSliderParams = {}

    return self
end

local function clamp(val, min, max)
    return math.max(math.min(val, max), min)
end

local function mix(a, b, t)
    return a * (1 - t) + b * t
end

local function cvtTable2Amaz(attrType, v)
    local value = nil
    if attrType == "number" then
        if #v == 1 then
            value = v[1]
        else
            Amaz.LOGE(AE_EFFECT_TAG, "Invalid keyframe: " .. key .. " size: " .. #v)
        end
    elseif attrType == "vector" then
        if #v == 2 then
            value = Amaz.Vector2f(v[1], v[2])
        elseif #v == 3 then
            value = Amaz.Vector3f(v[1], v[2], v[3])
        elseif #v == 4 then
            value = Amaz.Vector4f(v[1], v[2], v[3], v[4])
        else
            Amaz.LOGE(AE_EFFECT_TAG, "Invalid keyframe: " .. key .. " size: " .. #v)
        end
    elseif attrType == "color" then
        if #v == 3 then
            value = Amaz.Color(v[1], v[2], v[3], 1.0)
        elseif #v == 4 then
            value = Amaz.Color(v[1], v[2], v[3], v[4])
        else
            Amaz.LOGE(AE_EFFECT_TAG, "Invalid keyframe: " .. key .. " size: " .. #v)
        end
    else
        Amaz.LOGE(AE_EFFECT_TAG, "Invalid keyframe: " .. key .. "unsupported type: " .. attrType)
    end
    return value
end

local function getValueType(value)
    local valueType = type(value)
    if valueType == 'userdata' then
        local str = tostring(value)
        if string.sub(str, 1, 6) == 'Vector' then
            valueType = 'vector'
        elseif string.sub(str, 1, 5) == 'Color' then
            valueType = 'color'
        end
    end

    return valueType
end

local function add(value, number)
    local type = getValueType(value)
    if type == 'number' then
        return value + number
    elseif type == 'vector' then
        local ret = value:copy()
        ret.x = ret.x + number
        ret.y = ret.y + number
        if ret.z then ret.z = ret.z + number end
        if ret.w then ret.w = ret.w + number end
        return ret
    elseif type == 'color' then
        return Amaz.Color(
            value.r + number,
            value.g + number,
            value.b + number,
            value.a + number
        )
    end
    return nil
end

local function mul(value, number)
    local type = getValueType(value)
    if type == 'color' then
        return Amaz.Color(
            value.r * number,
            value.g * number,
            value.b * number,
            value.a * number
        )
    end
    return value * number
end

local function remap2(x, a, b)
    return x * (b - a) + a
end

function LumiParamsSetter:getCurrentDefaultParam(entity, key, time)
    local keyframeName = entity..'#'..key.."#"..'number'
    local keyframeType = 'number'
    if self.keyframes.attrs[keyframeName] == nil then
        keyframeName = entity..'#'..key.."#"..'vector'
        keyframeType = 'vector'
        if self.keyframes.attrs[keyframeName] == nil then
            keyframeName = entity..'#'..key.."#"..'color'
            keyframeType = 'color'
            if self.keyframes.attrs[keyframeName] == nil then
                keyframeName = nil
            end
        end
    end
    local value = nil
    if keyframeName ~= nil  then
        value = cvtTable2Amaz(keyframeType, self.keyframes:GetVal(keyframeName, time))
    else
        local entity = self.params[entity]
        if entity ~= nil then
            value = entity[key]
        end
    end
    return value
end

function LumiParamsSetter:updateSlider(lumi_obj, startTime, endTime, curTime, aeTime)
    -- local sliderIntensity = self.sliderParams['effects_adjust_blur']
    -- local gaussianIntensity = self:getCurrentDefaultParam("Gaussian_Blur_Root_353-effect2", 'intensity', aeTime)
    -- if sliderIntensity and gaussianIntensity then
    --     lumi_obj:setSubEffectAttr("Gaussian_Blur_Root_353-effect2", 'intensity', gaussianIntensity*sliderIntensity)
    -- end

    local function fsub(value, slider, valueA, valueB)
        -- (param - sliderValue) * valueA + valueB
        return (value - slider) * valueA + valueB
    end
    
    local function fmul(value, slider, valueA, valueB)
        -- (param - valueA) * sliderValue + valueB
        return (value - valueA) * slider + valueB
    end

    local function fadd(value, slider, valueA, valueB)
        -- (param - valueA) * valueB + sliderValue
        return (value - valueA) * valueB + slider
    end

    for sliderKey, paramsInfos in pairs(self.sliderInfos) do
        if self.sliderParams[sliderKey] ~= nil then
            for index, value in ipairs(paramsInfos) do
                local sliderIntensity = self.sliderParams[sliderKey]
                local entityName = value[1]
                local paramKey = value[2]
                local paramType = value[3]
                local paramDimensionFlag = value[4]
                local calcType = value[5]
                local maxValue = value[6]
                local minValue = value[7]
                local defaultValue = value[8]
                local valueA = value[9]
                local valueB = value[10]
                sliderIntensity = sliderIntensity * (maxValue - minValue) + minValue
                local oriValue = self:getCurrentDefaultParam(entityName, paramKey, aeTime)
                local newValue = oriValue
                local calcFunc = nil
                if calcType == 0 then
                    calcFunc = fsub
                elseif calcType == 1 then
                    calcFunc = fmul
                elseif calcType == 2 then
                    calcFunc = fadd
                else
                    Amaz.LOGE(AE_EFFECT_TAG, 'Unknown calcType: ' .. calcType)
                    return
                end
                if paramType == 'number' then
                    if paramDimensionFlag[1] then newValue = calcFunc(oriValue, sliderIntensity, valueA[1], valueB[1]) end
                elseif paramType == 'color' then
                    if paramDimensionFlag[1] then newValue.r = calcFunc(oriValue.r, sliderIntensity, valueA[1], valueB[1]) end
                    if paramDimensionFlag[2] then newValue.g = calcFunc(oriValue.g, sliderIntensity, valueA[2], valueB[2]) end
                    if paramDimensionFlag[3] then newValue.b = calcFunc(oriValue.b, sliderIntensity, valueA[3], valueB[3]) end
                    if paramDimensionFlag[4] then newValue.a = calcFunc(oriValue.a, sliderIntensity, valueA[4], valueB[4]) end
                elseif paramType == 'vector' then
                    if paramDimensionFlag[1] then newValue.x = calcFunc(oriValue.x, sliderIntensity, valueA[1], valueB[1]) end
                    if paramDimensionFlag[2] then newValue.y = calcFunc(oriValue.y, sliderIntensity, valueA[2], valueB[2]) end
                    if paramDimensionFlag[3] then newValue.z = calcFunc(oriValue.z, sliderIntensity, valueA[3], valueB[3]) end
                    if paramDimensionFlag[4] then newValue.w = calcFunc(oriValue.w, sliderIntensity, valueA[4], valueB[4]) end
                end
                lumi_obj:setSubEffectAttr(entityName, paramKey, newValue)
                self.afterSliderParams[entityName .. '#' .. paramKey] = newValue
            end
        end
    end
end

function LumiParamsSetter:updateFadeout(lumi_obj, remainingTime, aeTime)
    if self.fadeoutInfos == nil
    or self.fadeoutInfos.time == nil or self.fadeoutInfos.time <= 0
    or self.fadeoutInfos.infos == nil or #self.fadeoutInfos.infos == 0
    then
        return
    end

    local fadeoutTime = self.fadeoutInfos.time

    if remainingTime > fadeoutTime then
        return
    end

    local factor = remainingTime / fadeoutTime
    factor = clamp(factor, 0, 1)

    for i = 1, #self.fadeoutInfos.infos do
        local entityName = self.fadeoutInfos.infos[i][1]
        local paramKey = self.fadeoutInfos.infos[i][2]
        local valueType = self.fadeoutInfos.infos[i][3]
        local value = self.fadeoutInfos.infos[i][4]

        local curValue = nil
        if self.afterSliderParams[entityName .. '#' .. paramKey] ~= nil then
            curValue = self.afterSliderParams[entityName .. '#' .. paramKey]
        else
            curValue = self:getCurrentDefaultParam(entityName, paramKey, aeTime)
        end

        if curValue ~= nil then
            local needRenew = true
            if valueType == 'number' then
                curValue = mix(value[1], curValue, factor)
            elseif valueType == 'color' then
                curValue.r = mix(value[1], curValue.r, factor)
                curValue.g = mix(value[2], curValue.g, factor)
                curValue.b = mix(value[3], curValue.b, factor)
                curValue.a = mix(value[4], curValue.a, factor)
            elseif valueType == 'vector' then
                if #value <= 1 or #value >= 5 then
                    needRenew = false
                    Amaz.LOGE(AE_EFFECT_TAG, 'Invalid value size: ' .. #value)
                else
                    if #value >= 2 then
                        curValue.x = mix(value[1], curValue.x, factor)
                        curValue.y = mix(value[2], curValue.y, factor)
                    end
                    if #value >= 3 then
                        curValue.z = mix(value[3], curValue.z, factor)
                    end
                    if #value >= 4 then
                        curValue.w = mix(value[4], curValue.w, factor)
                    end
                end
            else
                needRenew = false
                Amaz.LOGE(AE_EFFECT_TAG, 'Invalid valye type: ' .. valueType)
            end
            if needRenew then
                lumi_obj:setSubEffectAttr(entityName, paramKey, curValue)
            end
        end
    end
end

function LumiParamsSetter:setValue(lumi_obj, value)
    --Amaz.LOGE(AE_EFFECT_TAG, '233 effects_adjust_intensity: ' .. value)
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "shadowMapParam", remap2(value, 0, 0.9))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "highlightMapParam", remap2(value, 0, 1.8))        
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "highlightIntensity", remap2(value, 0, 0.55))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "shadowIntensity", remap2(value, 0, 0.3))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "blackIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "whiteIntensity", remap2(value, 0, 0.33))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "exposureIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "offsetIntensity", remap2(value, 0, -0.07))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "contrastIntensity", remap2(value, 0, 0.08))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "contrastPivot", remap2(value, 0, 0.435))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "saturationSuppressionIntensity", remap2(value, 0, 0.05))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "saturationSuppressionThreshold", remap2(value, 0, 0.2))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "saturationIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "temperatureIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "tintIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "dahazeIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect0", "surfaceBlurIntensity", remap2(value, 0, 0))

    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "shadowMapParam", remap2(value, 0, 0.9))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "highlightMapParam", remap2(value, 0, 1.8))        
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "highlightIntensity", remap2(value, 0, 0.28))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "shadowIntensity", remap2(value, 0, 0.27))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "blackIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "whiteIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "exposureIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "offsetIntensity", remap2(value, 0, -0.18))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "contrastIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "contrastPivot", remap2(value, 0, 0.435))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "saturationSuppressionIntensity", remap2(value, 0, 0.05))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "saturationSuppressionThreshold", remap2(value, 0, 0.2))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "saturationIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "temperatureIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "tintIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "dahazeIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect1", "surfaceBlurIntensity", remap2(value, 0, 0))

    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "shadowMapParam", remap2(value, 0, 0.9))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "highlightMapParam", remap2(value, 0, 1.8))        
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "highlightIntensity", remap2(value, 0, 0.35))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "shadowIntensity", remap2(value, 0, 0.08))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "blackIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "whiteIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "exposureIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "offsetIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "contrastIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "contrastPivot", remap2(value, 0, 0.435))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "saturationSuppressionIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "saturationSuppressionThreshold", remap2(value, 0, 0.2))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "saturationIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "temperatureIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "tintIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "dahazeIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect2", "surfaceBlurIntensity", remap2(value, 0, 0))

    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "shadowMapParam", remap2(value, 0, 0.9))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "highlightMapParam", remap2(value, 0, 1.8))        
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "highlightIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "shadowIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "blackIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "whiteIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "exposureIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "offsetIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "contrastIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "contrastPivot", remap2(value, 0, 0.435))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "saturationSuppressionIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "saturationSuppressionThreshold", remap2(value, 0, 0.2))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "saturationIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "temperatureIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "tintIntensity", remap2(value, 0, 0))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "dahazeIntensity", remap2(value, 0, 0.55))
    lumi_obj:setSubEffectAttr("LumiColorAdjustBundle_25-effect3", "surfaceBlurIntensity", remap2(value, 0, 0))
end

function LumiParamsSetter:initParams(lumi_obj)
    if lumi_obj == nil then return end
    if self.params == nil then return end
    if self.init then return end

    for entityName, params in pairs(self.params) do
        for key, value in pairs(params) do
            lumi_obj:setSubEffectAttr(entityName, key, value)
        end
    end

    self.init = true
end

function LumiParamsSetter:updateKeyFrameData(lumi_obj, startTime, endTime, curTime, aeTime)
    if lumi_obj == nil then return end
    if self.keyframes == nil then return end

    -- local p = (curTime - startTime) / (endTime - startTime)
    local p = aeTime
    for key, _ in pairs(self.keyframes.attrs) do
        local keys = {}
        for substr in string.gmatch(key, "[^#]+") do
            table.insert(keys, substr)
        end
        if #keys == 3 then
            local entityName = keys[1]
            local attrName = keys[2]
            local attrType = keys[3]
            local v = self.keyframes:GetVal(key, p)
            local value = cvtTable2Amaz(attrType, v)
            if value then
                lumi_obj:setSubEffectAttr(entityName, attrName, value)
            end
        else
            Amaz.LOGE(AE_EFFECT_TAG, "Invalid keyframe: " .. key)
        end
    end
end

return LumiParamsSetter
