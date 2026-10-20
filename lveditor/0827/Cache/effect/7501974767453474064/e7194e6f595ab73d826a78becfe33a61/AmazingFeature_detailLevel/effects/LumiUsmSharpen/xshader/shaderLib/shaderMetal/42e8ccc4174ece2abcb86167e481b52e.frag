#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    float u_radiusX;
    float u_sigmaX;
    float u_dx;
};

struct main0_out
{
    float4 o_fragColor [[color(0)]];
};

struct main0_in
{
    float2 uv0 [[user(locn0)]];
};

static inline __attribute__((always_inline))
float _f0(thread const float& _p0, thread const float& _p1)
{
    return exp((((-0.5) * _p0) * _p0) / (_p1 * _p1));
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_inputTexture [[texture(0)]], sampler u_inputTextureSmplr [[sampler(0)]])
{
    main0_out out = {};
    float4 _t2 = u_inputTexture.sample(u_inputTextureSmplr, in.uv0);
    if (buffer.u_radiusX < 0.001000000047497451305389404296875)
    {
        out.o_fragColor = _t2;
        return out;
    }
    float _t3 = 0.0;
    float4 _t4 = float4(0.0);
    float _t5 = -10.0;
    while (_t5 <= 10.0)
    {
        if (_t5 > (buffer.u_radiusX + 0.001000000047497451305389404296875))
        {
            break;
        }
        float _78 = -buffer.u_radiusX;
        if (_t5 < _78)
        {
            _t5 = _78;
        }
        float2 _89 = in.uv0 + float2(_t5, 0.0);
        float2 _t6 = _89;
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
            float param_1 = buffer.u_sigmaX;
            float _110 = _f0(param, param_1);
            _t3 += _110;
            _t4 += (u_inputTexture.sample(u_inputTextureSmplr, _89) * _110);
        }
        _t5 += buffer.u_dx;
    }
    float4 _128 = _t4 / float4(_t3);
    _t2 = _128;
    out.o_fragColor = fast::clamp(_128, float4(0.0), float4(1.0));
    return out;
}

