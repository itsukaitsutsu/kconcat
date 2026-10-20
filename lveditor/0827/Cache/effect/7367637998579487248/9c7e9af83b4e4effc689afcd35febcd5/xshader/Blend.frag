precision highp float;

varying vec2 uv0;
varying vec4 v_color;
varying vec4 v_bloomPara;
varying vec4 v_bloomPara2;

uniform sampler2D _MainTex;
uniform sampler2D u_GlowBlurTex1;
uniform sampler2D u_GlowBlurTex2;
uniform sampler2D u_GlowBlurTex3;
uniform sampler2D u_GlowBlurTex4;
uniform sampler2D u_GlowBlurTex5;
uniform sampler2D u_GlowBlurTex6;
uniform sampler2D u_GlowBlurTex7;
uniform sampler2D u_GlowBlurTex8;
uniform float u_GlowIntensity;
uniform float u_gamma;
uniform int u_blendType;
uniform float u_radius;

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
    vec2 luv = getLocalUv();
    float radius = v_bloomPara.y * u_radius;

    vec4 resColor = texture2D(_MainTex, luv);
    vec4 blurColor1 = texture2D(u_GlowBlurTex1, uv0);
    vec4 blurColor2 = texture2D(u_GlowBlurTex2, uv0);
    vec4 blurColor3 = texture2D(u_GlowBlurTex3, uv0);
    vec4 blurColor4 = texture2D(u_GlowBlurTex4, uv0);
    vec4 blurColor5 = texture2D(u_GlowBlurTex5, uv0);
    vec4 blurColor6 = texture2D(u_GlowBlurTex6, uv0);
    vec4 blurColor7 = texture2D(u_GlowBlurTex7, uv0);
    vec4 blurColor8 = texture2D(u_GlowBlurTex8, uv0);

    vec4 dis1 = clamp(blurColor1, 0.0, 1.0);
    vec4 dis2 = clamp(blurColor2, 0.0, 1.0);
    vec4 dis3 = clamp(blurColor3, 0.0, 1.0);
    vec4 dis4 = clamp(blurColor4, 0.0, 1.0);
    vec4 dis5 = clamp(blurColor5, 0.0, 1.0);
    vec4 dis6 = clamp(blurColor6, 0.0, 1.0);
    vec4 dis7 = clamp(blurColor7, 0.0, 1.0);
    vec4 dis8 = clamp(blurColor8, 0.0, 1.0);
    
    float enableColorCustomized = v_bloomPara2.x;
    if (enableColorCustomized > 0.5)
    {
        dis1.rgb = v_color.rgb * dis1.a * v_color.a;
        dis2.rgb = v_color.rgb * dis2.a * v_color.a;
        dis3.rgb = v_color.rgb * dis3.a * v_color.a;
        dis4.rgb = v_color.rgb * dis4.a * v_color.a;
        dis5.rgb = v_color.rgb * dis5.a * v_color.a;
        dis6.rgb = v_color.rgb * dis6.a * v_color.a;
        dis7.rgb = v_color.rgb * dis7.a * v_color.a;
        dis8.rgb = v_color.rgb * dis8.a * v_color.a;
    }
    
    dis1 = pow(dis1, vec4(u_gamma));
    dis2 = pow(dis2, vec4(u_gamma));
    dis3 = pow(dis3, vec4(u_gamma));
    dis4 = pow(dis4, vec4(u_gamma));
    dis5 = pow(dis5, vec4(u_gamma));
    dis6 = pow(dis6, vec4(u_gamma));
    dis7 = pow(dis7, vec4(u_gamma));
    dis8 = pow(dis8, vec4(u_gamma));

    vec4 glowColor = vec4(0.0);
    vec4 oriColor = resColor;
    float glowIntensity = u_GlowIntensity * v_bloomPara.x;
    if (u_blendType == 0) {
        dis1 = 1. - (1. - dis1) * (1. - dis2 * 1.5 * smoothstep(0., 1., radius));
        dis1 = 1. - (1. - dis1) * (1. - dis3 * 1.45 * smoothstep(1., 2., radius));
        dis1 = 1. - (1. - dis1) * (1. - dis4 * 1.4 * smoothstep(2., 3., radius));
        dis1 = 1. - (1. - dis1) * (1. - dis5 * 1.3 * smoothstep(3., 4., radius));
        dis1 = 1. - (1. - dis1) * (1. - dis6 * 1.2 * smoothstep(4., 5., radius));
        dis1 = 1. - (1. - dis1) * (1. - dis7 * 1.1 * smoothstep(5., 6., radius));
        dis1 = 1. - (1. - dis1) * (1. - dis8 * 1.0 * smoothstep(6., 7., radius));
        glowColor = pow(dis1 * glowIntensity, vec4(1.0/u_gamma));
        glowColor = clamp(glowColor, 0.0, 1.0);
        resColor = (1. - (1. - glowColor) * (1. - resColor));
    }
    else {
        dis1 = dis1 + dis2 * 1.5 * smoothstep(0., 1., radius);
        dis1 = dis1 + dis3 * 1.45 * smoothstep(1., 2., radius);
        dis1 = dis1 + dis4 * 1.4 * smoothstep(2., 3., radius);
        dis1 = dis1 + dis5 * 1.3 * smoothstep(3., 4., radius);
        dis1 = dis1 + dis6 * 1.2 * smoothstep(4., 5., radius);
        dis1 = dis1 + dis7 * 1.1 * smoothstep(5., 6., radius);
        dis1 = dis1 + dis8 * 1.0 * smoothstep(6., 7., radius);
        glowColor = pow(dis1 * glowIntensity, vec4(1.0/u_gamma));
        glowColor = clamp(glowColor, 0.0, 1.0);
        resColor = glowColor + resColor;
    }
    oriColor.rgb = oriColor.a > 0. ? oriColor.rgb / oriColor.a : oriColor.rgb;
    gl_FragColor = oriColor.a * oriColor + (1. - oriColor.a) * resColor;
}
