precision highp float;
precision highp int;

uniform float u_Steps;
uniform float u_Sample;
uniform float u_mirrorEdge;
uniform float u_Angle;
uniform float u_ExpandFlag;
uniform vec4 u_ScreenParams;
uniform mediump sampler2D u_InputTexture;

varying vec2 v_uv;

float _f1(float _p0, float _p1)
{
    return (0.3989399969577789306640625 * exp((((-0.5) * _p0) * _p0) / (_p1 * _p1))) / _p1;
}

vec2 _f0(vec2 _p0)
{
    return abs(mod(_p0 - vec2(1.0), vec2(2.0)) - vec2(1.0));
}

vec4 _f2(mediump sampler2D _p0, vec2 _p1, vec2 _p2)
{
    float param = 0.0;
    float param_1 = 4.0;
    float _64 = _f1(param, param_1);
    vec4 _t2 = vec4(0.0);
    vec2 _74 = _p2 * u_Steps;
    float _t8 = _64;
    for (mediump int _t10 = 1; _t10 <= 1024; _t10++)
    {
        if (float(_t10) > u_Sample)
        {
            break;
        }
        vec2 _118 = _p1 + (_74 * float(_t10));
        vec2 _t11 = _118;
        vec2 _t12 = _p1 + (_74 * float(-_t10));
        float _130 = step(u_mirrorEdge, 0.5);
        float _135 = 1.0 - _130;
        vec2 param_2 = _118;
        _t11 = (_118 * _130) + (_f0(param_2) * _135);
        vec2 _143 = _t12;
        vec2 param_3 = _143;
        vec2 _152 = (_143 * _130) + (_f0(param_3) * _135);
        _t12 = _152;
        float param_4 = (float(_t10) / u_Sample) * 15.0;
        float param_5 = 4.0;
        float _170 = _f1(param_4, param_5);
        _t2 = (_t2 + (pow(texture2D(_p0, _t11), vec4(1.0)) * _170)) + (pow(texture2D(_p0, _152), vec4(1.0)) * _170);
        _t8 += (_170 * 2.0);
    }
    return clamp(pow((_t2 + (pow(texture2D(_p0, _p1), vec4(1.0)) * _64)) / vec4(_t8), vec4(1.0)), vec4(0.0), vec4(1.0));
}

void main()
{
    float _216 = (u_Angle * 3.141592502593994140625) / 180.0;
    vec2 param = v_uv;
    vec2 param_1 = vec2(cos(_216), sin(_216)) / ((u_ScreenParams.xy * ((1.0 + (u_ExpandFlag * 0.4000000059604644775390625)) * 720.0)) / vec2(min(u_ScreenParams.x, u_ScreenParams.y)));
    gl_FragData[0] = _f2(u_InputTexture, param, param_1);
}

