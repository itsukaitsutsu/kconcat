local data = {}

local ae_compDurations = {0, 30}
data.ae_compDurations = ae_compDurations

local ae_effectType = 'xtEffect'
data.ae_effectType = ae_effectType

local ae_transitionInputIndex = {
}
data.ae_transitionInputIndex = ae_transitionInputIndex

local ae_durations = {
    ['LumiSurfaceBlur_101-effect0'] = {
        ['nodeDuration'] = {{0, 30}, },
        ['texDuration'] = {
            ['InputTex'] = {{0, 30}, },
        },
    },
    ['UsmSharpen_101-effect1'] = {
        ['nodeDuration'] = {{0, 30}, },
        ['texDuration'] = {
            ['InputTex'] = {{0, 30}, },
        },
    },
    ['LuaStrongSharpen_101-effect2'] = {
        ['nodeDuration'] = {{0, 30}, },
        ['texDuration'] = {
            ['InputTex'] = {{0, 30}, },
        },
    },
    ['LumiLayer_101-blend'] = {
        ['nodeDuration'] = {{0, 30}, },
        ['texDuration'] = {
            ['InputTex'] = {{0, 30}, },
            ['baseTex'] = {{0, 30}, },
        },
    },
}
data.ae_durations = ae_durations

local ae_attribute = {
    ['LumiSurfaceBlur_101-effect0'] = {
        ['intensity'] = 0.03,
        ['blurIntensity'] = 0.03
    },
    ['UsmSharpen_101-effect1'] = {
        ['radius'] = 35,
        ['intensity'] = 0.4,
        ['threshold'] = 0.1,
    },
    ['LuaStrongSharpen_101-effect2'] = {
        ['strength'] = 0.4,
        ['range'] = 0.05,
        ['quality'] = 0.2,
    },
    ['LumiLayer_101-blend'] = {
        ['hasBlend'] = true,
        ['hasMatte'] = false,
        ['hasTransform'] = false,
        ['layerType'] = 'Adjustment',
        ['blendMode'] = 'Normal',
    },
}
data.ae_attribute = ae_attribute

local ae_sliderInfos = {
}
data.ae_sliderInfos = ae_sliderInfos

local ae_fadeinInfos = {
    time = 0,
    infos = {
    }
}
data.ae_fadeinInfos = ae_fadeinInfos

local ae_fadeoutInfos = {
    time = 0,
    infos = {
    }
}
data.ae_fadeoutInfos = ae_fadeoutInfos

local ae_animationInfos = {
    animationMode = 0,
    loopStart = 0,
    speedInfo = {1, 0, 1, },
}
data.ae_animationInfos = ae_animationInfos

return data
