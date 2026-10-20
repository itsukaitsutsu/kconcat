precision highp float;
precision highp int;

uniform float u_progress;
uniform float u_rotation;
uniform float u_feather;
uniform mediump sampler2D u_inputTexture;

varying vec2 v_uv;

vec2 _f0(vec2 _p0, float _p1)
{
    float _24 = sin(_p1);
    float _27 = cos(_p1);
    return ((_p0 - vec2(0.5)) * mat2(vec2(_27, -_24), vec2(_24, _27))) + vec2(0.5);
}

float _f1(vec2 _p0, float _p1, float _p2, float _p3)
{
    vec2 param = _p0;
    float param_1 = radians(_p2);
    vec2 _t3 = _f0(param, param_1);
    return mix(0.0, 1.0, smoothstep(_p1 - _p3, _p1 + _p3, _t3.y));
}

void main()
{
    vec2 param = v_uv;
    float param_1 = u_progress;
    float param_2 = u_rotation;
    float param_3 = u_feather;
    gl_FragData[0] = texture2D(u_inputTexture, v_uv) * _f1(param, param_1, param_2, param_3);
}

