precision highp float;
precision highp int;

uniform mediump sampler2D u_inputTexture;
uniform float u_threshold;
uniform float u_stepY;
uniform float u_stepX;

varying vec2 uv;

vec4 _f0(inout vec4 _p0, vec4 _p1, vec4 _p2)
{
    float _23;
    if (_p1.x < 9.9999997473787516355514526367188e-06)
    {
        _23 = _p2.x;
    }
    else
    {
        _23 = _p0.x / _p1.x;
    }
    _p0.x = _23;
    float _40;
    if (_p1.y < 9.9999997473787516355514526367188e-06)
    {
        _40 = _p2.y;
    }
    else
    {
        _40 = _p0.y / _p1.y;
    }
    _p0.y = _40;
    float _57;
    if (_p1.z < 9.9999997473787516355514526367188e-06)
    {
        _57 = _p2.z;
    }
    else
    {
        _57 = _p0.z / _p1.z;
    }
    _p0.z = _57;
    float _74;
    if (_p1.w < 9.9999997473787516355514526367188e-06)
    {
        _74 = _p2.w;
    }
    else
    {
        _74 = _p0.w / _p1.w;
    }
    _p0.w = _74;
    return _p0;
}

void main()
{
    vec4 _t0 = vec4(0.0);
    vec4 _t1 = vec4(0.0);
    mediump vec4 _104 = texture2D(u_inputTexture, uv);
    float _109 = max(u_threshold, 9.9999997473787516355514526367188e-06);
    for (mediump int _t4 = -7; _t4 <= 7; _t4++)
    {
        float _127 = float(_t4) * u_stepY;
        for (mediump int _t6 = -7; _t6 <= 7; _t6++)
        {
            mediump vec4 _152 = texture2D(u_inputTexture, uv + vec2(float(_t6) * u_stepX, _127));
            vec4 _167 = max(vec4(1.0) - (abs(_152 - _104) / vec4(2.5 * _109)), vec4(0.0));
            _t0 += (_152 * _167);
            _t1 += _167;
        }
    }
    vec4 param = _t0;
    vec4 param_1 = _t1;
    vec4 param_2 = _104;
    vec4 _187 = _f0(param, param_1, param_2);
    _t0 = _187;
    gl_FragData[0] = clamp(_187, vec4(0.0), vec4(1.0));
}

