precision highp float;
precision highp int;

uniform float u_stepX;
uniform float u_stepY;
uniform mediump sampler2D u_inputTexture;

varying vec2 uv0;

vec4 _f0(inout vec4 _p0, vec2 _p1, float _p2)
{
    vec2 _30 = _p1 * vec2(u_stepX, u_stepY) + uv0;
    vec2 _t0 = _30;
    vec4 _t1 = vec4(0.0, 0.0, 0.0, 1.0);
    bool _40 = _t0.x >= 0.0;
    bool _46;
    if (_40)
    {
        _46 = _t0.x <= 1.0;
    }
    else
    {
        _46 = _40;
    }
    bool _53;
    if (_46)
    {
        _53 = _t0.y >= 0.0;
    }
    else
    {
        _53 = _46;
    }
    bool _59;
    if (_53)
    {
        _59 = _t0.y <= 1.0;
    }
    else
    {
        _59 = _53;
    }
    if (_59)
    {
        _t1 = texture2D(u_inputTexture, _30);
    }
    vec4 _71 = _t1;
    vec3 _73 = _71.xyz * _p2;
    _t1.x = _73.x;
    _t1.y = _73.y;
    _t1.z = _73.z;
    _p0.x = min(_p0.x, _t1.x);
    _p0.y = min(_p0.y, _t1.y);
    _p0.z = min(_p0.z, _t1.z);
    _p0.w = min(_p0.w, _t1.w);
    return _p0;
}

void main()
{
    vec4 param = texture2D(u_inputTexture, uv0);
    vec2 param_1 = vec2(0.0, -1.0);
    float param_2 = 1.0;
    vec4 _119 = _f0(param, param_1, param_2);
    vec4 param_3 = _119;
    vec2 param_4 = vec2(0.0, 1.0);
    float param_5 = 1.0;
    vec4 _125 = _f0(param_3, param_4, param_5);
    vec4 param_6 = _125;
    vec2 param_7 = vec2(-1.0, 0.0);
    float param_8 = 1.0;
    vec4 _131 = _f0(param_6, param_7, param_8);
    vec4 param_9 = _131;
    vec2 param_10 = vec2(1.0, 0.0);
    float param_11 = 1.0;
    vec4 _137 = _f0(param_9, param_10, param_11);
    gl_FragData[0] = _137;
}

