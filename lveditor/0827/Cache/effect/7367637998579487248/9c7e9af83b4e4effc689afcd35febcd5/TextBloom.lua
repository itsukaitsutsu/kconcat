local exports = exports or {}
local TextBloom = TextBloom or {}
TextBloom.__index = TextBloom
function TextBloom.new(construct, ...)
    local self = setmetatable({}, TextBloom)

    if construct and TextBloom.constructor then TextBloom.constructor(self, ...) end
    return self
end


function TextBloom:onStart(comp) 
    self.text = comp.entity:getComponent('SDFText')
    if self.text == nil then
        local text = comp.entity:getComponent('Text')
        if text ~= nil then
            self.text = comp.entity:addComponent('SDFText')
            self.text:setTextWrapper(text)
        end
    end
    -- self.bloomMaterial = self.text:getBloomMaterial()
    self.parentTrans = comp.entity:getComponent("Transform").parent
end

function TextBloom:onUpdate(comp, deltaTime)
    local w = Amaz.BuiltinObject:getInputTextureWidth()
    local h = Amaz.BuiltinObject:getInputTextureHeight()
    if self.text == nil then return end
    local blurscale =  self.parentTrans.localScale.x
    local mainRtSize = self.text:getRectExpanded() 
    
    local main_width = mainRtSize.width*blurscale
    local main_height = mainRtSize.height*blurscale

    w = w < main_width and main_width or w
    h = h < main_height and main_height or h

    
    local getVersionNum = function(sdk_str)
        local sp_str = "."
        local splits = {}
        local sdk_version_num = 0
        if sdk_str and sdk_str ~= "" then
            -- normal split use gmatch
            local pattern = "[^" .. sp_str .. "]+"
            for str in string.gmatch(sdk_str, pattern) do
                table.insert(splits, str)
            end
        end
        local len = #splits
        local m_num = 10
        for i=len,1,-1 do
            sdk_version_num = sdk_version_num + tonumber(splits[i])*m_num
            m_num = m_num * 10
        end
        return sdk_version_num
    end

    if getVersionNum(EffectSdk.getSDKVersion())>= getVersionNum("14.0.0") then
        self.text.textWrapper.bloomRtSize = Amaz.Vector2f(w , h)
    else
        self.text.textWrapper.BloomRtSize = Amaz.Vector2f(w , h)
    end
    
end


exports.TextBloom = TextBloom
return exports
