#pragma clang diagnostic ignored "-Wmissing-prototypes"
#pragma clang diagnostic ignored "-Wmissing-braces"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

template<typename T, size_t Num>
struct spvUnsafeArray
{
    T elements[Num ? Num : 1];
    
    thread T& operator [] (size_t pos) thread
    {
        return elements[pos];
    }
    constexpr const thread T& operator [] (size_t pos) const thread
    {
        return elements[pos];
    }
    
    device T& operator [] (size_t pos) device
    {
        return elements[pos];
    }
    constexpr const device T& operator [] (size_t pos) const device
    {
        return elements[pos];
    }
    
    constexpr const constant T& operator [] (size_t pos) const constant
    {
        return elements[pos];
    }
    
    threadgroup T& operator [] (size_t pos) threadgroup
    {
        return elements[pos];
    }
    constexpr const threadgroup T& operator [] (size_t pos) const threadgroup
    {
        return elements[pos];
    }
};

struct buffer_t
{
    float2 u_ScreenParams;
    float4 u_offsets;
    float strength;
    float tv;
    float human;
    float4 u_sliderInfos;
    float alpha_s_full;
};

struct main0_out
{
    float4 o_fragColor [[color(0)]];
};

struct main0_in
{
    float2 uv [[user(locn0)]];
};

static inline __attribute__((always_inline))
float _f2(thread const float3& _p0, thread const float& _p1)
{
    return exp(((-0.5) * dot(_p0, _p0)) / (_p1 * _p1));
}

static inline __attribute__((always_inline))
float3 _f0(thread const float3& _p0)
{
    float4 _59 = mix(float4(_p0.zy, -1.0, 0.666666686534881591796875), float4(_p0.yz, 0.0, -0.3333333432674407958984375), float4(step(_p0.z, _p0.y)));
    float4 _t1 = _59;
    float4 _t2 = mix(float4(_59.xyw, _p0.x), float4(_p0.x, _59.yzx), float4(step(_t1.x, _p0.x)));
    float _94 = _t2.x - fast::min(_t2.w, _t2.y);
    return float3(abs(_t2.z + ((_t2.w - _t2.y) / ((6.0 * _94) + 1.0000000133514319600180897396058e-10))), _94 / (_t2.x + 1.0000000133514319600180897396058e-10), _t2.x);
}

static inline __attribute__((always_inline))
float3 _f1(thread const float3& _p0)
{
    return mix(float3(1.0), fast::clamp(abs((fract(_p0.xxx + float3(1.0, 0.666666686534881591796875, 0.3333333432674407958984375)) * 6.0) - float3(3.0)) - float3(1.0), float3(0.0), float3(1.0)), float3(_p0.y)) * _p0.z;
}

fragment main0_out main0(main0_in in [[stage_in]], constant buffer_t& buffer, texture2d<float> texture_source [[texture(0)]], texture2d<float> texture_skin [[texture(1)]], texture2d<float> texture_trans [[texture(2)]], texture2d<float> texture_curves [[texture(3)]], texture2d<float> texture_color [[texture(4)]], sampler texture_sourceSmplr [[sampler(0)]], sampler texture_skinSmplr [[sampler(1)]], sampler texture_transSmplr [[sampler(2)]], sampler texture_curvesSmplr [[sampler(3)]], sampler texture_colorSmplr [[sampler(4)]])
{
    main0_out out = {};
    float2 _180 = float2(in.uv.x, 1.0 - in.uv.y);
    float2 _t9 = float2(1.0 / buffer.u_ScreenParams.x, 1.0 / buffer.u_ScreenParams.y) * fast::min(buffer.u_ScreenParams.x, buffer.u_ScreenParams.y);
    float2 _205 = float2(buffer.u_offsets.w, 0.0);
    float2x2 _215 = float2x2(float2(_t9.x, 0.0), float2(0.0, _t9.y));
    float2x2 _224 = float2x2(float2(-_t9.x, 0.0), float2(0.0, _t9.y));
    float2x2 _233 = float2x2(float2(_t9.x, 0.0), float2(0.0, -_t9.y));
    float2x2 _243 = float2x2(float2(-_t9.x, 0.0), float2(0.0, -_t9.y));
    spvUnsafeArray<float2, 10> _t15;
    _t15[0] = in.uv + (_215 * buffer.u_offsets.xy);
    _t15[1] = in.uv + (_224 * buffer.u_offsets.xy);
    _t15[2] = in.uv + (_215 * buffer.u_offsets.zz);
    _t15[3] = in.uv + (_224 * buffer.u_offsets.zz);
    _t15[4] = in.uv + (_215 * _205);
    _t15[5] = in.uv + (_224 * _205);
    _t15[6] = in.uv + (_233 * buffer.u_offsets.zz);
    _t15[7] = in.uv + (_243 * buffer.u_offsets.zz);
    _t15[8] = in.uv + (_233 * buffer.u_offsets.xy);
    _t15[9] = in.uv + (_243 * buffer.u_offsets.xy);
    spvUnsafeArray<float, 10> _t16;
    _t16[0] = 0.895183026790618896484375;
    _t16[1] = 0.895183026790618896484375;
    _t16[2] = 0.9726979732513427734375;
    _t16[3] = 0.9726979732513427734375;
    _t16[4] = 0.895183026790618896484375;
    _t16[5] = 0.895183026790618896484375;
    _t16[6] = 0.9726979732513427734375;
    _t16[7] = 0.9726979732513427734375;
    _t16[8] = 0.895183026790618896484375;
    _t16[9] = 0.895183026790618896484375;
    float4 _t17 = texture_source.sample(texture_sourceSmplr, in.uv);
    if (buffer.strength > 0.0089999996125698089599609375)
    {
        float _t18 = 1.0;
        float3 _t19 = _t17.xyz;
        for (int _t20 = 0; _t20 < 10; _t20++)
        {
            float3 _377 = texture_source.sample(texture_sourceSmplr, _t15[_t20]).xyz;
            float3 param = _t17.xyz - _377;
            float param_1 = 0.0199999995529651641845703125 * buffer.strength;
            float _392 = _f2(param, param_1);
            _t18 += (_t16[_t20] * _392);
            _t19 += ((_377 * _t16[_t20]) * _392);
        }
        float3 _410 = _t19 / float3(_t18);
        _t17.x = _410.x;
        _t17.y = _410.y;
        _t17.z = _410.z;
    }
    float4 _421 = texture_skin.sample(texture_skinSmplr, _180);
    float _422 = _421.x;
    float4 _427 = texture_trans.sample(texture_transSmplr, _180);
    float _428 = _427.x;
    float _453 = mix(fast::clamp(buffer.strength * 1.5, 0.0, 1.0), buffer.strength * (1.0 - (_422 * 0.87999999523162841796875)), buffer.human);
    float3 param_2 = ((_t17.xyz - float3(0.800000011920928955078125)) / float3(mix(1.0, fast::max(1.0 - (_453 * 0.4000000059604644775390625), mix(_428, mix(_428, buffer.tv, fast::max(_422, 0.300000011920928955078125)), buffer.human)), buffer.u_sliderInfos.x))) + float3(0.800000011920928955078125);
    float3 _481 = _f0(param_2);
    float3 _t29 = _481;
    float3 _t33 = _481;
    float4 _504 = texture_curves.sample(texture_curvesSmplr, float2(fast::clamp(_t29.z, 0.0, 1.0), 0.0));
    float _505 = _504.z;
    _t33.z = mix(mix(_t29.z, _505, 0.0500000007450580596923828125 * _453), mix(_t29.z, _505, 0.2599999904632568359375 * _453), _422);
    float4 _529 = texture_curves.sample(texture_curvesSmplr, float2(fast::clamp(_t29.y, 0.0, 1.0), 0.0));
    float _530 = _529.y;
    _t33.y = mix(_t29.y, mix(_t29.y, _530, buffer.alpha_s_full * _453), _422);
    _t33.y = mix(_t33.y, mix(_t33.y, mix(_t33.y, _530, 0.60000002384185791015625 * _453), texture_curves.sample(texture_curvesSmplr, float2(fast::clamp(1.0 - texture_color.sample(texture_colorSmplr, in.uv).x, 0.0, 1.0), 0.0)).x), buffer.human);
    float3 param_3 = _t33;
    out.o_fragColor = float4(_f1(param_3), _t17.w);
    return out;
}

