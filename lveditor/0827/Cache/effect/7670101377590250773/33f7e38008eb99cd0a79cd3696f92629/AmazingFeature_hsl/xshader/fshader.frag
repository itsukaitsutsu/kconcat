precision highp float;
varying vec2 uv0;
uniform sampler2D inputImageTexture;

uniform vec3 u_defaultParam_1; // red
uniform vec3 u_defaultParam_2; // orange
uniform vec3 u_defaultParam_3; // yellow
uniform vec3 u_defaultParam_4; // green
uniform vec3 u_defaultParam_5; // cyan
uniform vec3 u_defaultParam_6; // blue
uniform vec3 u_defaultParam_7; // purple
uniform vec3 u_defaultParam_8; // magenta

#define USER_COLOR_NUM 10
uniform vec4 u_userColorHueRange[USER_COLOR_NUM];
uniform vec3 u_userParam[USER_COLOR_NUM];

const float Eps = 1e-5;


vec3 rgb2hsl(vec3 rgb)
{
    float h = 0.0, s = 0.0, l = 0.0;
    float r = rgb.r;
    float g = rgb.g;
    float b = rgb.b;
    float cmax = max(r, max(g, b));
    float cmin = min(r, min(g, b));
    float delta = cmax - cmin;
    l = (cmax + cmin) / 2.0;
    if (delta == 0.0) {
        s = 0.0;
        h = 0.0;
    } else {
        if (l <= 0.5)
            s = delta / (cmax + cmin);
        else
            s = delta / (2.0 - (cmax + cmin));
        if (cmax == r) {
            if (g >= b)
                h = 60.0 * (g - b) / delta;
            else
                h = 60.0 * (g - b) / delta + 360.0;
        } else if (cmax == g) {
            h = 60.0 * (b - r) / delta + 120.0;
        } else {
            h = 60.0 * (r - g) / delta + 240.0;
        }
    }
    return vec3(h, s, l);
}


float hue2rgb(float p, float q, float t)
{
    if (t < 0.0)
        t += 1.0;
    if (t > 1.0)
        t -= 1.0;
    if (t < 1.0 / 6.0)
        return p + (q - p) * 6.0 * t;
    if (t < 1.0 / 2.0)
        return q;
    if (t < 2.0 / 3.0)
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    return p;
}


vec3 hsl2rgb(vec3 hsl)
{
    float r, g, b;
    float h = hsl.x / 360.0;
    if (hsl.y == 0.0) {
        r = g = b = hsl.z;   // gray
    } else {
        float q = hsl.z < 0.5 ? hsl.z * (1.0 + hsl.y) : (hsl.z + hsl.y - hsl.z * hsl.y);
        float p = 2.0 * hsl.z - q;
        r = hue2rgb(p, q, h + 1.0 / 3.0);
        g = hue2rgb(p, q, h);
        b = hue2rgb(p, q, h - 1.0 / 3.0);
    }
    return vec3(r, g, b);
}


float regularizeHue(float hue)
{
    hue = mod(hue, 360.);
    if (hue > 360.0) {
        hue -= 360.0;
    }
    if (hue < 0.0) {
        hue += 360.0;
    }
    return hue;
}


vec3 pixelAdjust(float h, float hue, float saturation, float brightness, float left_left, float left, float right, float right_right, vec3 deltaHsl) {
    if (left_left < left && left > right && right < right_right) {
        if (h >= left && h <= 360.0) {
            deltaHsl.x += hue;
            deltaHsl.y += saturation;
            deltaHsl.z += brightness;
            return deltaHsl;
        }
        if (h >= 0.0 && h <= right) {
            deltaHsl.x += hue;
            deltaHsl.y += saturation;
            deltaHsl.z += brightness;
            return deltaHsl;
        }
        if (h >= left_left && h <= left) {
            deltaHsl.x += hue * (h - left_left) / (left - left_left);
            deltaHsl.y += saturation * (h - left_left) / (left - left_left);
            deltaHsl.z += brightness * (h - left_left) / (left - left_left);
            return deltaHsl;
        }
        if (h >= right && h <= right_right) {
            deltaHsl.x += hue * (right_right - h) / (right_right - right);
            deltaHsl.y += saturation * (right_right - h) / (right_right - right);
            deltaHsl.z += brightness * (right_right - h) / (right_right - right);
            return deltaHsl;
        }
    }
    if (left_left > left && left < right && right < right_right) {
        if (h >= left && h <= right) {
            deltaHsl.x += hue;
            deltaHsl.y += saturation;
            deltaHsl.z += brightness;
            return deltaHsl;
        }
        if (h >= 0.0 && h <= left) {
            deltaHsl.x += hue * (h + 360.0 - left_left) / (left + 360.0 - left_left);
            deltaHsl.y += saturation * (h + 360.0 - left_left) / (left + 360.0 - left_left);
            deltaHsl.z += brightness * (h + 360.0 - left_left) / (left + 360.0 - left_left);
            return deltaHsl;
        }
        if (h >= left_left && h <= 360.0) {
            deltaHsl.x += hue * (h - left_left) / (left + 360.0 - left_left);
            deltaHsl.y += saturation * (h - left_left) / (left + 360.0 - left_left);
            deltaHsl.z += brightness * (h - left_left) / (left + 360.0 - left_left);
            return deltaHsl;
        }
        if (h >= right && h <= right_right) {
            deltaHsl.x += hue * (right_right - h) / (right_right - right);
            deltaHsl.y += saturation * (right_right - h) / (right_right - right);
            deltaHsl.z += brightness * (right_right - h) / (right_right - right);
            return deltaHsl;
        }
    }
    if (left_left <= left && left < right && right <= right_right) {

        if (h >= left && h <= right) {
            deltaHsl.x += hue;
            deltaHsl.y += saturation;
            deltaHsl.z += brightness;
            return deltaHsl;
        }
        if (h >= left_left && h <= left) {
            deltaHsl.x += hue * (h - left_left) / (left - left_left);
            deltaHsl.y += saturation * (h - left_left) / (left - left_left);
            deltaHsl.z += brightness * (h - left_left) / (left - left_left);
            return deltaHsl;
        }
        if (h >= right && h <= right_right) {
            deltaHsl.x += hue * (right_right - h) / (right_right - right);
            deltaHsl.y += saturation * (right_right - h) / (right_right - right);
            deltaHsl.z += brightness * (right_right - h) / (right_right - right);
            return deltaHsl;
        }
    }
    if (left_left < left && left < right && right > right_right) {
        if (h >= left && h <= right) {
            deltaHsl.x += hue;
            deltaHsl.y += saturation;
            deltaHsl.z += brightness;
            return deltaHsl;
        }
        if (h >= left_left && h <= left) {
            deltaHsl.x += hue * (h - left_left) / (left - left_left);
            deltaHsl.y += saturation * (h - left_left) / (left - left_left);
            deltaHsl.z += brightness * (h - left_left) / (left - left_left);
            return deltaHsl;
        }
        if (h >= right && h <= 360.0) {
            deltaHsl.x += hue * (right_right + 360.0 - h) / (right_right + 360.0 - right);
            deltaHsl.y += saturation * (right_right + 360.0 - h) / (right_right + 360.0 - right);
            deltaHsl.z += brightness * (right_right + 360.0 - h) / (right_right + 360.0 - right);
            return deltaHsl;
        }
        if (h >= 0.0 && h <= right_right) {
            deltaHsl.x += hue * (right_right - h) / (right_right + 360.0 - right);
            deltaHsl.y += saturation * (right_right - h) / (right_right + 360.0 - right);
            deltaHsl.z += brightness * (right_right - h) / (right_right + 360.0 - right);
            return deltaHsl;
        }
    }
    return deltaHsl;
}

vec3 wrappedPixelAdjust(float originalHue, vec3 sliderParam, vec4 hueRange, vec3 deltaHsl)
{
    if (hueRange.x < Eps && hueRange.y < Eps && hueRange.z < Eps && hueRange.w < Eps) {
        return deltaHsl;
    } else {
        deltaHsl = pixelAdjust(originalHue, sliderParam.x, sliderParam.y, sliderParam.z, hueRange.x, hueRange.y, hueRange.z, hueRange.w, deltaHsl);
        return deltaHsl;
    }
}


void main() {
    vec4 baseColor = texture2D(inputImageTexture, uv0);

    // slider parameters mapping for default color
    vec3 hslRedParam;
    vec3 hslOrangeParam;
    vec3 hslYellowParam;
    vec3 hslGreenParam;
    vec3 hslCyanParam;
    vec3 hslBlueParam;
    vec3 hslPurpleParam;
    vec3 hslMagentaParam;
    hslRedParam = vec3(u_defaultParam_1.x * 0.25, u_defaultParam_1.y, u_defaultParam_1.z / 2.7);
    hslOrangeParam = vec3(u_defaultParam_2.x * 0.15, u_defaultParam_2.y, u_defaultParam_2.z / 2.7);
    hslYellowParam = vec3(u_defaultParam_3.x * 0.3, u_defaultParam_3.y, u_defaultParam_3.z / 2.7);
    hslGreenParam = vec3(u_defaultParam_4.x * 0.3, u_defaultParam_4.y * 1.1, u_defaultParam_4.z / 2.7);
    hslCyanParam = vec3(u_defaultParam_5.x * 0.4, u_defaultParam_5.y, u_defaultParam_5.z / 2.2);
    hslBlueParam = vec3(u_defaultParam_6.x * 0.4, u_defaultParam_6.y * 1.2, u_defaultParam_6.z / 2.0);
    hslPurpleParam = vec3(u_defaultParam_7.x * 0.4, u_defaultParam_7.y, u_defaultParam_7.z / 2.7);
    hslMagentaParam = vec3(u_defaultParam_8.x * 0.4, u_defaultParam_8.y, u_defaultParam_8.z / 2.7);

    // get delta hsl
    vec3 hsl = rgb2hsl(baseColor.rgb);
    vec3 deltaHsl = vec3(0.0);
    // for default colors
    deltaHsl = wrappedPixelAdjust(hsl.x, hslRedParam, vec4(315.0, 330.0, 5.0, 20.0), deltaHsl);
    deltaHsl = wrappedPixelAdjust(hsl.x, hslOrangeParam, vec4(350.0, 20.0, 40.0, 60.0), deltaHsl);
    deltaHsl = wrappedPixelAdjust(hsl.x, hslYellowParam, vec4(25.0, 50.0, 70.0, 90.0), deltaHsl);
    deltaHsl = wrappedPixelAdjust(hsl.x, hslGreenParam, vec4(50.0, 70.0, 160.0, 190.0), deltaHsl);
    deltaHsl = wrappedPixelAdjust(hsl.x, hslCyanParam, vec4(135.0, 165., 195.0, 225.0), deltaHsl);
    deltaHsl = wrappedPixelAdjust(hsl.x, vec3(hslBlueParam.x, 0., 0.), vec4(145.0, 180., 235.0, 275.0), deltaHsl);
    deltaHsl = wrappedPixelAdjust(hsl.x, vec3(0., hslBlueParam.y, hslBlueParam.z), vec4(145.0, 180., 235.0, 275.0), deltaHsl);
    deltaHsl = wrappedPixelAdjust(hsl.x, hslPurpleParam, vec4(235.0, 255.0, 315.0, 335.0), deltaHsl);
    deltaHsl = wrappedPixelAdjust(hsl.x, hslMagentaParam, vec4(255.0, 285.0, 335.0, 5.0), deltaHsl);
    // for user picked colors
    for (int i = 0; i < USER_COLOR_NUM; i++) {
        deltaHsl = wrappedPixelAdjust(hsl.x, u_userParam[i], u_userColorHueRange[i], deltaHsl);
    }
    
    // adjust hue
    hsl.x = hsl.x + deltaHsl.x;
    hsl.x = regularizeHue(hsl.x);

    // adjust saturation
    deltaHsl.y = clamp(deltaHsl.y / 100.0, -1.0, 1.0);
    if (deltaHsl.y < 0.0) {
        hsl.y = hsl.y * (1.0 + deltaHsl.y);
    } else {
        deltaHsl.y = deltaHsl.y / 2.0;   // TODO：移到业务层
        float temp = hsl.y * (1.0 - deltaHsl.y);
        hsl.y = hsl.y + (hsl.y - temp);
    }

    // adjust brightness
    deltaHsl.z = clamp(deltaHsl.z / 100.0, -1.0, 1.0);
    if (deltaHsl.z <= 0.0) {
        float radio = hsl.y;
        if (hsl.z >= 0.5) {
            radio = hsl.y * 1.0;
        }
        if (hsl.z < 0.5) {
            radio = hsl.y * 2.0 * hsl.z;
        }
        float temp = hsl.z - radio * (1.0 - hsl.z) * deltaHsl.z;
        hsl.z = hsl.z + (hsl.z - temp);
    } else {
        float radio = hsl.y;
        if (hsl.z >= 0.5) {
            radio = hsl.y * 1.0;
        }
        if (hsl.z < 0.5) {
            radio = hsl.y * 2.0 * hsl.z;
        }
        deltaHsl.z = (1.0 - deltaHsl.y) * deltaHsl.z;
        hsl.z = hsl.z + radio * (1.15 - hsl.z) * deltaHsl.z;
    }
    hsl.y = clamp(hsl.y, 0.0, 1.0);
    hsl.z = clamp(hsl.z, 0.0, 1.0);

    vec3 resColor = hsl2rgb(hsl);
    gl_FragColor = vec4(clamp(resColor, 0., 1.), baseColor.a);
}
