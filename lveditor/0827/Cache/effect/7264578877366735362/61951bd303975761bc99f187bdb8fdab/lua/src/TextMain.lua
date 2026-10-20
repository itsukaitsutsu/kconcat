

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


return TextMain