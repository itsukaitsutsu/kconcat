#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    float u_sampleY;
    float u_sigmaY;
    float u_dy;
    int u_borderType;
    float u_strength;
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

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_blurMidTexture [[texture(0)]], texture2d<float> u_inputTexture [[texture(1)]], sampler u_blurMidTextureSmplr [[sampler(0)]], sampler u_inputTextureSmplr [[sampler(1)]])
{
    main0_out out = {};
    float4 _37 = u_blurMidTexture.sample(u_blurMidTextureSmplr, in.v_uv);
    float4 _t1 = _37;
    if (buffer.u_sampleY > 9.9999997473787516355514526367188e-06)
    {
        float param = 0.0;
        float param_1 = buffer.u_sigmaY;
        float _54 = _f0(param, param_1);
        float _t2 = _54;
        _t1 = _37 * _54;
        float2 _t3 = in.v_uv;
        for (int _t4 = 1; _t4 <= 128; _t4++)
        {
            float _75 = float(_t4);
            if (_75 > buffer.u_sampleY)
            {
                break;
            }
            float _86 = _75 * buffer.u_dy;
            float param_2 = _86;
            float param_3 = buffer.u_sigmaY;
            float _92 = _f0(param_2, param_3);
            _t3.y = in.v_uv.y - _86;
            if (_t3.y < 0.0)
            {
                if (buffer.u_borderType == 1)
                {
                    _t3.y = 0.0;
                    _t1 += (u_blurMidTexture.sample(u_blurMidTextureSmplr, _t3) * _92);
                    _t2 += _92;
                }
                else
                {
                    if (buffer.u_borderType == 2)
                    {
                        _t1 += (float4(0.0, 0.0, 0.0, 1.0) * _92);
                        _t2 += _92;
                    }
                    else
                    {
                        if (buffer.u_borderType == 3)
                        {
                            _t3.y = -_t3.y;
                            _t1 += (u_blurMidTexture.sample(u_blurMidTextureSmplr, _t3) * _92);
                            _t2 += _92;
                        }
                    }
                }
            }
            else
            {
                _t1 += (u_blurMidTexture.sample(u_blurMidTextureSmplr, _t3) * _92);
                _t2 += _92;
            }
            _t3.y = in.v_uv.y + _86;
            if (_t3.y > 1.0)
            {
                if (buffer.u_borderType == 1)
                {
                    _t3.y = 1.0;
                    _t1 += (u_blurMidTexture.sample(u_blurMidTextureSmplr, _t3) * _92);
                    _t2 += _92;
                }
                else
                {
                    if (buffer.u_borderType == 2)
                    {
                        _t1 += (float4(0.0, 0.0, 0.0, 1.0) * _92);
                        _t2 += _92;
                    }
                    else
                    {
                        if (buffer.u_borderType == 3)
                        {
                            _t3.y = 2.0 - _t3.y;
                            _t1 += (u_blurMidTexture.sample(u_blurMidTextureSmplr, _t3) * _92);
                            _t2 += _92;
                        }
                    }
                }
            }
            else
            {
                _t1 += (u_blurMidTexture.sample(u_blurMidTextureSmplr, _t3) * _92);
                _t2 += _92;
            }
        }
        _t1 /= float4(_t2);
    }
    float4 _247 = u_inputTexture.sample(u_inputTextureSmplr, in.v_uv);
    float4 _t8 = _247;
    _t1.w = _t8.w;
    float4 _259 = _t1;
    float3 _263 = (_247.xyz * (1.0 + buffer.u_strength)) - (_259.xyz * buffer.u_strength);
    _t1.x = _263.x;
    _t1.y = _263.y;
    _t1.z = _263.z;
    out.o_fragColor = fast::clamp(_t1, float4(0.0), float4(1.0));
    return out;
}

