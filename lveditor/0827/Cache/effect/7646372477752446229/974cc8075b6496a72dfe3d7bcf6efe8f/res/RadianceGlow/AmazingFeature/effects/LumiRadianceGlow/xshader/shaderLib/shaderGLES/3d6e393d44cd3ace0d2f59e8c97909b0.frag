precision highp float;
precision highp int;

uniform vec4 u_ScreenParams;
uniform float u_sample;
uniform mediump sampler2D u_inputTexture;
uniform float u_sigma;
uniform float u_spaceDither;
uniform float u_stepX;
uniform float u_stepY;
uniform mediump int u_borderType;
uniform float u_exposure;

varying vec2 uv0;

float _f2(float _p0, float _p1)
{
    return exp((((-0.5) * _p0) * _p0) / (_p1 * _p1));
}

float _f1(vec2 _p0)
{
    vec3 _55 = fract(vec3((_p0 * u_ScreenParams.xy).xyx) * 0.103100001811981201171875);
    vec3 _t1 = _55 + vec3(dot(_55, _55.yzx + vec3(33.3300018310546875)));
    return fract(fract((_t1.x + _t1.y) * _t1.z)) * 2.0 + (-1.0);
}

float _f0(inout float _p0)
{
    _p0 = abs(_p0);
    return abs(floor(ceil(_p0) * 0.5) * 2.0 + (-_p0));
}

void main()
{
    if (u_sample < 9.9999997473787516355514526367188e-06)
    {
        gl_FragData[0] = texture2D(u_inputTexture, uv0);
        return;
    }
    float param = 0.0;
    float param_1 = u_sigma;
    float _130 = _f2(param, param_1);
    float _t4 = _130;
    vec4 _t5 = texture2D(u_inputTexture, uv0) * _130;
    vec2 _t6 = uv0;
    if (u_spaceDither > 9.9999997473787516355514526367188e-06)
    {
        vec2 param_2 = uv0;
        _t6 += (vec2(u_stepX, u_stepY) * (u_spaceDither * _f1(param_2)));
    }
    vec2 _t8 = _t6;
    for (mediump int _t9 = 1; _t9 <= 1024; _t9++)
    {
        mediump float _173 = float(_t9);
        if (_173 > u_sample)
        {
            break;
        }
        vec2 _185 = vec2(u_stepX, u_stepY) * _173;
        float param_3 = length(_185);
        float param_4 = u_sigma;
        float _194 = _f2(param_3, param_4);
        _t8 = _t6 - _185;
        bool _200 = _t8.x < 0.0;
        bool _207;
        if (!_200)
        {
            _207 = _t8.y < 0.0;
        }
        else
        {
            _207 = _200;
        }
        bool _214;
        if (!_207)
        {
            _214 = _t8.x > 1.0;
        }
        else
        {
            _214 = _207;
        }
        bool _221;
        if (!_214)
        {
            _221 = _t8.y > 1.0;
        }
        else
        {
            _221 = _214;
        }
        if (_221)
        {
            if (u_borderType == 0)
            {
                _t4 += _194;
            }
            else
            {
                if (u_borderType == 1)
                {
                    float param_5 = _t8.x;
                    float _242 = _f0(param_5);
                    _t8.x = _242;
                    float param_6 = _t8.y;
                    float _247 = _f0(param_6);
                    _t8.y = _247;
                    _t5 += (texture2D(u_inputTexture, _t8) * _194);
                    _t4 += _194;
                }
            }
        }
        else
        {
            _t5 += (texture2D(u_inputTexture, _t8) * _194);
            _t4 += _194;
        }
        _t8 = _t6 + _185;
        bool _275 = _t8.x < 0.0;
        bool _282;
        if (!_275)
        {
            _282 = _t8.y < 0.0;
        }
        else
        {
            _282 = _275;
        }
        bool _289;
        if (!_282)
        {
            _289 = _t8.x > 1.0;
        }
        else
        {
            _289 = _282;
        }
        bool _296;
        if (!_289)
        {
            _296 = _t8.y > 1.0;
        }
        else
        {
            _296 = _289;
        }
        if (_296)
        {
            if (u_borderType == 0)
            {
                _t4 += _194;
            }
            else
            {
                if (u_borderType == 1)
                {
                    float param_7 = _t8.x;
                    float _314 = _f0(param_7);
                    _t8.x = _314;
                    float param_8 = _t8.y;
                    float _319 = _f0(param_8);
                    _t8.y = _319;
                    _t5 += (texture2D(u_inputTexture, _t8) * _194);
                    _t4 += _194;
                }
            }
        }
        else
        {
            _t5 += (texture2D(u_inputTexture, _t8) * _194);
            _t4 += _194;
        }
    }
    vec4 _345 = _t5;
    vec4 _347 = _345 / vec4(_t4);
    _t5 = _347;
    vec3 _352 = _347.xyz * u_exposure;
    _t5.x = _352.x;
    _t5.y = _352.y;
    _t5.z = _352.z;
    gl_FragData[0] = clamp(_t5, vec4(0.0), vec4(1.0));
}

