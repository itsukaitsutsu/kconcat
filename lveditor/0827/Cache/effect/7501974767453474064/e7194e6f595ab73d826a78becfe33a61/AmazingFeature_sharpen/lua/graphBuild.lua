local exports = exports or {}
local graphBuild = graphBuild or {}

---@class graphBuild : ScriptComponent
----@field inputTexture Texture  [UI(Type="Texture")]
----@field curTime Double [UI(Range={0, 3}, Slider)]
----@field sharpness Double [UI(Range={0.0, 1.0}, Slider)]
----@export "prefabs"
graphBuild.__index = graphBuild

function graphBuild.new(construct, ...)
    local o = setmetatable({}, graphBuild)
    -- o.renderChain = includeRelativePath("renderChain.lua")
    o.curTime = 0
    o.sharpness = 0
    o.Utils = includeRelativePath("Utils.lua")
    o.RTCounter = {}
    o.entities = {}
    return o
end

function graphBuild:init(comp)
    local entities = comp.entity.scene.entities
    local link = self.renderChain.link
    local tmpLink = {}
    for i = 1, #link do
        if link[i].subLink then
            local subLink = link[i].subLink
            for j = 1, #subLink do
                for key, value in pairs(subLink[j].input) do
                    local splitResult = self.Utils.split(value, ".")
                    if #splitResult == 2 and splitResult[1] == "subLinkInput" then
                        subLink[j].input[key] = link[i].subLinkInput[splitResult[2]]
                    end
                end
                table.insert(tmpLink, subLink[j])
            end
            for j = i + 1, #link do
                if link[j].input then
                    for key, value in pairs(link[j].input) do
                        if value == link[i].subLinkName then
                            link[j].input[key] = link[i].subLink[#link[i].subLink].entityName
                        end
                    end
                end
                if link[j].subLinkInput then
                    for key, value in pairs(link[j].subLinkInput) do
                        if value == link[i].subLinkName then
                            link[j].subLinkInput[key] = link[i].subLink[#link[i].subLink].entityName
                        end
                    end
                end
            end
        else
            table.insert(tmpLink, link[i])
        end
    end
    self.renderChain.link = tmpLink
    link = tmpLink
    local path = comp.entity.scene.assetMgr.rootDir
    --  remove all instantiated entities
    for i = 1, #self.entities do
        comp.entity.scene:removeEntity(self.entities[i])
    end
    self.entities = {}
    for i = 1, #link do
        -- local prefab = Amaz.PrefabManager.loadPrefab(path, path .. "prefabs/" .. link[i].entityName .. ".prefab")
        -- if not prefab then
        --     Amaz.LOGE("link", "can't find prefab:" .. link[i].entityName)
        -- end
        -- comp.entity.scene:addInstantiatedPrefab(prefab)
        local entity = comp.entity.scene:findEntityBy(link[i].entityName)
        if entity then
            entity.visible = link[i].visible
        end
        table.insert(self.entities, entity)
        local renderer =
            entity:getComponent("MeshRenderer") and entity:getComponent("MeshRenderer") or
            entity:getComponent("Sprite2DRenderer")
        if renderer then
            link[i].material = renderer.material
        else
            Amaz.LOGE("link", "can't find render:" .. link[i].entityName)
        end
    end
    self.Utils.buildRenderChain(self, comp)
    self.Utils.setMaterialProp(self, comp)
end

function graphBuild:onStart(comp, sys)
    local path = comp.entity.scene.assetMgr.rootDir
    self.jsonParser = cjson.new()
    local file = io.open(path .. "lua/renderChain.json", "r")
    local jsonRaw = file:read("*a")
    file:close()
    self.renderChain = cjson.decode(jsonRaw)
    self:init(comp)
end

if Amaz.Macros and Amaz.Macros.EditorSDK then
    function graphBuild:onUpdate(comp)
        local link = self.renderChain.link
        for i = 1, #link do
            if link[i].entityName == "sharp" then
                link[i].material:setFloat("sharpness",self.sharpness)
            end
        end
    end
end

-- function graphBuild:onEvent(comp, event)
--     if event.type == Amaz.EventType.TOUCH then
--         local pointer = event.args:get(0)
--         local type = pointer.type
--         if type == Amaz.TouchType.TOUCH_ENDED then
--             -- self:init(comp)
--             if Amaz.Macros and Amaz.Macros.EditorSDK then
--                 local path = comp.entity.scene.assetMgr.rootDir
--                 local link = self.renderChain.link
--                 for i = 1, #link do
--                     local RTIndex = link[i].renderTextureIndex
--                     if RTIndex ~= 1 then
--                         if link[i].visible then
--                             Amaz.LOGE("RenderTex", "saveToFile:" .. path .. "rt/__tmp_rt_" .. RTIndex - 1 .. ".png")
--                             local rt = self.RTCounter[RTIndex].rt
--                             rt:saveToFile(path .. "rt/__tmp_rt_" .. RTIndex - 1 .. ".png")
--                         end
--                     end
--                 end
--             end
--         -- self.renderChain.link[#self.renderChain.link] = nil
--         -- self:init(comp)
--         end
--     end
-- end

exports.graphBuild = graphBuild
return exports
