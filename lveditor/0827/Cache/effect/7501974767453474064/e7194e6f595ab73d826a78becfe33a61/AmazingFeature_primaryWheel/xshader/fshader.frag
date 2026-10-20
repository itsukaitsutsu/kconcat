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

const float EPS = 0.00001;

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
    if (delta < EPS) {
        s = 0.;
        h = 0.;
    } else {
        if (l <= .5) {
            s = delta / (cmax + cmin);
        } else {
            s = delta / (2. - (cmax + cmin));
        }
        
        if (cmax - r < EPS) {
            if (g >= b) {
                h = 60. * (g - b) / delta;
            } else {
                h = 60. * (g - b) / delta + 360.;
            } 
        } else if (cmax - g < EPS) {
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
    resColor = clamp(resColor, 0.0, 1.0);

    // process saturation
    resY = rgb2gray(resColor.rgb);
    vec3 hsl = rgb2hsl(resColor.rgb);
    float deltaS = OffsetS;
    deltaS += processWheel(resY, LiftS, 1.0, 1.0, 0.0) - resY;
    deltaS += processWheel(resY, 0.0, GammaS, 1.0, 0.0) - resY;
    deltaS += processWheel(resY, 0.0, 1.0, GainS, 0.0) - resY;
    deltaS = clamp(deltaS, -1.0, 2.0);

    hsl.y = hsl.y * (1. + deltaS);
    hsl.y = clamp(hsl.y, 0., 1.);

    if (abs(deltaS) > EPS) {
        resColor.rgb = hsl2rgb(hsl);
    }

    resColor.rgb = mix(oriColor.rgb, resColor.rgb, Intensity);
    gl_FragColor = clamp(resColor, 0., 1.);
}
