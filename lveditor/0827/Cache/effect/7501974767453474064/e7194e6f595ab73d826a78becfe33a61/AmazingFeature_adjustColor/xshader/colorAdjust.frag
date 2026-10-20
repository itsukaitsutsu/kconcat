precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;
uniform sampler2D u_lightSensationMinTexture;
uniform sampler2D u_lightSensationMaxTexture;
uniform sampler2D u_fadeMaxTexture;

uniform int u_enableTemperatureTint;
uniform int u_enableHighlight;
uniform int u_enableShadow;
uniform int u_enableSaturation;
uniform int u_enableBrightness;
uniform int u_enableContrast;
uniform int u_enableBlackWhite;
uniform int u_enableLightSensation;
uniform int u_enableFade;

uniform vec3 u_temperatureTintRedVec3;
uniform vec3 u_temperatureTintGreenVec3;
uniform vec3 u_temperatureTintBlueVec3;
uniform float u_saturationParam;
uniform float u_brightnessParam;
uniform float u_contrastParam;
uniform float u_contrastPivot;
uniform float u_contrastXFactor;
uniform float u_contrastLeftDiff;
uniform float u_contrastRightDiff;
uniform float u_contrastLeftSlopeDiff;
uniform float u_contrastRightSlopeDiff;
uniform float u_contrastPivotSlope;
uniform float u_highlightParam;
uniform float u_shadowParam;
uniform float u_blackWhiteSlope;
uniform float u_blackWhiteBias;
uniform float u_lightSensationParam;
uniform float u_fadeParam;

#define TRUE 1
#define FALSE 0

const vec3 SaturationLuminanceFactor = vec3(0.208540, 0.702086, 0.089374);

vec3 adjustShadow(vec3 color, float p)
{
    vec3 color2 = color * color;
    vec3 color3 = color2 * color;
    vec3 resColor = pow(color, vec3(p)) + (p - 1.0) * (color2 - color3);
    return resColor;
}

vec3 adjustHighlight(vec3 color, float p)
{
    vec3 t = 1. - color;
    vec3 t2 = t * t;
    vec3 t3 = t2 * t;
    vec3 resColor = 1.0 - pow(t, vec3(p)) - (p - 1.0) * (t2 - t3);
    return resColor;
}

float getContrastSigmoidValue(float x, float xFactor, float pivot)
{
    float res = 1.0 / (1.0 + exp(-xFactor * (x - pivot))) + pivot - 0.5;
    return res;
}

float getContrastSigmoidDerivative(float x, float xFactor, float pivot)
{
    float s = 1.0 / (1.0 + exp(-xFactor * (x - pivot)));
    float k = xFactor * s * (1.0 - s);
    return k;
}

float enchanceContrast(float x, float xFactor, float pivot)
{
    float res = getContrastSigmoidValue(x, xFactor, pivot);
    float k = getContrastSigmoidDerivative(x, xFactor, pivot);
    if (x <= pivot) {
        float scale = (u_contrastPivotSlope - k) / u_contrastLeftSlopeDiff;
        scale = scale * scale;
        res = res + scale * u_contrastLeftDiff;
    } else {
        float scale = (u_contrastPivotSlope - k) / u_contrastRightSlopeDiff;
        scale = scale * scale;
        res = res + scale * u_contrastRightDiff;
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
    vec4 color1 = texture2D(lutTexture, texturePos);

    // get color2, the computation of `gridPos` and `texturePos` are exactly the same with preceding code
    gridIndex = ceil(blue);
    gridPos.y = floor(gridIndex / gridNumX);
    gridPos.x = gridIndex - gridPos.y * gridNumX;
    texturePos.x = gridPos.x * gridSizeX + 0.5 / lutWidth + scaleX * color.r;
    texturePos.y = gridPos.y * gridSizeY + 0.5 / lutHeight + scaleY * color.g;
    vec4 color2 = texture2D(lutTexture, texturePos);

    vec4 resultColor = mix(color1, color2, fract(blue));
    return resultColor;
}

void main()
{
    vec4 color = texture2D(u_inputTexture, uv0);

    if (u_enableBrightness == TRUE) {
        // the following part is to finetune parameter for pow function.
        float p = 0.;
        if (u_brightnessParam > 0.) {
            p = 1.0 + u_brightnessParam * 5.0;
        } else {
            p = 1.0 / (1.0 - u_brightnessParam * 2.5);
            color.rgb -= -u_brightnessParam * 0.01; // add a small negative offset when darkening brightness.
        }
        // the following equation `y = 1 - pow(1 - x, p)` is the result of
        // rotating `y = pow(x, p)` 180 degree around (0.5, 0.5)
        color.rgb = 1. - pow(1. - color.rgb, vec3(p));
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    if (u_enableContrast == TRUE) {
        if (u_contrastParam <= 1.0) {
            color.rgb = u_contrastParam * (color.rgb - u_contrastPivot) + u_contrastPivot;
        } else {
            color.r = enchanceContrast(color.r, u_contrastXFactor, u_contrastPivot);
            color.g = enchanceContrast(color.g, u_contrastXFactor, u_contrastPivot);
            color.b = enchanceContrast(color.b, u_contrastXFactor, u_contrastPivot);
        }
        color.rgb = clamp(color.rgb, 0., 1.);
    }

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

    if (u_enableHighlight == TRUE) {
        color.rgb = adjustHighlight(color.rgb, u_highlightParam);
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    if (u_enableShadow == TRUE) {
        color.rgb = adjustShadow(color.rgb, u_shadowParam);
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    if (u_enableBlackWhite == TRUE) {
        color.rgb = u_blackWhiteSlope * color.rgb + u_blackWhiteBias;
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    if (u_enableTemperatureTint == TRUE) {
        vec3 rgb = color.rgb;
        color.r = dot(rgb, u_temperatureTintRedVec3);
        color.g = dot(rgb, u_temperatureTintGreenVec3);
        color.b = dot(rgb, u_temperatureTintBlueVec3);
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    if (u_enableFade == TRUE) {
        vec4 fadeColor = applyLut3D(color, u_fadeMaxTexture, 17., 17., 17., 1.);
        color.rgb = mix(color.rgb, fadeColor.rgb, u_fadeParam);
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    if (u_enableLightSensation == TRUE) {
        vec4 newColor = color;
        if (u_lightSensationParam <= 0.0) {
            newColor = applyLut3D(color, u_lightSensationMinTexture, 17., 17., 17., 1.);
        } else {
            newColor = applyLut3D(color, u_lightSensationMaxTexture, 17., 17., 17., 1.);
        }
        color.rgb = mix(color.rgb, newColor.rgb, abs(u_lightSensationParam));
        color.rgb = clamp(color.rgb, 0., 1.);
    }

    gl_FragColor = clamp(color, 0., 1.);
}
