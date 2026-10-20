#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    float u_exposure;
    int u_directionNum;
    float3 u_glowColor;
    int u_displayGlow;
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
float4 _f0(thread const float4& _p0, thread const float4& _p1)
{
    return fma(-_p0, _p1, _p0 + _p1);
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_tex1 [[texture(0)]], texture2d<float> u_tex2 [[texture(1)]], texture2d<float> u_tex3 [[texture(2)]], texture2d<float> u_tex4 [[texture(3)]], texture2d<float> u_inputTexture [[texture(4)]], sampler u_tex1Smplr [[sampler(0)]], sampler u_tex2Smplr [[sampler(1)]], sampler u_tex3Smplr [[sampler(2)]], sampler u_tex4Smplr [[sampler(3)]], sampler u_inputTextureSmplr [[sampler(4)]])
{
    main0_out out = {};
    float4 _t0 = float4(0.0);
    if (buffer.u_exposure > 9.9999997473787516355514526367188e-06)
    {
        _t0 = u_tex1.sample(u_tex1Smplr, in.uv0);
    }
    if (buffer.u_directionNum >= 2)
    {
        float4 param = _t0;
        float4 param_1 = u_tex2.sample(u_tex2Smplr, in.uv0);
        _t0 = _f0(param, param_1);
    }
    if (buffer.u_directionNum >= 3)
    {
        float4 param_2 = _t0;
        float4 param_3 = u_tex3.sample(u_tex3Smplr, in.uv0);
        _t0 = _f0(param_2, param_3);
    }
    if (buffer.u_directionNum >= 4)
    {
        float4 param_4 = _t0;
        float4 param_5 = u_tex4.sample(u_tex4Smplr, in.uv0);
        _t0 = _f0(param_4, param_5);
    }
    float4 _96 = _t0;
    float3 _98 = _96.xyz * buffer.u_glowColor;
    _t0.x = _98.x;
    _t0.y = _98.y;
    _t0.z = _98.z;
    if (buffer.u_displayGlow == 1)
    {
        out.o_fragColor = _t0;
    }
    else
    {
        float4 param_6 = _t0;
        float4 param_7 = u_inputTexture.sample(u_inputTextureSmplr, in.uv0);
        _t0 = _f0(param_6, param_7);
    }
    out.o_fragColor = fast::clamp(_t0, float4(0.0), float4(1.0));
    return out;
}

