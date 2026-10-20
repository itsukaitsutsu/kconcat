#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct buffer_t
{
    float u_stepX;
    float u_stepY;
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
float4 _f0(thread float4& _p0, thread const float2& _p1, thread const float& _p2, thread float2& uv0, constant float& u_stepX, constant float& u_stepY, texture2d<float> u_inputTexture, sampler u_inputTextureSmplr)
{
    float2 _30 = fma(_p1, float2(u_stepX, u_stepY), uv0);
    float2 _t0 = _30;
    float4 _t1 = float4(0.0, 0.0, 0.0, 1.0);
    bool _40 = _t0.x >= 0.0;
    bool _46;
    if (_40)
    {
        _46 = _t0.x <= 1.0;
    }
    else
    {
        _46 = _40;
    }
    bool _53;
    if (_46)
    {
        _53 = _t0.y >= 0.0;
    }
    else
    {
        _53 = _46;
    }
    bool _59;
    if (_53)
    {
        _59 = _t0.y <= 1.0;
    }
    else
    {
        _59 = _53;
    }
    if (_59)
    {
        _t1 = u_inputTexture.sample(u_inputTextureSmplr, _30);
    }
    float4 _71 = _t1;
    float3 _73 = _71.xyz * _p2;
    _t1.x = _73.x;
    _t1.y = _73.y;
    _t1.z = _73.z;
    _p0.x = fast::min(_p0.x, _t1.x);
    _p0.y = fast::min(_p0.y, _t1.y);
    _p0.z = fast::min(_p0.z, _t1.z);
    _p0.w = fast::min(_p0.w, _t1.w);
    return _p0;
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_inputTexture [[texture(0)]], sampler u_inputTextureSmplr [[sampler(0)]])
{
    main0_out out = {};
    float4 param = u_inputTexture.sample(u_inputTextureSmplr, in.uv0);
    float2 param_1 = float2(0.0, -1.0);
    float param_2 = 1.0;
    float4 _119 = _f0(param, param_1, param_2, in.uv0, buffer.u_stepX, buffer.u_stepY, u_inputTexture, u_inputTextureSmplr);
    float4 param_3 = _119;
    float2 param_4 = float2(0.0, 1.0);
    float param_5 = 1.0;
    float4 _125 = _f0(param_3, param_4, param_5, in.uv0, buffer.u_stepX, buffer.u_stepY, u_inputTexture, u_inputTextureSmplr);
    float4 param_6 = _125;
    float2 param_7 = float2(-1.0, 0.0);
    float param_8 = 1.0;
    float4 _131 = _f0(param_6, param_7, param_8, in.uv0, buffer.u_stepX, buffer.u_stepY, u_inputTexture, u_inputTextureSmplr);
    float4 param_9 = _131;
    float2 param_10 = float2(1.0, 0.0);
    float param_11 = 1.0;
    float4 _137 = _f0(param_9, param_10, param_11, in.uv0, buffer.u_stepX, buffer.u_stepY, u_inputTexture, u_inputTextureSmplr);
    out.o_fragColor = _137;
    return out;
}

