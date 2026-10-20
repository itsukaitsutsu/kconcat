precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;
uniform sampler2D u_lutTexture;
uniform sampler2D u_skinSegTexture;
uniform vec4 u_ScreenParams;

// parameters for color range and shadow/highlight map
uniform float u_shadowMapParam;
uniform float u_highlightMapParam;
uniform int u_displayOriginalColor;
uniform int u_displayRangeMap;
uniform int u_displaySaturationSuppressionFactor;
uniform int u_useRangeMap;
uniform int u_colorRange;
uniform int u_mapType;

// flag for enabling color adjustment
uniform int u_enableHighlight;
uniform int u_enableShadow;
uniform int u_enableLinearOps;
uniform int u_enableContrast;
uniform int u_enableTemperatureTint;
uniform int u_enableSaturation;
uniform int u_enableLut;
uniform int u_enableDither;
uniform int u_enableSaturationSuppresion;

// parameters for color adjustment
uniform float u_highlightParam;
uniform float u_shadowParam;
uniform float u_blackWhiteSlope;
uniform float u_blackWhiteBias;
uniform float u_exposure;
uniform float u_offsetIntensity;
uniform float u_contrastIntensity;
uniform float u_contrastPivot;
uniform float u_saturationSuppressionIntensity;
uniform float u_saturationSuppressionThreshold;
uniform vec3 u_temperatureTintRedVec3;
uniform vec3 u_temperatureTintGreenVec3;
uniform vec3 u_temperatureTintBlueVec3;
uniform float u_saturationParam;
uniform float u_lutIntensity;
uniform float u_lutColorSample;
uniform float u_lutHorizontalGridNum;
uniform float u_lutVerticalGridNum;

#define FALSE 0
#define TRUE 1
#define MAP_TYPE_HARD 0
#define MAP_TYPE_SOFT 1
#define COLOR_RANGE_SHADOW 0
#define COLOR_RANGE_HIGHLIGHT 1
#define COLOR_RANGE_SKIN 2
#define BLUR_SAMPLE 2

const float Eps = 1e-5;
const vec3 SaturationLuminanceFactor = vec3(0.208540, 0.702086, 0.089374);

// --------- BEGIN: functions for color model conversion ---------
vec3 rgb2hsl(vec3 rgb) 
{
    float h = 0., s = 0., l = 0.;
    float r = rgb.r;
    float g = rgb.g;
    float b = rgb.b;
    float cmax = max(r, max(g, b));
    float cmin = min(r, min(g, b));
    float delta = cmax - cmin;
    l = (cmax + cmin) / 2.;
    if (delta < Eps) {
        s = 0.;
        h = 0.;
    } else {
        if (l <= .5) {
            s = delta / (cmax + cmin);
        } else {
            s = delta / (2. - (cmax + cmin));
        }
        
        if (cmax - r < Eps) {
            if (g >= b) {
                h = 60. * (g - b) / delta;
            } else {
                h = 60. * (g - b) / delta + 360.;
            } 
        } else if (cmax - g < Eps) {
            h = 60. * (b - r) / delta + 120.;
        } else {
            h = 60. * (r - g) / delta + 240.;
        }
    }
    return vec3(h, s, l);
}

float hue2rgb(float p, float q, float t) 
{
    if (t < 0.) {
        t += 1.;
    }
    if (t > 1.) {
        t -= 1.;
    }
    if (t < 1. / 6.) {
        return p + (q - p) * 6. * t;
    }
    if (t < 1. / 2.) {
        return q;
    }
    if (t < 2. / 3.) {
        return p + (q - p) * (2. / 3. - t) * 6.;
    }
    return p;
}

vec3 hsl2rgb(vec3 hsl) 
{
    float r, g, b;
    float h = hsl.x / 360.;
    if (hsl.y == 0.) {
        r = g = b = hsl.z;  // gray
    } else {
        float q = hsl.z < .5 ? hsl.z * (1. + hsl.y) : (hsl.z + hsl.y - hsl.z * hsl.y);
        float p = 2. * hsl.z - q;
        r = hue2rgb(p, q, h + 1. / 3.);
        g = hue2rgb(p, q, h);
        b = hue2rgb(p, q, h - 1. / 3.);
    }
    return vec3(r, g, b);
}

float rgb2gray(vec3 rgb)
{
    // get Y channel from BT.709
    return dot(rgb, vec3(0.2126, 0.7152, 0.0722));
}
// --------- END: functions for color model conversion ---------


// --------- BEGIN: functions for shadow/highlight/skin map selection ---------
float getShadowMap(float color, float p)
{
    float res = 1.;
    if (u_mapType == MAP_TYPE_HARD) {
        if (p < 1.) {
            res = 1. - color / max(p, Eps);
        } else {
            res = 1. - (2. - p) * color;
        }
    } else {
        res = pow(1. - color, (2. - p) / max(p, Eps));
    }
    return res;
}

float getHighlightMap(float color, float p)
{
    float res = 1.;
    if (u_mapType == MAP_TYPE_HARD) {
        if (p < 1.) {
            res = (color - 1.) / max(p, Eps) + 1.;
        } else {
            res = (2. - p) * (color - 1.) + 1.;
        }
    } else {
        res = pow(color, (2. - p) / max(p, Eps));
    }
    return res;
}

float getSkinMap(sampler2D skinSegTex, vec2 uv)
{
    return texture2D(skinSegTex, vec2(uv.x, 1. - uv.y)).a;
}

float getMap(float color)
{
    float res = 1.;
    if (u_colorRange == COLOR_RANGE_SHADOW) {
        res = getShadowMap(color, u_shadowMapParam);
    } else if (u_colorRange == COLOR_RANGE_HIGHLIGHT) {
        res = getHighlightMap(color, u_highlightMapParam);
    } else {
        res = getSkinMap(u_skinSegTexture, uv0);
    }
    res = clamp(res, 0., 1.);
    return res;
}

float getSaturationSuppressionMap(vec3 color)
{
    float threshold = u_saturationSuppressionThreshold;
    float r = color.r < threshold ? 0. : (color.r - threshold) / (1. - threshold);
    float g = color.g < threshold ? 0. : (color.g - threshold) / (1. - threshold);
    float b = color.b < threshold ? 0. : (color.b - threshold) / (1. - threshold);
    float map = (r + g + b) / max(color.r + color.g + color.b, Eps);
    map = 1. - map;
    map = map * map;
    return map;
}
// --------- END: functions for shadow/highlight map selection ---------


// --------- BEGIN: functions for color adjustment ---------
vec3 adjustShadow(vec3 color, float p)
{
    vec3 resColor = color;
    if (p <= 0.) {
        resColor = pow(color, vec3(p + 1.)) - 0.1 * p * pow(1. - color, vec3(5.));
    } else {
        resColor = pow(color, vec3(1. / max(1. - p, Eps)));
    }
    return resColor;
}

vec3 adjustHighlight(vec3 color, float p)
{
    vec3 resColor = color;
    if (p < 0.) {
        resColor = 1. - pow(1. - color - 0.1 * p * color, vec3(p + 1.));
    } else {
        resColor = 1. - pow(1. - color, vec3(1. / max(1. - p, Eps)));
    }
    return resColor;
}

float enchanceContrast(float x, float pivot, float p)
{
    float res = x;
    if (x <= pivot) {
        res = pivot * pow(x / max(pivot, Eps), p);
    } else {
        float pivotInv = 1. - pivot;
        res = 1. - pivotInv * pow((1. - x) / max(pivotInv, Eps), p);
    }
    return res;
}

vec4 applyLut3D(vec4 color, sampler2D lutTexture, float gridSampleX, float gridSampleY, float gridNumX, float gridNumY)
{
    // gridSample and gridNum: for a ordinary 512*512 3D LUT, gridSampleX = gridSampleY = 64, gridNumX = gridNumY = 8

    float gridNum = gridNumX * gridNumY;
    float lutWidth = gridSampleX * gridNumX;
    float lutHeight = gridSampleY * gridNumY;
    float gridSizeX = 1. / gridNumX; // grid size is in uv scale (0, 1)
    float gridSizeY = 1. / gridNumY;

    // 1. find adjacent color grids using blue channel
    // 2. look up (interpolate) red and green values in two grids respectively
    // 3. interoplate color1 and color2 using fract(blue) as weight to get final result

    // get color1
    float blue = color.b * (gridNum - 1.);
    vec2 gridPos = vec2(0.); // gridPos is 2D index for red-green grid, which is integer
    float gridIndex = floor(blue);
    gridPos.y = floor(gridIndex / gridNumX);
    gridPos.x = gridIndex - gridPos.y * gridNumX;
    float scaleX = 1. / gridNumX - 1. / lutWidth;
    float scaleY = 1. / gridNumY - 1. / lutHeight;
    vec2 texturePos = vec2(0.); // texturePos is uv coordinate
    texturePos.x = gridPos.x * gridSizeX + 0.5 / lutWidth + scaleX * color.r;
    texturePos.y = gridPos.y * gridSizeY + 0.5 / lutHeight + scaleY * color.g;
    texturePos.y = 1. - texturePos.y;
    vec4 color1 = texture2D(lutTexture, texturePos);

    // get color2, the computation of `gridPos` and `texturePos` are exactly the same with preceding code
    gridIndex = ceil(blue);
    gridPos.y = floor(gridIndex / gridNumX);
    gridPos.x = gridIndex - gridPos.y * gridNumX;
    texturePos.x = gridPos.x * gridSizeX + 0.5 / lutWidth + scaleX * color.r;
    texturePos.y = gridPos.y * gridSizeY + 0.5 / lutHeight + scaleY * color.g;
    texturePos.y = 1. - texturePos.y;
    vec4 color2 = texture2D(lutTexture, texturePos);

    vec4 resultColor = mix(color1, color2, fract(blue));
    return resultColor;
}
// --------- END: functions for color adjustment ---------


void main()
{
    vec4 oriColor = texture2D(u_inputTexture, uv0);
    float oriGray = rgb2gray(oriColor.rgb);
    vec4 color = oriColor;
    float map = getMap(oriGray);

    // luminance adjustment
    if (u_enableHighlight == TRUE) {
        color.rgb = adjustHighlight(color.rgb, u_highlightParam);
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    if (u_enableShadow == TRUE) {
        color.rgb = adjustShadow(color.rgb, u_shadowParam);
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    if (u_enableLinearOps == TRUE) {
        color.rgb = u_blackWhiteSlope * color.rgb + u_blackWhiteBias;
        color.rgb = u_exposure * color.rgb + u_offsetIntensity;
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    if (u_enableContrast == TRUE) {
        if (u_contrastIntensity <= 0.) {
            float contrastParam = u_contrastIntensity + 1.;
            color.rgb = contrastParam * (color.rgb - u_contrastPivot) + u_contrastPivot;
        } else {
            float contrastParam = 1. / max(1. - u_contrastIntensity, Eps);
            color.r = enchanceContrast(color.r, u_contrastPivot, contrastParam);
            color.g = enchanceContrast(color.g, u_contrastPivot, contrastParam);
            color.b = enchanceContrast(color.b, u_contrastPivot, contrastParam);
        }
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    // result of luminance adjustment
    if (u_useRangeMap == TRUE) {
        color = mix(oriColor, color, map);
    }

    // Saturation Suppresion
    float satSupMap = getSaturationSuppressionMap(oriColor.rgb);    // satSup here stand for saturation suppression
    float satSupFactor = u_saturationSuppressionIntensity * satSupMap;
    if (u_enableSaturationSuppresion == TRUE) {
        vec3 hsl = rgb2hsl(color.rgb);
        float satLowLimit = mix(hsl.y, 0., satSupMap);
        hsl.y = mix(hsl.y, satLowLimit, satSupFactor);
        color.rgb = hsl2rgb(hsl);
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    if (u_displayOriginalColor == TRUE) {
        color = oriColor;
    } else if (u_displayRangeMap == TRUE) {
        color = vec4(map, map, map, 1.0);
    } else if (u_displaySaturationSuppressionFactor == TRUE) {
        color = vec4(satSupFactor, satSupFactor, satSupFactor, 1.0);
    }

    // chroma adjustment
    if (u_enableSaturation == TRUE) {
        vec3 rgb = color.rgb;
        // use a color vector to adjust saturation
        vec3 baseAdjustVec = SaturationLuminanceFactor * (1. - u_saturationParam);

        vec3 adjustVec = baseAdjustVec + vec3(u_saturationParam, 0., 0.);
        color.r = dot(rgb, adjustVec);

        adjustVec = baseAdjustVec + vec3(0., u_saturationParam, 0.);
        color.g = dot(rgb, adjustVec);

        adjustVec = baseAdjustVec + vec3(0., 0., u_saturationParam);
        color.b = dot(rgb, adjustVec);
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    if (u_enableTemperatureTint == TRUE) {
        vec3 rgb = color.rgb;
        color.r = dot(rgb, u_temperatureTintRedVec3);
        color.g = dot(rgb, u_temperatureTintGreenVec3);
        color.b = dot(rgb, u_temperatureTintBlueVec3);
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    // apply 3D LUT
    if (u_enableLut == TRUE) {
        vec4 lutColor = applyLut3D(color, u_lutTexture, u_lutColorSample, u_lutColorSample, u_lutHorizontalGridNum,
                                   u_lutVerticalGridNum);
        color = mix(color, lutColor, u_lutIntensity);
    }
    gl_FragColor = clamp(color, 0., 1.);
}
