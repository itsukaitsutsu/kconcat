#version 300 es
precision highp float;

layout(location = 0) out vec4 FragColor;

in vec2 o_uv;

uniform sampler2D _MainTex;
uniform sampler2D u_picture1;

uniform float u_alphaIns;
uniform vec2 u_pos;
uniform vec2 u_pos2;
uniform float u_progress;
uniform float u_scale;
const float EPS = 1e-5;

float uvProtect(vec2 uvTemp)
{
    return step(0., uvTemp.x) * step(0., uvTemp.y) * step(uvTemp.x, 1.0) * step(uvTemp.y, 1.0);
}

vec4 blendPassNormal(vec4 srcColor, vec4 baseColor)
{
    srcColor.rgb /= max(srcColor.a, EPS);
    baseColor.rgb /= max(baseColor.a, EPS);
    
    // normal
    vec4 blendColor = baseColor;
    blendColor.rgb = srcColor.rgb;

    vec4 result = vec4(0.);
    result.rgb = baseColor.rgb * baseColor.a * (1.0 - srcColor.a) +
        srcColor.rgb * srcColor.a * (1.0 - baseColor.a) +
        srcColor.a * baseColor.a * blendColor.rgb;
    result.a = srcColor.a + baseColor.a * (1.0 - srcColor.a);
    
    return result;
}

float sdRoundBox(in vec2 p, in vec2 b, in vec4 r) {
    r.xy = (p.x > 0.0) ? r.xy : r.zw;
    r.x = (p.y > 0.0) ? r.x : r.y;
    vec2 q = abs(p) - b + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r.x;
}

float getBox(vec4 AABB, vec2 u_textRect, float u_Scale)
{
    vec2 uv_sdf = o_uv;
    uv_sdf -= AABB.xy;
    uv_sdf.y *= u_textRect.y/u_textRect.x;
    vec2 curSize = AABB.zw;
    // curSize.y *= u_textRect.y/u_textRect.x*1.15;
    curSize.y *= u_textRect.y/u_textRect.x;
    float angleValue = 0.0001;
    // angleValue *= fontSize * 0.02;
    // angleValue *= 1.0 * 0.02;
    float d = sdRoundBox(uv_sdf, curSize*u_Scale, vec4(angleValue) * 720.0 / u_textRect.x);
    d = smoothstep(0.00, 0.001 * 720.0 / u_textRect.x, d);
    return d;
}

vec2 offset(vec2 pos)
{
    vec2 offset = vec2(0.0);
    return offset;
}

// vec2 affine(vec2 uv, float theta, vec2 offset, vec2 anchor, vec2 scale)
// {
//     // offset
//     uv -= offset;
//     // rotate
//     uv -= 0.5;
//     uv.y *= u_ScreenParams.y / u_ScreenParams.x;
//     float sint = sin(theta);
//     float cost = cos(theta);
//     mat2 rot = mat2(
//         cost, sint,
//         -sint, cost
//     );
//     uv = rot * uv;
//     uv.y *= u_ScreenParams.x / u_ScreenParams.y;
//     // scale
//     uv /= scale;
//     uv += 0.5;
//     uv += anchor;
//     return uv;
// }

void main(void) 
{   
    float offset = mix(-u_pos.x, u_pos.x, u_progress);
    float offset2 = mix(-u_pos2.x, u_pos.x, u_progress);
    vec2 offsetUV1 = o_uv;
    offsetUV1 -= vec2(offset, 0.1);
    offsetUV1 -= 0.5;
    offsetUV1 /= u_scale;
    // offsetUV1.x /= 0.5;
    offsetUV1.y /= 1.5;
    offsetUV1 += 0.5;

    vec2 offsetUV2 = o_uv;
    offsetUV2 -= vec2(-offset2, -0.1);
    offsetUV2 -= 0.5;
    offsetUV2 /= u_scale;
    // offsetUV2.x /= 0.5;
    offsetUV2.y /= 1.5;
    offsetUV2 += 0.5;
    // offsetUV -= vec2(u_pos.x, 0.4);
    // offsetUV -= vec2(-0.16, 0.4);

    vec4 textColor = texture(_MainTex, o_uv);
    vec4 lightCol1 = texture(u_picture1, offsetUV1);
    vec4 lightCol2 = texture(u_picture1, offsetUV2);
    lightCol1 *= u_alphaIns; 
    lightCol2 *= u_alphaIns; 

    // lightCol.rgb = clamp(lightCol.rgb / lightCol.a, 0.0, 1.0);

    // vec4 resCol = mix(textColor, lightCol, lightCol.a);

    vec4 finalColor = blendPassNormal(lightCol1, textColor);
    finalColor = blendPassNormal(lightCol2, finalColor);

    // FragColor = textColor + lightCol1 + lightCol2 +  vec4(0.3);
    // FragColor = vec4(lightCol.a);
    FragColor = finalColor;
}