precision highp float;

uniform sampler2D u_src;
uniform sampler2D u_lut;
uniform float u_intensity;

varying vec2 v_uv;

vec3 lut8x8in16x8 (sampler2D lut, vec3 src, float area)
{
    src *= 63.0;
    float b = src.b;

    vec2 i01 = vec2(floor(b), ceil(b));
    vec2 r01 = floor(i01 * 0.125);
    vec2 c01 = i01 - r01 * 8.0;

    vec4 uv0 = vec4(c01, r01).xzyw * 0.125;
    vec2 uv1 = (src.rg + 0.5) * 0.001953125;
    uv0 += vec4(uv1, uv1);

    uv0.y = (uv0.y + area) * 0.5;
    uv0.w = (uv0.w + area) * 0.5;

    vec3 c0 = texture2D(lut, uv0.xy).rgb;
    vec3 c1 = texture2D(lut, uv0.zw).rgb;
    return mix(c0, c1, b - i01.x);
}

vec4 mapping (vec4 color, sampler2D lut, float intensity)
{
    vec3 src = color.rgb;
    float area = step(0.80001, intensity); 
    vec3 res = lut8x8in16x8(lut, src, area);
    vec3 res80 = lut8x8in16x8(lut, src, 0.0);

    float weight0 = smoothstep(0.0, 0.8, intensity);
    float weight1 = 1.0;
    float weight = mix(weight0, weight1, area); 

    float mix_weight = (1.0 - intensity) / 0.2;
    mix_weight = mix_weight > 1.0 ? 0.0 : mix_weight;
    res = mix_weight * res80 + (1.0 - mix_weight) * res;

    res = mix(src, res, weight);

    return vec4(res, 1.0 * color.a);
}

void main ()
{
    vec4 color = clamp(texture2D(u_src, v_uv), 0.0, 1.0);
    color = mapping(color, u_lut, u_intensity);
    gl_FragColor = color;
}