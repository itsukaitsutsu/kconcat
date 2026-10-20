precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;

uniform int u_enableTemperatureTint;
uniform int u_enableHighlight;
uniform int u_enableShadow;
uniform int u_enableSaturation;
uniform int u_enableBrightness;
uniform int u_enableContrast;
uniform int u_enableBlackWhite;

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

#define TRUE 1
#define FALSE 0

const vec3 SaturationLuminanceFactor = vec3(0.208540, 0.702086, 0.089374);

vec3 adjustShadow(vec3 color, float p)
{
    vec3 color2 = color * color; // color2 = color ^ 2, color3 = color ^ 3, and so on
    vec3 color3 = color2 * color;
    vec3 resColor = pow(color, vec3(p)) + (p - 1.0) * (color2 - color3);
    return resColor;
}

vec3 adjustHighlight(vec3 color, float p)
{
    vec3 t = 1. - color;
    vec3 t2 = t * t; // t2 = t ^ 2, t3 = t ^ 3, and so on
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


void main()
{
    // sequence for color correction: 
    // contrast > highlight > shadow > while > black > brightness > temperature > tint > saturation
    vec4 color = texture2D(u_inputTexture, uv0);

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

    if (u_enableTemperatureTint == TRUE) {
        vec3 rgb = color.rgb;
        color.r = dot(rgb, u_temperatureTintRedVec3);
        color.g = dot(rgb, u_temperatureTintGreenVec3);
        color.b = dot(rgb, u_temperatureTintBlueVec3);
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

    gl_FragColor = clamp(color, 0., 1.);
}
