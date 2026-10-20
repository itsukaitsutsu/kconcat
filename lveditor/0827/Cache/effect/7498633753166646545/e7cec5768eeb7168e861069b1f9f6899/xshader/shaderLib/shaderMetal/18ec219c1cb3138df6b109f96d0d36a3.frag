#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

// Implementation of the GLSL mod() function, which is slightly different than Metal fmod()
template<typename Tx, typename Ty>
inline Tx mod(Tx x, Ty y)
{
    return x - y * floor(x / y);
}

struct buffer_t
{
    float u_Steps;
    float u_Sample;
    float u_mirrorEdge;
    float u_Angle;
    float u_ExpandFlag;
    float4 u_ScreenParams;
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
float _f1(thread const float& _p0, thread const float& _p1)
{
    return (0.3989399969577789306640625 * exp((((-0.5) * _p0) * _p0) / (_p1 * _p1))) / _p1;
}

static inline __attribute__((always_inline))
float2 _f0(thread const float2& _p0)
{
    return abs(mod(_p0 - float2(1.0), float2(2.0)) - float2(1.0));
}

static inline __attribute__((always_inline))
float4 _f2(texture2d<float> _p0, sampler _p0Smplr, thread const float2& _p1, thread const float2& _p2, constant float& u_Steps, constant float& u_Sample, constant float& u_mirrorEdge)
{
    float param = 0.0;
    float param_1 = 4.0;
    float _64 = _f1(param, param_1);
    float4 _t2 = float4(0.0);
    float2 _74 = _p2 * u_Steps;
    float _t8 = _64;
    for (int _t10 = 1; _t10 <= 1024; _t10++)
    {
        if (float(_t10) > u_Sample)
        {
            break;
        }
        float2 _118 = _p1 + (_74 * float(_t10));
        float2 _t11 = _118;
        float2 _t12 = _p1 + (_74 * float(-_t10));
        float _130 = step(u_mirrorEdge, 0.5);
        float _135 = 1.0 - _130;
        float2 param_2 = _118;
        _t11 = (_118 * _130) + (_f0(param_2) * _135);
        float2 _143 = _t12;
        float2 param_3 = _143;
        float2 _152 = (_143 * _130) + (_f0(param_3) * _135);
        _t12 = _152;
        float param_4 = (float(_t10) / u_Sample) * 15.0;
        float param_5 = 4.0;
        float _170 = _f1(param_4, param_5);
        _t2 = (_t2 + (pow(_p0.sample(_p0Smplr, _t11), float4(1.0)) * _170)) + (pow(_p0.sample(_p0Smplr, _152), float4(1.0)) * _170);
        _t8 += (_170 * 2.0);
    }
    return fast::clamp(pow((_t2 + (pow(_p0.sample(_p0Smplr, _p1), float4(1.0)) * _64)) / float4(_t8), float4(1.0)), float4(0.0), float4(1.0));
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> u_InputTexture [[texture(0)]], sampler u_InputTextureSmplr [[sampler(0)]])
{
    main0_out out = {};
    float _216 = (buffer.u_Angle * 3.141592502593994140625) / 180.0;
    float2 param = in.v_uv;
    float2 param_1 = float2(cos(_216), sin(_216)) / ((buffer.u_ScreenParams.xy * ((1.0 + (buffer.u_ExpandFlag * 0.4000000059604644775390625)) * 720.0)) / float2(fast::min(buffer.u_ScreenParams.x, buffer.u_ScreenParams.y)));
    out.o_fragColor = _f2(u_InputTexture, u_InputTextureSmplr, param, param_1, buffer.u_Steps, buffer.u_Sample, buffer.u_mirrorEdge);
    return out;
}

