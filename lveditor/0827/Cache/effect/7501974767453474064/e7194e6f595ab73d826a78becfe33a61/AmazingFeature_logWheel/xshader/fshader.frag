precision highp float;
varying highp vec2 uv0;

// Interface parameters
uniform sampler2D inputImageTexture;
uniform float Intensity;
uniform float RngDown;
uniform float RngUp;
uniform float ShadowR;
uniform float ShadowG;
uniform float ShadowB;
uniform float ShadowS;
uniform float MidtoneR;
uniform float MidtoneG;
uniform float MidtoneB;
uniform float MidtoneS;
uniform float HightlightsR;
uniform float HightlightsG;
uniform float HightlightsB;
uniform float HightlightsS;
uniform float OffsetR;
uniform float OffsetG;
uniform float OffsetB;
uniform float OffsetS;

// NONE-interface parameters
uniform vec4 s2mR;
uniform vec4 s2mG;
uniform vec4 s2mB;
uniform vec4 s2mS;
uniform vec4 h2mR;
uniform vec4 h2mG;
uniform vec4 h2mB;
uniform vec4 h2mS;
uniform vec4 s2hR;
uniform vec4 s2hG;
uniform vec4 s2hB;
uniform vec4 s2hS;

const float EPS = 0.000001;
const float MAX_SLOPE = 9.0;    // maximum slope for shadow or highlights range
const float RNG_T = 0.05;       // half length for transition range


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
    if (delta < .00001) {
        s = 0.;
        h = 0.;
    } else {
        if (l <= .5) {
            s = delta / (cmax + cmin);
        } else {
            s = delta / (2. - (cmax + cmin));
        }
        
        if (cmax - r < .00001) {
            if (g >= b) {
                h = 60. * (g - b) / delta;
            } else {
                h = 60. * (g - b) / delta + 360.;
            } 
        } else if (cmax - g < .00001) {
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
        r = g = b = hsl.z; // gray
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


float getShadow(float x, float shadow, float rngDown)
{
    float res = 0.0;
    if (shadow > 0.0) {
        res = x * (1.0 - shadow) + rngDown * shadow;
    } else {
        float slope = (rngDown - shadow) / max(rngDown, EPS);
        if (slope <= MAX_SLOPE) {
            res = x * slope + shadow;
        } else {
            res = x * MAX_SLOPE + (1.0 - MAX_SLOPE) * rngDown;
        }
    }
    return res;
}

float getHighlights(float x, float highlights, float rngUp)
{
    float res = 0.0;
    if (highlights > 0.0) {
        float slope = (1.0 - rngUp + highlights) / max(1.0 - rngUp, EPS);
        if (slope <= MAX_SLOPE) {
            res = x * slope - rngUp * highlights / max(1.0 - rngUp, EPS);
        } else {
            res = x * MAX_SLOPE + (1.0 - MAX_SLOPE) * rngUp;
        }
    } else {
        res = x * (1.0 + highlights) - rngUp * highlights;
    }
    return res;
}


float mappingMidtoneParam(float midtone)
{
    float m2 = midtone * midtone;
    float m3 = m2 * midtone;
    float m4 = m3 * midtone;
    float m5 = m4 * midtone;
    float m6 = m5 * midtone;
    float m7 = m6 * midtone;
    float res = 1.0 - 1.10145 * midtone + 0.906076 * m2 - 0.681492 * m3 - 
                0.253415 * m4 + 0.272653 * m5 + 1.09324 * m6 - 0.909166 * m7;
    return res;
}


float getMidtone(float x, float midtone, float rngDown, float rngUp)
{
    float res = 0.0;
    float rngDiff = rngUp - rngDown;
    float m = mappingMidtoneParam(midtone);

    // adjust rng for horizontal direction
    x = (x - rngDown) / max(rngDiff, EPS);
    res = 1.0 - pow(x, m);
    res = 1.0 - pow(res, 1.0 / m);
    // adjust rng for vertical direction
    res = res * rngDiff + rngDown;
    return res;
}


float getCubic(float x, vec4 c)
{
    float x2 = x * x;   // square
    float x3 = x2 * x;  // cubic
    float res = c.x + c.y * x + c.z * x2 + c.w * x3;
    return res;
}


float processWheel(float x, float shadow, float midtone, float highlights, float offset, 
                   float rngDown, float rngUp, vec4 s2m, vec4 h2m, vec4 s2h)
{
    float res = 0.0;
    x = x + offset;

    float rngMid = (rngDown + rngUp) / 2.0;
    float rngDiff = rngUp - rngDown;

    if ((rngDiff < 0.05) && (rngDown - RNG_T < x) && (x < rngUp + RNG_T)) {
        // shadow directly transitions to highlights
        res = getCubic(x, s2h);
    } else if (x <= rngDown - RNG_T) {
        res = getShadow(x, shadow, rngDown);
    } else if ((rngDown - RNG_T < x) && (x < min(rngDown + RNG_T, rngMid))) {
        // transitions between shadow and midtone
        res = getCubic(x, s2m);
    } else if ((rngDown + RNG_T <= x) && (x <= rngUp - RNG_T)) {
        res = getMidtone(x, midtone, rngDown, rngUp);
    } else if ((max(rngUp - RNG_T, rngMid) < x) && (x < rngUp + RNG_T)) {
        // transitions between midtone and highlights
        res = getCubic(x, h2m);
    } else {
        res = getHighlights(x, highlights, rngUp);
    }
    return res;
}


void main(void){
    vec4 oriColor = texture2D(inputImageTexture, uv0);
    vec4 res = oriColor;

    // adjust shadow, midtone, highlights, offset
    float rngDownMod = min(RngDown, RngUp);
    res.r = processWheel(oriColor.r, ShadowR, MidtoneR, HightlightsR, OffsetR, rngDownMod, RngUp, s2mR, h2mR, s2hR);
    res.g = processWheel(oriColor.g, ShadowG, MidtoneG, HightlightsG, OffsetG, rngDownMod, RngUp, s2mG, h2mG, s2hG);
    res.b = processWheel(oriColor.b, ShadowB, MidtoneB, HightlightsB, OffsetB, rngDownMod, RngUp, s2mB, h2mB, s2hB);
    res = clamp(res, 0.0, 1.0);

    // adjust saturation
    vec3 hsl = rgb2hsl(res.rgb);
    float resY = rgb2gray(res.rgb);
    float dstY = processWheel(resY, ShadowS, MidtoneS, HightlightsS, 0.0, rngDownMod, RngUp, s2mS, h2mS, s2hS);
    float deltaS = (dstY - resY) * 12.0 + OffsetS;
    hsl.y = hsl.y * (1.0 + deltaS);
    hsl.y = clamp(hsl.y, 0.0, 1.0);
    if (abs(deltaS) > 0.00001) {
        res.rgb = hsl2rgb(hsl);
    }
    
    res.rgb = mix(oriColor.rgb, res.rgb, Intensity);
    gl_FragColor = clamp(res, 0., 1.);
}

