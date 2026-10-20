precision highp float;
precision highp int;

uniform float u_sampleX;
uniform mediump sampler2D u_inputTexture;
uniform float u_sigmaX;
uniform float u_dx;
uniform mediump int u_borderType;

varying vec2 v_uv;

float _f0(float _p0, float _p1)
{
    return exp((((-0.5) * _p0) * _p0) / (_p1 * _p1));
}

void main()
{
    if (u_sampleX < 9.9999997473787516355514526367188e-06)
    {
        gl_FragData[0] = texture2D(u_inputTexture, v_uv);
        return;
    }
    float param = 0.0;
    float param_1 = u_sigmaX;
    float _58 = _f0(param, param_1);
    float _t1 = _58;
    vec4 _t2 = texture2D(u_inputTexture, v_uv) * _58;
    vec2 _t3 = v_uv;
    for (mediump int _t4 = 1; _t4 <= 128; _t4++)
    {
        mediump float _80 = float(_t4);
        if (_80 > u_sampleX)
        {
            break;
        }
        float _91 = _80 * u_dx;
        float param_2 = _91;
        float param_3 = u_sigmaX;
        float _97 = _f0(param_2, param_3);
        _t3.x = v_uv.x - _91;
        if (_t3.x < 0.0)
        {
            if (u_borderType == 1)
            {
                _t3.x = 0.0;
                _t2 += (texture2D(u_inputTexture, _t3) * _97);
                _t1 += _97;
            }
            else
            {
                if (u_borderType == 2)
                {
                    _t2 += (vec4(0.0, 0.0, 0.0, 1.0) * _97);
                    _t1 += _97;
                }
                else
                {
                    if (u_borderType == 3)
                    {
                        _t3.x = -_t3.x;
                        _t2 += (texture2D(u_inputTexture, _t3) * _97);
                        _t1 += _97;
                    }
                }
            }
        }
        else
        {
            _t2 += (texture2D(u_inputTexture, _t3) * _97);
            _t1 += _97;
        }
        _t3.x = v_uv.x + _91;
        if (_t3.x > 1.0)
        {
            if (u_borderType == 1)
            {
                _t3.x = 1.0;
                _t2 += (texture2D(u_inputTexture, _t3) * _97);
                _t1 += _97;
            }
            else
            {
                if (u_borderType == 2)
                {
                    _t2 += (vec4(0.0, 0.0, 0.0, 1.0) * _97);
                    _t1 += _97;
                }
                else
                {
                    if (u_borderType == 3)
                    {
                        _t3.x = 2.0 - _t3.x;
                        _t2 += (texture2D(u_inputTexture, _t3) * _97);
                        _t1 += _97;
                    }
                }
            }
        }
        else
        {
            _t2 += (texture2D(u_inputTexture, _t3) * _97);
            _t1 += _97;
        }
    }
    vec4 _245 = _t2;
    vec4 _247 = _245 / vec4(_t1);
    _t2 = _247;
    gl_FragData[0] = _247;
}

