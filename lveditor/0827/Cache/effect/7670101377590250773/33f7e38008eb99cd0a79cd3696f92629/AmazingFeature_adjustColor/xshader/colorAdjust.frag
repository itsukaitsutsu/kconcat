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

float adjustShadow(float x, float p)
{
    float res = x;
    if (p < 1.) {
        if (x < 0.) {
            float bias = (pow(0.01, p) + 0.000099 * (p - 1.)) * (1. - p);
            res = x + bias;
        } else if (x <= 1.) {
            float t = 0.99 * x + 0.01;
            float t2 = t * t;
            float t3 = t2 * t;
            res = x * p + (pow(t, p) + (p - 1.) * (t2 - t3)) * (1. - p);
        }
    } else {
        if (x <= 1. && x >= 0.) {
            float x2 = x * x;
            float x3 = x2 * x;
            res = pow(x, p) + (p - 1.) * (x2 - x3) + 0.035 * (1. - p) * (1. - x);
        }
    }
    return res;
}

float adjustHighlight(float x, float p)
{
    float res = x;
    if (p > 1.) {
        if (x >= 0. && x <= 1.) {
            float t = 1. - x;
            float t2 = t * t;
            float t3 = t2 * t;
            res = 1. - pow(t, p) - (p - 1.) * (t2 - t3) - 0.033 * (1. - p) * x;
        }
    } else {
        if (x > 1.) {
            float bias = -1. + p + (1. - pow(0.01, p) - 0.000099 * (p - 1.)) * (1. - p);
            res = x + bias;
        } else if (x >= 0.) {
            float t = -0.99 * x + 1.;
            float t2 = t * t;
            float t3 = t2 * t;
            res = x * p + (1. - pow(t, p) - (p - 1.) * (t2 - t3)) * (1. - p);
        }
    }
    return res;
}

float adjustFade(float x, float p)
{
    float res = x;
    if (x < 0.5) {
        float s = -0.82 * p;
        res = x - s * pow(0.5 - x, 1.6);
        if (x < 0.) {
            res = (1. + 1.6 * s * pow(0.5, 0.6)) * x - s * pow(0.5, 1.6);
        }
    } else {
        float s = -0.59 * p;
        res = x + s * pow(0.5 - x, 2.);
        if (x > 1.) {
            res = (1. + s) * x - 0.75 * s;
        }
    }
    return res;
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
    float gridNum = gridNumX * gridNumY;
    float lutWidth = gridSampleX * gridNumX;
    float lutHeight = gridSampleY * gridNumY;
    float gridSizeX = 1. / gridNumX;
    float gridSizeY = 1. / gridNumY;

    float blue = color.b * (gridNum - 1.);
    vec2 gridPos = vec2(0.);
    float gridIndex = floor(blue);
    gridPos.y = floor(gridIndex / gridNumX);
    gridPos.x = gridIndex - gridPos.y * gridNumX;
    float scaleX = 1. / gridNumX - 1. / lutWidth;
    float scaleY = 1. / gridNumY - 1. / lutHeight;
    vec2 texturePos = vec2(0.);
    texturePos.x = gridPos.x * gridSizeX + 0.5 / lutWidth + scaleX * color.r;
    texturePos.y = gridPos.y * gridSizeY + 0.5 / lutHeight + scaleY * color.g;
    vec4 color1 = texture2D(lutTexture, texturePos);

    gridIndex = ceil(blue);
    gridPos.y = floor(gridIndex / gridNumX);
    gridPos.x = gridIndex - gridPos.y * gridNumX;
    texturePos.x = gridPos.x * gridSizeX + 0.5 / lutWidth + scaleX * color.r;
    texturePos.y = gridPos.y * gridSizeY + 0.5 / lutHeight + scaleY * color.g;
    vec4 color2 = texture2D(lutTexture, texturePos);

    vec4 resultColor = mix(color1, color2, fract(blue));
    return resultColor;
}

vec3 processOverHighlight(vec3 dstColor, vec3 srcColor, float xBias, float yBias, float slope)
{
    vec3 tmpColor = srcColor * slope + yBias;
    if (srcColor.r > xBias) {
        dstColor.r = tmpColor.r;
    }
    if (srcColor.g > xBias) {
        dstColor.g = tmpColor.g;
    }
    if (srcColor.b > xBias) {
        dstColor.b = tmpColor.b;
    }
    return dstColor;
}

vec3 processUnderShadow(vec3 dstColor, vec3 srcColor, float xBias, float yBias, float slope)
{
    vec3 tmpColor = srcColor * slope + yBias;
    if (srcColor.r < xBias) {
        dstColor.r = tmpColor.r;
    }
    if (srcColor.g < xBias) {
        dstColor.g = tmpColor.g;
    }
    if (srcColor.b < xBias) {
        dstColor.b = tmpColor.b;
    }
    return dstColor;
}

void main()
{
    vec4 color = texture2D(u_inputTexture, uv0);

    if (u_enableBlackWhite == TRUE) {
        color.rgb = u_blackWhiteSlope * color.rgb + u_blackWhiteBias;
    }

    if (u_enableBrightness == TRUE) {
        vec4 oriColor = color;
        if (u_brightnessParam > 0.) {
            float p = 1.0 + u_brightnessParam * 5.0;
            color.rgb = 1. - pow(1. - color.rgb, vec3(p));
            float yBias = pow(0.001, 1. / p);
            float xBias = 1. - yBias;
            color.rgb = processOverHighlight(color.rgb, oriColor.rgb, xBias, yBias, 1.);
            color.rgb = processUnderShadow(color.rgb, oriColor.rgb, 0., 0., 1.);
        } else {
            float p = 1.0 / (1.0 - u_brightnessParam * 2.5);
            color.rgb += u_brightnessParam * 0.01 * color.rgb;
            color.rgb = 1. - pow(1. - color.rgb, vec3(p));
            float yBias = -pow(-u_brightnessParam * 0.01, p);
            color.rgb = processOverHighlight(color.rgb, oriColor.rgb, 1., yBias, 1.);
            color.rgb = processUnderShadow(color.rgb, oriColor.rgb, 0., 0., 1.);
        }
    }

    if (u_enableHighlight == TRUE) {
        color.r = adjustHighlight(color.r, u_highlightParam);
        color.g = adjustHighlight(color.g, u_highlightParam);
        color.b = adjustHighlight(color.b, u_highlightParam);
    }

    if (u_enableShadow == TRUE) {
        color.r = adjustShadow(color.r, u_shadowParam);
        color.g = adjustShadow(color.g, u_shadowParam);
        color.b = adjustShadow(color.b, u_shadowParam);
    }

    if (u_enableContrast == TRUE) {
        if (u_contrastParam <= 1.0) {
            color.rgb = u_contrastParam * (color.rgb - u_contrastPivot) + u_contrastPivot;
        } else {
            color.r = enchanceContrast(color.r, u_contrastXFactor, u_contrastPivot);
            color.g = enchanceContrast(color.g, u_contrastXFactor, u_contrastPivot);
            color.b = enchanceContrast(color.b, u_contrastXFactor, u_contrastPivot);
        }
    }

    if (u_enableFade == TRUE) {
        color.r = adjustFade(color.r, u_fadeParam);
        color.g = adjustFade(color.g, u_fadeParam);
        color.b = adjustFade(color.b, u_fadeParam);
    }

    if (u_enableSaturation == TRUE) {
        vec3 oriColor = color.rgb;
        vec3 baseAdjustVec = SaturationLuminanceFactor * (1. - u_saturationParam);

        vec3 adjustVec = baseAdjustVec + vec3(u_saturationParam, 0., 0.);
        color.r = dot(oriColor, adjustVec);

        adjustVec = baseAdjustVec + vec3(0., u_saturationParam, 0.);
        color.g = dot(oriColor, adjustVec);

        adjustVec = baseAdjustVec + vec3(0., 0., u_saturationParam);
        color.b = dot(oriColor, adjustVec);
    }

    if (u_enableTemperatureTint == TRUE) {
        vec3 rgb = color.rgb;
        color.r = dot(rgb, u_temperatureTintRedVec3);
        color.g = dot(rgb, u_temperatureTintGreenVec3);
        color.b = dot(rgb, u_temperatureTintBlueVec3);
    }

    if (u_enableLightSensation == TRUE) {
        vec4 newColor = color;
        vec4 oriColor = color;
        if (u_lightSensationParam <= 0.0) {
            newColor = applyLut3D(color, u_lightSensationMinTexture, 17., 17., 17., 1.);
        } else {
            newColor = applyLut3D(color, u_lightSensationMaxTexture, 17., 17., 17., 1.);
        }
        color.rgb = mix(color.rgb, newColor.rgb, abs(u_lightSensationParam));
        color.rgb = processOverHighlight(color.rgb, oriColor.rgb, 1., 0., 1.);
        color.rgb = processUnderShadow(color.rgb, oriColor.rgb, 0., 0., 1.);
    }

    gl_FragColor = color;
}
