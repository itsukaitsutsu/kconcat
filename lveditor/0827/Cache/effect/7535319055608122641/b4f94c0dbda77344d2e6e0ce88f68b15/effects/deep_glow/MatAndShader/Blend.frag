precision highp float;
varying highp vec2 uv0;
uniform sampler2D u_inputTexture;
uniform sampler2D u_GlowBlurTex1;
uniform sampler2D u_GlowBlurTex2;
uniform sampler2D u_GlowBlurTex3;
uniform sampler2D u_GlowBlurTex4;
uniform sampler2D u_GlowBlurTex5;
uniform sampler2D u_GlowBlurTex6;
uniform sampler2D u_GlowBlurTex7;
uniform float u_GlowIntensity;
uniform float u_Alpha;

float gaussianWeight(float x, float sigma)
{
    return exp(-0.5 * x * x / (sigma * sigma));
}
vec4 glow1(vec2 uv, float ins, vec3 tintColor)
{
    vec4 oriColor = texture2D(u_inputTexture, uv);
    vec4 blurColor1 = texture2D(u_GlowBlurTex1, uv);
    vec4 blurColor2 = texture2D(u_GlowBlurTex2, uv);
    vec4 blurColor3 = texture2D(u_GlowBlurTex3, uv);
    vec4 blurColor4 = texture2D(u_GlowBlurTex4, uv);
    vec4 blurColor5 = texture2D(u_GlowBlurTex5, uv);
    vec4 blurColor6 = texture2D(u_GlowBlurTex6, uv);
    vec4 blurColor7 = texture2D(u_GlowBlurTex7, uv);
    float intensity = pow(ins * 0.5, 1./2.2);
    vec4 dis1 = clamp(blurColor1 * intensity, 0.0, 1.0);
    vec4 dis2 = clamp(blurColor2 * intensity, 0.0, 1.0);
    vec4 dis3 = clamp(blurColor3 * intensity, 0.0, 1.0);
    vec4 dis4 = clamp(blurColor4 * intensity, 0.0, 1.0);
    vec4 dis5 = clamp(blurColor5 * intensity, 0.0, 1.0);
    vec4 dis6 = clamp(blurColor6 * intensity, 0.0, 1.0);
    vec4 dis7 = clamp(blurColor7 * intensity, 0.0, 1.0);
    dis1 = 1. - (1. - dis1) * (1. - dis2);
    dis1 = 1. - (1. - dis1) * (1. - dis3);
    dis1 = 1. - (1. - dis1) * (1. - dis4);
    dis1 = 1. - (1. - dis1) * (1. - dis5);
    dis1 = 1. - (1. - dis1) * (1. - dis6);
    dis1 = 1. - (1. - dis1) * (1. - dis7);
    vec4 glowColor = clamp(vec4(dis1) * max(intensity, 1.0), 0.0, 1.0) * vec4(tintColor, 1.0);
    glowColor *= u_Alpha;

    return oriColor + glowColor * (1.0 - oriColor.a);
}

void main()
{
    gl_FragColor = glow1(uv0, u_GlowIntensity, vec3(1,1,1));
}
