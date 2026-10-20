local exports = exports or {}
local TextBloom = TextBloom or {}
TextBloom.__index = TextBloom
function TextBloom.new(construct, ...)
    local self = setmetatable({}, TextBloom)

    if construct and TextBloom.constructor then TextBloom.constructor(self, ...) end
    return self
end

function TextBloom:constructor()

end


function TextBloom:getTextFontSize()
    local text = self.text1
    local fontSize = 24
    if text then 
        
        if text.forceFlushCommandQueue then
            text:forceFlushCommandQueue()
        end
        local letters = text.letters
        if letters:size() > 0 then
            local letter0 = letters:get(0)
            fontSize = letter0 and letter0.letterStyle and letter0.letterStyle.fontSize
        end
    else
        fontSize = self.text.fontSize
    end
    return fontSize
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
    self.trans = comp.entity:getComponent("Transform")
    self.renderer = nil
	if self.text ~= nil then
		self.renderer = comp.entity:getComponent("MeshRenderer")
    end
    self.comp = comp

    self.trans = comp.entity:getComponent("Transform")
    self.parentTrans = self.trans.parent
end

function TextBloom:initAnim()

end

function TextBloom:onUpdate(comp, deltaTime)
    local w = Amaz.BuiltinObject:getInputTextureWidth()*1
    local h = Amaz.BuiltinObject:getInputTextureHeight()*1
    if self.text then
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
        
        local maxSize = 4096*0.9
        if w > maxSize or h > maxSize then
            if w > h then
                local scale = h/w
                w = maxSize
                h = maxSize*scale
            else
                local scale = w/h
                w = maxSize*scale
                h = maxSize
            end
        end
        
        if self.text.forceFlushCommandQueue then
            self.text:forceFlushCommandQueue()
        end
        
        local text = comp.entity:getComponent('Text')
        if text and  text.getBloomMaterial then
            self.bloomMaterial = text:getBloomMaterial()
    
        else
            self.bloomMaterial = nil
        end
    
        local scale = 0.25

        if self.text.renderToRT then
            scale=0.5
            self.bloomMaterial:setFloat("u_BlurScale",1.75*0.820)
        else
            self.bloomMaterial:setFloat("u_BlurScale",2.5*1.20)

            blurscale = 1.0
        end

            local rt2 = self.bloomMaterial:getTex("inputTextureY")
            local rt1 = self.bloomMaterial:getTex("u_X1InputTex")
            local rt3 = self.bloomMaterial:getTex("u_BloomTex1")
            local rt4 = self.bloomMaterial:getTex("u_BloomTex2")
            
            rt1.width = w*scale
            rt1.height = h*scale
            rt3.width = w*scale
            rt3.height = h*scale
            rt4.width = w*scale
            rt4.height = h*scale

            rt2.width = w
            rt2.height = h

    end


end

function TextBloom:onEnter()
	-- self.isInit = false
end

function TextBloom:onLeave()

end


exports.TextBloom = TextBloom
return exports
