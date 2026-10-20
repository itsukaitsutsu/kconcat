local Utils = require("common/Utils")


---@class AEAdapter
local AEAdapter = {}
AEAdapter.__index = AEAdapter


function AEAdapter:new ()
    local self = setmetatable({}, AEAdapter)
    self._tracks = {}
    return self
end


---@param layerName string
---@param layerData table<string, table>
function AEAdapter:addKeyframes (layerName, layerData)
    for trackName, trackData in pairs(layerData) do
        local trackPath = layerName.."/"..trackName
        trackData.from_ae = true
        self._tracks[trackPath] = trackData
        if trackData.v then
            AEAdapter._solveBezierLength(trackData)
        end
    end
end


---@param layerName string
---@param layerData table
function AEAdapter:addFrames (layerName, layerData)
    local fps = layerData.frameRate
    layerData = layerData.layer0
    for attrName, attrData in pairs(layerData) do
        if type(attrData) == "table" then
            local trackPath = layerName.."."..attrName
            attrData.fps = fps
            self._tracks[trackPath] = attrData
        end
    end
end


---@param layerName string
---@param layerData table
function AEAdapter:addAnimations (layerName, layerData)
    for attrName, attrData in pairs(layerData) do
        local trackPath = layerName..":"..attrName
        self._tracks[trackPath] = attrData
    end
end


---@param path string
---@param time number
---@return number|number[]
function AEAdapter:get (path, time)
    local track = self._tracks[path]
    if not track then
        return
    end

    if track.fps then
        return self:_solveFrame(track, time * track.fps)
    elseif track.from_ae then
        return self:_solveKeyframe(track, time)
    else
        return self:_solveAnimation(track, time)
    end
end
---@param path string
---@param time number
---@param delta number|nil
---@return number|number[]
function AEAdapter:getVelocity (path, time, delta)
    delta = delta or 0.001
    local v0 = self:get(path, time - delta)
    local v1 = self:get(path, time + delta)
    delta = delta + delta
    local v = {}
    for i = 1, #v0 do
        v[i] = (v1[i] - v0[i]) / delta
    end
    return v
end


function AEAdapter:_solveFrame (track, frame)
    if #track > 0 then
        local v = self._interpolateFrame(track, frame)
        return type(v) == "number" and {v} or v
    else
        local x = self._interpolateFrame(track.x, frame)
        local y = self._interpolateFrame(track.y, frame)
        return {x, y}
    end
end
function AEAdapter._interpolateFrame (array, frame)
    local n = #array
    local t, k0, k1

    if frame <= 0 then
        k0 = array[1]
        k1 = array[1]
        t = 0
    elseif frame >= n - 1 then
        k0 = array[n]
        k1 = array[n]
        t = 0
    else
        local i = math.floor(frame)
        k0 = array[i + 1]
        k1 = array[i + 2]
        t = frame - i
    end

    if type(k0) == "table" then
        local x = k0.x + (k1.x - k0.x) * t
        local y = k0.y + (k1.y - k0.y) * t
        if k0.z then
            local z = k0.z + (k1.z - k0.z) * t
            return {x, y, z}
        else
            return {x, y}
        end
    else
        return k0 + (k1 - k0) * t
    end
end


function AEAdapter:_solveKeyframe (track, time)
    local K = track.k
    local N = #K
    if time <= K[1][1] then
        return Utils.table_slice(K[1], 2)
    elseif time >= K[N][1] then
        return Utils.table_slice(K[N], 2)
    end

    local interpolator = track.v and self._interpolateVector or self._interpolateScalar
    for i = 2, N do
        if time < K[i][1] then
            return interpolator(K[i - 1], K[i], track.s[i - 1], time)
        end
    end
    return Utils.table_slice(K[N], 2)
end
function AEAdapter._interpolateScalar (K0, K1, S, T)
    if S.h then
        return Utils.table_slice(K0, 2)
    end
    local C = S.c
    local y = {}
    for i = 1, #K0 - 1 do
        local x1 = K0[1]
        local x2 = C[i][1]
        local x3 = C[i][3]
        local x4 = K1[1]
        local y1 = K0[i + 1]
        local y2 = C[i][2]
        local y3 = C[i][4]
        local y4 = K1[i + 1]
        y[i] = Utils.bezier4x2y(x1, x2, x3, x4, y1, y2, y3, y4, T)
    end
    return y
end
function AEAdapter._interpolateVector (K0, K1, S, T)
    if S.h then
        return Utils.table_slice(K0, 2)
    end

    local C = S.c
    local P1 = S.p1
    local P2 = S.p2
    local L = S.l

    local t1 = K0[1]
    local t2 = C[1]
    local t3 = C[3]
    local t4 = K1[1]
    local l1 = 0
    local l2 = C[2]
    local l3 = C[4]
    local l4 = L[#L]
    local l = Utils.bezier4x2y(t1, t2, t3, t4, l1, l2, l3, l4, T)
    l = math.min(l, L[#L])

    local si = 1
    local ei = #L
    while si < ei - 1 do
        local i = math.floor((si + ei) * 0.5)
        if l < L[i] then
            ei = i
        else
            si = i
        end
    end

    local t = Utils.step(L[si], L[ei], l)
    t = ((si-1) + t) / (#L-1)

    local x1 = K0[2]
    local x2 = P1[1]
    local x3 = P2[1]
    local x4 = K1[2]
    local y1 = K0[3]
    local y2 = P1[2]
    local y3 = P2[2]
    local y4 = K1[3]
    local z1, z2, z3, z4
    if K0[4] then
        z1 = K0[4]
        z2 = P1[3]
        z3 = P2[3]
        z4 = K1[4]
    end
    local x, y, z = Utils.bezier4(t, x1, x2, x3, x4, y1, y2, y3, y4, z1, z2, z3, z4)
    return {x, y, z}
end


function AEAdapter:_solveAnimation (track, time)
    local N = #track
    if time <= track[1][1] then
        return track[1][3]
    elseif time >= track[N][2] then
        return track[N][4]
    end

    for i = 1, N do
        local frag = track[i]
        if time <= frag[2] then
            local x = Utils.step(frag[1], frag[2], time)
            local m = frag[5]
            local y = type(m) == "function" and m(x) or Utils.bezier4x2y(0, m[1], m[3], 1, 0, m[2], m[4], 1, x)
            local v0 = frag[3]
            local v1 = frag[4]
            local v = {}
            for j = 1, #v0 do
                v[j] = Utils.mix(v0[j], v1[j], y)
            end
            return v
        end
    end
    return track[N][4]
end

function AEAdapter._splitBezierCurve (segments, K0, P1, P2, K3)
    local L = {0}
    local x0, y0, z0 = K0[2], K0[3], K0[4]
    for i = 1, segments do
        local t = i / segments
        local x1, y1, z1 = Utils.bezier4(t, K0[2], P1[1], P2[1], K3[2], K0[3], P1[2], P2[2], K3[3], K0[4], P1[3], P2[3], K3[4])
        local dx = x1 - x0
        local dy = y1 - y0
        local dz = z0 and z1 - z0 or 0
        local dL = math.sqrt(dx*dx + dy*dy + dz*dz)
        table.insert(L, L[#L] + dL)
        x0, y0, z0 = x1, y1, z1
    end
    return L
end
function AEAdapter._solveBezierLength (track)
    for i, s in ipairs(track.s) do
        if not s.h then
            local k0 = track.k[i]
            local k3 = track.k[i+1]
            s.l = AEAdapter._splitBezierCurve(128, k0, s.p1, s.p2, k3)
        end
    end
end


return AEAdapter

