#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

// Implementation of the GLSL radians() function
template<typename T>
inline T radians(T d)
{
    return d * T(0.01745329251);
}

struct buffer_t
{
    float u_progress;
    float u_rotation;
    float u_feather;
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
float2 _f0(thread const float2& _p0, thread const float& _p1)
{
    float _24 = sin(_p1);
    float _27 = cos(_p1);
    return ((_p0 - float2(0.5)) * float2x2(float2(_27, -_24), float2(_24, _27))) + float2(0.5);
}

static inline __attribute__((always_inline))
float _f1(thread const float2& _p0, thread const float& _p1, thread const float& _p2, thread const float& _p3)
{
    float2 param = _p0;
    float param_1 = radians(_p2);
    float2 _t3 = _f0(param, param_1);
    return mix(0.0, 1.0, smoothstep(_p1 - _p3, _p1 + _p3, _t3.y));
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_inputTexture [[texture(0)]], sampler u_inputTextureSmplr [[sampler(0)]])
{
    main0_out out = {};
    float2 param = in.v_uv;
    float param_1 = buffer.u_progress;
    float param_2 = buffer.u_rotation;
    float param_3 = buffer.u_feather;
    out.o_fragColor = u_inputTexture.sample(u_inputTextureSmplr, in.v_uv) * _f1(param, param_1, param_2, param_3);
    return out;
}

