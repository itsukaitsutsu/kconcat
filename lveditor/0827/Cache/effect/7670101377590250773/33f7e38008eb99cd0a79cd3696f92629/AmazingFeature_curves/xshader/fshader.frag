precision highp float;
varying highp vec2 uv0;

uniform sampler2D inputImageTexture;
uniform sampler2D lutY;
uniform sampler2D lutR;
uniform sampler2D lutG;
uniform sampler2D lutB;
uniform float intensityY;
uniform float intensityR;
uniform float intensityG;
uniform float intensityB;

const float BORDER_SLOPE_STEP = 0.00390625; // = 1/256
const float BORDER_SLOPE_LIMIT = 10.;

float vec2ToFloat(vec2 val){
    float res = val.x + val.y/255.0;
    return res;
}

float vec4ToFloat(vec4 val){
    float res = val.x * 255.0 + val.y + val.z/255.0;
    res = val.w < 0.5 ? res : -res;
    return res;
}

float rgb2gray(vec3 rgb)
{
    return 0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b;
}

float applyLutCurve(float src, sampler2D lutCurve)
{
    float res = src;
    if (src < 0.0) {
        float y1 = vec4ToFloat(texture2D(lutCurve, vec2(0., 0.5)));
        float y2 = vec4ToFloat(texture2D(lutCurve, vec2(BORDER_SLOPE_STEP, 0.5)));
        float slope = (y2 - y1) / BORDER_SLOPE_STEP;
        slope = clamp(slope, -BORDER_SLOPE_LIMIT, BORDER_SLOPE_LIMIT);
        res = y1 + slope * src;
    } else if (src > 1.0) {
        float y1 = vec4ToFloat(texture2D(lutCurve, vec2(1. - BORDER_SLOPE_STEP, 0.5)));
        float y2 = vec4ToFloat(texture2D(lutCurve, vec2(1., 0.5)));
        float slope = (y2 - y1) / BORDER_SLOPE_STEP;
        slope = clamp(slope, -BORDER_SLOPE_LIMIT, BORDER_SLOPE_LIMIT);
        res = y2 + slope * (src - 1.0);
    } else {
        res = vec4ToFloat(texture2D(lutCurve, vec2(src, 0.5)));
    }
    return res;
}

vec4 processY(vec4 srcColor, vec4 lastColor)
{
    vec4 resColor = lastColor;
    float ySrc = rgb2gray(srcColor.rgb);
    float yLast = rgb2gray(lastColor.rgb);
    float yNew = applyLutCurve(ySrc, lutY);
    float yIncr = (yNew - ySrc) * intensityY - (yLast - ySrc);
    resColor.rgb += yIncr;
    return resColor;
}

vec4 processR(vec4 srcColor, vec4 lastColor)
{
    vec4 resColor = lastColor;
    resColor.r = applyLutCurve(srcColor.r, lutR);
    float rIncr = (resColor.r - srcColor.r) * intensityR;
    resColor.r = lastColor.r + rIncr;
    return resColor;
}

vec4 processG(vec4 srcColor, vec4 lastColor)
{
    vec4 resColor = lastColor;
    resColor.g = applyLutCurve(srcColor.g, lutG);
    float gIncr = (resColor.g - srcColor.g) * intensityG;
    resColor.g = lastColor.g + gIncr;
    return resColor;
}

vec4 processB(vec4 srcColor, vec4 lastColor)
{
    vec4 resColor = lastColor;
    resColor.b = applyLutCurve(srcColor.b, lutB);
    float bIncr = (resColor.b - srcColor.b) * intensityB;
    resColor.b = lastColor.b + bIncr;
    return resColor;
}

void main(void) {
    vec4 srcColor = texture2D(inputImageTexture, uv0);

    vec4 resColor = srcColor;
    resColor = processR(srcColor, resColor);
    resColor = processG(srcColor, resColor);
    resColor = processB(srcColor, resColor);
    resColor = processY(srcColor, resColor);
    gl_FragColor = resColor;
}
