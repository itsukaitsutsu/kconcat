#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    float u_sampleX;
    float u_sigmaX;
    float u_dx;
    int u_borderType;
};

struct main0_out
{
    float4 o_fragColor [[color(0)]];
};

struct main0_in
{
    float2 v_uv [[user(locn0)]];
};

static inline __attribute__((always_inline))
float _f0(thread const float& _p0, thread const float& _p1)
{
    return exp((((-0.5) * _p0) * _p0) / (_p1 * _p1));
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_inputTexture [[texture(0)]], sampler u_inputTextureSmplr [[sampler(0)]])
{
    main0_out out = {};
    if (buffer.u_sampleX < 9.9999997473787516355514526367188e-06)
    {
        out.o_fragColor = u_inputTexture.sample(u_inputTextureSmplr, in.v_uv);
        return out;
    }
    float param = 0.0;
    float param_1 = buffer.u_sigmaX;
    float _58 = _f0(param, param_1);
    float _t1 = _58;
    float4 _t2 = u_inputTexture.sample(u_inputTextureSmplr, in.v_uv) * _58;
    float2 _t3 = in.v_uv;
    for (int _t4 = 1; _t4 <= 128; _t4++)
    {
        float _80 = float(_t4);
        if (_80 > buffer.u_sampleX)
        {
            break;
        }
        float _91 = _80 * buffer.u_dx;
        float param_2 = _91;
        float param_3 = buffer.u_sigmaX;
        float _97 = _f0(param_2, param_3);
        _t3.x = in.v_uv.x - _91;
        if (_t3.x < 0.0)
        {
            if (buffer.u_borderType == 1)
            {
                _t3.x = 0.0;
                _t2 += (u_inputTexture.sample(u_inputTextureSmplr, _t3) * _97);
                _t1 += _97;
            }
            else
            {
                if (buffer.u_borderType == 2)
                {
                    _t2 += (float4(0.0, 0.0, 0.0, 1.0) * _97);
                    _t1 += _97;
                }
                else
                {
                    if (buffer.u_borderType == 3)
                    {
                        _t3.x = -_t3.x;
                        _t2 += (u_inputTexture.sample(u_inputTextureSmplr, _t3) * _97);
                        _t1 += _97;
                    }
                }
            }
        }
        else
        {
            _t2 += (u_inputTexture.sample(u_inputTextureSmplr, _t3) * _97);
            _t1 += _97;
        }
        _t3.x = in.v_uv.x + _91;
        if (_t3.x > 1.0)
        {
            if (buffer.u_borderType == 1)
            {
                _t3.x = 1.0;
                _t2 += (u_inputTexture.sample(u_inputTextureSmplr, _t3) * _97);
                _t1 += _97;
            }
            else
            {
                if (buffer.u_borderType == 2)
                {
                    _t2 += (float4(0.0, 0.0, 0.0, 1.0) * _97);
                    _t1 += _97;
                }
                else
                {
                    if (buffer.u_borderType == 3)
                    {
                        _t3.x = 2.0 - _t3.x;
                        _t2 += (u_inputTexture.sample(u_inputTextureSmplr, _t3) * _97);
                        _t1 += _97;
                    }
                }
            }
        }
        else
        {
            _t2 += (u_inputTexture.sample(u_inputTextureSmplr, _t3) * _97);
            _t1 += _97;
        }
    }
    float4 _245 = _t2;
    float4 _247 = _245 / float4(_t1);
    _t2 = _247;
    out.o_fragColor = _247;
    return out;
}

