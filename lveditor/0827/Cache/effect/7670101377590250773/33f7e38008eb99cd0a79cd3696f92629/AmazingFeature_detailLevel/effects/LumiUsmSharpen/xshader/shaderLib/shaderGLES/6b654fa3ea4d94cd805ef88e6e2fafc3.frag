precision highp float;
precision highp int;

uniform mediump sampler2D u_inputTexture;
uniform mediump sampler2D u_albedo;
uniform float u_radiusY;
uniform float u_sigmaY;
uniform float u_dy;
uniform float u_threshold;
uniform float u_intensity;
uniform mediump sampler2D mask123;

varying vec2 uv0;

float _f0(float _p0, float _p1)
{
    return exp((((-0.5) * _p0) * _p0) / (_p1 * _p1));
}

void main()
{
    vec4 _t2 = texture2D(u_inputTexture, uv0);
    if (u_radiusY < 0.001000000047497451305389404296875)
    {
        gl_FragData[0] = _t2;
        return;
    }
    float _t4 = 0.0;
    vec4 _t5 = vec4(0.0);
    float _t6 = -10.0;
    while (_t6 <= 10.0)
    {
        if (_t6 > (u_radiusY + 0.001000000047497451305389404296875))
        {
            break;
        }
        float _83 = -u_radiusY;
        if (_t6 < _83)
        {
            _t6 = _83;
        }
        vec2 _94 = uv0 + vec2(0.0, _t6);
        vec2 _t7 = _94;
        bool _99 = _t7.y >= 0.0;
        bool _106;
        if (_99)
        {
            _106 = _t7.y <= 1.0;
        }
        else
        {
            _106 = _99;
        }
        if (_106)
        {
            float param = _t6;
            float param_1 = u_sigmaY;
            float _115 = _f0(param, param_1);
            _t4 += _115;
            _t5 += (texture2D(u_albedo, _94) * _115);
        }
        _t6 += u_dy;
    }
    vec4 _135 = _t2;
    vec4 _137 = _135 - (_t5 / vec4(_t4));
    vec3 _157 = smoothstep(vec4(u_threshold / 5.0), vec4(u_threshold), abs(_137)).xyz;
    vec3 _160 = _137.xyz;
    vec3 _162 = _135.xyz + ((_157 * u_intensity) * _160);
    _t2.x = _162.x;
    _t2.y = _162.y;
    _t2.z = _162.z;
    vec4 _t12 = texture2D(mask123, uv0);
    vec4 _177 = _t2;
    vec4 _t13 = _177;
    vec3 _187 = _177.xyz + ((_157 * 0.20000000298023223876953125) * _160);
    _t13.x = _187.x;
    _t13.y = _187.y;
    _t13.z = _187.z;
    vec4 _199 = mix(_177, _t13, vec4(_t12.y));
    _t2 = _199;
    gl_FragData[0] = clamp(_199, vec4(0.0), vec4(1.0));
}

