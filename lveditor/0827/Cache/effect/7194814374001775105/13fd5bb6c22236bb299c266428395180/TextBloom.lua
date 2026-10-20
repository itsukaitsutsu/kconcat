local exports = exports or {}
local TextBloom = TextBloom or {}
TextBloom.__index = TextBloom
function TextBloom.new(construct, ...)
    local self = setmetatable({}, TextBloom)
    self.pre_main_size = {width = 0,height = 0}
    self.pre_target_size = {width = 0,height = 0}
    self.pre_bloom_range = 1.0
    self.first = true
    if construct and TextBloom.constructor then TextBloom.constructor(self, ...) end
    return self
end

function TextBloom:constructor()

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
    local w = Amaz.BuiltinObject:getInputTextureWidth()*1.0
    local h = Amaz.BuiltinObject:getInputTextureHeight()*1.0

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
        
        if self.text.forceFlushCommandQueue then
            self.text:forceFlushCommandQueue()
        end
        
        local text = comp.entity:getComponent('Text')
        if text and  text.getBloomMaterial then
            self.bloomMaterial = text:getBloomMaterial()
    
        else
            self.bloomMaterial = nil
        end
    
        if self.bloomMaterial then
            local rt1 = self.bloomMaterial:getTex("inputTexture")
            local rt2 = self.bloomMaterial:getTex("inputTextureX")
            local rt3 = self.bloomMaterial:getTex("inputTextureY")
            -- local mainrt =  self.bloomMaterial:getTex("u_TextTex")

            if self.text.renderToRT then

                self.bloomMaterial:setFloat("blurscale",1)
                if blurscale < 0.5 then
                    blurscale = 0.5
                end
                self.bloomMaterial:setFloat("blurscale1",1.0/blurscale)
            else
                blurscale = 1.0
                self.bloomMaterial:setFloat("blurscale",1.0)
            end
            
            local scale = 0.3
            -- scale = 1
            local extraScale = 1
            rt1.width = w * scale
            rt1.height = h * scale
            rt2.width = w * scale * extraScale
            rt2.height = h * scale * extraScale
            rt3.width = w*scale * extraScale
            rt3.height = h*scale * extraScale

            -- local maxSize = 4096*0.9
            -- if w > maxSize or h > maxSize then
            --     if w > h then
            --         mainrt.width = maxSize
            --         mainrt.height = maxSize*h/w
            --     else
            --         mainrt.height = maxSize
            --         mainrt.width = maxSize*w/h
            --     end

            -- else
            --     mainrt.width = w
            --     mainrt.height = h
            -- end

            self.bloomMaterial:setVec2("u_Center", Amaz.Vector2f((self.parentTrans.localPosition.x * 0.5) * h / w , self.parentTrans.localPosition.y * 0.5 ))
    
        
        end
    end


end

function TextBloom:seek(time)

end

function TextBloom:onEnter()
	-- self.isInit = false
end

function TextBloom:onLeave()

end


exports.TextBloom = TextBloom
return exports
