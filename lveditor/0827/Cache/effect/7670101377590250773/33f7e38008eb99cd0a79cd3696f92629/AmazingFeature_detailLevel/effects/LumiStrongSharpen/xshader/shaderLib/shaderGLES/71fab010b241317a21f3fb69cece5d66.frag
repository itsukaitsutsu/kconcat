precision highp float;
precision highp int;

uniform mediump sampler2D u_blurMidTexture;
uniform float u_sampleY;
uniform float u_sigmaY;
uniform float u_dy;
uniform mediump int u_borderType;
uniform mediump sampler2D u_inputTexture;
uniform float u_strength;

varying vec2 v_uv;

float _f0(float _p0, float _p1)
{
    return exp((((-0.5) * _p0) * _p0) / (_p1 * _p1));
}

void main()
{
    mediump vec4 _37 = texture2D(u_blurMidTexture, v_uv);
    vec4 _t1 = _37;
    if (u_sampleY > 9.9999997473787516355514526367188e-06)
    {
        float param = 0.0;
        float param_1 = u_sigmaY;
        float _54 = _f0(param, param_1);
        float _t2 = _54;
        _t1 = _37 * _54;
        vec2 _t3 = v_uv;
        for (mediump int _t4 = 1; _t4 <= 128; _t4++)
        {
            mediump float _75 = float(_t4);
            if (_75 > u_sampleY)
            {
                break;
            }
            float _86 = _75 * u_dy;
            float param_2 = _86;
            float param_3 = u_sigmaY;
            float _92 = _f0(param_2, param_3);
            _t3.y = v_uv.y - _86;
            if (_t3.y < 0.0)
            {
                if (u_borderType == 1)
                {
                    _t3.y = 0.0;
                    _t1 += (texture2D(u_blurMidTexture, _t3) * _92);
                    _t2 += _92;
                }
                else
                {
                    if (u_borderType == 2)
                    {
                        _t1 += (vec4(0.0, 0.0, 0.0, 1.0) * _92);
                        _t2 += _92;
                    }
                    else
                    {
                        if (u_borderType == 3)
                        {
                            _t3.y = -_t3.y;
                            _t1 += (texture2D(u_blurMidTexture, _t3) * _92);
                            _t2 += _92;
                        }
                    }
                }
            }
            else
            {
                _t1 += (texture2D(u_blurMidTexture, _t3) * _92);
                _t2 += _92;
            }
            _t3.y = v_uv.y + _86;
            if (_t3.y > 1.0)
            {
                if (u_borderType == 1)
                {
                    _t3.y = 1.0;
                    _t1 += (texture2D(u_blurMidTexture, _t3) * _92);
                    _t2 += _92;
                }
                else
                {
                    if (u_borderType == 2)
                    {
                        _t1 += (vec4(0.0, 0.0, 0.0, 1.0) * _92);
                        _t2 += _92;
                    }
                    else
                    {
                        if (u_borderType == 3)
                        {
                            _t3.y = 2.0 - _t3.y;
                            _t1 += (texture2D(u_blurMidTexture, _t3) * _92);
                            _t2 += _92;
                        }
                    }
                }
            }
            else
            {
                _t1 += (texture2D(u_blurMidTexture, _t3) * _92);
                _t2 += _92;
            }
        }
        _t1 /= vec4(_t2);
    }
    mediump vec4 _247 = texture2D(u_inputTexture, v_uv);
    vec4 _t8 = _247;
    _t1.w = _t8.w;
    vec4 _259 = _t1;
    vec3 _263 = (_247.xyz * (1.0 + u_strength)) - (_259.xyz * u_strength);
    _t1.x = _263.x;
    _t1.y = _263.y;
    _t1.z = _263.z;
    gl_FragData[0] = clamp(_t1, vec4(0.0), vec4(1.0));
}

