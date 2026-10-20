




---------- TextMain.lua ----------


local TextMain = {}
TextMain.__index = TextMain
function TextMain.new ()
    local self = setmetatable({}, TextMain)
    self.FRAMES = {1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1, 0, 0}
    return self
end


function TextMain:onCreate (env)
    ---#ifdef DEV
--//    env.rootTextOld.fontSize = 24
--//    env.rootTextOld.str = "First Line\nSecond Line"
--//    env.duration = 1
    ---#endif
end

function TextMain:onShow (env)
end

function TextMain:onHide (env)
    local chars = env.rootTextOld.chars
    for i = 0, chars:size() - 1 do
        local char = chars:get(i)
        char.scale = Amaz.Vector3f(1, 1, 1)
    end
end

function TextMain:onChange (env)
    local text = env.rootTextOld
    text:forceTypeSetting()
end

function TextMain:onUpdate (env, elapsed)
    local i = elapsed % env.duration / env.duration
    i = math.floor(i * #self.FRAMES) + 1

    local visible = self.FRAMES[i] > 0
    local scale = visible and Amaz.Vector3f(1, 1, 1) or Amaz.Vector3f(0, 0, 0)
    local chars = env.rootTextOld.chars
    for j = 0, chars:size() - 1 do
        local char = chars:get(j)
        char.scale = scale
    end
end







---------- info_sticker/Patch0.lua ----------

--- Detect user configuration changes
local Patch0 = {}
Patch0.__index = Patch0


function Patch0.new (env, compNew, compOld)
    local self = setmetatable({}, Patch0)
    self.compNew = compNew
    self.compOld = compOld
    self.compOld:forceTypeSetting()
    self:_cacheInitialPosition()
    return self
end


function Patch0:isDirty (env)
    self.compOld:forceTypeSetting()
    if self:_checkInitialPosition() then
        self:_cacheInitialPosition()
        return true
    end
    return false
end

function Patch0:_cacheInitialPosition ()
    self._initialPosition = {}
    local cache = {}
    local chars = self.compOld.chars
    for i = 0, chars:size() - 1 do
        local char = chars:get(i)
        table.insert(cache, char.initialPosition:copy())
    end
    self.cache = cache
end

function Patch0:_checkInitialPosition ()
    local cache = self.cache
    local chars = self.compOld.chars
    if chars:size() ~= #cache then
        return true
    end
    for i = 0, chars:size() - 1 do
        local char = chars:get(i)
        local p0 = char.initialPosition
        local p1 = cache[i + 1]
        if p0.x ~= p1.x or p0.y ~= p1.y or p0.z ~= p1.z then
            return true
        end
    end
    return false
end







---------- TextEntry.lua ----------


local TextAnim = {}
TextAnim.__index = TextAnim


function TextAnim.new ()
    local self = setmetatable({}, TextAnim)
    self.curTime = 0.0
    self.duration = 5.0
    self.w = 0
    self.h = 0
    self._visible = false
    self._dirty = false
    self._main = TextMain.new()
    return self
end

function TextAnim:onStart (comp)
    self.scene = comp.entity.scene
    local cameraEntity = self.scene:findEntityBy("InfoSticker_camera_entity")
    if cameraEntity then
        self.camera = cameraEntity:getComponent("Camera")
    end
    self.rootPath = debug.getinfo(1, "S").source:match("@?(.*/)")
    self.rootNode = comp.entity:getComponent("Transform")
    self.rootText = comp.entity:getComponent("Text")
    self.rootTextOld = comp.entity:getComponent("SDFText")
    if not self.rootTextOld then
        if self.rootText then
            self.rootTextOld = comp.entity:addComponent("SDFText")
            self.rootTextOld:setTextWrapper(self.rootText)
        end
    end
    self._main:onCreate(self)
    self.patch0 = Patch0.new(self, self.rootText, self.rootTextOld)
end

function TextAnim:onEnter ()
    if not self.visible then
        self._main:onShow(self)
        self._visible = true
    end
    self._dirty = true
end

function TextAnim:onLeave ()
    if self._visible then
        self._main:onHide(self)
        self._visible = false
    end
end

function TextAnim:clear ()
    self:onLeave()
end

function TextAnim:setDuration (duration)
    self.duration = duration
end

function TextAnim:seek (time)
    if not self._visible then
        return
    end
    local w = Amaz.BuiltinObject:getOutputTextureWidth()
    local h = Amaz.BuiltinObject:getOutputTextureHeight()
    if self.w ~= w or self.h ~= h then
        self.w = w
        self.h = h
        self._dirty = true
    end
    if self.patch0:isDirty(self) then
        self._dirty = true
    end
    if self._dirty then
        self._main:onChange(self)
        self._dirty = false
    end
    self._main:onUpdate(self, time)
end


---#ifdef DEV
--//function TextAnim:onUpdate (comp, dt)
--//    self:seek(self.seekTime or math.min(self.curTime, self.duration))
--//    self.curTime = self.curTime + dt
--//end
--//function TextAnim:onEvent (comp, event)
--//    if event.type ~= Amaz.EventType.TOUCH then
--//        return
--//    end
--//    local pointer = event.args:get(0)
--//    if pointer.type == Amaz.TouchType.TOUCH_BEGAN then
--//        self.curTime = 0
--//        self:onEnter()
--//    elseif pointer.type == Amaz.TouchType.TOUCH_MOVED then
--//        self.seekTime = pointer.x * self.duration
--//    elseif pointer.type == Amaz.TouchType.TOUCH_ENDED or pointer.type == Amaz.TouchType.TOUCH_CANCELLED then
--//        self.seekTime = nil
--//        self:onLeave()
--//    end
--//end
---#endif


local exports = exports or {}
exports.TextAnim = TextAnim
return exports