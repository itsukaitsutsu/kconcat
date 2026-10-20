precision highp float;
varying highp vec2 uv0;
varying highp vec4 v_color;
varying highp vec4 v_bloomPara;
varying highp vec4 v_bloomPara2;

uniform sampler2D _MainTex;
uniform sampler2D screen_Texture;
uniform sampler2D u_BloomTex2;
uniform float u_Exposure;
uniform float u_GlowIntensity;
uniform float u_Exposure2;
uniform float u_GlowIntensity2;
uniform float u_Exposure3;
//uniform float u_hue;
//uniform float u_light;
uniform float u_GlowIntensity3;
uniform vec3  u_lightColor;

#define PI 3.1415926


vec3 rgb2hsv(vec3 c)
{
    vec4 K = vec4(0.0, -1.0 / 3.0, 2.0 / 3.0, -1.0);
    vec4 p = mix(vec4(c.bg, K.wz), vec4(c.gb, K.xy), step(c.b, c.g));
    vec4 q = mix(vec4(p.xyw, c.r), vec4(c.r, p.yzx), step(p.x, c.r));

    float d = q.x - min(q.w, q.y);
    float e = 1.0e-10;
    return vec3(abs(q.z +  (q.w - q.y) / (6.0 * d + e)), d / (q.x + e), q.x);
}

vec3 hsv2rgb(vec3 c)
{
    vec4 K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    vec3 p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, 0.0, 1.0), c.y);
}


vec3 ChangeHue(vec3 col, float h){
    float shift = 2.0*h*3.1415926535;
    vec3 m = vec3(cos(shift), -sin(shift) * .57735, 0);
    m = vec3(m.xy, -m.y) + (1. - m.x) * .33333;
    return mat3(m, m.zxy, m.yzx) * col;
}

varying vec2 v_local_uv;
varying vec2 v_screen_uv;
varying vec2 m;
varying vec2 n;

vec2 getLocalUv(){
    vec2 uv = v_local_uv;
    vec2 x = vec2(0.0);
    vec2 y = vec2(0.0);
    x = (m + n) / (2.0 * (v_screen_uv));
    y = (m - n) / (2.0 * (1. - v_screen_uv));
    float adapt_width = x.x - y.x;
    float adapt_height = x.y - y.y;
    uv.x -= (x.x + y.x) * 0.5;
    uv.y += (x.y + y.y) * 0.5;
    uv.x /= (adapt_width * 0.5);
    uv.y /= (adapt_height * 0.5);
    uv = uv * 0.5 + 0.5;
    // uv.x = 1.-uv.x;
    uv.y = 1.-uv.y;
    return uv;
}

void main()
{
    float light = v_bloomPara.x;
    float enableColorCustomized = v_bloomPara2.x;

    vec4 oriColor = texture2D(_MainTex, getLocalUv());
    vec4 bloomColor1 = texture2D(screen_Texture, getLocalUv());
    vec4 bloomColor2 = texture2D(u_BloomTex2, getLocalUv());

    //bloomColor1.rgb = u_lightColor;
    //bloomColor2.rgb = u_lightColor;

    if (enableColorCustomized > 0.5)
    {
        bloomColor1.rgb = v_color.rgb * bloomColor1.a * v_color.a;
        bloomColor2.rgb = v_color.rgb * bloomColor2.a * v_color.a;
    }
    
    bloomColor1 = bloomColor1 * light*1.8;
    bloomColor2 = bloomColor2 * light*1.8;

    float d = pow(bloomColor1.a * u_GlowIntensity, (u_Exposure));
    float d1 = pow(bloomColor2.a * u_GlowIntensity2, (u_Exposure2));
    float d2 = pow(oriColor.a * u_GlowIntensity3, (u_Exposure3));
    d = pow(d, (1.0 / 2.2));
    d1 = pow(d1, (1.0 / 2.2));
    d2 = pow(d2, (1.0 / 2.2));
    bloomColor1 = bloomColor1 * d;
    bloomColor2 = bloomColor2 * d1;
    oriColor = oriColor * d2;
    bloomColor1 = clamp(bloomColor1, 0.0, 1.0);
    bloomColor2 = clamp(bloomColor2, 0.0, 1.0);
    oriColor = clamp(oriColor, 0.0, 1.0);
    vec4 outlineColor = bloomColor1;


    oriColor.rgb = rgb2hsv(oriColor.rgb*0.7);
    oriColor.g *= 0.75;
    oriColor.b *= 1.4;
    oriColor.rgb = hsv2rgb(oriColor.rgb);


    bloomColor1 = 1. - (1. - bloomColor1) * (1. - bloomColor2);
    gl_FragColor = 1. - (1. - oriColor) * (1. - bloomColor1);
    gl_FragColor = oriColor + bloomColor1*(1. - clamp(oriColor.a, 0.0, 1.0));
    // gl_FragColor =gl_FragColor + (1.0-gl_FragColor.a)*outlineColor;

    // gl_FragColor = texture2D(u_BloomTex2, getLocalUv());
}
