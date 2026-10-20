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

const float SCALE = 1020.0/1023.0;
const float INV_SCALE = 1023.0/1020.0;

float vec2ToFloat(vec2 val){
    float res = val.x + val.y/255.0;
    return res;
}
float vec4ToFloat(vec4 val){
    float res = val.x * 255.0 + val.y + val.z/255.0;
    res = val.w < 0.5 ? res : -res;
    return res;
}

vec4 processY(vec4 srcColor, vec4 lastColor)
{
    vec4 resColor = lastColor;
    float ySrc = 0.2126 * srcColor.r + 0.7152 * srcColor.g + 0.0722 * srcColor.b;
    float yLast = 0.2126 * lastColor.r + 0.7152 * lastColor.g + 0.0722 * lastColor.b;
    // float yNew = vec2ToFloat(texture2D(lutY, vec2(ySrc, 0.5)).xy);
    float yNew = vec4ToFloat(texture2D(lutY, vec2(ySrc, 0.5)));
    float yIncr = (yNew - ySrc) * intensityY - (yLast - ySrc);
    resColor.rgb += yIncr;
    return resColor;
}

vec4 processR(vec4 srcColor, vec4 lastColor)
{
    vec4 resColor = lastColor;
    // resColor.r = vec2ToFloat(texture2D(lutR, vec2(srcColor.r, 0.5)).xy);
    resColor.r = vec4ToFloat(texture2D(lutR, vec2(srcColor.r, 0.5)));
    float rIncr = (resColor.r - srcColor.r) * intensityR;
    resColor.r = lastColor.r + rIncr;
    // resColor.rgb -= 0.2126 * rIncr;
    return resColor;
}

vec4 processG(vec4 srcColor, vec4 lastColor)
{
    vec4 resColor = lastColor;
    // resColor.g = vec2ToFloat(texture2D(lutG, vec2(srcColor.g, 0.5)).xy);
    resColor.g = vec4ToFloat(texture2D(lutG, vec2(srcColor.g, 0.5)));
    float gIncr = (resColor.g - srcColor.g) * intensityG;
    resColor.g = lastColor.g + gIncr;
    // resColor.rgb -= 0.7152 * gIncr;
    return resColor;
}

vec4 processB(vec4 srcColor, vec4 lastColor)
{
    vec4 resColor = lastColor;
    // resColor.b = vec2ToFloat(texture2D(lutB, vec2(srcColor.b, 0.5)).xy);
    resColor.b = vec4ToFloat(texture2D(lutB, vec2(srcColor.b, 0.5)));
    float bIncr = (resColor.b - srcColor.b) * intensityB;
    resColor.b = lastColor.b + bIncr;
    // resColor.rgb -= 0.0722 * bIncr;
    return resColor;
}

void main(void) {
    vec4 srcColor = texture2D(inputImageTexture, uv0);

    vec4 resColor = srcColor;
    resColor = processR(srcColor, resColor);
    resColor = processG(srcColor, resColor);
    resColor = processB(srcColor, resColor);
    resColor = processY(srcColor, resColor);

    gl_FragColor = clamp(resColor, 0., 1.);
    // gl_FragColor = vec4(1.0);
}
