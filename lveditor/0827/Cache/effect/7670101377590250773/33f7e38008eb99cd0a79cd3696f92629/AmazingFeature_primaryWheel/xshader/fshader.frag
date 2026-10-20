precision highp float;
varying highp vec2 uv0;

uniform sampler2D inputImageTexture;
uniform float Intensity;
uniform float LiftY;
uniform float LiftR;
uniform float LiftG;
uniform float LiftB;
uniform float LiftS;
uniform float GammaY;
uniform float GammaR;
uniform float GammaG;
uniform float GammaB;
uniform float GammaS;
uniform float GainY;
uniform float GainR;
uniform float GainG;
uniform float GainB;
uniform float GainS;
uniform float OffsetR;
uniform float OffsetG;
uniform float OffsetB;
uniform float OffsetS;
uniform float LumaMix;


float rgb2gray(vec3 rgb)
{
    return 0.2126 * rgb.r + 0.7152 * rgb.g + 0.0722 * rgb.b;
}


float processWheel(float srcColor, float lift, float gamma, float gain, float offset)
{
    lift = min(lift, 0.4995);
    float liftX = lift / (lift - 0.5);
    gain = max(gain, 0.001);
    float gainX = 1.0 / gain;
    liftX = min(liftX, gainX - 0.0005);

    float resColor = (srcColor - offset - liftX) / (gainX - liftX) + offset;
    if (resColor >= 0.0) {
        resColor = pow(resColor, gamma);
    } else {
        resColor = -pow(-resColor, gamma);
    }
    return resColor;
}


vec3 adjustSaturation(vec3 color, float p)
{
    // p = [0, xxx], 0 will get gray image (saturation=0), 1 will get original color, >1 will enhance saturation
    
    // use a color vector to adjust saturation
    vec3 oriColor = color;
    vec3 baseAdjustVec = vec3(0.2126, 0.7152, 0.0722) * (1. - p);

    vec3 adjustVec = baseAdjustVec + vec3(p, 0., 0.);
    color.r = dot(oriColor, adjustVec);

    adjustVec = baseAdjustVec + vec3(0., p, 0.);
    color.g = dot(oriColor, adjustVec);

    adjustVec = baseAdjustVec + vec3(0., 0., p);
    color.b = dot(oriColor, adjustVec);
    return color;
}


void main(void) {
    vec4 oriColor = texture2D(inputImageTexture, uv0);

    // process offset
    vec4 offset = vec4(OffsetR, OffsetG, OffsetB, 0.0);
    vec4 resColor = oriColor + offset;

    // get Y with offset for the following LumaMix
    float offsetGray = rgb2gray(offset.rgb);
    float baseY = rgb2gray(resColor.rgb);
    baseY = processWheel(baseY, LiftY, GammaY, GainY, offsetGray);

    // process RGB
    resColor.r = processWheel(resColor.r, LiftR, GammaR, GainR, offset.r);
    resColor.g = processWheel(resColor.g, LiftG, GammaG, GainG, offset.g);
    resColor.b = processWheel(resColor.b, LiftB, GammaB, GainB, offset.b);

    // process LumaMix (to preseve Y)
    float resY = rgb2gray(resColor.rgb);
    resColor.rgb += (baseY - resY) * LumaMix;

    // process saturation
    resY = rgb2gray(resColor.rgb);
    float deltaS = OffsetS;
    deltaS += processWheel(resY, LiftS, 1.0, 1.0, 0.) - resY;
    deltaS += processWheel(resY, 0.0, GammaS, 1.0, 0.0) - resY;
    deltaS += processWheel(resY, 0.0, 1.0, GainS, 0.0) - resY;
    float satParam = clamp(deltaS + 1., 0., 3.);
    resColor.rgb = adjustSaturation(resColor.rgb, satParam);

    resColor.rgb = mix(oriColor.rgb, resColor.rgb, Intensity);
    gl_FragColor = resColor;
}
