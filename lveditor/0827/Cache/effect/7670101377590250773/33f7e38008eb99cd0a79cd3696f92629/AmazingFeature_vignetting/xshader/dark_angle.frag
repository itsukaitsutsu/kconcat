precision highp float;

varying vec2 fTexCoord;
uniform vec2 radius;
uniform float huagan;
uniform float out_huagan;

uniform vec2 u_ScreenParams;

uniform sampler2D _MainTex;

const float sqrt_2 = 1.415;

uniform int blendMode;
uniform float alphaFactor;

// normal
vec3 blendNormal(vec3 base, vec3 blend) {
    return blend;
}

vec3 blendNormal(vec3 base, vec3 blend, float opacity) {
    return (blendNormal(base, blend) * opacity + blend * (1.0 - opacity));
}

void main() 
{
    vec2 uv0 = 2. * fTexCoord - 1.;
    // uv0 *= u_ScreenParams/max(u_ScreenParams.x, u_ScreenParams.y);
    float l = length(uv0);
    vec3 col = vec3(1.);
    if(huagan>0.0001){
        col = vec3(0.0001);
    }

    vec4 mainCol = texture2D(_MainTex, vec2(fTexCoord.x, 1.-fTexCoord.y));
    // col = vec3(0.);

    // float innerR = radius.x;
    // float outR = radius.y * 


    vec3 resCol = blendNormal(mainCol.rgb, col) * alphaFactor + mainCol.rgb * (1.0 - alphaFactor);

    float one_minus_radius_x = 1. - radius.x;
    vec2 r = radius;
    if(huagan>0.0001){
        r.x = 0.1;
        r.y = 0.85;
    }

    // float c_alpha = smoothstep(mix(1., r.x, abs(huagan)) * sqrt_2, mix(3., r.y, abs(out_huagan)) * sqrt_2, l);

    if(huagan>0.0001)
    {
        float c_alpha = smoothstep(
            mix(1., r.x, abs(out_huagan)) * sqrt_2, 
            mix(1.4, r.y, abs(out_huagan)) * sqrt_2, 
            l);
        gl_FragColor.rgb = vec3(0,0,0);
        gl_FragColor.a = c_alpha * out_huagan;
    }
    else
    {
        float c_alpha = smoothstep(
            mix(1., r.x, abs(huagan)) * sqrt_2, 
            mix(1.5, r.y, abs(out_huagan)) * sqrt_2, 
            l);
        vec4 angle_col = vec4(c_alpha);
        gl_FragColor = angle_col;
        gl_FragColor.rgb = clamp(resCol, 0., 1.);
    }
    // gl_FragColor = vec4(1.);
}
