---@class LumiUsmSharpen: ScriptComponent
---@field radius double [UI(Range={0, 100}, Slider)]
---@field intensity double [UI(Range={0, 5}, Slider)]
---@field threshold double [UI(Range={0, 0.2}, Slider)]
---@field InputTex Texture
---@field OutputTex Texture
---@field lumiSharedRt Vector [UI(Type="Texture")]

local exports = exports or {}
local LumiUsmSharpen = LumiUsmSharpen or {}
LumiUsmSharpen.__index = LumiUsmSharpen


function LumiUsmSharpen.new(construct, ...)
    local self = setmetatable({}, LumiUsmSharpen)

    if construct and LumiUsmSharpen.constructor then LumiUsmSharpen.constructor(self, ...) end
    -- interface parameters to users
    self.radius = 0.0
    self.intensity = 0.0
    self.threshold = 0.0

    -- constant parameters
    self.SigmaScale = 2.5   -- for computing gaussian blur sigma, which is `radius / SigmaScale`
    self.MinSample = 9.0
    self.MaxSample = 31.0

    -- Lua input and output textures
    self.InputTex = nil
    self.OutputTex = nil

    return self
end


function LumiUsmSharpen:constructor()
end


function LumiUsmSharpen:onStart(comp)
    self.matGaussianBlurX = comp.entity:searchEntity("EntityGaussianBlurX"):getComponent("MeshRenderer").material
    self.matGaussianBlurY = comp.entity:searchEntity("EntityGaussianBlurY"):getComponent("MeshRenderer").material
    self.camGaussianBlurX = comp.entity:searchEntity("CameraGaussianBlurX"):getComponent("Camera")
    self.camGaussianBlurY = comp.entity:searchEntity("CameraGaussianBlurY"):getComponent("Camera")

    if self.lumiSharedRt and self.lumiSharedRt:size() > 0 then
        self.midTex = self.lumiSharedRt:get(0)
    end

    local textureWidth = self.OutputTex.width
    local textureHeight = self.OutputTex.height
    if self.midTex.width ~= textureWidth or self.midTex.height ~= textureHeight then
        self.midTex.width = textureWidth
        self.midTex.height = textureHeight
    end
end

local function remap(x, a, b)
    -- rectify x to between [a, b]
    return math.min(math.max(x, a), b)
end

function LumiUsmSharpen:onUpdate(comp, deltaTime)
    -- set Lua input and output
    self.matGaussianBlurX:setTex("u_inputTexture", self.InputTex)
    self.matGaussianBlurY:setTex("u_inputTexture", self.InputTex)
    self.camGaussianBlurX.renderTexture = self.midTex
    self.camGaussianBlurY.renderTexture = self.OutputTex
    self.matGaussianBlurY:setTex("u_albedo", self.midTex)

    -- compute gaussian blur parameters
    local textureWidth = Amaz.BuiltinObject.getOutputTextureWidth()
    local textureHeight = Amaz.BuiltinObject.getOutputTextureHeight()
    local radiusX = self.radius / textureWidth
    local radiusY = self.radius / textureHeight
    local sigmaX = radiusX / self.SigmaScale
    local sigmaY = radiusY / self.SigmaScale
    local sample = math.ceil(self.radius) * 2.0 + 1.0
    sample = remap(sample, self.MinSample, self.MaxSample)
    local dx = 2.0 * radiusX / (sample - 1.0)
    local dy = 2.0 * radiusY / (sample - 1.0)

    -- set parameters for GaussianBlurX material
    self.matGaussianBlurX:setFloat("u_radiusX", radiusX)
    self.matGaussianBlurX:setFloat("u_sigmaX", sigmaX)
    self.matGaussianBlurX:setFloat("u_dx", dx)

    -- set parameters for GaussianBlurY material. Sharpen is also implemented in this pass
    self.matGaussianBlurY:setFloat("u_radiusY", radiusY)
    self.matGaussianBlurY:setFloat("u_sigmaY", sigmaY)
    self.matGaussianBlurY:setFloat("u_dy", dy)
    self.matGaussianBlurY:setFloat("u_intensity", self.intensity)
    self.matGaussianBlurY:setFloat("u_threshold", self.threshold)
end


exports.LumiUsmSharpen = LumiUsmSharpen
return exports
