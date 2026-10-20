precision highp float;
precision highp int;

uniform mediump sampler2D u_inputTexture;
uniform float u_radiusX;
uniform float u_sigmaX;
uniform float u_dx;

varying vec2 uv0;

float _f0(float _p0, float _p1)
{
    return exp((((-0.5) * _p0) * _p0) / (_p1 * _p1));
}

void main()
{
    vec4 _t2 = texture2D(u_inputTexture, uv0);
    if (u_radiusX < 0.001000000047497451305389404296875)
    {
        gl_FragData[0] = _t2;
        return;
    }
    float _t3 = 0.0;
    vec4 _t4 = vec4(0.0);
    float _t5 = -10.0;
    while (_t5 <= 10.0)
    {
        if (_t5 > (u_radiusX + 0.001000000047497451305389404296875))
        {
            break;
        }
        float _78 = -u_radiusX;
        if (_t5 < _78)
        {
            _t5 = _78;
        }
        vec2 _89 = uv0 + vec2(_t5, 0.0);
        vec2 _t6 = _89;
        bool _94 = _t6.x >= 0.0;
        bool _101;
        if (_94)
        {
            _101 = _t6.x <= 1.0;
        }
        else
        {
            _101 = _94;
        }
        if (_101)
        {
            float param = _t5;
            float param_1 = u_sigmaX;
            float _110 = _f0(param, param_1);
            _t3 += _110;
            _t4 += (texture2D(u_inputTexture, _89) * _110);
        }
        _t5 += u_dx;
    }
    vec4 _128 = _t4 / vec4(_t3);
    _t2 = _128;
    gl_FragData[0] = clamp(_128, vec4(0.0), vec4(1.0));
}

